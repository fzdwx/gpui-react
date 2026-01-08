use std::sync::Arc;

use gpui::{AnyElement, App, Bounds, DispatchPhase, Element, ElementId, GlobalElementId, Hitbox, InspectorElementId, IntoElement, LayoutId, MouseButton, MouseUpEvent, Pixels, Window, div, prelude::*, px, rgb};

use super::{ElementStyle, ReactElement};
use crate::event_types::{props, types};
use crate::renderer::{EventData, dispatch_event_to_js};

/// A React element that implements GPUI's Element trait directly
pub struct ReactDivElement {
	element:      Arc<ReactElement>,
	window_id:    u64,
	parent_style: Option<ElementStyle>,
	children:     Vec<AnyElement>,
}

/// State returned from request_layout, containing child layout IDs
pub struct DivLayoutState {
	child_layout_ids: Vec<LayoutId>,
}

/// State returned from prepaint
pub struct DivPrepaintState {
	hitbox: Option<Hitbox>,
}

impl ReactDivElement {
	pub fn new(
		element: Arc<ReactElement>,
		window_id: u64,
		parent_style: Option<ElementStyle>,
	) -> Self {
		Self { element, window_id, parent_style, children: Vec::new() }
	}

	fn has_click_handler(&self) -> bool {
		self.element.event_handlers.as_ref().and_then(|v| v.get(props::ON_CLICK)).is_some()
	}
}

impl Element for ReactDivElement {
	type RequestLayoutState = DivLayoutState;
	type PrepaintState = DivPrepaintState;

	fn id(&self) -> Option<ElementId> { Some(ElementId::Integer(self.element.global_id)) }

	fn source_location(&self) -> Option<&'static std::panic::Location<'static>> { None }

	fn request_layout(
		&mut self,
		_id: Option<&GlobalElementId>,
		_inspector_id: Option<&InspectorElementId>,
		window: &mut Window,
		cx: &mut App,
	) -> (LayoutId, Self::RequestLayoutState) {
		let style = self.element.build_gpui_style(None);
		let inherited_style = self.element.effective_style(self.parent_style.as_ref());

		// Build child elements with inherited style
		self.children = self
			.element
			.children
			.iter()
			.map(|child| {
				super::create_element(child.clone(), self.window_id, Some(inherited_style.clone()))
					.into_any_element()
			})
			.collect();

		// If element has text content, add it as a child using GPUI's text element
		if let Some(ref text) = self.element.text {
			if !text.is_empty() {
				// Use inherited text styles
				let text_color = inherited_style.text_color.unwrap_or(0xffffff);
				let text_size = inherited_style.text_size.unwrap_or(14.0);

				let text_element =
					div().text_color(rgb(text_color)).text_size(px(text_size)).child(text.clone());
				self.children.push(text_element.into_any_element());
			}
		}

		// Request layout for children
		let child_layout_ids: Vec<LayoutId> =
			self.children.iter_mut().map(|child| child.request_layout(window, cx)).collect();

		// Request our own layout
		let layout_id = window.request_layout(style, child_layout_ids.iter().copied(), cx);

		(layout_id, DivLayoutState { child_layout_ids })
	}

	fn prepaint(
		&mut self,
		_id: Option<&GlobalElementId>,
		_inspector_id: Option<&InspectorElementId>,
		bounds: Bounds<Pixels>,
		request_layout: &mut Self::RequestLayoutState,
		window: &mut Window,
		cx: &mut App,
	) -> Self::PrepaintState {
		// Prepaint children
		for child in &mut self.children {
			child.prepaint(window, cx);
		}

		// Insert hitbox if we have event handlers
		let hitbox = if self.has_click_handler() {
			Some(window.insert_hitbox(bounds, gpui::HitboxBehavior::Normal))
		} else {
			None
		};

		DivPrepaintState { hitbox }
	}

	fn paint(
		&mut self,
		_id: Option<&GlobalElementId>,
		_inspector_id: Option<&InspectorElementId>,
		bounds: Bounds<Pixels>,
		_request_layout: &mut Self::RequestLayoutState,
		prepaint: &mut Self::PrepaintState,
		window: &mut Window,
		cx: &mut App,
	) {
		let style = self.element.build_gpui_style(None);

		// Paint background and children
		style.paint(bounds, window, cx, |window, cx| {
			// Use shared helper for overflow clipping
			super::paint_children_with_clip(
				&mut self.children,
				bounds,
				self.element.style.should_clip(),
				window,
				cx,
				|child, window, cx| child.paint(window, cx),
			);
		});

		// Register click handler
		if let Some(hitbox) = prepaint.hitbox.as_ref() {
			let element_id = self.element.global_id;
			let window_id = self.window_id;
			let hitbox = hitbox.clone();

			window.on_mouse_event(move |event: &MouseUpEvent, phase, window, _cx| {
				if phase == DispatchPhase::Bubble
					&& event.button == MouseButton::Left
					&& hitbox.is_hovered(window)
				{
					// Extract mouse position (convert Pixels to f32)
					let position = event.position;
					let client_x: f32 = position.x.into();
					let client_y: f32 = position.y.into();

					let event_data = EventData {
						client_x: Some(client_x),
						client_y: Some(client_y),
						button:   Some(match event.button {
							MouseButton::Left => 0,
							MouseButton::Right => 2,
							MouseButton::Middle => 1,
							MouseButton::Navigate(_) => 3,
						}),
					};

					log::info!(
						"[Rust] onClick triggered: window_id={}, element_id={}, position=({}, {})",
						window_id,
						element_id,
						client_x,
						client_y
					);
					dispatch_event_to_js(window_id, element_id, types::CLICK, Some(event_data));
				}
			});
		}
	}
}

impl IntoElement for ReactDivElement {
	type Element = Self;

	fn into_element(self) -> Self::Element { self }
}
