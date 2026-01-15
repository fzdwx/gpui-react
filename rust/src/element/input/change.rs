use std::ops::Range;

/// Represents a text change for undo/redo.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Change {
	/// The range in the original text that was replaced.
	pub old_range: Range<usize>,
	/// The old text that was replaced.
	pub old_text:  String,
	/// The new text that replaced it.
	pub new_text:  String,
}

impl Change {
	/// Create a new change.
	pub fn new(old_range: Range<usize>, old_text: String, new_text: String) -> Self {
		Self { old_range, old_text, new_text }
	}

	/// Create the inverse of this change (for undo).
	pub fn inverse(&self) -> Self {
		let new_range_start = self.old_range.start;
		let new_range_end = self.old_range.start + self.new_text.len();

		Self {
			old_range: new_range_start..new_range_end,
			old_text:  self.new_text.clone(),
			new_text:  self.old_text.clone(),
		}
	}

	/// Check if this change can be merged with another.
	pub fn can_merge(&self, other: &Change) -> bool {
		// Can merge if other starts where this ends
		let this_end = self.old_range.start + self.new_text.len();
		other.old_range.start == this_end
			&& self.old_text.is_empty()
			&& other.old_text.is_empty()
			&& !self.new_text.ends_with('\n')
			&& !other.new_text.starts_with('\n')
	}

	/// Merge another change into this one.
	pub fn merge(&mut self, other: Change) { self.new_text.push_str(&other.new_text); }
}

/// History manager for undo/redo.
#[derive(Debug, Clone, Default)]
pub struct History {
	/// Stack of changes for undo.
	undo_stack: Vec<Change>,
	/// Stack of changes for redo.
	redo_stack: Vec<Change>,
	/// Maximum number of changes to keep.
	max_size:   usize,
	/// Whether to group changes together.
	grouping:   bool,
}

impl History {
	/// Create a new history with default max size.
	pub fn new() -> Self {
		Self { undo_stack: Vec::new(), redo_stack: Vec::new(), max_size: 1000, grouping: false }
	}

	/// Create a new history with specified max size.
	pub fn with_max_size(max_size: usize) -> Self {
		Self { undo_stack: Vec::new(), redo_stack: Vec::new(), max_size, grouping: false }
	}

	/// Push a change to the history.
	pub fn push(&mut self, change: Change) {
		// Clear redo stack on new change
		self.redo_stack.clear();

		// Try to merge with last change if grouping
		if self.grouping {
			if let Some(last) = self.undo_stack.last_mut() {
				if last.can_merge(&change) {
					last.merge(change);
					return;
				}
			}
		}

		self.undo_stack.push(change);

		// Trim if over max size
		if self.undo_stack.len() > self.max_size {
			self.undo_stack.remove(0);
		}
	}

	/// Pop the last change for undo.
	pub fn pop_undo(&mut self) -> Option<Change> {
		if let Some(change) = self.undo_stack.pop() {
			let inverse = change.inverse();
			self.redo_stack.push(change);
			Some(inverse)
		} else {
			None
		}
	}

	/// Pop the last change for redo.
	pub fn pop_redo(&mut self) -> Option<Change> {
		if let Some(change) = self.redo_stack.pop() {
			let inverse = change.inverse();
			self.undo_stack.push(change);
			Some(inverse)
		} else {
			None
		}
	}

	/// Check if undo is available.
	pub fn can_undo(&self) -> bool { !self.undo_stack.is_empty() }

	/// Check if redo is available.
	pub fn can_redo(&self) -> bool { !self.redo_stack.is_empty() }

	/// Clear all history.
	pub fn clear(&mut self) {
		self.undo_stack.clear();
		self.redo_stack.clear();
	}

	/// Start grouping changes.
	pub fn start_grouping(&mut self) { self.grouping = true; }

	/// Stop grouping changes.
	pub fn stop_grouping(&mut self) { self.grouping = false; }
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_change_inverse() {
		let change = Change::new(5..10, "hello".to_string(), "world".to_string());
		let inverse = change.inverse();

		assert_eq!(inverse.old_range, 5..10);
		assert_eq!(inverse.old_text, "world");
		assert_eq!(inverse.new_text, "hello");
	}

	#[test]
	fn test_history_undo_redo() {
		let mut history = History::new();

		history.push(Change::new(0..0, "".to_string(), "hello".to_string()));

		assert!(history.can_undo());
		assert!(!history.can_redo());

		let undo = history.pop_undo().unwrap();
		assert_eq!(undo.new_text, "");
		assert_eq!(undo.old_text, "hello");

		assert!(!history.can_undo());
		assert!(history.can_redo());

		let redo = history.pop_redo().unwrap();
		assert_eq!(redo.new_text, "hello");
	}
}
