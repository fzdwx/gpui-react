use super::TabSize;

/// Input mode configuration.
#[derive(Debug, Clone)]
pub enum InputMode {
    /// Plain text input.
    PlainText {
        /// Whether multi-line is enabled.
        multi_line: bool,
        /// Tab size for indentation.
        tab: TabSize,
        /// Number of visible rows (for multi-line).
        rows: usize,
    },
    /// Auto-growing text area.
    AutoGrow {
        /// Current number of rows.
        rows: usize,
        /// Minimum rows.
        min_rows: usize,
        /// Maximum rows.
        max_rows: usize,
        /// Tab size.
        tab: TabSize,
    },
    /// Code editor mode with syntax highlighting.
    CodeEditor {
        /// Whether multi-line is enabled.
        multi_line: bool,
        /// Tab size.
        tab: TabSize,
        /// Number of visible rows.
        rows: usize,
        /// Whether to show line numbers.
        line_number: bool,
        /// Language for syntax highlighting.
        language: Option<String>,
    },
}

impl Default for InputMode {
    fn default() -> Self {
        Self::plain_text()
    }
}

impl InputMode {
    /// Create a plain text single-line input.
    pub fn plain_text() -> Self {
        Self::PlainText {
            multi_line: false,
            tab: TabSize::default(),
            rows: 1,
        }
    }

    /// Create a multi-line plain text input.
    pub fn multi_line() -> Self {
        Self::PlainText {
            multi_line: true,
            tab: TabSize::default(),
            rows: 3,
        }
    }

    /// Create a multi-line input with specified rows.
    pub fn multi_line_with_rows(rows: usize) -> Self {
        Self::PlainText {
            multi_line: true,
            tab: TabSize::default(),
            rows,
        }
    }

    /// Create an auto-growing text area.
    pub fn auto_grow(min_rows: usize, max_rows: usize) -> Self {
        Self::AutoGrow {
            rows: min_rows,
            min_rows,
            max_rows,
            tab: TabSize::default(),
        }
    }

    /// Create a code editor.
    pub fn code_editor(language: Option<String>) -> Self {
        Self::CodeEditor {
            multi_line: true,
            tab: TabSize::default(),
            rows: 10,
            line_number: true,
            language,
        }
    }

    /// Check if this is a single-line input.
    pub fn is_single_line(&self) -> bool {
        match self {
            Self::PlainText { multi_line, .. } => !multi_line,
            Self::AutoGrow { .. } => false,
            Self::CodeEditor { multi_line, .. } => !multi_line,
        }
    }

    /// Check if this is a multi-line input.
    pub fn is_multi_line(&self) -> bool {
        !self.is_single_line()
    }

    /// Check if this is auto-grow mode.
    pub fn is_auto_grow(&self) -> bool {
        matches!(self, Self::AutoGrow { .. })
    }

    /// Check if this is code editor mode.
    pub fn is_code_editor(&self) -> bool {
        matches!(self, Self::CodeEditor { .. })
    }

    /// Get the tab size.
    pub fn tab_size(&self) -> &TabSize {
        match self {
            Self::PlainText { tab, .. } => tab,
            Self::AutoGrow { tab, .. } => tab,
            Self::CodeEditor { tab, .. } => tab,
        }
    }

    /// Get the number of rows.
    pub fn rows(&self) -> usize {
        match self {
            Self::PlainText { rows, .. } => *rows,
            Self::AutoGrow { rows, .. } => *rows,
            Self::CodeEditor { rows, .. } => *rows,
        }
    }

    /// Get max rows (for auto-grow).
    pub fn max_rows(&self) -> usize {
        match self {
            Self::AutoGrow { max_rows, .. } => *max_rows,
            _ => self.rows(),
        }
    }

    /// Get min rows (for auto-grow).
    pub fn min_rows(&self) -> usize {
        match self {
            Self::AutoGrow { min_rows, .. } => *min_rows,
            _ => self.rows(),
        }
    }

    /// Set the number of rows (for auto-grow).
    pub fn set_rows(&mut self, new_rows: usize) {
        match self {
            Self::PlainText { rows, .. } => *rows = new_rows,
            Self::AutoGrow {
                rows,
                min_rows,
                max_rows,
                ..
            } => {
                *rows = new_rows.max(*min_rows).min(*max_rows);
            }
            Self::CodeEditor { rows, .. } => *rows = new_rows,
        }
    }

    /// Check if line numbers should be shown.
    pub fn line_number(&self) -> bool {
        match self {
            Self::CodeEditor { line_number, .. } => *line_number,
            _ => false,
        }
    }

    /// Get the language for syntax highlighting.
    pub fn language(&self) -> Option<&str> {
        match self {
            Self::CodeEditor { language, .. } => language.as_deref(),
            _ => None,
        }
    }

    /// Enable or disable multi-line.
    pub fn with_multi_line(mut self, enabled: bool) -> Self {
        match &mut self {
            Self::PlainText { multi_line, .. } => *multi_line = enabled,
            Self::CodeEditor { multi_line, .. } => *multi_line = enabled,
            _ => {}
        }
        self
    }

    /// Set tab size.
    pub fn with_tab_size(mut self, tab_size: TabSize) -> Self {
        match &mut self {
            Self::PlainText { tab, .. } => *tab = tab_size,
            Self::AutoGrow { tab, .. } => *tab = tab_size,
            Self::CodeEditor { tab, .. } => *tab = tab_size,
        }
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plain_text() {
        let mode = InputMode::plain_text();
        assert!(mode.is_single_line());
        assert!(!mode.is_multi_line());
    }

    #[test]
    fn test_auto_grow() {
        let mut mode = InputMode::auto_grow(2, 10);
        assert!(mode.is_auto_grow());
        assert_eq!(mode.min_rows(), 2);
        assert_eq!(mode.max_rows(), 10);

        mode.set_rows(5);
        assert_eq!(mode.rows(), 5);

        mode.set_rows(1); // Below min
        assert_eq!(mode.rows(), 2);

        mode.set_rows(20); // Above max
        assert_eq!(mode.rows(), 10);
    }

    #[test]
    fn test_code_editor() {
        let mode = InputMode::code_editor(Some("rust".to_string()));
        assert!(mode.is_code_editor());
        assert!(mode.line_number());
        assert_eq!(mode.language(), Some("rust"));
    }
}
