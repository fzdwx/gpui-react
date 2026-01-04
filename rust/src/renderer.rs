use crate::element::ReactElement;
use crate::global_state::GLOBAL_STATE;
use crate::host_command;
use gpui::{
    div, prelude::*, px, rgb, Application as GpuiApp, Bounds, Entity, Point, Render, Size, Window,
    WindowBounds, WindowOptions,
};

pub struct RootState {
    pub render_count: u64,
}

pub struct RootView {
    pub state: Entity<RootState>,
    pub last_render: u64,
    pub window_id: u64,
}

impl RootView {
    fn update_state(&mut self, cx: &mut gpui::Context<Self>) {
        let window_state = GLOBAL_STATE.get_window_state(self.window_id);
        let trigger = window_state.get_render_count();
        log::trace!(
            "update_state: trigger={}, last_render={}",
            trigger,
            self.last_render
        );

        if trigger != self.last_render {
            log::debug!(
                "update_state: trigger changed from {} to {}",
                self.last_render,
                trigger
            );
            self.last_render = trigger;
            self.state.update(cx, |state, _cx| {
                state.render_count = trigger;
            });
        }
    }
}

impl Render for RootView {
    fn render(
        &mut self,
        _window: &mut Window,
        cx: &mut gpui::Context<Self>,
    ) -> impl gpui::IntoElement {
        let render_start = std::time::Instant::now();
        self.update_state(cx);

        let window_state = GLOBAL_STATE.get_window_state(self.window_id);
        let tree = window_state
            .element_tree
            .lock()
            .expect("Failed to acquire element_tree lock in RootView.render");
        log::debug!(
            "RootView.render: tree={:?}",
            tree.as_ref().map(|e| (e.global_id, &e.element_type))
        );

        let result = div().size(px(800.0)).bg(rgb(0x1e1e1e)).child(match &*tree {
            Some(element) => render_element_to_gpui(&element),
            None => div()
                .child("Waiting for React...")
                .text_color(rgb(0x888888)),
        });

        let render_duration = render_start.elapsed();
        log::debug!("RootView.render completed in {:?}", render_duration);

        result
    }
}

fn render_element_to_gpui(element: &ReactElement) -> gpui::Div {
    log::debug!(
        "render_element_to_gpui: type={}, text={:?}, style={:?}",
        element.element_type,
        element.text,
        element.style
    );

    match element.element_type.as_str() {
        "div" => {
            let children: Vec<gpui::Div> = element
                .children
                .iter()
                .map(|c| render_element_to_gpui(c))
                .collect();
            log::trace!("  div has {} children", children.len());

            let is_flex = element.style.display.as_ref().map(|s| s.as_str()) == Some("flex");

            let mut div = if is_flex { div().flex() } else { div() };

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

            div.children(children)
        }
        "text" => {
            let text = element.text.clone().unwrap_or_default();
            log::trace!("  rendering text: '{}'", text);

            let mut text_element = div().child(text);

            if let Some(color) = element.style.text_color {
                text_element = text_element.text_color(rgb(color));
            } else {
                text_element = text_element.text_color(rgb(0xffffff));
            }

            if let Some(size) = element.style.text_size {
                text_element = text_element.text_size(px(size));
            }

            text_element
        }
        "span" => {
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

            let mut span_element = div().child(text);

            if let Some(color) = element.style.text_color {
                span_element = span_element.text_color(rgb(color));
            } else {
                span_element = span_element.text_color(rgb(0xffffff));
            }

            if let Some(size) = element.style.text_size {
                span_element = span_element.text_size(px(size));
            }

            span_element
        }
        "img" => {
            log::trace!("  rendering img");

            let mut img_element = if let Some(ref src) = element.style.src {
                div().child(format!("[Image: {}]", src))
            } else if let Some(ref alt) = element.style.alt {
                div().child(format!("[Image: {}]", alt))
            } else {
                div().child("[Image]")
            };

            if let Some(width) = element.style.width {
                img_element = img_element.w(px(width));
            }

            if let Some(height) = element.style.height {
                img_element = img_element.h(px(height));
            }

            img_element
        }
        _ => div().child(format!("[Unknown: {}]", element.element_type)),
    }
}

pub fn start_gpui_thread(width: f32, height: f32) {
    log::info!("start_gpui_thread: spawning thread...");

    std::thread::spawn(move || {
        log::info!("GPUI thread: starting...");
        GLOBAL_STATE.set_thread_started(true);

        let app = GpuiApp::new();
        log::debug!("GPUI thread: app created");

        app.run(move |cx: &mut gpui::App| {
            log::debug!("GPUI thread: app.run() callback entered");
            host_command::init(cx);

            log::info!("GPUI thread: initialized, window creation via gpui_create_window");
        });

        log::debug!("GPUI thread: app.run() returned");
    });

    log::info!("start_gpui_thread: thread spawned");
}

use std::sync::atomic::{AtomicU64, Ordering};

pub static NEXT_WINDOW_ID: AtomicU64 = AtomicU64::new(1);
