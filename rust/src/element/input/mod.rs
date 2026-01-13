mod blink_cursor;
mod change;
mod indent;
mod mode;
mod movement;
mod rope_ext;
mod selection;
mod state;
mod text_wrapper;

pub use blink_cursor::*;
pub use change::*;
pub use indent::*;
pub use mode::*;
pub use movement::*;
pub use rope_ext::*;
pub use selection::*;
pub use state::*;
pub use text_wrapper::*;

pub use ropey::Rope;

/// Cursor width in pixels.
pub const CURSOR_WIDTH: gpui::Pixels = gpui::px(2.0);

/// The context name for Input key bindings.
pub const CONTEXT: &str = "Input";
