use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, OnceLock};

use gpui::{App, AsyncApp};

use crate::global_state::GLOBAL_STATE;

#[derive(Debug)]
pub enum HostCommand {
    CreateWindow { width: f32, height: f32 },
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
}

impl Inner {
    fn is_shutting_down(&self) -> bool {
        self.shutdown.load(Ordering::SeqCst)
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
    });

    if BUS.set(inner.clone()).is_ok() {
        cx.spawn(async move |cx: &mut AsyncApp| {
            run_loop(inner, receiver, cx).await;
        })
        .detach();
    }
}

async fn run_loop(inner: Arc<Inner>, receiver: async_channel::Receiver<Command>, cx: &mut AsyncApp) {
    while let Ok(command) = receiver.recv().await {
        if inner.is_shutting_down() {
            break;
        }

        let result = match command {
            Command::Host(cmd) => cx.update(|app| crate::renderer::handle_on_app_thread(cmd, app)),
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

pub fn sender() -> Result<CommandSender, CommandError> {
    BUS.get()
        .map(|inner| CommandSender { inner: inner.clone() })
        .ok_or(CommandError::NotInitialized)
}

pub fn send_host_command(command: HostCommand) {
    // Retry a few times if bus not ready yet
    for _ in 0..100 {
        if let Ok(s) = sender() {
            let _ = s.send_host(command);
            return;
        }
        std::thread::sleep(std::time::Duration::from_millis(1));
    }
    // Last try without retry
    if let Ok(s) = sender() {
        let _ = s.send_host(command);
    }
}

pub fn get_current_window() -> u64 {
    GLOBAL_STATE.get_current_window()
}