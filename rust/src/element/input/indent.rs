/// Tab size configuration for indentation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TabSize {
	/// Number of spaces per tab.
	pub size:      usize,
	/// Whether to use hard tabs (\t) instead of spaces.
	pub hard_tabs: bool,
}

impl Default for TabSize {
	fn default() -> Self { Self { size: 4, hard_tabs: false } }
}

impl TabSize {
	/// Create a new TabSize with the given size.
	pub fn new(size: usize) -> Self { Self { size, hard_tabs: false } }

	/// Set whether to use hard tabs.
	pub fn with_hard_tabs(mut self, hard_tabs: bool) -> Self {
		self.hard_tabs = hard_tabs;
		self
	}

	/// Get the indent string for one level.
	pub fn indent_str(&self) -> String {
		if self.hard_tabs { "\t".to_string() } else { " ".repeat(self.size) }
	}

	/// Count the indent level of a line.
	pub fn indent_count(&self, line: &str) -> usize {
		let mut spaces = 0;
		for c in line.chars() {
			match c {
				' ' => spaces += 1,
				'\t' => spaces += self.size,
				_ => break,
			}
		}
		spaces / self.size
	}

	/// Get the leading whitespace of a line.
	pub fn leading_whitespace(line: &str) -> &str {
		let end =
			line.char_indices().find(|(_, c)| !c.is_whitespace()).map(|(i, _)| i).unwrap_or(line.len());
		&line[..end]
	}

	/// Indent a line by one level.
	pub fn indent_line(&self, line: &str) -> String { format!("{}{}", self.indent_str(), line) }

	/// Outdent a line by one level.
	pub fn outdent_line(&self, line: &str) -> String {
		let leading = Self::leading_whitespace(line);
		let rest = &line[leading.len()..];

		if leading.is_empty() {
			return line.to_string();
		}

		// Remove one level of indentation
		let mut removed = 0;
		let mut new_leading = String::new();

		for c in leading.chars() {
			let char_spaces = if c == '\t' { self.size } else { 1 };

			if removed + char_spaces <= self.size {
				removed += char_spaces;
			} else {
				new_leading.push(c);
			}
		}

		format!("{}{}", new_leading, rest)
	}
}

/// Indent multiple lines.
pub fn indent_lines(lines: &str, tab_size: &TabSize) -> String {
	lines.lines().map(|line| tab_size.indent_line(line)).collect::<Vec<_>>().join("\n")
}

/// Outdent multiple lines.
pub fn outdent_lines(lines: &str, tab_size: &TabSize) -> String {
	lines.lines().map(|line| tab_size.outdent_line(line)).collect::<Vec<_>>().join("\n")
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_tab_size() {
		let tab = TabSize::new(4);
		assert_eq!(tab.indent_str(), "    ");

		let hard = TabSize::new(4).with_hard_tabs(true);
		assert_eq!(hard.indent_str(), "\t");
	}

	#[test]
	fn test_indent_count() {
		let tab = TabSize::new(4);
		assert_eq!(tab.indent_count("hello"), 0);
		assert_eq!(tab.indent_count("    hello"), 1);
		assert_eq!(tab.indent_count("        hello"), 2);
		assert_eq!(tab.indent_count("\thello"), 1);
	}

	#[test]
	fn test_indent_line() {
		let tab = TabSize::new(4);
		assert_eq!(tab.indent_line("hello"), "    hello");
	}

	#[test]
	fn test_outdent_line() {
		let tab = TabSize::new(4);
		assert_eq!(tab.outdent_line("    hello"), "hello");
		assert_eq!(tab.outdent_line("hello"), "hello");
		assert_eq!(tab.outdent_line("        hello"), "    hello");
	}
}
