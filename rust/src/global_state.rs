use std::{
    collections::HashMap,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, RwLock,
    },
};

use gpui::Global;

use crate::window_state::WindowState;

pub struct GlobalState {
    gpui_initialized: AtomicBool,
    gpui_thread_started: AtomicBool,
    window_states: RwLock<HashMap<u64, Arc<WindowState>>>,
}

impl Global for GlobalState {}

impl GlobalState {
    pub fn new() -> Self {
        Self {
            gpui_initialized: AtomicBool::new(false),
            gpui_thread_started: AtomicBool::new(false),
            window_states: RwLock::new(HashMap::new()),
        }
    }

    pub fn is_initialized(&self) -> bool {
        self.gpui_initialized.load(Ordering::SeqCst)
    }

    pub fn set_initialized(
        &self,
        value: bool,
    ) {
        self.gpui_initialized.store(value, Ordering::SeqCst);
    }

    pub fn is_thread_started(&self) -> bool {
        self.gpui_thread_started.load(Ordering::SeqCst)
    }

    pub fn set_thread_started(
        &self,
        value: bool,
    ) {
        self.gpui_thread_started.store(value, Ordering::SeqCst);
    }

    pub fn get_window_state(
        &self,
        window_id: u64,
    ) -> Arc<WindowState> {
        let mut states = self
            .window_states
            .write()
            .expect("Failed to acquire window_states write lock");
        states
            .entry(window_id)
            .or_insert_with(|| Arc::new(WindowState::new()))
            .clone()
    }

    pub fn get_window_state_ref(
        &self,
        window_id: u64,
    ) -> Option<Arc<WindowState>> {
        let states = self
            .window_states
            .read()
            .expect("Failed to acquire window_states read lock");
        states.get(&window_id).cloned()
    }

    pub fn remove_window_state(
        &self,
        window_id: u64,
    ) {
        let mut states = self
            .window_states
            .write()
            .expect("Failed to acquire window_states write lock");
        states.remove(&window_id);
    }
}

impl Default for GlobalState {
    fn default() -> Self {
        Self::new()
    }
}

lazy_static::lazy_static! {
    pub static ref GLOBAL_STATE: GlobalState = GlobalState::new();
}
