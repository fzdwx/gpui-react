use std::{collections::{HashMap, VecDeque}, sync::{Arc, Mutex, atomic::{AtomicU64, Ordering}}};

use gpui::{AnyWindowHandle, App, AppContext};

use crate::element::{ElementKind, ElementStyle, ReactElement};

/// Event message to be sent to JS
#[derive(Clone, Debug)]
pub struct EventMessage {
	pub window_id: u64,
	pub element_id: u64,
	pub event_type: String,
	pub payload: String, // JSON payload
}

pub struct Window {
	/// The GPUI window handle
	h:         AnyWindowHandle,
	/// The React element state for this window
	state:     Arc<WindowState>,
	window_id: u64,
}

impl Window {
	/// Create a new window with the given GPUI handle
	pub fn new(h: AnyWindowHandle) -> Self {
		let window_id = h.window_id().as_u64();
		Self { h, state: Arc::new(WindowState::new()), window_id }
	}

	pub fn refresh(&self, app: &mut App) {
		if let Err(e) = app.update_window(self.h, |_view, w, app| {
			self.state.increment_render_count();
			w.refresh();
			log::trace!("Calling window.refresh() for window {}", self.window_id);
		}) {
			log::error!("window refresh err {}", e)
		}
	}

	/// Get the window state
	pub fn state(&self) -> &Arc<WindowState> { &self.state }

	/// Get mutable access to the window state
	pub fn state_mut(&mut self) -> &mut Arc<WindowState> { &mut self.state }

	/// Render a single element with its children
	/// This method sets the root element ID and rebuilds the element tree
	/// It should be called after batch_update_elements has populated the
	/// element_map
	pub fn render_element(
		&self,
		global_id: u64,
		_element_type: String,
		_text: Option<String>,
		children: &[u64],
	) {
		// Don't create a new element if it already exists in the map
		// batch_update_elements should have already populated the elements with proper
		// styles
		let mut element_map =
			self.state.element_map.lock().expect("Failed to acquire element_map lock in render_element");

		// Only create placeholder elements for children that don't exist
		for &child_id in children {
			if !element_map.contains_key(&child_id) {
				let placeholder = Arc::new(ReactElement {
					global_id:         child_id,
					element_type:      "placeholder".to_string(),
					element_kind:      ElementKind::Unknown,
					text:              None,
					children:          Vec::new(),
					style:             ElementStyle::default(),
					event_handlers:    None,
					cached_gpui_style: None,
				});
				element_map.insert(child_id, placeholder);
			}
		}

		drop(element_map);

		self.state.set_root_element_id(global_id);
		self.state.rebuild_tree(global_id, children);
		self.state.update_element_tree();
	}

	/// Batch update multiple elements from JSON data
	pub fn batch_update_elements(&self, elements: &serde_json::Value) {
		let elements_array = elements.as_array().expect("Elements must be an array");

		let mut element_map = self
			.state
			.element_map
			.lock()
			.expect("Failed to acquire element_map lock in batch_update_elements");

		// First pass: create all elements
		for elem_value in elements_array {
			if let Some(elem_obj) = elem_value.as_object() {
				let global_id = elem_obj.get("globalId").and_then(|v| v.as_u64()).unwrap_or(0);

				let element_type = elem_obj.get("type").and_then(|v| v.as_str()).unwrap_or("").to_string();

				let text = elem_obj.get("text").and_then(|v| v.as_str()).map(|s| s.to_string());

				let style = elem_obj.get("style").map(ElementStyle::from_json).unwrap_or_default();

				let event_handlers = elem_obj.get("eventHandlers").cloned();

				// Pre-compute GPUI Style (div and span have no default background)
				let cached_gpui_style = Some(style.build_gpui_style(None));

				let element_kind = ElementKind::from_str(&element_type);
				let element = Arc::new(ReactElement {
					global_id,
					element_type,
					element_kind,
					text,
					children: Vec::new(),
					style,
					event_handlers,
					cached_gpui_style,
				});

				element_map.insert(global_id, element);
			}
		}

		// Second pass: update children references
		for elem_value in elements_array {
			if let Some(elem_obj) = elem_value.as_object() {
				if let Some(global_id) = elem_obj.get("globalId").and_then(|v| v.as_u64()) {
					if let Some(children_arr) = elem_obj.get("children").and_then(|v| v.as_array()) {
						let children_ids: Vec<u64> = children_arr.iter().filter_map(|c| c.as_u64()).collect();

						let mut child_refs: Vec<Arc<ReactElement>> = Vec::new();

						for &cid in &children_ids {
							if let Some(child) = element_map.get(&cid) {
								child_refs.push(child.clone());
							}
						}

						if let Some(element) = element_map.get_mut(&global_id) {
							let element_mut = Arc::make_mut(element);
							element_mut.children = child_refs;
						}
					}
				}
			}
		}
	}
}

pub struct WindowState {
	pub root_element_id: AtomicU64,
	pub element_map:     Mutex<HashMap<u64, Arc<ReactElement>>>,
	pub element_tree:    Arc<Mutex<Option<Arc<ReactElement>>>>,
	pub render_count:    AtomicU64,
	/// Event queue for JS polling (thread-safe)
	pub event_queue:     Mutex<VecDeque<EventMessage>>,
}

impl WindowState {
	pub fn new() -> Self {
		Self {
			root_element_id: AtomicU64::new(0),
			element_map:     Mutex::new(HashMap::new()),
			element_tree:    Arc::new(Mutex::new(None)),
			render_count:    AtomicU64::new(0),
			event_queue:     Mutex::new(VecDeque::new()),
		}
	}

	/// Push an event to the queue
	pub fn push_event(&self, event: EventMessage) {
		if let Ok(mut queue) = self.event_queue.lock() {
			queue.push_back(event);
		}
	}

	/// Drain all events from the queue
	pub fn drain_events(&self) -> Vec<EventMessage> {
		if let Ok(mut queue) = self.event_queue.lock() {
			queue.drain(..).collect()
		} else {
			Vec::new()
		}
	}

	pub fn get_root_element_id(&self) -> u64 { self.root_element_id.load(Ordering::SeqCst) }

	pub fn set_root_element_id(&self, id: u64) { self.root_element_id.store(id, Ordering::SeqCst); }

	pub fn get_render_count(&self) -> u64 { self.render_count.load(Ordering::SeqCst) }

	pub fn increment_render_count(&self) -> u64 { self.render_count.fetch_add(1, Ordering::SeqCst) }

	pub fn rebuild_tree(&self, root_id: u64, children: &[u64]) {
		let element_map = self.element_map.lock().expect("Failed to acquire element_map lock");

		if let Some(_root) = element_map.get(&root_id) {
			let child_elements: Vec<Arc<ReactElement>> =
				children.iter().filter_map(|id| element_map.get(id).cloned()).collect();

			drop(element_map);

			let mut element_map =
				self.element_map.lock().expect("Failed to acquire element_map lock (second)");
			if let Some(root) = element_map.get_mut(&root_id) {
				let root_mut = Arc::make_mut(root);
				root_mut.children = child_elements;
				// Note: Don't reset style here - it was already parsed from JSON in
				// batch_update_elements
			}
		}
	}

	pub fn update_element_tree(&self) {
		let mut tree = self.element_tree.lock().expect("Failed to acquire element_tree lock");

		let root_id = self.get_root_element_id();
		if root_id == 0 {
			return;
		}

		let element_map = self.element_map.lock().expect("Failed to acquire element_map lock");

		if let Some(root) = element_map.get(&root_id) {
			let mut new_tree = (**root).clone();

			fn update_children(
				element: &mut ReactElement,
				element_map: &HashMap<u64, Arc<ReactElement>>,
			) {
				let children_ids: Vec<u64> =
					element.children.iter().filter_map(|c| Some(c.global_id)).collect();

				let mut new_children = Vec::new();
				for &cid in &children_ids {
					if let Some(child) = element_map.get(&cid) {
						let mut child_clone = (**child).clone();
						update_children(&mut child_clone, element_map);
						new_children.push(Arc::new(child_clone));
					}
				}

				if !new_children.is_empty() {
					element.children = new_children;
				}
			}

			update_children(&mut new_tree, &element_map);
			*tree = Some(Arc::new(new_tree));
		}
	}
}

impl Default for WindowState {
	fn default() -> Self { Self::new() }
}
