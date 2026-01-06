use std::{collections::HashMap, sync::{Arc, RwLock, atomic::{AtomicBool, Ordering}}};

use gpui::{AnyWindowHandle, Global, WindowHandle};
use crate::renderer::RootView;
use crate::window::Window;

pub struct GlobalState {
	gpui_initialized:    AtomicBool,
	gpui_thread_started: AtomicBool,
	windows:             RwLock<HashMap<u64, Arc<Window>>>,
}

impl Global for GlobalState {}

impl GlobalState {
	pub fn new() -> Self {
		Self {
			gpui_initialized:    AtomicBool::new(false),
			gpui_thread_started: AtomicBool::new(false),
			windows:             RwLock::new(HashMap::new()),
		}
	}

	pub fn is_initialized(&self) -> bool { self.gpui_initialized.load(Ordering::SeqCst) }

	pub fn set_initialized(&self, value: bool) {
		self.gpui_initialized.store(value, Ordering::SeqCst);
	}

	pub fn is_thread_started(&self) -> bool { self.gpui_thread_started.load(Ordering::SeqCst) }

	pub fn set_thread_started(&self, value: bool) {
		self.gpui_thread_started.store(value, Ordering::SeqCst);
	}

	/// Add a window with its GPUI handle
	pub fn add_window(&self, handle: WindowHandle<RootView>) {
		let window_id = handle.window_id().as_u64();
		let mut windows = self.windows.write().expect("Failed to acquire windows write lock");
		windows.insert(window_id, Arc::new(Window::new(handle)));
	}

	/// Get a window by ID, returns None if not found
	pub fn get_window(&self, window_id: u64) -> Option<Arc<Window>> {
		let windows = self.windows.read().expect("Failed to acquire windows read lock");
		windows.get(&window_id).cloned()
	}

	pub fn get_window_ref(&self, window_id: u64) -> Option<Arc<Window>> {
		let windows = self.windows.read().expect("Failed to acquire windows read lock");
		windows.get(&window_id).cloned()
	}

	pub fn remove_window(&self, window_id: u64) {
		let mut windows = self.windows.write().expect("Failed to acquire windows write lock");
		windows.remove(&window_id);
	}
}

impl Default for GlobalState {
	fn default() -> Self { Self::new() }
}

lazy_static::lazy_static! {
		pub static ref GLOBAL_STATE: GlobalState = GlobalState::new();
}
