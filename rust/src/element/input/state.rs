use std::ops::Range;

use gpui::{App, Bounds, ClipboardItem, EventEmitter, FocusHandle, MouseButton, MouseDownEvent, MouseMoveEvent, MouseUpEvent, Pixels, Point, ScrollHandle, Size, Window, px};
use ropey::Rope;

use super::{BlinkCursor, Change, History, InputMode, LastLayout, RopeExt, Selection, TextPoint, TextWrapper, movement, select_word};

/// Events emitted by InputState.
#[derive(Debug, Clone)]
pub enum InputEvent {
	/// Text content changed.
	Change(String),
	/// Enter key pressed.
	PressEnter,
	/// Input gained focus.
	Focus,
	/// Input lost focus.
	Blur,
}

/// Core state for text input.
pub struct InputState {
	/// The text content.
	pub text:                   Rope,
	/// Current selection (cursor position and selection range).
	pub selected_range:         Selection,
	/// Previous selection for tracking changes.
	pub last_selected_range:    Option<Selection>,
	/// IME marked text range.
	pub ime_marked_range:       Option<Range<usize>>,
	/// Focus handle for keyboard focus.
	pub focus_handle:           FocusHandle,
	/// Cursor blink state.
	pub blink_cursor:           BlinkCursor,
	/// Whether selection is reversed (anchor > head).
	pub selection_reversed:     bool,
	/// Preferred column for vertical movement.
	pub preferred_column:       Option<usize>,
	/// Edit history for undo/redo.
	pub history:                History,
	/// Input mode configuration.
	pub mode:                   InputMode,
	/// Whether input is disabled.
	pub disabled:               bool,
	/// Whether input is loading.
	pub loading:                bool,
	/// Whether text is masked (password).
	pub masked:                 bool,
	/// Placeholder text.
	pub placeholder:            String,
	/// Text wrapper for layout.
	pub text_wrapper:           TextWrapper,
	/// Last layout information.
	pub last_layout:            Option<LastLayout>,
	/// Last bounds.
	pub last_bounds:            Option<Bounds<Pixels>>,
	/// Last cursor position.
	pub last_cursor:            Option<usize>,
	/// Input bounds.
	pub input_bounds:           Option<Bounds<Pixels>>,
	/// Scroll handle.
	pub scroll_handle:          ScrollHandle,
	/// Scroll size.
	pub scroll_size:            Size<Pixels>,
	/// Deferred scroll offset.
	pub deferred_scroll_offset: Option<Point<Pixels>>,
	/// Whether soft wrap is enabled.
	pub soft_wrap:              bool,
	/// Mouse drag state.
	drag_start:                 Option<usize>,
	/// Click count for multi-click detection.
	click_count:                u32,
	/// Element ID for FFI.
	pub element_id:             u64,
}

impl InputState {
	/// Create a new InputState.
	pub fn new(focus_handle: FocusHandle) -> Self {
		Self {
			text: Rope::new(),
			selected_range: Selection::cursor(0),
			last_selected_range: None,
			ime_marked_range: None,
			focus_handle,
			blink_cursor: BlinkCursor::new(),
			selection_reversed: false,
			preferred_column: None,
			history: History::new(),
			mode: InputMode::default(),
			disabled: false,
			loading: false,
			masked: false,
			placeholder: String::new(),
			text_wrapper: TextWrapper::default(),
			last_layout: None,
			last_bounds: None,
			last_cursor: None,
			input_bounds: None,
			scroll_handle: ScrollHandle::default(),
			scroll_size: Size::default(),
			deferred_scroll_offset: None,
			soft_wrap: false,
			drag_start: None,
			click_count: 0,
			element_id: 0,
		}
	}

	/// Create with element ID for FFI.
	pub fn with_element_id(mut self, id: u64) -> Self {
		self.element_id = id;
		self
	}

	/// Set the input mode.
	pub fn with_mode(mut self, mode: InputMode) -> Self {
		self.mode = mode;
		self
	}

	/// Set placeholder text.
	pub fn with_placeholder(mut self, placeholder: impl Into<String>) -> Self {
		self.placeholder = placeholder.into();
		self
	}

	/// Set masked mode (for passwords).
	pub fn with_masked(mut self, masked: bool) -> Self {
		self.masked = masked;
		self
	}

	/// Get the current text value.
	pub fn value(&self) -> String { self.text.to_string() }

	/// Set the text value.
	pub fn set_value(&mut self, value: &str, _cx: &mut App) {
		let old_text = self.text.to_string();
		self.text = Rope::from(value);
		self.selected_range = Selection::cursor(value.len());
		self.text_wrapper.set_default_text(&self.text);

		// Record change for undo
		self.history.push(Change::new(0..old_text.len(), old_text, value.to_string()));
	}

	/// Get the cursor position.
	pub fn cursor(&self) -> usize { self.selected_range.end }

	/// Get cursor position as (row, column).
	pub fn cursor_position(&self) -> TextPoint { self.text.offset_to_point(self.cursor()) }

	/// Set cursor position.
	pub fn set_cursor(&mut self, offset: usize) {
		let offset = offset.min(self.text.len_bytes());
		self.selected_range = Selection::cursor(offset);
		self.blink_cursor.pause();
	}

	/// Get the selected text.
	pub fn selected_text(&self) -> String {
		let range = self.selected_range.range();
		self.text.slice(range).to_string()
	}

	/// Replace text in the given range.
	pub fn replace_text(
		&mut self,
		range: Range<usize>,
		new_text: &str,
		_window: &mut Window,
		cx: &mut App,
	) {
		let old_text = self.text.slice(range.clone()).to_string();

		// Update text
		self.text.replace(range.clone(), new_text);

		// Update selection
		let new_cursor = range.start + new_text.len();
		self.selected_range = Selection::cursor(new_cursor);

		// Update text wrapper
		self.text_wrapper.update(&self.text, &range, &Rope::from(new_text));

		// Record change
		self.history.push(Change::new(range, old_text, new_text.to_string()));

		self.blink_cursor.pause();
		// Notify is handled at FFI level
	}

	/// Insert text at cursor.
	pub fn insert(&mut self, text: &str, window: &mut Window, cx: &mut App) {
		let range = self.selected_range.range();
		self.replace_text(range, text, window, cx);
	}

	/// Delete selected text or character before cursor.
	pub fn backspace(&mut self, window: &mut Window, cx: &mut App) {
		if self.disabled {
			return;
		}

		let range = if self.selected_range.is_empty() {
			let cursor = self.cursor();
			if cursor == 0 {
				return;
			}
			let prev = self.text.prev_grapheme_boundary(cursor);
			prev..cursor
		} else {
			self.selected_range.range()
		};

		self.replace_text(range, "", window, cx);
	}

	/// Delete selected text or character after cursor.
	pub fn delete(&mut self, window: &mut Window, cx: &mut App) {
		if self.disabled {
			return;
		}

		let range = if self.selected_range.is_empty() {
			let cursor = self.cursor();
			if cursor >= self.text.len_bytes() {
				return;
			}
			let next = self.text.next_grapheme_boundary(cursor);
			cursor..next
		} else {
			self.selected_range.range()
		};

		self.replace_text(range, "", window, cx);
	}

	/// Clear all text.
	pub fn clear(&mut self, window: &mut Window, cx: &mut App) {
		self.replace_text(0..self.text.len_bytes(), "", window, cx);
	}

	/// Select all text.
	pub fn select_all(&mut self, _cx: &mut App) {
		self.selected_range = movement::select_all(&self.text);
	}

	/// Move cursor left.
	pub fn move_left(&mut self, _cx: &mut App) {
		self.selected_range = movement::move_left(&self.text, &self.selected_range);
		self.preferred_column = None;
		self.blink_cursor.pause();
	}

	/// Move cursor right.
	pub fn move_right(&mut self, _cx: &mut App) {
		self.selected_range = movement::move_right(&self.text, &self.selected_range);
		self.preferred_column = None;
		self.blink_cursor.pause();
	}

	/// Move cursor up.
	pub fn move_up(&mut self, _cx: &mut App) {
		if self.preferred_column.is_none() {
			self.preferred_column = Some(self.text.offset_to_point(self.cursor()).column);
		}
		self.selected_range =
			movement::move_up(&self.text, &self.selected_range, self.preferred_column);
		self.blink_cursor.pause();
	}

	/// Move cursor down.
	pub fn move_down(&mut self, _cx: &mut App) {
		if self.preferred_column.is_none() {
			self.preferred_column = Some(self.text.offset_to_point(self.cursor()).column);
		}
		self.selected_range =
			movement::move_down(&self.text, &self.selected_range, self.preferred_column);
		self.blink_cursor.pause();
	}

	/// Move to start of line.
	pub fn move_to_line_start(&mut self, _cx: &mut App) {
		self.selected_range = movement::move_to_line_start(&self.text, &self.selected_range);
		self.preferred_column = None;
		self.blink_cursor.pause();
	}

	/// Move to end of line.
	pub fn move_to_line_end(&mut self, _cx: &mut App) {
		self.selected_range = movement::move_to_line_end(&self.text, &self.selected_range);
		self.preferred_column = None;
		self.blink_cursor.pause();
	}

	/// Move to start of text.
	pub fn move_to_start(&mut self, _cx: &mut App) {
		self.selected_range = movement::move_to_start(&self.text, &self.selected_range);
		self.preferred_column = None;
		self.blink_cursor.pause();
	}

	/// Move to end of text.
	pub fn move_to_end(&mut self, _cx: &mut App) {
		self.selected_range = movement::move_to_end(&self.text, &self.selected_range);
		self.preferred_column = None;
		self.blink_cursor.pause();
	}

	/// Move to previous word.
	pub fn move_to_prev_word(&mut self, _cx: &mut App) {
		self.selected_range = movement::move_to_prev_word(&self.text, &self.selected_range);
		self.preferred_column = None;
		self.blink_cursor.pause();
	}

	/// Move to next word.
	pub fn move_to_next_word(&mut self, _cx: &mut App) {
		self.selected_range = movement::move_to_next_word(&self.text, &self.selected_range);
		self.preferred_column = None;
		self.blink_cursor.pause();
	}

	/// Extend selection left.
	pub fn select_left(&mut self, _cx: &mut App) {
		self.selected_range = movement::select_left(&self.text, &self.selected_range);
		self.selection_reversed = self.selected_range.is_reversed();
		self.blink_cursor.pause();
	}

	/// Extend selection right.
	pub fn select_right(&mut self, _cx: &mut App) {
		self.selected_range = movement::select_right(&self.text, &self.selected_range);
		self.selection_reversed = self.selected_range.is_reversed();
		self.blink_cursor.pause();
	}

	/// Extend selection up.
	pub fn select_up(&mut self, _cx: &mut App) {
		if self.preferred_column.is_none() {
			self.preferred_column = Some(self.text.offset_to_point(self.cursor()).column);
		}
		self.selected_range =
			movement::select_up(&self.text, &self.selected_range, self.preferred_column);
		self.selection_reversed = self.selected_range.is_reversed();
		self.blink_cursor.pause();
	}

	/// Extend selection down.
	pub fn select_down(&mut self, _cx: &mut App) {
		if self.preferred_column.is_none() {
			self.preferred_column = Some(self.text.offset_to_point(self.cursor()).column);
		}
		self.selected_range =
			movement::select_down(&self.text, &self.selected_range, self.preferred_column);
		self.selection_reversed = self.selected_range.is_reversed();
		self.blink_cursor.pause();
	}

	/// Extend selection to line start.
	pub fn select_to_line_start(&mut self, _cx: &mut App) {
		self.selected_range = movement::select_to_line_start(&self.text, &self.selected_range);
		self.selection_reversed = self.selected_range.is_reversed();
		self.blink_cursor.pause();
	}

	/// Extend selection to line end.
	pub fn select_to_line_end(&mut self, _cx: &mut App) {
		self.selected_range = movement::select_to_line_end(&self.text, &self.selected_range);
		self.selection_reversed = self.selected_range.is_reversed();
		self.blink_cursor.pause();
	}

	/// Extend selection to previous word.
	pub fn select_to_prev_word(&mut self, _cx: &mut App) {
		self.selected_range = movement::select_to_prev_word(&self.text, &self.selected_range);
		self.selection_reversed = self.selected_range.is_reversed();
		self.blink_cursor.pause();
	}

	/// Extend selection to next word.
	pub fn select_to_next_word(&mut self, _cx: &mut App) {
		self.selected_range = movement::select_to_next_word(&self.text, &self.selected_range);
		self.selection_reversed = self.selected_range.is_reversed();
		self.blink_cursor.pause();
	}

	/// Extend selection to start.
	pub fn select_to_start(&mut self, _cx: &mut App) {
		self.selected_range = movement::select_to_start(&self.text, &self.selected_range);
		self.selection_reversed = self.selected_range.is_reversed();
		self.blink_cursor.pause();
	}

	/// Extend selection to end.
	pub fn select_to_end(&mut self, _cx: &mut App) {
		self.selected_range = movement::select_to_end(&self.text, &self.selected_range);
		self.selection_reversed = self.selected_range.is_reversed();
		self.blink_cursor.pause();
	}

	/// Undo last change.
	pub fn undo(&mut self, window: &mut Window, cx: &mut App) {
		if let Some(change) = self.history.pop_undo() {
			self.text.replace(change.old_range.clone(), &change.new_text);
			self.selected_range = Selection::cursor(change.old_range.start + change.new_text.len());
			self.text_wrapper.set_default_text(&self.text);
			// Notify is handled at FFI level
		}
	}

	/// Redo last undone change.
	pub fn redo(&mut self, window: &mut Window, cx: &mut App) {
		if let Some(change) = self.history.pop_redo() {
			self.text.replace(change.old_range.clone(), &change.new_text);
			self.selected_range = Selection::cursor(change.old_range.start + change.new_text.len());
			self.text_wrapper.set_default_text(&self.text);
			// Notify is handled at FFI level
		}
	}

	/// Copy selected text to clipboard.
	pub fn copy(&mut self, _window: &mut Window, cx: &mut App) {
		let text = self.selected_text();
		if !text.is_empty() {
			cx.write_to_clipboard(ClipboardItem::new_string(text));
		}
	}

	/// Cut selected text to clipboard.
	pub fn cut(&mut self, window: &mut Window, cx: &mut App) {
		let text = self.selected_text();
		if !text.is_empty() {
			cx.write_to_clipboard(ClipboardItem::new_string(text));
			self.replace_text(self.selected_range.range(), "", window, cx);
		}
	}

	/// Paste from clipboard.
	pub fn paste(&mut self, window: &mut Window, cx: &mut App) {
		if self.disabled {
			return;
		}

		if let Some(item) = cx.read_from_clipboard() {
			if let Some(text) = item.text() {
				// For single-line, remove newlines
				let text = if self.mode.is_single_line() {
					text.replace('\n', " ").replace('\r', "")
				} else {
					text.to_string()
				};
				self.insert(&text, window, cx);
			}
		}
	}

	/// Handle Enter key.
	pub fn enter(&mut self, window: &mut Window, cx: &mut App) {
		if self.mode.is_multi_line() {
			self.insert("\n", window, cx);
		}
		// Emit PressEnter event will be handled by the element
	}

	/// Handle Tab key.
	pub fn tab(&mut self, window: &mut Window, cx: &mut App) {
		if self.mode.is_multi_line() {
			let indent = self.mode.tab_size().indent_str();
			self.insert(&indent, window, cx);
		}
	}

	/// Focus the input.
	pub fn focus(&mut self, window: &mut Window, _cx: &mut App) {
		self.focus_handle.focus(window);
		self.blink_cursor.start();
	}

	/// Blur the input.
	pub fn blur(&mut self, window: &mut Window, _cx: &mut App) {
		window.blur();
		self.blink_cursor.stop();
	}

	/// Check if focused.
	pub fn is_focused(&self, window: &Window) -> bool { self.focus_handle.is_focused(window) }

	/// Set masked mode.
	pub fn set_masked(&mut self, masked: bool, _window: &mut Window, cx: &mut App) {
		self.masked = masked;
		// Notify is handled at FFI level
	}

	/// Check if cursor should be visible.
	pub fn show_cursor(&self, window: &Window, _cx: &App) -> bool {
		self.is_focused(window) && !self.disabled && self.blink_cursor.visible()
	}

	/// Handle mouse down.
	pub fn on_mouse_down(&mut self, event: &MouseDownEvent, window: &mut Window, cx: &mut App) {
		if self.disabled {
			return;
		}

		self.focus(window, cx);

		if event.button == MouseButton::Left {
			self.click_count = event.click_count as u32;

			if let Some(offset) = self.offset_for_position(event.position) {
				match event.click_count {
					1 => {
						// Single click - set cursor
						self.selected_range = Selection::cursor(offset);
						self.drag_start = Some(offset);
					}
					2 => {
						// Double click - select word
						self.selected_range = select_word(&self.text, offset);
					}
					3 => {
						// Triple click - select line
						let point = self.text.offset_to_point(offset);
						let line_start = self.text.line_start_offset(point.row);
						let line_end = self.text.line_end_offset(point.row);
						self.selected_range = Selection::new(line_start, line_end);
					}
					_ => {
						// Quad+ click - select all
						self.select_all(cx);
					}
				}
			}
		}

		self.blink_cursor.pause();
		// Notify is handled at FFI level
	}

	/// Handle mouse up.
	pub fn on_mouse_up(&mut self, _event: &MouseUpEvent, _window: &mut Window, cx: &mut App) {
		self.drag_start = None;
		// Notify is handled at FFI level
	}

	/// Handle mouse move (for drag selection).
	pub fn on_mouse_move(&mut self, event: &MouseMoveEvent, _window: &mut Window, cx: &mut App) {
		if event.pressed_button != Some(MouseButton::Left) {
			return;
		}

		if let (Some(drag_start), Some(offset)) =
			(self.drag_start, self.offset_for_position(event.position))
		{
			self.selected_range = Selection::new(drag_start, offset);
			self.selection_reversed = self.selected_range.is_reversed();
			// Notify is handled at FFI level
		}
	}

	/// Handle drag move.
	pub fn on_drag_move(&mut self, event: &MouseMoveEvent, _window: &mut Window, cx: &mut App) {
		self.on_mouse_move(event, _window, cx);
	}

	/// Get byte offset for screen position.
	fn offset_for_position(&self, position: Point<Pixels>) -> Option<usize> {
		let bounds = self.input_bounds.as_ref()?;

		// Simple offset calculation - should be improved with actual text layout
		let relative_x = position.x - bounds.origin.x;
		let relative_y = position.y - bounds.origin.y;

		let line_height = self.last_layout.as_ref().map(|l| l.line_height).unwrap_or(px(20.));

		let row = (relative_y / line_height).max(0.) as usize;
		let row = row.min(self.text.lines_len().saturating_sub(1));

		let line_start = self.text.line_start_offset(row);
		let line = self.text.slice_line(row);
		let line_len = line.len();

		// Estimate column based on x position (simplified)
		let char_width = px(8.); // Approximate
		let col = (relative_x / char_width).max(0.) as usize;
		let col = col.min(line_len);

		Some(line_start + col)
	}

	/// Set input bounds.
	pub fn set_input_bounds(&mut self, bounds: Bounds<Pixels>, _cx: &mut App) {
		self.input_bounds = Some(bounds);
	}

	/// Update scroll offset.
	pub fn update_scroll_offset(&mut self, offset: Option<Point<Pixels>>, _cx: &mut App) {
		if let Some(offset) = offset {
			self.scroll_handle.set_offset(offset);
		}
	}

	/// Insert IME text.
	pub fn insert_ime(&mut self, text: &str, window: &mut Window, cx: &mut App) {
		if let Some(marked_range) = self.ime_marked_range.take() {
			self.replace_text(marked_range, text, window, cx);
		} else {
			self.insert(text, window, cx);
		}
	}

	/// Replace IME marked text.
	pub fn replace_ime(
		&mut self,
		text: &str,
		new_selection: Option<Range<usize>>,
		_window: &mut Window,
		cx: &mut App,
	) {
		let range = self.ime_marked_range.take().unwrap_or_else(|| {
			let cursor = self.cursor();
			cursor..cursor
		});

		// Update text
		self.text.replace(range.clone(), text);

		// Set marked range
		let new_start = range.start;
		let new_end = range.start + text.len();
		self.ime_marked_range = if text.is_empty() { None } else { Some(new_start..new_end) };

		// Update selection within marked text
		if let Some(sel) = new_selection {
			self.selected_range = Selection::new(new_start + sel.start, new_start + sel.end);
		} else {
			self.selected_range = Selection::cursor(new_end);
		}

		self.text_wrapper.set_default_text(&self.text);
		// Notify is handled at FFI level
	}

	/// Delete to beginning of line.
	pub fn delete_to_beginning_of_line(&mut self, window: &mut Window, cx: &mut App) {
		if self.disabled {
			return;
		}
		let cursor = self.cursor();
		let line_start = self.text.line_start_offset(self.text.offset_to_point(cursor).row);
		self.replace_text(line_start..cursor, "", window, cx);
	}

	/// Delete to end of line.
	pub fn delete_to_end_of_line(&mut self, window: &mut Window, cx: &mut App) {
		if self.disabled {
			return;
		}
		let cursor = self.cursor();
		let line_end = self.text.line_end_offset(self.text.offset_to_point(cursor).row);
		self.replace_text(cursor..line_end, "", window, cx);
	}

	/// Delete previous word.
	pub fn delete_prev_word(&mut self, window: &mut Window, cx: &mut App) {
		if self.disabled {
			return;
		}
		let cursor = self.cursor();
		let prev_word = movement::move_to_prev_word(&self.text, &Selection::cursor(cursor));
		self.replace_text(prev_word.end..cursor, "", window, cx);
	}

	/// Delete next word.
	pub fn delete_next_word(&mut self, window: &mut Window, cx: &mut App) {
		if self.disabled {
			return;
		}
		let cursor = self.cursor();
		let next_word = movement::move_to_next_word(&self.text, &Selection::cursor(cursor));
		self.replace_text(cursor..next_word.end, "", window, cx);
	}
}

impl EventEmitter<InputEvent> for InputState {}
