use gpui::{px, prelude::*, div};

use super::{ElementStyle, ReactElement};

pub struct ImgComponent;

impl ImgComponent {
    pub fn from_element(
        element: &ReactElement,
        _parent_style: Option<&ElementStyle>,
        _window_id: u64,
    ) -> gpui::Stateful<gpui::Div> {
        log::trace!("  rendering img");

        let div = div().id(element.global_id as usize);
        let mut img_element = if let Some(ref src) = element.style.src {
            div.child(format!("[Image: {}]", src))
        } else if let Some(ref alt) = element.style.alt {
            div.child(format!("[Image: {}]", alt))
        } else {
            div.child("[Image]")
        };

        if let Some(width) = element.style.width {
            img_element = img_element.w(px(width));
        }

        if let Some(height) = element.style.height {
            img_element = img_element.h(px(height));
        }

        img_element
    }
}