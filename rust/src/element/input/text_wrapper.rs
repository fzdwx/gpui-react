//! Text wrapping for multi-line display
//!
//! Handles soft wrapping of text for multi-line text areas.

use gpui::{Pixels, px};

use super::display_point::{DisplayPoint, WrappedLine};
use super::text_content::TextContent;

/// Handles soft wrapping of text for display
#[derive(Clone, Debug)]
pub struct TextWrapper {
	/// Maximum width for wrapping (in pixels), None = no wrapping
	wrap_width: Option<Pixels>,
	/// Cached wrapped lines
	lines: Vec<WrappedLine>,
	/// Total display row count
	display_rows: usize,
	/// Version counter for cache invalidation
	version: usize,
}

impl Default for TextWrapper {
	fn default() -> Self {
		Self::new(None)
	}
}

impl TextWrapper {
	/// Create a new text wrapper
	pub fn new(wrap_width: Option<Pixels>) -> Self {
		Self { wrap_width, lines: Vec::new(), display_rows: 0, version: 0 }
	}

	/// Set the wrap width
	pub fn set_wrap_width(&mut self, wrap_width: Option<Pixels>) {
		if self.wrap_width != wrap_width {
			self.wrap_width = wrap_width;
			self.version += 1;
			self.lines.clear();
		}
	}

	/// Get the wrap width
	pub fn wrap_width(&self) -> Option<Pixels> {
		self.wrap_width
	}

	/// Update wrapped lines from text content
	///
	/// For now, this is a simple implementation without actual pixel-based wrapping.
	/// Wrapping is done at newlines only.
	pub fn update(&mut self, content: &TextContent) {
		self.lines.clear();
		self.display_rows = 0;

		if content.is_empty() {
			// Even empty content has one line
			self.lines.push(WrappedLine::new(0, 0, 0));
			self.display_rows = 1;
			return;
		}

		let mut offset = 0;
		let total_len = content.len();

		for physical_line in 0..content.line_count() {
			let line_start = content.line_start_offset(physical_line);
			let line_end = content.line_end_offset(physical_line);

			// For now, no soft wrapping - just use physical lines
			// TODO: Implement pixel-based soft wrapping when wrap_width is set
			self.lines.push(WrappedLine {
				start_offset: line_start,
				end_offset: line_end,
				physical_line,
				visual_row: 0,
				width: None,
			});
			self.display_rows += 1;

			// Include newline in offset tracking
			offset = line_end;
			if offset < total_len {
				// Skip newline character
				if let Some(c) = content.char_at(offset) {
					if c == '\n' {
						offset += 1;
					}
				}
			}
		}

		self.version += 1;
	}

	/// Get total display row count
	pub fn display_row_count(&self) -> usize {
		self.display_rows.max(1)
	}

	/// Get the wrapped lines
	pub fn lines(&self) -> &[WrappedLine] {
		&self.lines
	}

	/// Get a specific wrapped line by display row
	pub fn line(&self, display_row: usize) -> Option<&WrappedLine> {
		self.lines.get(display_row)
	}

	/// Convert byte offset to display point
	pub fn offset_to_display_point(&self, content: &TextContent, offset: usize) -> DisplayPoint {
		let offset = offset.min(content.len());

		for (row, line) in self.lines.iter().enumerate() {
			if offset >= line.start_offset && offset <= line.end_offset {
				return DisplayPoint::new(row, offset - line.start_offset);
			}
		}

		// If not found, return end of last line
		if let Some(last) = self.lines.last() {
			DisplayPoint::new(self.lines.len() - 1, last.len())
		} else {
			DisplayPoint::origin()
		}
	}

	/// Convert display point to byte offset
	pub fn display_point_to_offset(&self, point: DisplayPoint) -> usize {
		if let Some(line) = self.lines.get(point.row) {
			line.start_offset + point.column.min(line.len())
		} else if let Some(last) = self.lines.last() {
			// Past the end - return end of content
			last.end_offset
		} else {
			0
		}
	}

	/// Move cursor up by one display row
	pub fn move_up(&self, content: &TextContent, offset: usize, preferred_column: Option<usize>) -> usize {
		let point = self.offset_to_display_point(content, offset);
		if point.row == 0 {
			// Already at top, stay at same position
			return offset;
		}

		let target_row = point.row - 1;
		let column = preferred_column.unwrap_or(point.column);

		if let Some(line) = self.lines.get(target_row) {
			line.start_offset + column.min(line.len())
		} else {
			offset
		}
	}

	/// Move cursor down by one display row
	pub fn move_down(&self, content: &TextContent, offset: usize, preferred_column: Option<usize>) -> usize {
		let point = self.offset_to_display_point(content, offset);
		if point.row + 1 >= self.display_row_count() {
			// Already at bottom, stay at same position
			return offset;
		}

		let target_row = point.row + 1;
		let column = preferred_column.unwrap_or(point.column);

		if let Some(line) = self.lines.get(target_row) {
			line.start_offset + column.min(line.len())
		} else {
			offset
		}
	}

	/// Get visible lines in a viewport range
	pub fn visible_lines(&self, start_row: usize, end_row: usize) -> &[WrappedLine] {
		let start = start_row.min(self.lines.len());
		let end = end_row.min(self.lines.len());
		&self.lines[start..end]
	}

	/// Find which display row contains a byte offset
	pub fn display_row_for_offset(&self, offset: usize) -> usize {
		for (row, line) in self.lines.iter().enumerate() {
			if offset >= line.start_offset && offset <= line.end_offset {
				return row;
			}
		}
		self.lines.len().saturating_sub(1)
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_basic_wrapping() {
		let content = TextContent::from_str("line1\nline2\nline3");
		let mut wrapper = TextWrapper::new(None);
		wrapper.update(&content);

		assert_eq!(wrapper.display_row_count(), 3);
		assert_eq!(wrapper.lines().len(), 3);

		assert_eq!(wrapper.lines()[0].start_offset, 0);
		assert_eq!(wrapper.lines()[0].end_offset, 5);
		assert_eq!(wrapper.lines()[1].start_offset, 6);
		assert_eq!(wrapper.lines()[1].end_offset, 11);
	}

	#[test]
	fn test_offset_to_display_point() {
		let content = TextContent::from_str("hello\nworld");
		let mut wrapper = TextWrapper::new(None);
		wrapper.update(&content);

		assert_eq!(wrapper.offset_to_display_point(&content, 0), DisplayPoint::new(0, 0));
		assert_eq!(wrapper.offset_to_display_point(&content, 3), DisplayPoint::new(0, 3));
		assert_eq!(wrapper.offset_to_display_point(&content, 5), DisplayPoint::new(0, 5));
		assert_eq!(wrapper.offset_to_display_point(&content, 6), DisplayPoint::new(1, 0));
		assert_eq!(wrapper.offset_to_display_point(&content, 11), DisplayPoint::new(1, 5));
	}

	#[test]
	fn test_display_point_to_offset() {
		let content = TextContent::from_str("hello\nworld");
		let mut wrapper = TextWrapper::new(None);
		wrapper.update(&content);

		assert_eq!(wrapper.display_point_to_offset(DisplayPoint::new(0, 0)), 0);
		assert_eq!(wrapper.display_point_to_offset(DisplayPoint::new(0, 3)), 3);
		assert_eq!(wrapper.display_point_to_offset(DisplayPoint::new(1, 0)), 6);
		assert_eq!(wrapper.display_point_to_offset(DisplayPoint::new(1, 5)), 11);
	}

	#[test]
	fn test_move_up_down() {
		let content = TextContent::from_str("hello\nworld\ntest");
		let mut wrapper = TextWrapper::new(None);
		wrapper.update(&content);

		// Start at "world" line, column 2
		let offset = 8; // "wo" in "world"
		assert_eq!(wrapper.move_up(&content, offset, None), 2); // column 2 in "hello"
		assert_eq!(wrapper.move_down(&content, offset, None), 14); // column 2 in "test"

		// At top, move up stays
		assert_eq!(wrapper.move_up(&content, 2, None), 2);

		// At bottom, move down stays
		assert_eq!(wrapper.move_down(&content, 14, None), 14);
	}

	#[test]
	fn test_empty_content() {
		let content = TextContent::new();
		let mut wrapper = TextWrapper::new(None);
		wrapper.update(&content);

		assert_eq!(wrapper.display_row_count(), 1);
		assert_eq!(wrapper.lines().len(), 1);
	}
}
