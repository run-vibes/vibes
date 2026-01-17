//! Command input state management.

/// Manages command-mode input state.
#[derive(Debug, Default, Clone)]
pub struct CommandInput {
    /// Current input buffer (without the leading ':').
    pub buffer: String,
    /// Cursor position within buffer.
    pub cursor: usize,
    /// Current completion candidates.
    pub completions: Vec<String>,
    /// Selected completion index (None = no selection).
    pub completion_idx: Option<usize>,
    /// Result message to display after command executes.
    /// Tuple of (message, is_error).
    pub message: Option<(String, bool)>,
}

impl CommandInput {
    /// Insert a character at the cursor position.
    pub fn insert(&mut self, c: char) {
        self.buffer.insert(self.cursor, c);
        self.cursor += 1;
    }

    /// Delete the character before the cursor.
    pub fn backspace(&mut self) {
        if self.cursor > 0 {
            self.cursor -= 1;
            self.buffer.remove(self.cursor);
        }
    }

    /// Delete the character at the cursor.
    pub fn delete(&mut self) {
        if self.cursor < self.buffer.len() {
            self.buffer.remove(self.cursor);
        }
    }

    /// Move cursor left.
    pub fn move_left(&mut self) {
        if self.cursor > 0 {
            self.cursor -= 1;
        }
    }

    /// Move cursor right.
    pub fn move_right(&mut self) {
        if self.cursor < self.buffer.len() {
            self.cursor += 1;
        }
    }

    /// Move cursor to start of buffer.
    pub fn move_to_start(&mut self) {
        self.cursor = 0;
    }

    /// Move cursor to end of buffer.
    pub fn move_to_end(&mut self) {
        self.cursor = self.buffer.len();
    }

    /// Clear all input state.
    pub fn clear(&mut self) {
        self.buffer.clear();
        self.cursor = 0;
        self.completions.clear();
        self.completion_idx = None;
        self.message = None;
    }

    /// Set completion candidates.
    pub fn set_completions(&mut self, completions: Vec<String>) {
        self.completions = completions;
        self.completion_idx = None;
    }

    /// Cycle to the next completion.
    pub fn next_completion(&mut self) {
        if self.completions.is_empty() {
            return;
        }

        self.completion_idx = Some(match self.completion_idx {
            None => 0,
            Some(idx) => (idx + 1) % self.completions.len(),
        });
    }

    /// Accept the current completion into the buffer.
    pub fn accept_completion(&mut self) {
        if let Some(idx) = self.completion_idx
            && let Some(completion) = self.completions.get(idx)
        {
            self.buffer = format!("{} ", completion);
            self.cursor = self.buffer.len();
            self.completions.clear();
            self.completion_idx = None;
        }
    }

    /// Get the currently selected completion.
    pub fn current_completion(&self) -> Option<&str> {
        self.completion_idx
            .and_then(|idx| self.completions.get(idx))
            .map(|s| s.as_str())
    }

    /// Set a message to display.
    pub fn set_message(&mut self, msg: impl Into<String>, is_error: bool) {
        self.message = Some((msg.into(), is_error));
    }

    /// Clear the message.
    pub fn clear_message(&mut self) {
        self.message = None;
    }
}

#[cfg(test)]
#[allow(clippy::field_reassign_with_default)]
mod tests {
    use super::*;

    #[test]
    fn command_input_default_is_empty() {
        let input = CommandInput::default();
        assert!(input.buffer.is_empty());
        assert_eq!(input.cursor, 0);
        assert!(input.completions.is_empty());
        assert!(input.completion_idx.is_none());
        assert!(input.message.is_none());
    }

    // === Insert tests ===

    #[test]
    fn insert_adds_char_at_cursor() {
        let mut input = CommandInput::default();
        input.insert('h');
        input.insert('e');
        input.insert('l');
        input.insert('l');
        input.insert('o');
        assert_eq!(input.buffer, "hello");
        assert_eq!(input.cursor, 5);
    }

    #[test]
    fn insert_in_middle_of_buffer() {
        let mut input = CommandInput::default();
        input.buffer = "helo".into();
        input.cursor = 3;
        input.insert('l');
        assert_eq!(input.buffer, "hello");
        assert_eq!(input.cursor, 4);
    }

    // === Backspace tests ===

    #[test]
    fn backspace_removes_char_before_cursor() {
        let mut input = CommandInput::default();
        input.buffer = "hello".into();
        input.cursor = 5;
        input.backspace();
        assert_eq!(input.buffer, "hell");
        assert_eq!(input.cursor, 4);
    }

    #[test]
    fn backspace_at_start_does_nothing() {
        let mut input = CommandInput::default();
        input.buffer = "hello".into();
        input.cursor = 0;
        input.backspace();
        assert_eq!(input.buffer, "hello");
        assert_eq!(input.cursor, 0);
    }

    #[test]
    fn backspace_in_middle() {
        let mut input = CommandInput::default();
        input.buffer = "hello".into();
        input.cursor = 3;
        input.backspace();
        assert_eq!(input.buffer, "helo");
        assert_eq!(input.cursor, 2);
    }

    // === Delete tests ===

    #[test]
    fn delete_removes_char_at_cursor() {
        let mut input = CommandInput::default();
        input.buffer = "hello".into();
        input.cursor = 0;
        input.delete();
        assert_eq!(input.buffer, "ello");
        assert_eq!(input.cursor, 0);
    }

    #[test]
    fn delete_at_end_does_nothing() {
        let mut input = CommandInput::default();
        input.buffer = "hello".into();
        input.cursor = 5;
        input.delete();
        assert_eq!(input.buffer, "hello");
        assert_eq!(input.cursor, 5);
    }

    // === Cursor movement tests ===

    #[test]
    fn move_left_decrements_cursor() {
        let mut input = CommandInput::default();
        input.buffer = "hello".into();
        input.cursor = 3;
        input.move_left();
        assert_eq!(input.cursor, 2);
    }

    #[test]
    fn move_left_at_start_does_nothing() {
        let mut input = CommandInput::default();
        input.buffer = "hello".into();
        input.cursor = 0;
        input.move_left();
        assert_eq!(input.cursor, 0);
    }

    #[test]
    fn move_right_increments_cursor() {
        let mut input = CommandInput::default();
        input.buffer = "hello".into();
        input.cursor = 2;
        input.move_right();
        assert_eq!(input.cursor, 3);
    }

    #[test]
    fn move_right_at_end_does_nothing() {
        let mut input = CommandInput::default();
        input.buffer = "hello".into();
        input.cursor = 5;
        input.move_right();
        assert_eq!(input.cursor, 5);
    }

    #[test]
    fn move_to_start() {
        let mut input = CommandInput::default();
        input.buffer = "hello".into();
        input.cursor = 3;
        input.move_to_start();
        assert_eq!(input.cursor, 0);
    }

    #[test]
    fn move_to_end() {
        let mut input = CommandInput::default();
        input.buffer = "hello".into();
        input.cursor = 2;
        input.move_to_end();
        assert_eq!(input.cursor, 5);
    }

    // === Clear tests ===

    #[test]
    fn clear_resets_all_state() {
        let mut input = CommandInput::default();
        input.buffer = "hello".into();
        input.cursor = 3;
        input.completions = vec!["one".into(), "two".into()];
        input.completion_idx = Some(1);
        input.message = Some(("test".into(), false));

        input.clear();

        assert!(input.buffer.is_empty());
        assert_eq!(input.cursor, 0);
        assert!(input.completions.is_empty());
        assert!(input.completion_idx.is_none());
        assert!(input.message.is_none());
    }

    // === Completion tests ===

    #[test]
    fn set_completions_stores_candidates() {
        let mut input = CommandInput::default();
        input.set_completions(vec!["theme".into(), "quit".into()]);
        assert_eq!(input.completions.len(), 2);
        assert!(input.completion_idx.is_none());
    }

    #[test]
    fn next_completion_cycles_through() {
        let mut input = CommandInput::default();
        input.completions = vec!["one".into(), "two".into(), "three".into()];

        input.next_completion();
        assert_eq!(input.completion_idx, Some(0));

        input.next_completion();
        assert_eq!(input.completion_idx, Some(1));

        input.next_completion();
        assert_eq!(input.completion_idx, Some(2));

        input.next_completion();
        assert_eq!(input.completion_idx, Some(0)); // Wraps around
    }

    #[test]
    fn next_completion_with_empty_list_does_nothing() {
        let mut input = CommandInput::default();
        input.next_completion();
        assert!(input.completion_idx.is_none());
    }

    #[test]
    fn accept_completion_replaces_buffer() {
        let mut input = CommandInput::default();
        input.buffer = "the".into();
        input.cursor = 3;
        input.completions = vec!["theme".into(), "three".into()];
        input.completion_idx = Some(0);

        input.accept_completion();

        assert_eq!(input.buffer, "theme ");
        assert_eq!(input.cursor, 6);
        assert!(input.completions.is_empty());
        assert!(input.completion_idx.is_none());
    }

    #[test]
    fn accept_completion_with_no_selection_does_nothing() {
        let mut input = CommandInput::default();
        input.buffer = "the".into();
        input.cursor = 3;
        input.completions = vec!["theme".into()];
        // No selection

        input.accept_completion();

        assert_eq!(input.buffer, "the");
    }

    #[test]
    fn current_completion_returns_selected() {
        let mut input = CommandInput::default();
        input.completions = vec!["one".into(), "two".into()];
        input.completion_idx = Some(1);

        assert_eq!(input.current_completion(), Some("two"));
    }

    // === Message tests ===

    #[test]
    fn set_message_stores_success() {
        let mut input = CommandInput::default();
        input.set_message("Done!", false);
        assert_eq!(input.message, Some(("Done!".into(), false)));
    }

    #[test]
    fn set_message_stores_error() {
        let mut input = CommandInput::default();
        input.set_message("Failed!", true);
        assert_eq!(input.message, Some(("Failed!".into(), true)));
    }

    #[test]
    fn clear_message_removes_message() {
        let mut input = CommandInput::default();
        input.message = Some(("test".into(), false));
        input.clear_message();
        assert!(input.message.is_none());
    }
}
