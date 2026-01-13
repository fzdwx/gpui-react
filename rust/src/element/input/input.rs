use std::{panic::Location, sync::Arc};

use gpui::{App, Bounds, Element, ElementId, GlobalElementId, InspectorElementId, IntoElement, LayoutId, Pixels, RenderOnce, Window};

use crate::element::{ElementStyle, ReactElement};

pub struct ReactInputElement {
	element:      Arc<ReactElement>,
	window_id:    u64,
	#[allow(dead_code)]
	parent_style: Option<ElementStyle>,
}

impl Element for ReactInputElement {
	type PrepaintState = ();
	type RequestLayoutState = ();

	fn id(&self) -> Option<ElementId> { todo!() }

	fn source_location(&self) -> Option<&'static Location<'static>> { todo!() }

	fn request_layout(
		&mut self,
		id: Option<&GlobalElementId>,
		inspector_id: Option<&InspectorElementId>,
		window: &mut Window,
		cx: &mut App,
	) -> (LayoutId, Self::RequestLayoutState) {
		todo!()
	}

	fn prepaint(
		&mut self,
		id: Option<&GlobalElementId>,
		inspector_id: Option<&InspectorElementId>,
		bounds: Bounds<Pixels>,
		request_layout: &mut Self::RequestLayoutState,
		window: &mut Window,
		cx: &mut App,
	) -> Self::PrepaintState {
		todo!()
	}

	fn paint(
		&mut self,
		id: Option<&GlobalElementId>,
		inspector_id: Option<&InspectorElementId>,
		bounds: Bounds<Pixels>,
		request_layout: &mut Self::RequestLayoutState,
		prepaint: &mut Self::PrepaintState,
		window: &mut Window,
		cx: &mut App,
	) {
		todo!()
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


impl IntoElement for ReactInputElement {
	type Element = Self;

	fn into_element(self) -> Self::Element {
		self
	}
}
