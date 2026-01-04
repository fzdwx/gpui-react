use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, OnceLock};

use gpui::{App, AppContext, AsyncApp};

use crate::global_state::GLOBAL_STATE;

#[derive(Debug)]
pub enum HostCommand {
    CreateWindow { width: f32, height: f32, window_id: u64 },
    RefreshWindow,
    TriggerRender,
    UpdateElementTree,
}

pub enum Command {
    Host(HostCommand),
    Shutdown,
}

#[derive(Debug, thiserror::Error)]
pub enum CommandError {
    #[error("Command bus not initialized")]
    NotInitialized,
    #[error("Receiver channel closed")]
    ReceiverGone,
    #[error("System is shutting down")]
    ShuttingDown,
}

struct Inner {
    sender: async_channel::Sender<Command>,
    shutdown: AtomicBool,
    ready: AtomicBool,
}

impl Inner {
    fn is_shutting_down(&self) -> bool {
        self.shutdown.load(Ordering::SeqCst)
    }

    fn is_ready(&self) -> bool {
        self.ready.load(Ordering::SeqCst)
    }
}

#[derive(Clone)]
pub struct CommandSender {
    inner: Arc<Inner>,
}

impl CommandSender {
    fn send(&self, command: Command) -> Result<(), CommandError> {
        if self.inner.is_shutting_down() {
            return Err(CommandError::ShuttingDown);
        }
        self.inner.sender.send_blocking(command).map_err(|_| CommandError::ReceiverGone)
    }

    pub fn send_host(&self, command: HostCommand) -> Result<(), CommandError> {
        self.send(Command::Host(command))
    }
}

static BUS: OnceLock<Arc<Inner>> = OnceLock::new();

pub fn init(cx: &mut App) {
    if BUS.get().is_some() {
        return;
    }

    let (sender, receiver) = async_channel::unbounded();
    let inner = Arc::new(Inner {
        sender,
        shutdown: AtomicBool::new(false),
        ready: AtomicBool::new(false),
    });

    if BUS.set(inner.clone()).is_ok() {
        let inner_for_spawn = inner.clone();
        cx.spawn(async move |cx: &mut AsyncApp| {
            run_loop(inner_for_spawn, receiver, cx).await;
        })
        .detach();

        inner.ready.store(true, Ordering::SeqCst);
    }
}

async fn run_loop(inner: Arc<Inner>, receiver: async_channel::Receiver<Command>, cx: &mut AsyncApp) {
    while let Ok(command) = receiver.recv().await {
        if inner.is_shutting_down() {
            break;
        }

        let result = match command {
            Command::Host(cmd) => cx.update(|app| handle_on_app_thread(cmd, app)),
            Command::Shutdown => {
                inner.shutdown.store(true, Ordering::SeqCst);
                break;
            }
        };

        if let Err(err) = result {
            log::error!("host_command: failed to handle command: {err}");
        }
    }

    while receiver.try_recv().is_ok() {}
}

pub fn handle_on_app_thread(command: crate::host_command::HostCommand, app: &mut gpui::App) {
    log::trace!("handle_on_app_thread: {:?}", command);

    match command {
        crate::host_command::HostCommand::CreateWindow { width, height, window_id } => {
            let size = gpui::Size {
                width: gpui::px(width),
                height: gpui::px(height),
            };
            let origin = gpui::Point {
                x: gpui::px(100.0),
                y: gpui::px(100.0),
            };
            let bounds = gpui::Bounds { origin, size };

            let _window = app.open_window(
                gpui::WindowOptions {
                    window_bounds: Some(gpui::WindowBounds::Windowed(bounds)),
                    titlebar: Some(gpui::TitlebarOptions {
                        title: Some("R22222eact-GPUI".into()),
                        ..Default::default()
                    }),
                    ..Default::default()
                },
                |_window, cx| {
                    let state = cx.new(|_| crate::renderer::RootState { render_count: 0 });
                    cx.new(|_| crate::renderer::RootView {
                        state,
                        last_render: 0,
                        window_id,
                    })
                },
            );

            log::info!("Created window with id: {}", window_id);
        }
        HostCommand::RefreshWindow
        | HostCommand::TriggerRender
        | HostCommand::UpdateElementTree => {
            if let Some(window) = app.windows().first() {
                window.update(app, |_, window, _cx| {
                    log::trace!("Calling window.refresh()");
                    window.refresh();
                });
            } else {
                log::warn!("No windows found to refresh");
            }
        }
    }
}

pub fn sender() -> Result<CommandSender, CommandError> {
    BUS.get()
        .map(|inner| CommandSender { inner: inner.clone() })
        .ok_or(CommandError::NotInitialized)
}

pub fn is_bus_ready() -> bool {
    BUS.get().map(|inner| inner.is_ready()).unwrap_or(false)
}

pub fn send_host_command(command: HostCommand) {
    for _ in 0..100 {
        if let Ok(s) = sender() {
            let _ = s.send_host(command);
            return;
        }
        std::thread::sleep(std::time::Duration::from_millis(1));
    }
    if let Ok(s) = sender() {
        let _ = s.send_host(command);
    }
}