//! Display point types for multi-line text positioning
//!
//! Provides types for representing positions in wrapped text.

use gpui::Pixels;

/// Position in wrapped/displayed text
///
/// This represents a position in the visual display, accounting for soft
/// wrapping. Unlike `Point` which uses physical line numbers, `DisplayPoint`
/// uses visual rows.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct DisplayPoint {
	/// Display row (includes soft wraps, 0-based)
	pub row:    usize,
	/// Column byte offset within the display row
	pub column: usize,
}

impl DisplayPoint {
	pub fn new(row: usize, column: usize) -> Self { Self { row, column } }

	/// Create a display point at the origin
	pub fn origin() -> Self { Self { row: 0, column: 0 } }
}

/// A wrapped line segment for display
#[derive(Clone, Debug)]
pub struct WrappedLine {
	/// Starting byte offset in source text
	pub start_offset:  usize,
	/// Ending byte offset in source text (exclusive)
	pub end_offset:    usize,
	/// Physical line index (0-based, counts only newlines)
	pub physical_line: usize,
	/// Visual row within the display (0-based, increments per line)
	pub visual_row:    usize,
	/// Width of this line segment in pixels (cached)
	pub width:         Option<Pixels>,
}

impl WrappedLine {
	pub fn new(
		start_offset: usize,
		end_offset: usize,
		physical_line: usize,
		visual_row: usize,
	) -> Self {
		Self { start_offset, end_offset, physical_line, visual_row, width: None }
	}

	/// Get the byte length of this line segment
	pub fn len(&self) -> usize { self.end_offset - self.start_offset }

	/// Check if this line segment is empty
	pub fn is_empty(&self) -> bool { self.start_offset == self.end_offset }

	/// Check if a byte offset is within this line segment
	pub fn contains_offset(&self, offset: usize) -> bool {
		offset >= self.start_offset && offset < self.end_offset
	}

	/// Convert a global byte offset to a local offset within this line
	pub fn to_local_offset(&self, offset: usize) -> usize { offset.saturating_sub(self.start_offset) }

	/// Convert a local offset to a global byte offset
	pub fn to_global_offset(&self, local_offset: usize) -> usize {
		self.start_offset + local_offset.min(self.len())
	}
}

/// Scroll position for multi-line text
#[derive(Clone, Copy, Debug, Default)]
pub struct ScrollPosition {
	/// Vertical scroll offset in display rows
	pub row: usize,
	/// Horizontal scroll offset in pixels
	pub x:   Pixels,
}

impl ScrollPosition {
	pub fn new(row: usize, x: Pixels) -> Self { Self { row, x } }
}

#[cfg(test)]
mod tests {
	use gpui::px;

	use super::*;

	#[test]
	fn test_display_point() {
		let dp = DisplayPoint::new(5, 10);
		assert_eq!(dp.row, 5);
		assert_eq!(dp.column, 10);

		let origin = DisplayPoint::origin();
		assert_eq!(origin.row, 0);
		assert_eq!(origin.column, 0);
	}

	#[test]
	fn test_wrapped_line() {
		let line = WrappedLine::new(10, 20, 1, 0);
		assert_eq!(line.len(), 10);
		assert!(!line.is_empty());
		assert!(line.contains_offset(10));
		assert!(line.contains_offset(19));
		assert!(!line.contains_offset(20));

		assert_eq!(line.to_local_offset(15), 5);
		assert_eq!(line.to_global_offset(5), 15);
	}

	#[test]
	fn test_scroll_position() {
		let scroll = ScrollPosition::new(10, px(50.0));
		assert_eq!(scroll.row, 10);
		assert_eq!(scroll.x, px(50.0));
	}
}
