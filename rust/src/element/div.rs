use std::sync::Arc;

use gpui::{
	AnyElement, App, Bounds, Element, ElementId, GlobalElementId, Hitbox, InspectorElementId,
	IntoElement, LayoutId, Pixels, Window, div, prelude::*, px, rgb,
};

use super::events::{EventHandlerFlags, insert_hitbox_if_needed, register_event_handlers};
use super::{ElementStyle, ReactElement};

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
	event_flags: EventHandlerFlags,
}

impl ReactDivElement {
	pub fn new(
		element: Arc<ReactElement>,
		window_id: u64,
		parent_style: Option<ElementStyle>,
	) -> Self {
		Self { element, window_id, parent_style, children: Vec::new() }
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
		_request_layout: &mut Self::RequestLayoutState,
		window: &mut Window,
		cx: &mut App,
	) -> Self::PrepaintState {
		// Prepaint children
		for child in &mut self.children {
			child.prepaint(window, cx);
		}

		// Check event handlers and insert hitbox if needed
		let event_flags = EventHandlerFlags::from_handlers(
			self.element.event_handlers.as_ref(),
			self.element.style.tab_index,
		);
		let hitbox = insert_hitbox_if_needed(&event_flags, bounds, window);

		DivPrepaintState { hitbox, event_flags }
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

		// Register event handlers using shared module
		register_event_handlers(
			&prepaint.event_flags,
			prepaint.hitbox.as_ref(),
			self.window_id,
			self.element.global_id,
			window,
		);
	}
}

impl IntoElement for ReactDivElement {
	type Element = Self;

	fn into_element(self) -> Self::Element { self }
}
