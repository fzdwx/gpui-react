//! Focus state management for keyboard events and focus/blur events
//!
//! This module provides a simple focus management system that tracks which
//! element is currently focused. This allows keyboard events to be properly
//! dispatched to focusable elements.
//!
//! Note: This is a simplified implementation. GPUI has a more sophisticated
//! focus system with FocusHandle, but integrating it with custom Element
//! implementations requires a different approach.

use std::{collections::HashMap, sync::{Arc, Mutex}};

use lazy_static::lazy_static;

/// Focus state for a single window
pub struct WindowFocusState {
	/// The currently focused element ID (if any)
	focused_element: Option<u64>,
	/// Map of element IDs to their tab indices for Tab navigation
	tab_order:       HashMap<u64, i32>,
}

impl WindowFocusState {
	pub fn new() -> Self { Self { focused_element: None, tab_order: HashMap::new() } }

	/// Get the currently focused element
	pub fn get_focused(&self) -> Option<u64> { self.focused_element }

	/// Set focus to an element. Returns (previous_focused, new_focused) for event
	/// dispatch.
	pub fn set_focus(&mut self, element_id: u64) -> (Option<u64>, Option<u64>) {
		let previous = self.focused_element;
		self.focused_element = Some(element_id);
		(previous, Some(element_id))
	}

	/// Clear focus. Returns the previously focused element (if any).
	pub fn clear_focus(&mut self) -> Option<u64> {
		let previous = self.focused_element;
		self.focused_element = None;
		previous
	}

	/// Check if a specific element is focused
	pub fn is_focused(&self, element_id: u64) -> bool { self.focused_element == Some(element_id) }

	/// Register an element's tab index for Tab navigation
	pub fn register_tab_index(&mut self, element_id: u64, tab_index: i32) {
		self.tab_order.insert(element_id, tab_index);
	}

	/// Unregister an element from tab order
	pub fn unregister_tab_index(&mut self, element_id: u64) { self.tab_order.remove(&element_id); }

	/// Get the next focusable element in tab order (Tab key navigation)
	pub fn get_next_focusable(&self) -> Option<u64> {
		if self.tab_order.is_empty() {
			return None;
		}

		// Get all elements sorted by tab index
		let mut sorted: Vec<_> = self
            .tab_order
            .iter()
            .filter(|(_, idx)| **idx >= 0) // Only positive tab indices participate in tab navigation
            .collect();
		sorted.sort_by_key(|(_, idx)| *idx);

		if sorted.is_empty() {
			return None;
		}

		match self.focused_element {
			Some(current_id) => {
				// Find current element's position
				let current_pos = sorted.iter().position(|(id, _)| **id == current_id);
				match current_pos {
					Some(pos) => {
						// Move to next element, wrap around
						let next_pos = (pos + 1) % sorted.len();
						Some(*sorted[next_pos].0)
					}
					None => {
						// Current element not in tab order, start from beginning
						Some(*sorted[0].0)
					}
				}
			}
			None => {
				// No current focus, start from first element
				Some(*sorted[0].0)
			}
		}
	}

	/// Get the previous focusable element in tab order (Shift+Tab navigation)
	pub fn get_prev_focusable(&self) -> Option<u64> {
		if self.tab_order.is_empty() {
			return None;
		}

		let mut sorted: Vec<_> = self.tab_order.iter().filter(|(_, idx)| **idx >= 0).collect();
		sorted.sort_by_key(|(_, idx)| *idx);

		if sorted.is_empty() {
			return None;
		}

		match self.focused_element {
			Some(current_id) => {
				let current_pos = sorted.iter().position(|(id, _)| **id == current_id);
				match current_pos {
					Some(pos) => {
						let prev_pos = if pos == 0 { sorted.len() - 1 } else { pos - 1 };
						Some(*sorted[prev_pos].0)
					}
					None => Some(*sorted[sorted.len() - 1].0),
				}
			}
			None => Some(*sorted[sorted.len() - 1].0),
		}
	}

	/// Clear all state (for window cleanup)
	pub fn clear(&mut self) {
		self.focused_element = None;
		self.tab_order.clear();
	}
}

impl Default for WindowFocusState {
	fn default() -> Self { Self::new() }
}

/// Global focus state manager - manages focus state per window
pub struct FocusManager {
	/// Map of window ID to focus state
	windows: HashMap<u64, WindowFocusState>,
}

impl FocusManager {
	pub fn new() -> Self { Self { windows: HashMap::new() } }

	/// Get or create focus state for a window
	pub fn get_window_state(&mut self, window_id: u64) -> &mut WindowFocusState {
		self.windows.entry(window_id).or_insert_with(WindowFocusState::new)
	}

	/// Remove focus state for a window (cleanup)
	pub fn remove_window(&mut self, window_id: u64) { self.windows.remove(&window_id); }

	/// Clear all state
	pub fn clear(&mut self) { self.windows.clear(); }
}

impl Default for FocusManager {
	fn default() -> Self { Self::new() }
}

lazy_static! {
		/// Global focus manager
		static ref FOCUS_MANAGER: Arc<Mutex<FocusManager>> = Arc::new(Mutex::new(FocusManager::new()));
}

/// Get a reference to the global focus manager
pub fn get_focus_manager() -> &'static Arc<Mutex<FocusManager>> { &FOCUS_MANAGER }

/// Set focus to an element. Returns (blur_element_id, focus_element_id) for
/// event dispatch.
pub fn set_focus(window_id: u64, element_id: u64) -> (Option<u64>, Option<u64>) {
	if let Ok(mut manager) = FOCUS_MANAGER.lock() {
		let state = manager.get_window_state(window_id);
		state.set_focus(element_id)
	} else {
		(None, None)
	}
}

/// Clear focus for a window. Returns the previously focused element (if any).
pub fn clear_focus(window_id: u64) -> Option<u64> {
	if let Ok(mut manager) = FOCUS_MANAGER.lock() {
		let state = manager.get_window_state(window_id);
		state.clear_focus()
	} else {
		None
	}
}

/// Check if a specific element is focused
pub fn is_focused(window_id: u64, element_id: u64) -> bool {
	if let Ok(mut manager) = FOCUS_MANAGER.lock() {
		let state = manager.get_window_state(window_id);
		state.is_focused(element_id)
	} else {
		false
	}
}

/// Get the currently focused element for a window
pub fn get_focused(window_id: u64) -> Option<u64> {
	if let Ok(mut manager) = FOCUS_MANAGER.lock() {
		let state = manager.get_window_state(window_id);
		state.get_focused()
	} else {
		None
	}
}

/// Register an element's tab index
pub fn register_tab_index(window_id: u64, element_id: u64, tab_index: i32) {
	if let Ok(mut manager) = FOCUS_MANAGER.lock() {
		let state = manager.get_window_state(window_id);
		state.register_tab_index(element_id, tab_index);
	}
}

/// Unregister an element from tab order
pub fn unregister_tab_index(window_id: u64, element_id: u64) {
	if let Ok(mut manager) = FOCUS_MANAGER.lock() {
		let state = manager.get_window_state(window_id);
		state.unregister_tab_index(element_id);
	}
}

/// Focus the next element in tab order
pub fn focus_next(window_id: u64) -> (Option<u64>, Option<u64>) {
	if let Ok(mut manager) = FOCUS_MANAGER.lock() {
		let state = manager.get_window_state(window_id);
		if let Some(next_id) = state.get_next_focusable() {
			state.set_focus(next_id)
		} else {
			(None, None)
		}
	} else {
		(None, None)
	}
}

/// Focus the previous element in tab order
pub fn focus_prev(window_id: u64) -> (Option<u64>, Option<u64>) {
	if let Ok(mut manager) = FOCUS_MANAGER.lock() {
		let state = manager.get_window_state(window_id);
		if let Some(prev_id) = state.get_prev_focusable() {
			state.set_focus(prev_id)
		} else {
			(None, None)
		}
	} else {
		(None, None)
	}
}
