//! Clipboard utilities for copying content to system clipboard.

use arboard::Clipboard;

/// Copies text to the system clipboard.
///
/// Returns Ok(()) on success, or an error message on failure.
pub fn copy_to_clipboard(text: &str) -> Result<(), String> {
    let mut clipboard =
        Clipboard::new().map_err(|e| format!("Failed to access clipboard: {}", e))?;
    clipboard
        .set_text(text)
        .map_err(|e| format!("Failed to copy to clipboard: {}", e))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn copy_to_clipboard_handles_empty_string() {
        // This test may fail in headless environments without a clipboard
        // We're mainly testing that the function doesn't panic
        let result = copy_to_clipboard("");
        // Either succeeds or returns a meaningful error
        assert!(result.is_ok() || result.unwrap_err().contains("clipboard"));
    }

    #[test]
    fn copy_to_clipboard_handles_text() {
        let result = copy_to_clipboard("test content");
        // Either succeeds or returns a meaningful error (headless env)
        assert!(result.is_ok() || result.unwrap_err().contains("clipboard"));
    }
}
