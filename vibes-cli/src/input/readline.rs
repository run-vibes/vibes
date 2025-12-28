//! Readline-like input with history support using crossterm
//!
//! Provides line editing with:
//! - Up/Down arrows for history navigation
//! - Left/Right arrows for cursor movement
//! - Backspace/Delete for character deletion
//! - Home/End for line navigation

use std::io::{self, Stdout, Write};

use crossterm::{
    ExecutableCommand,
    cursor::{self, MoveLeft, MoveRight, MoveToColumn},
    event::{self, Event, KeyCode, KeyEvent, KeyModifiers},
    terminal::{self, ClearType},
};

use super::InputHistory;

/// Result of reading a line
pub enum ReadlineResult {
    /// User entered a line
    Line(String),
    /// User pressed Ctrl+C
    Interrupted,
    /// User pressed Ctrl+D on empty line (EOF)
    Eof,
}

/// Readline-like input handler
pub struct Readline {
    history: InputHistory,
    prompt: String,
}

impl Readline {
    /// Create a new Readline with the given prompt
    pub fn new(prompt: impl Into<String>) -> Self {
        Self {
            history: InputHistory::new(),
            prompt: prompt.into(),
        }
    }

    /// Read a line with history support
    ///
    /// Returns the input line, or signals for Ctrl+C/Ctrl+D.
    pub fn readline(&mut self) -> io::Result<ReadlineResult> {
        let mut stdout = io::stdout();

        // Print prompt
        print!("{}", self.prompt);
        stdout.flush()?;

        // Check if we're in a TTY - if not, use simple line reading
        if !terminal::is_raw_mode_enabled().unwrap_or(false) && !atty::is(atty::Stream::Stdin) {
            return self.read_line_simple();
        }

        // Enable raw mode for key-by-key input
        terminal::enable_raw_mode()?;
        let result = self.read_line_raw(&mut stdout);
        terminal::disable_raw_mode()?;

        // Move to next line after input
        println!();

        result
    }

    /// Simple line reading for non-TTY input
    fn read_line_simple(&mut self) -> io::Result<ReadlineResult> {
        let mut input = String::new();
        match io::stdin().read_line(&mut input) {
            Ok(0) => Ok(ReadlineResult::Eof),
            Ok(_) => {
                let trimmed = input.trim_end().to_string();
                if !trimmed.is_empty() {
                    self.history.push(trimmed.clone());
                }
                Ok(ReadlineResult::Line(trimmed))
            }
            Err(e) => Err(e),
        }
    }

    /// Read line in raw mode with full editing support
    fn read_line_raw(&mut self, stdout: &mut Stdout) -> io::Result<ReadlineResult> {
        let mut buffer = String::new();
        let mut cursor_pos: usize = 0;
        let prompt_len = self.prompt.len() as u16;

        loop {
            if let Event::Key(key_event) = event::read()? {
                match key_event {
                    // Ctrl+C - interrupt
                    KeyEvent {
                        code: KeyCode::Char('c'),
                        modifiers: KeyModifiers::CONTROL,
                        ..
                    } => {
                        return Ok(ReadlineResult::Interrupted);
                    }

                    // Ctrl+D - EOF (only on empty line)
                    KeyEvent {
                        code: KeyCode::Char('d'),
                        modifiers: KeyModifiers::CONTROL,
                        ..
                    } => {
                        if buffer.is_empty() {
                            return Ok(ReadlineResult::Eof);
                        }
                    }

                    // Enter - submit line
                    KeyEvent {
                        code: KeyCode::Enter,
                        ..
                    } => {
                        let line = buffer.clone();
                        if !line.is_empty() {
                            self.history.push(line.clone());
                        }
                        return Ok(ReadlineResult::Line(line));
                    }

                    // Up arrow - previous history
                    KeyEvent {
                        code: KeyCode::Up, ..
                    } => {
                        if let Some(prev) = self.history.navigate_up(&buffer).map(|s| s.to_string())
                        {
                            self.replace_line(
                                stdout,
                                &mut buffer,
                                &mut cursor_pos,
                                &prev,
                                prompt_len,
                            )?;
                        }
                    }

                    // Down arrow - next history
                    KeyEvent {
                        code: KeyCode::Down,
                        ..
                    } => {
                        if let Some(next) = self.history.navigate_down().map(|s| s.to_string()) {
                            self.replace_line(
                                stdout,
                                &mut buffer,
                                &mut cursor_pos,
                                &next,
                                prompt_len,
                            )?;
                        }
                    }

                    // Left arrow - move cursor left
                    KeyEvent {
                        code: KeyCode::Left,
                        ..
                    } => {
                        if cursor_pos > 0 {
                            cursor_pos -= 1;
                            stdout.execute(MoveLeft(1))?;
                        }
                    }

                    // Right arrow - move cursor right
                    KeyEvent {
                        code: KeyCode::Right,
                        ..
                    } => {
                        if cursor_pos < buffer.len() {
                            cursor_pos += 1;
                            stdout.execute(MoveRight(1))?;
                        }
                    }

                    // Home - move to start
                    KeyEvent {
                        code: KeyCode::Home,
                        ..
                    } => {
                        cursor_pos = 0;
                        stdout.execute(MoveToColumn(prompt_len))?;
                    }

                    // End - move to end
                    KeyEvent {
                        code: KeyCode::End, ..
                    } => {
                        cursor_pos = buffer.len();
                        stdout.execute(MoveToColumn(prompt_len + buffer.len() as u16))?;
                    }

                    // Backspace - delete char before cursor
                    KeyEvent {
                        code: KeyCode::Backspace,
                        ..
                    } => {
                        if cursor_pos > 0 {
                            buffer.remove(cursor_pos - 1);
                            cursor_pos -= 1;
                            self.redraw_from_cursor(stdout, &buffer, cursor_pos, prompt_len)?;
                        }
                    }

                    // Delete - delete char at cursor
                    KeyEvent {
                        code: KeyCode::Delete,
                        ..
                    } => {
                        if cursor_pos < buffer.len() {
                            buffer.remove(cursor_pos);
                            self.redraw_from_cursor(stdout, &buffer, cursor_pos, prompt_len)?;
                        }
                    }

                    // Regular character input
                    KeyEvent {
                        code: KeyCode::Char(c),
                        modifiers,
                        ..
                    } if !modifiers.contains(KeyModifiers::CONTROL) => {
                        buffer.insert(cursor_pos, c);
                        cursor_pos += 1;

                        if cursor_pos == buffer.len() {
                            // Simple case: appending at end
                            print!("{}", c);
                            stdout.flush()?;
                        } else {
                            // Inserting in middle: need to redraw
                            self.redraw_from_cursor(stdout, &buffer, cursor_pos, prompt_len)?;
                        }
                    }

                    _ => {}
                }
            }
        }
    }

    /// Replace the entire line with new content
    fn replace_line(
        &self,
        stdout: &mut Stdout,
        buffer: &mut String,
        cursor_pos: &mut usize,
        new_content: &str,
        prompt_len: u16,
    ) -> io::Result<()> {
        // Move to start of input
        stdout.execute(MoveToColumn(prompt_len))?;
        // Clear to end of line
        stdout.execute(terminal::Clear(ClearType::UntilNewLine))?;
        // Write new content
        print!("{}", new_content);
        stdout.flush()?;

        *buffer = new_content.to_string();
        *cursor_pos = buffer.len();

        Ok(())
    }

    /// Redraw from cursor position to end of line
    fn redraw_from_cursor(
        &self,
        stdout: &mut Stdout,
        buffer: &str,
        cursor_pos: usize,
        prompt_len: u16,
    ) -> io::Result<()> {
        // Save cursor position
        stdout.execute(cursor::SavePosition)?;
        // Move to current position
        stdout.execute(MoveToColumn(prompt_len + cursor_pos as u16))?;
        // Clear to end
        stdout.execute(terminal::Clear(ClearType::UntilNewLine))?;
        // Print remaining text
        print!("{}", &buffer[cursor_pos..]);
        stdout.flush()?;
        // Restore cursor
        stdout.execute(cursor::RestorePosition)?;

        // Actually move cursor to correct position
        stdout.execute(MoveToColumn(prompt_len + cursor_pos as u16))?;

        Ok(())
    }
}

/// Check if stdin is a TTY
mod atty {
    pub enum Stream {
        Stdin,
    }

    pub fn is(stream: Stream) -> bool {
        match stream {
            Stream::Stdin => {
                #[cfg(unix)]
                {
                    unsafe { libc::isatty(libc::STDIN_FILENO) != 0 }
                }
                #[cfg(windows)]
                {
                    // On Windows, assume TTY for now
                    true
                }
                #[cfg(not(any(unix, windows)))]
                {
                    true
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn readline_new_creates_with_prompt() {
        let rl = Readline::new("> ");
        assert_eq!(rl.prompt, "> ");
    }

    #[test]
    fn readline_history_starts_empty() {
        let rl = Readline::new("> ");
        assert!(rl.history.is_empty());
    }
}
