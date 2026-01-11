//! Cursor blink animation
//!
//! Manages the blinking state of the input cursor.

use std::time::Duration;

/// Blink interval in milliseconds
pub const BLINK_INTERVAL: Duration = Duration::from_millis(500);

/// Pause duration after typing before resuming blink
pub const PAUSE_DURATION: Duration = Duration::from_millis(300);

/// Cursor width in pixels (platform-specific)
#[cfg(not(target_os = "macos"))]
pub const CURSOR_WIDTH: f32 = 2.0;
#[cfg(target_os = "macos")]
pub const CURSOR_WIDTH: f32 = 1.5;

/// Cursor blink animation state
#[derive(Clone, Debug)]
pub struct BlinkCursor {
	/// Whether cursor is currently visible
	visible: bool,

	/// Whether blinking is paused (e.g., during typing)
	paused: bool,

	/// Epoch counter for canceling stale timers
	epoch: usize,

	/// Timestamp of last state change (for timer management)
	last_change_ms: u64,
}

impl Default for BlinkCursor {
	fn default() -> Self { Self::new() }
}

impl BlinkCursor {
	pub fn new() -> Self { Self { visible: true, paused: false, epoch: 0, last_change_ms: 0 } }

	/// Reset cursor to visible and pause blinking temporarily
	/// Called after any text input or cursor movement
	pub fn pause_and_show(&mut self) {
		self.visible = true;
		self.paused = true;
		self.epoch += 1;
	}

	/// Resume blinking after pause
	pub fn resume(&mut self) { self.paused = false; }

	/// Toggle visibility (called by timer)
	/// Returns true if state changed
	pub fn toggle(&mut self) -> bool {
		if !self.paused {
			self.visible = !self.visible;
			true
		} else {
			false
		}
	}

	/// Check if cursor should be visible
	pub fn is_visible(&self) -> bool { self.visible || self.paused }

	/// Get current epoch for timer cancellation
	pub fn epoch(&self) -> usize { self.epoch }

	/// Check if paused
	pub fn is_paused(&self) -> bool { self.paused }

	/// Update based on elapsed time
	/// Returns true if a repaint is needed
	pub fn update(&mut self, current_time_ms: u64) -> bool {
		if self.paused {
			// Check if pause duration has elapsed
			if current_time_ms - self.last_change_ms >= PAUSE_DURATION.as_millis() as u64 {
				self.paused = false;
				self.last_change_ms = current_time_ms;
				return true;
			}
			return false;
		}

		// Check if blink interval has elapsed
		if current_time_ms - self.last_change_ms >= BLINK_INTERVAL.as_millis() as u64 {
			self.visible = !self.visible;
			self.last_change_ms = current_time_ms;
			return true;
		}

		false
	}

	/// Record current time for timer management
	pub fn set_last_change(&mut self, time_ms: u64) { self.last_change_ms = time_ms; }
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_initial_state() {
		let cursor = BlinkCursor::new();
		assert!(cursor.is_visible());
		assert!(!cursor.is_paused());
	}

	#[test]
	fn test_pause_and_show() {
		let mut cursor = BlinkCursor::new();
		cursor.toggle(); // Now hidden
		assert!(!cursor.visible);

		cursor.pause_and_show();
		assert!(cursor.is_visible());
		assert!(cursor.is_paused());
	}

	#[test]
	fn test_toggle_when_not_paused() {
		let mut cursor = BlinkCursor::new();
		assert!(cursor.is_visible());

		let changed = cursor.toggle();
		assert!(changed);
		assert!(!cursor.visible);

		let changed = cursor.toggle();
		assert!(changed);
		assert!(cursor.visible);
	}

	#[test]
	fn test_toggle_when_paused() {
		let mut cursor = BlinkCursor::new();
		cursor.pause_and_show();

		let changed = cursor.toggle();
		assert!(!changed); // Should not toggle when paused
		assert!(cursor.is_visible());
	}

	#[test]
	fn test_epoch_increment() {
		let mut cursor = BlinkCursor::new();
		let epoch1 = cursor.epoch();

		cursor.pause_and_show();
		let epoch2 = cursor.epoch();

		assert!(epoch2 > epoch1);
	}
}
