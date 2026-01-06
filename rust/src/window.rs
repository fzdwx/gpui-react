use std::{collections::HashMap, sync::{Arc, Mutex, atomic::{AtomicU64, Ordering}}};

use gpui::{AnyWindowHandle, App, AppContext, WindowHandle};

use crate::{element::ReactElement, renderer::RootView};

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
}


pub struct WindowState {
	pub root_element_id: AtomicU64,
	pub element_map:     Mutex<HashMap<u64, Arc<ReactElement>>>,
	pub element_tree:    Arc<Mutex<Option<Arc<ReactElement>>>>,
	pub render_count:    AtomicU64,
}

impl WindowState {
	pub fn new() -> Self {
		Self {
			root_element_id: AtomicU64::new(0),
			element_map:     Mutex::new(HashMap::new()),
			element_tree:    Arc::new(Mutex::new(None)),
			render_count:    AtomicU64::new(0),
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
				root_mut.style = crate::element::ElementStyle::default();
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
				element_map: &std::collections::HashMap<u64, Arc<ReactElement>>,
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
