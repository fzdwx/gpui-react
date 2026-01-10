//! Hover state tracking for mouseenter/mouseleave events
//!
//! This module tracks which elements are currently hovered to detect
//! state transitions for triggering mouseenter and mouseleave events.

use std::{collections::HashSet, sync::{Arc, Mutex}};

use lazy_static::lazy_static;

/// Tracks the current hover state of elements
pub struct HoverState {
	/// Set of element IDs that are currently hovered
	hovered_elements: HashSet<u64>,
}

impl HoverState {
	pub fn new() -> Self { Self { hovered_elements: HashSet::new() } }

	/// Check if an element is currently hovered
	pub fn is_hovered(&self, element_id: u64) -> bool { self.hovered_elements.contains(&element_id) }

	/// Mark an element as hovered. Returns true if this is a new hover (enter).
	pub fn set_hovered(&mut self, element_id: u64) -> bool {
		self.hovered_elements.insert(element_id)
	}

	/// Mark an element as not hovered. Returns true if it was previously hovered
	/// (leave).
	pub fn set_not_hovered(&mut self, element_id: u64) -> bool {
		self.hovered_elements.remove(&element_id)
	}

	/// Clear all hover states (called on window change or cleanup)
	pub fn clear(&mut self) { self.hovered_elements.clear(); }
}

impl Default for HoverState {
	fn default() -> Self { Self::new() }
}

lazy_static! {
		/// Global hover state manager
		/// Each window could have its own, but for simplicity we use a global one
		static ref HOVER_STATE: Arc<Mutex<HoverState>> = Arc::new(Mutex::new(HoverState::new()));
}

/// Get a reference to the global hover state
pub fn get_hover_state() -> &'static Arc<Mutex<HoverState>> { &HOVER_STATE }

/// Clear all hover states (call when window closes or during cleanup)
pub fn clear_hover_state() {
	if let Ok(mut state) = HOVER_STATE.lock() {
		state.clear();
	}
}
