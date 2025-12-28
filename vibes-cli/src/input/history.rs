//! In-memory input history for up/down navigation

/// Manages input history for CLI sessions
///
/// TODO: Integrate with CLI input loop using crossterm for arrow key detection.
/// This struct is currently prepared but not yet integrated into the CLI's
/// input loop. Integration will require terminal raw mode handling.
#[allow(dead_code)]
#[derive(Debug, Default)]
pub struct InputHistory {
    /// Previous inputs
    entries: Vec<String>,
    /// Current navigation position (None = not navigating)
    position: Option<usize>,
    /// Draft input when starting navigation
    draft: String,
}

#[allow(dead_code)]
impl InputHistory {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Add input to history
    pub fn push(&mut self, input: String) {
        // Don't add duplicates consecutively
        if self.entries.last() != Some(&input) {
            self.entries.push(input);
        }
        self.position = None;
    }

    /// Navigate up (older entries)
    pub fn navigate_up(&mut self, current: &str) -> Option<&str> {
        match self.position {
            None if !self.entries.is_empty() => {
                self.draft = current.to_string();
                self.position = Some(self.entries.len() - 1);
                self.entries.last().map(|s| s.as_str())
            }
            Some(0) => None,
            Some(pos) => {
                self.position = Some(pos - 1);
                Some(&self.entries[pos - 1])
            }
            _ => None,
        }
    }

    /// Navigate down (newer entries, back to draft)
    pub fn navigate_down(&mut self) -> Option<&str> {
        match self.position {
            None => None,
            Some(pos) if pos >= self.entries.len() - 1 => {
                self.position = None;
                Some(&self.draft)
            }
            Some(pos) => {
                self.position = Some(pos + 1);
                Some(&self.entries[pos + 1])
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_history_is_empty() {
        let history = InputHistory::new();
        assert!(history.entries.is_empty());
        assert!(history.position.is_none());
    }

    #[test]
    fn push_adds_entry() {
        let mut history = InputHistory::new();
        history.push("first".to_string());
        history.push("second".to_string());
        assert_eq!(history.len(), 2);
    }

    #[test]
    fn push_deduplicates_consecutive() {
        let mut history = InputHistory::new();
        history.push("same".to_string());
        history.push("same".to_string());
        assert_eq!(history.len(), 1);
    }

    #[test]
    fn navigate_up_returns_previous() {
        let mut history = InputHistory::new();
        history.push("first".to_string());
        history.push("second".to_string());

        assert_eq!(history.navigate_up("current"), Some("second"));
        assert_eq!(history.navigate_up("current"), Some("first"));
        assert_eq!(history.navigate_up("current"), None); // No more
    }

    #[test]
    fn navigate_down_returns_next() {
        let mut history = InputHistory::new();
        history.push("first".to_string());
        history.push("second".to_string());

        history.navigate_up("draft");
        history.navigate_up("draft");

        assert_eq!(history.navigate_down(), Some("second"));
        assert_eq!(history.navigate_down(), Some("draft")); // Returns to draft
        assert_eq!(history.navigate_down(), None);
    }

    #[test]
    fn push_resets_navigation() {
        let mut history = InputHistory::new();
        history.push("first".to_string());
        history.navigate_up("draft");

        history.push("new".to_string());

        assert!(history.position.is_none());
    }
}
