use gpui::{div, prelude::*, px, rgb};

use super::{ElementStyle, ReactElement};
use crate::renderer::dispatch_event_to_js;

pub struct DivComponent;

impl DivComponent {
	pub fn from_element(
		element: &ReactElement,
		parent_style: Option<&ElementStyle>,
		window_id: u64,
	) -> gpui::Stateful<gpui::Div> {
		log::debug!("DivComponent::from_element: global_id={}", element.global_id);

		let children: Vec<gpui::Stateful<gpui::Div>> = element
			.children
			.iter()
			.map(|c| super::render_to_gpui(c, Some(&element.style), window_id))
			.collect();

		log::trace!("  div has {} children", children.len());

		let is_flex = element.style.display.as_ref().map(|s| s.as_str()) == Some("flex");

		let mut div = div().id(element.global_id as usize);
		div = div.when(is_flex, |div| div.flex());

		div = match element.style.flex_direction.as_ref().map(|s| s.as_str()) {
			Some("row") => div.flex_row(),
			Some("column") => div.flex_col(),
			_ => div,
		};

		div = match element.style.justify_content.as_ref().map(|s| s.as_str()) {
			Some("flex-start") => div.justify_start(),
			Some("center") => div.justify_center(),
			Some("flex-end") => div.justify_end(),
			Some("space-between") => div.justify_between(),
			Some("space-around") => div.justify_around(),
			_ => div,
		};

		div = match element.style.align_items.as_ref().map(|s| s.as_str()) {
			Some("flex-start") => div.items_start(),
			Some("center") => div.items_center(),
			Some("flex-end") => div.items_end(),
			_ => div,
		};

		if let Some(bg) = element.style.bg_color {
			div = div.bg(rgb(bg));
		} else {
			div = div.bg(rgb(0x2d2d2d));
		}

		if let Some(width) = element.style.width {
			div = div.w(px(width));
		}

		if let Some(height) = element.style.height {
			div = div.h(px(height));
		}

		if let (Some(pt), Some(pr), Some(pb), Some(pl)) = (
			element.style.padding_top,
			element.style.padding_right,
			element.style.padding_bottom,
			element.style.padding_left,
		) {
			div = div.pt(px(pt)).pr(px(pr)).pb(px(pb)).pl(px(pl));
		}

		if let (Some(mt), Some(mr), Some(mb), Some(ml)) = (
			element.style.margin_top,
			element.style.margin_right,
			element.style.margin_bottom,
			element.style.margin_left,
		) {
			div = div.mt(px(mt)).mr(px(mr)).mb(px(mb)).ml(px(ml));
		}

		if let Some(border_radius) = element.style.border_radius {
			div = div.rounded(px(border_radius));
		}

		if let Some(gap) = element.style.gap {
			div = div.gap(px(gap));
		}

		if let Some(opacity) = element.style.opacity {
			div = div.opacity(opacity as f32);
		}

		if element.event_handlers.as_ref().and_then(|v| v.get("onClick")).is_some() {
			let element_id = element.global_id;
			div = div.on_click(move |_event, _window, _cx| {
				log::info!("[Rust] onClick triggered: window_id={}, element_id={}", window_id, element_id);
				// Test: call dispatch_event_to_js only on actual click, not during setup
				crate::renderer::dispatch_event_to_js(window_id, element_id, "onClick", None);
			});
		}

		div.children(children)
	}
}
