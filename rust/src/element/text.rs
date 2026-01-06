use gpui::{div, prelude::*, px, rgb};

use super::{ElementStyle, ReactElement};

pub struct TextComponent;

impl TextComponent {
	pub fn from_element(
		element: &ReactElement,
		parent_style: Option<&ElementStyle>,
		_window_id: u64,
	) -> gpui::Stateful<gpui::Div> {
		let text = element.text.clone().unwrap_or_default();
		log::trace!("  rendering text: '{}'", text);

		let mut text_element = div().id(element.global_id as usize).child(text);

		let effective_text_color = element.style.text_color.or(parent_style.and_then(|s| s.text_color));
		let effective_text_size = element.style.text_size.or(parent_style.and_then(|s| s.text_size));

		if let Some(color) = effective_text_color {
			text_element = text_element.text_color(rgb(color));
		} else {
			text_element = text_element.text_color(rgb(0xffffff));
		}

		if let Some(size) = effective_text_size {
			text_element = text_element.text_size(px(size));
		}

		text_element
	}
}
