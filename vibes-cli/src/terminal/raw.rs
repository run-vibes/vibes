//! Raw terminal mode handling for PTY I/O

use crossterm::{
    event::{self, Event},
    terminal::{self},
};
use std::io::{self, Write};

/// RAII wrapper for raw terminal mode
///
/// Enables raw mode on construction and restores the previous state on drop.
/// Raw mode disables line buffering and special key processing, allowing
/// direct character-by-character input.
pub struct RawTerminal {
    was_raw: bool,
}

impl RawTerminal {
    /// Enable raw terminal mode
    ///
    /// If the terminal is already in raw mode, this is a no-op.
    /// The previous mode is restored on drop.
    pub fn new() -> io::Result<Self> {
        let was_raw = terminal::is_raw_mode_enabled()?;
        if !was_raw {
            terminal::enable_raw_mode()?;
        }
        Ok(Self { was_raw })
    }

    /// Poll for and read a terminal event with a short timeout
    ///
    /// Returns `None` if no event is available within 10ms.
    pub fn read_event(&self) -> io::Result<Option<Event>> {
        if event::poll(std::time::Duration::from_millis(10))? {
            Ok(Some(event::read()?))
        } else {
            Ok(None)
        }
    }

    /// Write data directly to stdout
    pub fn write(&self, data: &[u8]) -> io::Result<()> {
        let mut stdout = io::stdout();
        stdout.write_all(data)?;
        stdout.flush()?;
        Ok(())
    }

    /// Get the current terminal size (cols, rows)
    pub fn size(&self) -> io::Result<(u16, u16)> {
        terminal::size()
    }
}

impl Drop for RawTerminal {
    fn drop(&mut self) {
        if !self.was_raw {
            let _ = terminal::disable_raw_mode();
        }
    }
}
