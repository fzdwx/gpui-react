use std::sync::Arc;

use gpui::{
	div, prelude::*, px, rgb, AnyElement, App, Bounds, Element, ElementId, GlobalElementId,
	InspectorElementId, IntoElement, LayoutId, Pixels, Style, Window,
};

use super::{ElementStyle, ReactElement};

/// A specialized text element that renders text content
/// Uses GPUI's built-in text rendering for proper layout integration
pub struct ReactTextElement {
	element:      Arc<ReactElement>,
	parent_style: Option<ElementStyle>,
	text_child:   Option<AnyElement>,
}

pub struct TextLayoutState {
	child_layout_id: Option<LayoutId>,
}

pub struct TextPrepaintState;

impl ReactTextElement {
	pub fn new(element: Arc<ReactElement>, _window_id: u64, parent_style: Option<ElementStyle>) -> Self {
		Self { element, parent_style, text_child: None }
	}

	/// Get effective style with inheritance applied
	fn effective_style(&self) -> ElementStyle {
		let mut style = self.element.style.clone();
		if let Some(parent) = &self.parent_style {
			style.inherit_from(parent);
		}
		style
	}
}

impl Element for ReactTextElement {
	type RequestLayoutState = TextLayoutState;
	type PrepaintState = TextPrepaintState;

	fn id(&self) -> Option<ElementId> {
		Some(ElementId::Integer(self.element.global_id))
	}

	fn source_location(&self) -> Option<&'static std::panic::Location<'static>> {
		None
	}

	fn request_layout(
		&mut self,
		_id: Option<&GlobalElementId>,
		_inspector_id: Option<&InspectorElementId>,
		window: &mut Window,
		cx: &mut App,
	) -> (LayoutId, Self::RequestLayoutState) {
		let effective = self.effective_style();
		let text = self.element.text.clone().unwrap_or_default();

		// Build style for the container
		let mut style = Style::default();

		// Apply sizing if provided
		if let Some(width) = effective.width {
			style.size.width = gpui::Length::Definite(gpui::DefiniteLength::Absolute(
				gpui::AbsoluteLength::Pixels(px(width)),
			));
		}
		if let Some(height) = effective.height {
			style.size.height = gpui::Length::Definite(gpui::DefiniteLength::Absolute(
				gpui::AbsoluteLength::Pixels(px(height)),
			));
		}

		// Create child text element if we have text content
		let child_layout_id = if !text.is_empty() {
			let text_color = effective.text_color.unwrap_or(0xffffff);
			let text_size = effective.text_size.unwrap_or(14.0);

			let mut text_element = div()
				.text_color(rgb(text_color))
				.text_size(px(text_size))
				.child(text);

			// Apply font weight if specified
			if let Some(weight) = effective.font_weight {
				text_element = text_element.font_weight(gpui::FontWeight(weight as f32));
			}

			let mut child = text_element.into_any_element();
			let layout_id = child.request_layout(window, cx);
			self.text_child = Some(child);
			Some(layout_id)
		} else {
			None
		};

		// Request layout with child
		let layout_id = if let Some(child_id) = child_layout_id {
			window.request_layout(style, std::iter::once(child_id), cx)
		} else {
			window.request_layout(style, std::iter::empty(), cx)
		};

		(layout_id, TextLayoutState { child_layout_id })
	}

	fn prepaint(
		&mut self,
		_id: Option<&GlobalElementId>,
		_inspector_id: Option<&InspectorElementId>,
		_bounds: Bounds<Pixels>,
		_request_layout: &mut Self::RequestLayoutState,
		window: &mut Window,
		cx: &mut App,
	) -> Self::PrepaintState {
		// Prepaint child
		if let Some(ref mut child) = self.text_child {
			child.prepaint(window, cx);
		}
		TextPrepaintState
	}

	fn paint(
		&mut self,
		_id: Option<&GlobalElementId>,
		_inspector_id: Option<&InspectorElementId>,
		_bounds: Bounds<Pixels>,
		_request_layout: &mut Self::RequestLayoutState,
		_prepaint: &mut Self::PrepaintState,
		window: &mut Window,
		cx: &mut App,
	) {
		// Paint child text element
		if let Some(ref mut child) = self.text_child {
			child.paint(window, cx);
		}
	}
}

impl IntoElement for ReactTextElement {
	type Element = Self;

	fn into_element(self) -> Self::Element {
		self
	}
}
