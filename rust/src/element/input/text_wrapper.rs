use std::ops::Range;

use gpui::{Font, Pixels, px};
use ropey::Rope;

use super::RopeExt;

/// A line item with soft-wrapped line information.
#[derive(Debug, Clone)]
pub struct LineItem {
	/// The original line text length in bytes (without ending \n).
	len:               usize,
	/// Byte ranges of soft-wrapped lines within this line.
	pub wrapped_lines: Vec<Range<usize>>,
}

impl LineItem {
	/// Create a new line item.
	pub fn new(len: usize, wrapped_lines: Vec<Range<usize>>) -> Self { Self { len, wrapped_lines } }

	/// Get the byte length of this line.
	pub fn len(&self) -> usize { self.len }

	/// Get the number of wrapped lines (including the first line).
	pub fn lines_len(&self) -> usize { self.wrapped_lines.len().max(1) }

	/// Get the height of this line item given line height.
	pub fn height(&self, line_height: Pixels) -> Pixels { self.lines_len() as f32 * line_height }
}

/// Tracks the longest row in the text.
#[derive(Debug, Default, Clone)]
pub struct LongestRow {
	/// The 0-based row index.
	pub row: usize,
	/// The byte length of the longest line.
	pub len: usize,
}

/// Manages text wrapping and line layout.
#[derive(Debug)]
pub struct TextWrapper {
	/// The text content.
	text:            Rope,
	/// Total number of soft-wrapped lines.
	soft_lines:      usize,
	/// Font for measuring text.
	font:            Font,
	/// Font size.
	font_size:       Pixels,
	/// Wrap width (None for no wrapping).
	wrap_width:      Option<Pixels>,
	/// The longest row information.
	pub longest_row: LongestRow,
	/// Line items with wrapping info.
	pub lines:       Vec<LineItem>,
	/// Whether initialized.
	_initialized:    bool,
}

impl TextWrapper {
	/// Create a new TextWrapper.
	pub fn new(font: Font, font_size: Pixels, wrap_width: Option<Pixels>) -> Self {
		Self {
			text: Rope::new(),
			soft_lines: 0,
			font,
			font_size,
			wrap_width,
			longest_row: LongestRow::default(),
			lines: Vec::new(),
			_initialized: false,
		}
	}

	/// Create with default font.
	pub fn default_font() -> Self { Self::new(gpui::font("monospace"), px(14.), None) }

	/// Set the default text.
	pub fn set_default_text(&mut self, text: &Rope) { self.text = text.clone(); }

	/// Get the total number of soft-wrapped lines.
	pub fn len(&self) -> usize { self.soft_lines.max(1) }

	/// Check if empty.
	pub fn is_empty(&self) -> bool { self.text.len_bytes() == 0 }

	/// Get a line item by row index.
	pub fn line(&self, row: usize) -> Option<&LineItem> { self.lines.get(row) }

	/// Set the wrap width.
	pub fn set_wrap_width(&mut self, wrap_width: Option<Pixels>) {
		if wrap_width == self.wrap_width {
			return;
		}
		self.wrap_width = wrap_width;
		self.update_all(&self.text.clone());
	}

	/// Set the font.
	pub fn set_font(&mut self, font: Font, font_size: Pixels) {
		if self.font == font && self.font_size == font_size {
			return;
		}
		self.font = font;
		self.font_size = font_size;
		self.update_all(&self.text.clone());
	}

	/// Prepare if needed.
	pub fn prepare_if_needed(&mut self, text: &Rope) {
		if self._initialized && self.text.len_bytes() == text.len_bytes() {
			return;
		}
		self._initialized = true;
		self.update_all(text);
	}

	/// Update the text wrapper with changed text.
	pub fn update(&mut self, text: &Rope, range: &Range<usize>, new_text: &Rope) {
		// For simplicity, just update all for now
		// A more sophisticated implementation would only update affected lines
		self.update_all(text);
		let _ = (range, new_text); // Silence unused warnings
	}

	/// Update all lines.
	fn update_all(&mut self, text: &Rope) {
		self.text = text.clone();
		self.lines.clear();
		self.longest_row = LongestRow::default();

		let mut longest_len = 0;
		let mut longest_row = 0;

		for (row, line) in text.iter_lines().enumerate() {
			let line_len = line.len();

			if line_len > longest_len {
				longest_len = line_len;
				longest_row = row;
			}

			// For now, no soft wrapping - each line is one wrapped line
			let wrapped_lines = vec![0..line_len];

			self.lines.push(LineItem::new(line_len, wrapped_lines));
		}

		// Ensure at least one line
		if self.lines.is_empty() {
			self.lines.push(LineItem::new(0, vec![0..0]));
		}

		self.longest_row = LongestRow { row: longest_row, len: longest_len };

		self.soft_lines = self.lines.iter().map(|l| l.lines_len()).sum();
	}

	/// Convert byte offset to display point.
	pub fn offset_to_display_point(&self, offset: usize) -> DisplayPoint {
		let point = self.text.offset_to_point(offset);
		let row = point.row;
		let column = point.column;

		// Calculate wrapped row
		let wrapped_row: usize = self.lines.iter().take(row).map(|l| l.lines_len()).sum();

		DisplayPoint { row: wrapped_row, local_row: 0, column }
	}

	/// Convert display point to byte offset.
	pub fn display_point_to_offset(&self, point: DisplayPoint) -> usize {
		let mut wrapped_row = 0;

		for (row, line) in self.lines.iter().enumerate() {
			if wrapped_row + line.lines_len() > point.row {
				let line_start = self.text.line_start_offset(row);
				let local_row = point.row.saturating_sub(wrapped_row);

				if let Some(range) = line.wrapped_lines.get(local_row) {
					return line_start + (range.start + point.column).min(range.end);
				} else {
					return line_start + line.len();
				}
			}
			wrapped_row += line.lines_len();
		}

		self.text.len_bytes()
	}
}

impl Default for TextWrapper {
	fn default() -> Self { Self::default_font() }
}

/// Display point with soft-wrap information.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct DisplayPoint {
	/// The 0-based soft-wrapped row index.
	pub row:       usize,
	/// The 0-based local row within the line.
	pub local_row: usize,
	/// The 0-based column byte index.
	pub column:    usize,
}

impl DisplayPoint {
	/// Create a new display point.
	pub fn new(row: usize, local_row: usize, column: usize) -> Self {
		Self { row, local_row, column }
	}
}

/// Layout information for a visible line.
#[derive(Debug, Clone, Default)]
pub struct LineLayout {
	/// Total byte length of this line.
	len:               usize,
	/// Shaped lines for rendering.
	pub wrapped_lines: Vec<ShapedLineStub>,
	/// Longest width among wrapped lines.
	pub longest_width: Pixels,
}

/// Stub for ShapedLine (actual implementation depends on GPUI).
#[derive(Debug, Clone, Default)]
pub struct ShapedLineStub {
	pub text:  String,
	pub width: Pixels,
	pub len:   usize,
}

impl LineLayout {
	/// Create a new line layout.
	pub fn new() -> Self { Self::default() }

	/// Get the byte length.
	pub fn len(&self) -> usize { self.len }

	/// Check if empty.
	pub fn is_empty(&self) -> bool { self.len == 0 }

	/// Set the wrapped lines.
	pub fn set_wrapped_lines(&mut self, lines: Vec<ShapedLineStub>) {
		self.len = lines.iter().map(|l| l.len).sum();
		self.longest_width = lines
			.iter()
			.map(|l| l.width)
			.max_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
			.unwrap_or_default();
		self.wrapped_lines = lines;
	}
}

/// Cached layout information.
#[derive(Debug, Clone)]
pub struct LastLayout {
	/// Visible row range (0-based, unwrapped).
	pub visible_range:        Range<usize>,
	/// Top position of first visible line.
	pub visible_top:          Pixels,
	/// Byte range of visible content.
	pub visible_range_offset: Range<usize>,
	/// Line height.
	pub line_height:          Pixels,
	/// Wrap width.
	pub wrap_width:           Option<Pixels>,
	/// Width of line number gutter.
	pub line_number_width:    Pixels,
	/// Laid out lines.
	pub lines:                Vec<LineLayout>,
	/// Content width.
	pub content_width:        Pixels,
}

impl Default for LastLayout {
	fn default() -> Self {
		Self {
			visible_range:        0..1,
			visible_top:          px(0.),
			visible_range_offset: 0..0,
			line_height:          px(20.),
			wrap_width:           None,
			line_number_width:    px(0.),
			lines:                Vec::new(),
			content_width:        px(0.),
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_text_wrapper() {
		let mut wrapper = TextWrapper::default_font();
		let text = Rope::from("Hello\nWorld\nTest");

		wrapper.update_all(&text);

		assert_eq!(wrapper.lines.len(), 3);
		assert_eq!(wrapper.len(), 3);
		assert_eq!(wrapper.longest_row.len, 5); // "Hello" or "World"
	}

	#[test]
	fn test_display_point() {
		let mut wrapper = TextWrapper::default_font();
		let text = Rope::from("Hello\nWorld");

		wrapper.update_all(&text);

		let point = wrapper.offset_to_display_point(6); // Start of "World"
		assert_eq!(point.row, 1);
		assert_eq!(point.column, 0);

		let offset = wrapper.display_point_to_offset(DisplayPoint::new(1, 0, 3));
		assert_eq!(offset, 9); // "Wor|ld"
	}
}
