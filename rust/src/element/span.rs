use std::sync::Arc;

use gpui::{
	div, prelude::*, px, rgb, AnyElement, App, Bounds, ContentMask, Element, ElementId,
	GlobalElementId, InspectorElementId, IntoElement, LayoutId, Pixels, Style, Window,
};

use super::{ElementStyle, ReactElement};

/// A span element - similar to div but:
/// - No default background (transparent by default)
/// - Conceptually for inline/text content grouping
/// - Supports children and text
pub struct ReactSpanElement {
	element:      Arc<ReactElement>,
	window_id:    u64,
	parent_style: Option<ElementStyle>,
	children:     Vec<AnyElement>,
}

pub struct SpanLayoutState {
	child_layout_ids: Vec<LayoutId>,
}

pub struct SpanPrepaintState;

impl ReactSpanElement {
	pub fn new(element: Arc<ReactElement>, window_id: u64, parent_style: Option<ElementStyle>) -> Self {
		Self { element, window_id, parent_style, children: Vec::new() }
	}

	/// Get effective style with inheritance applied
	fn effective_style(&self) -> ElementStyle {
		let mut style = self.element.style.clone();
		if let Some(parent) = &self.parent_style {
			style.inherit_from(parent);
		}
		style
	}

	/// Convert ElementStyle to GPUI Style using shared builder - no default background
	fn build_style(&self) -> Style {
		// span uses no default background (transparent)
		self.element.style.build_gpui_style(None)
	}
}

impl Element for ReactSpanElement {
	type RequestLayoutState = SpanLayoutState;
	type PrepaintState = SpanPrepaintState;

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
		let style = self.build_style();
		let inherited_style = self.effective_style();

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

		// If element has text content, add it as a child
		if let Some(ref text) = self.element.text {
			if !text.is_empty() {
				let text_color = inherited_style.text_color.unwrap_or(0xffffff);
				let text_size = inherited_style.text_size.unwrap_or(14.0);

				let text_element = div()
					.text_color(rgb(text_color))
					.text_size(px(text_size))
					.child(text.clone());
				self.children.push(text_element.into_any_element());
			}
		}

		// Request layout for children
		let child_layout_ids: Vec<LayoutId> = self
			.children
			.iter_mut()
			.map(|child| child.request_layout(window, cx))
			.collect();

		let layout_id = window.request_layout(style, child_layout_ids.iter().copied(), cx);
		(layout_id, SpanLayoutState { child_layout_ids })
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
		for child in &mut self.children {
			child.prepaint(window, cx);
		}
		SpanPrepaintState
	}

	fn paint(
		&mut self,
		_id: Option<&GlobalElementId>,
		_inspector_id: Option<&InspectorElementId>,
		bounds: Bounds<Pixels>,
		_request_layout: &mut Self::RequestLayoutState,
		_prepaint: &mut Self::PrepaintState,
		window: &mut Window,
		cx: &mut App,
	) {
		let style = self.build_style();

		// Paint background and children
		style.paint(bounds, window, cx, |window, cx| {
			// Apply content mask for overflow clipping
			if self.element.style.should_clip() {
				let mask = ContentMask { bounds };
				window.with_content_mask(Some(mask), |window| {
					for child in &mut self.children {
						child.paint(window, cx);
					}
				});
			} else {
				for child in &mut self.children {
					child.paint(window, cx);
				}
			}
		});
	}
}

impl IntoElement for ReactSpanElement {
	type Element = Self;

	fn into_element(self) -> Self::Element {
		self
	}
}
