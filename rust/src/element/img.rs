use std::sync::Arc;

use gpui::{
	AnyElement, App, Bounds, Element, ElementId, GlobalElementId, Hitbox, InspectorElementId,
	IntoElement, LayoutId, Pixels, Style, Window, div, prelude::*, px, rgb,
};

use super::events::{EventHandlerFlags, insert_hitbox_if_needed, register_event_handlers};
use super::{ElementStyle, ReactElement};

/// An image element
/// - Displays images from src URL/path
/// - Falls back to alt text or placeholder
/// - Supports width/height sizing
pub struct ReactImgElement {
	element:           Arc<ReactElement>,
	window_id:         u64,
	parent_style:      Option<ElementStyle>,
	placeholder_child: Option<AnyElement>,
}

pub struct ImgLayoutState {
	child_layout_id: Option<LayoutId>,
}

pub struct ImgPrepaintState {
	hitbox: Option<Hitbox>,
	event_flags: EventHandlerFlags,
}

impl ReactImgElement {
	pub fn new(
		element: Arc<ReactElement>,
		window_id: u64,
		parent_style: Option<ElementStyle>,
	) -> Self {
		Self { element, window_id, parent_style, placeholder_child: None }
	}

	fn build_style(&self) -> Style {
		let es = &self.element.style;
		let mut style = Style::default();

		// Apply size
		if let Some(width) = es.width {
			style.size.width = gpui::Length::Definite(gpui::DefiniteLength::Absolute(
				gpui::AbsoluteLength::Pixels(px(width)),
			));
		}
		if let Some(height) = es.height {
			style.size.height = gpui::Length::Definite(gpui::DefiniteLength::Absolute(
				gpui::AbsoluteLength::Pixels(px(height)),
			));
		}

		// Apply padding if specified
		if let Some(pt) = es.padding_top {
			style.padding.top = gpui::DefiniteLength::Absolute(gpui::AbsoluteLength::Pixels(px(pt)));
		}
		if let Some(pr) = es.padding_right {
			style.padding.right = gpui::DefiniteLength::Absolute(gpui::AbsoluteLength::Pixels(px(pr)));
		}
		if let Some(pb) = es.padding_bottom {
			style.padding.bottom = gpui::DefiniteLength::Absolute(gpui::AbsoluteLength::Pixels(px(pb)));
		}
		if let Some(pl) = es.padding_left {
			style.padding.left = gpui::DefiniteLength::Absolute(gpui::AbsoluteLength::Pixels(px(pl)));
		}

		// Border radius for rounded images
		if let Some(radius) = es.border_radius {
			let r = gpui::AbsoluteLength::Pixels(px(radius));
			style.corner_radii.top_left = r;
			style.corner_radii.top_right = r;
			style.corner_radii.bottom_left = r;
			style.corner_radii.bottom_right = r;
		}

		// Background color (placeholder background)
		if let Some(bg) = es.bg_color {
			style.background = Some(gpui::Fill::Color(rgb(bg).into()));
		} else {
			// Default placeholder background
			style.background = Some(gpui::Fill::Color(rgb(0x444444).into()));
		}

		// Opacity
		if let Some(opacity) = es.opacity {
			style.opacity = Some(opacity);
		}

		// Center content
		style.display = gpui::Display::Flex;
		style.justify_content = Some(gpui::JustifyContent::Center);
		style.align_items = Some(gpui::AlignItems::Center);

		style
	}
}

impl Element for ReactImgElement {
	type PrepaintState = ImgPrepaintState;
	type RequestLayoutState = ImgLayoutState;

	fn id(&self) -> Option<ElementId> { Some(ElementId::Integer(self.element.global_id)) }

	fn source_location(&self) -> Option<&'static std::panic::Location<'static>> { None }

	fn request_layout(
		&mut self,
		_id: Option<&GlobalElementId>,
		_inspector_id: Option<&InspectorElementId>,
		window: &mut Window,
		cx: &mut App,
	) -> (LayoutId, Self::RequestLayoutState) {
		let es = &self.element.style;
		let effective = self.element.effective_style(self.parent_style.as_ref());
		let style = self.build_style();

		// Create placeholder text
		let placeholder_text = if let Some(ref src) = es.src {
			format!("[Image: {}]", src)
		} else if let Some(ref alt) = es.alt {
			format!("[{}]", alt)
		} else {
			"[Image]".to_string()
		};

		// Create placeholder child element
		let text_color = effective.text_color.unwrap_or(0x888888);
		let text_size = effective.text_size.unwrap_or(12.0);

		let placeholder =
			div().text_color(rgb(text_color)).text_size(px(text_size)).child(placeholder_text);

		let mut child = placeholder.into_any_element();
		let child_layout_id = child.request_layout(window, cx);
		self.placeholder_child = Some(child);

		let layout_id = window.request_layout(style, std::iter::once(child_layout_id), cx);
		(layout_id, ImgLayoutState { child_layout_id: Some(child_layout_id) })
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
		if let Some(ref mut child) = self.placeholder_child {
			child.prepaint(window, cx);
		}

		// Check event handlers and insert hitbox if needed
		let event_flags = EventHandlerFlags::from_handlers(
			self.element.event_handlers.as_ref(),
			self.element.style.tab_index,
		);
		let hitbox = insert_hitbox_if_needed(&event_flags, bounds, window);

		ImgPrepaintState { hitbox, event_flags }
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
		let style = self.build_style();

		// Paint background and child
		style.paint(bounds, window, cx, |window, cx| {
			if let Some(ref mut child) = self.placeholder_child {
				child.paint(window, cx);
			}
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

impl IntoElement for ReactImgElement {
	type Element = Self;

	fn into_element(self) -> Self::Element { self }
}
