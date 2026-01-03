use gpui::{
    div, prelude::*, px, rgb, Application as GpuiApp, Bounds, Entity, Point, Render, Size, Window,
    WindowBounds, WindowOptions,
};

use crate::element_store::{ReactElement, ELEMENT_TREE, RENDER_TRIGGER};
use crate::GPUI_THREAD_STARTED;

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

        // Wrap in a container div to ensure consistent return type
        div().size(px(800.0)).bg(rgb(0x1e1e1e)).child(match &*tree {
            Some(element) => render_element_to_gpui(element),
            None => div()
                .child("Waiting for React...")
                .text_color(rgb(0x888888)),
        })
    }
}

/// Render a ReactElement to GPUI elements
fn render_element_to_gpui(element: &ReactElement) -> gpui::Div {
    match element.element_type.as_str() {
        "div" => {
            let children: Vec<gpui::Div> = element
                .children
                .iter()
                .map(|c| render_element_to_gpui(c))
                .collect();
            div()
                .flex()
                .justify_center()
                .items_center()
                .bg(rgb(0x2d2d2d))
                .children(children)
        }
        "text" => {
            let text = element.text.clone().unwrap_or_default();
            div().child(text).text_color(rgb(0xffffff))
        }
        _ => div().child(format!("[Unknown: {}]", element.element_type)),
    }
}

pub fn start_gpui_thread() {
    eprintln!("start_gpui_thread: spawning thread...");

    std::thread::spawn(|| {
        eprintln!("GPUI thread: starting...");
        GPUI_THREAD_STARTED.store(true, std::sync::atomic::Ordering::SeqCst);

        let app = GpuiApp::new();
        eprintln!("GPUI thread: app created");

        app.run(move |cx: &mut gpui::App| {
            eprintln!("GPUI thread: app.run() callback entered");

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
                    eprintln!("GPUI thread: creating RootView");
                    let state = cx.new(|_| RootState { render_count: 0 });
                    cx.new(|_| RootView {
                        state,
                        last_render: 0,
                    })
                },
            );

            eprintln!("GPUI thread: window opened successfully!");
        });

        eprintln!("GPUI thread: app.run() returned");
    });

    eprintln!("start_gpui_thread: thread spawned");
}
