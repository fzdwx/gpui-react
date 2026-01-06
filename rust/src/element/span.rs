use gpui::{px, rgb, prelude::*, div};

use super::{ElementStyle, ReactElement};

pub struct SpanComponent;

impl SpanComponent {
    pub fn from_element(
        element: &ReactElement,
        parent_style: Option<&ElementStyle>,
        _window_id: u64,
    ) -> gpui::Stateful<gpui::Div> {
        let text = if let Some(ref t) = element.text {
            t.clone()
        } else {
            element
                .children
                .iter()
                .filter(|c| c.element_type == "text")
                .filter_map(|c| c.text.as_ref())
                .cloned()
                .collect::<Vec<_>>()
                .join("")
        };

        log::trace!("  rendering span (inline text): '{}'", text);

        let mut span_element = div().id(element.global_id as usize).child(text);

        let effective_text_color = element.style.text_color.or(parent_style.and_then(|s| s.text_color));
        let effective_text_size = element.style.text_size.or(parent_style.and_then(|s| s.text_size));

        if let Some(color) = effective_text_color {
            span_element = span_element.text_color(rgb(color));
        } else {
            span_element = span_element.text_color(rgb(0xffffff));
        }

        if let Some(size) = effective_text_size {
            span_element = span_element.text_size(px(size));
        }

        span_element
    }
}