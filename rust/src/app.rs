use gpui::{
    div, prelude::*, px, rgb, Application as GpuiApp, Bounds, Empty, Entity, Point, Render, Size,
    Window, WindowBounds, WindowOptions,
};

use crate::element_store::{ReactElement, ELEMENT_TREE, RENDER_TRIGGER};

#[derive(Clone)]
struct RootState {
    pub render_count: u64,
}

struct RootView {
    state: Entity<RootState>,
    pub last_render: u64,
}

impl RootView {
    fn update_state(&mut self, cx: &mut gpui::Context<Self>) {
        let trigger = RENDER_TRIGGER.load(std::sync::atomic::Ordering::SeqCst);
        if trigger != self.last_render {
            self.last_render = trigger;
            self.state.update(cx, |state, _| {
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
        self.update_state(cx);

        let tree = ELEMENT_TREE.lock().unwrap();

        div()
            .flex()
            .justify_center()
            .items_center()
            .size(px(800.0))
            .bg(rgb(0x1e1e1e))
            .child(match &*tree {
                Some(element) => render_element_to_string(element),
                None => "Waiting for React...".to_string(),
            })
    }
}

/// Render a ReactElement to a string representation
fn render_element_to_string(element: &ReactElement) -> String {
    match element.element_type.as_str() {
        "div" => {
            let children_str: String = element
                .children
                .iter()
                .map(|c| render_element_to_string(c))
                .collect();
            format!("[div: {}]", children_str)
        }
        "text" => element.text.clone().unwrap_or_default(),
        _ => format!("[Unknown: {}]", element.element_type),
    }
}

pub fn start_gpui_thread() {
    std::thread::spawn(|| {
        let app = GpuiApp::new();

        app.run(move |cx: &mut gpui::App| {
            let size = Size {
                width: px(800.0),
                height: px(600.0),
            };
            let origin = Point {
                x: px(100.0),
                y: px(100.0),
            };
            let bounds = Bounds { origin, size };
            let _window = cx.open_window(
                WindowOptions {
                    window_bounds: Some(WindowBounds::Windowed(bounds)),
                    titlebar: Some(gpui::TitlebarOptions {
                        title: Some("React-GPUI".into()),
                        ..Default::default()
                    }),
                    ..Default::default()
                },
                |_window, cx| {
                    let state = cx.new(|_| RootState { render_count: 0 });
                    cx.new(|_| RootView {
                        state,
                        last_render: 0,
                    })
                },
            );
        });
    });
}
