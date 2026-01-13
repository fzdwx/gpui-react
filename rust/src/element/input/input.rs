use std::{panic::Location, sync::Arc};

use gpui::{div, App, Bounds, Context, Element, ElementId, GlobalElementId, InspectorElementId, IntoElement, LayoutId, Pixels, RenderOnce, Window};

use crate::element::{ElementStyle, ReactElement};
use crate::renderer::RootView;

#[derive(IntoElement)]
pub struct ReactInputElement {
	element:      Arc<ReactElement>,
	window_id:    u64,
	#[allow(dead_code)]
	parent_style: Option<ElementStyle>,
}

impl RenderOnce for ReactInputElement {
	fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
		div()
	}
}

impl ReactInputElement {
	pub fn new(
		element: Arc<ReactElement>,
		window_id: u64,
		parent_style: Option<ElementStyle>,
	) -> Self {
		Self { element, window_id, parent_style }
	}
}


