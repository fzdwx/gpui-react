use std::sync::{Arc, OnceLock, atomic::{AtomicBool, Ordering}};

use gpui::{App, AppContext, AsyncApp};
use tokio::sync::oneshot;

use crate::global_state::GLOBAL_STATE;

#[derive(Debug)]
pub enum HostCommand {
	CreateWindow { options: super::ffi_types::WindowOptions, response_tx: oneshot::Sender<u64> },
	TriggerRender { window_id: u64 },
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
	sender:   async_channel::Sender<Command>,
	shutdown: AtomicBool,
	ready:    AtomicBool,
}

impl Inner {
	fn is_shutting_down(&self) -> bool { self.shutdown.load(Ordering::SeqCst) }

	fn is_ready(&self) -> bool { self.ready.load(Ordering::SeqCst) }
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
	let inner =
		Arc::new(Inner { sender, shutdown: AtomicBool::new(false), ready: AtomicBool::new(false) });

	if BUS.set(inner.clone()).is_ok() {
		let inner_for_spawn = inner.clone();
		cx.spawn(async move |cx: &mut AsyncApp| {
			run_loop(inner_for_spawn, receiver, cx).await;
		})
		.detach();

		inner.ready.store(true, Ordering::SeqCst);
	}
}

async fn run_loop(
	inner: Arc<Inner>,
	receiver: async_channel::Receiver<Command>,
	cx: &mut AsyncApp,
) {
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

pub fn handle_on_app_thread(command: HostCommand, app: &mut App) {
	log::trace!("handle_on_app_thread: {:?}", command);

	match command {
		HostCommand::CreateWindow { options, response_tx } => {
			let title = options.title.as_deref().unwrap_or("React-GPUI");
			log::debug!("Creating window: {} ({}x{})", title, options.width, options.height);
			let window_options: gpui::WindowOptions = options.into();
			app
				.open_window(window_options, |window, cx| {
					let window_handle = window.window_handle();
					let window_id = window_handle.window_id().as_u64();
					let state = cx.new(|_| crate::renderer::RootState { render_count: 0 });
					log::debug!("Created window with id: {}", window_id);
					let _ = response_tx.send(window_id);
					GLOBAL_STATE.add_window(window_handle);
					cx.new(|_| crate::renderer::RootView { state, last_render: 0, window_id })
				})
				.unwrap();
		}
		HostCommand::TriggerRender { window_id } => {
			let Some(window) = GLOBAL_STATE.get_window(window_id) else {
				log::warn!("TriggerRender: window {} not found", window_id);
				return;
			};
			window.refresh(app);
		}
	}
}

pub fn sender() -> Result<CommandSender, CommandError> {
	BUS.get().map(|inner| CommandSender { inner: inner.clone() }).ok_or(CommandError::NotInitialized)
}

pub fn is_bus_ready() -> bool { BUS.get().map(|inner| inner.is_ready()).unwrap_or(false) }

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
