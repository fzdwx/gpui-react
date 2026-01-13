use gpui::{px, Pixels};
use std::time::Duration;

/// Blink interval for cursor.
pub const BLINK_INTERVAL: Duration = Duration::from_millis(500);

/// Pause duration before resuming blink.
pub const BLINK_PAUSE: Duration = Duration::from_millis(300);

/// Cursor width varies by platform.
#[cfg(target_os = "macos")]
pub const CURSOR_WIDTH: Pixels = px(1.5);

#[cfg(not(target_os = "macos"))]
pub const CURSOR_WIDTH: Pixels = px(2.0);

/// Manages cursor blinking state.
#[derive(Debug, Clone)]
pub struct BlinkCursor {
    /// Whether the cursor is currently visible.
    visible: bool,
    /// Whether blinking is enabled.
    enabled: bool,
    /// Whether blinking is paused (e.g., during typing).
    paused: bool,
    /// Epoch counter for tracking blink state changes.
    epoch: usize,
}

impl Default for BlinkCursor {
    fn default() -> Self {
        Self {
            visible: true,
            enabled: false,
            paused: false,
            epoch: 0,
        }
    }
}

impl BlinkCursor {
    /// Create a new BlinkCursor.
    pub fn new() -> Self {
        Self::default()
    }

    /// Start blinking the cursor.
    pub fn start(&mut self) {
        self.enabled = true;
        self.visible = true;
        self.paused = false;
        self.epoch += 1;
    }

    /// Stop blinking the cursor.
    pub fn stop(&mut self) {
        self.enabled = false;
        self.visible = true;
        self.paused = false;
        self.epoch += 1;
    }

    /// Pause blinking temporarily (e.g., during typing).
    /// The cursor will remain visible while paused.
    pub fn pause(&mut self) {
        if self.enabled {
            self.paused = true;
            self.visible = true;
            self.epoch += 1;
        }
    }

    /// Resume blinking after pause.
    pub fn resume(&mut self) {
        if self.enabled && self.paused {
            self.paused = false;
            self.epoch += 1;
        }
    }

    /// Toggle cursor visibility (called by timer).
    pub fn toggle(&mut self) {
        if self.enabled && !self.paused {
            self.visible = !self.visible;
        }
    }

    /// Check if cursor should be visible.
    pub fn visible(&self) -> bool {
        self.visible
    }

    /// Check if blinking is enabled.
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Check if blinking is paused.
    pub fn is_paused(&self) -> bool {
        self.paused
    }

    /// Get current epoch (for change tracking).
    pub fn epoch(&self) -> usize {
        self.epoch
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_blink_cursor() {
        let mut cursor = BlinkCursor::new();
        assert!(cursor.visible());
        assert!(!cursor.is_enabled());

        cursor.start();
        assert!(cursor.is_enabled());
        assert!(cursor.visible());

        cursor.toggle();
        assert!(!cursor.visible());

        cursor.toggle();
        assert!(cursor.visible());

        cursor.pause();
        assert!(cursor.is_paused());
        assert!(cursor.visible());

        cursor.toggle(); // Should not toggle while paused
        assert!(cursor.visible());

        cursor.resume();
        assert!(!cursor.is_paused());

        cursor.stop();
        assert!(!cursor.is_enabled());
        assert!(cursor.visible());
    }
}
