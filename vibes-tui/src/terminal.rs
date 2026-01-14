//! Terminal setup and teardown for vibes TUI.

use std::io::{self, Stdout};
use std::panic;

use crossterm::{
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{Terminal, backend::CrosstermBackend};

/// The terminal type used throughout the TUI.
pub type VibesTerminal = Terminal<CrosstermBackend<Stdout>>;

/// Sets up the terminal for TUI rendering.
///
/// Enables raw mode and enters the alternate screen. The returned terminal
/// should be passed to `restore_terminal` on exit to clean up properly.
pub fn setup_terminal() -> io::Result<VibesTerminal> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    Terminal::new(backend)
}

/// Restores the terminal to its normal state.
///
/// Disables raw mode and leaves the alternate screen. Should be called
/// on exit and in panic hooks to avoid leaving the terminal in a bad state.
pub fn restore_terminal(terminal: &mut VibesTerminal) -> io::Result<()> {
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    Ok(())
}

/// Installs a panic hook that restores the terminal on crash.
///
/// This ensures the terminal is returned to a usable state even if the
/// application panics. Should be called once at startup before entering
/// the TUI.
pub fn install_panic_hook() {
    let original_hook = panic::take_hook();

    panic::set_hook(Box::new(move |panic_info| {
        // Best-effort terminal restoration - ignore errors
        let _ = disable_raw_mode();
        let _ = execute!(io::stdout(), LeaveAlternateScreen);

        // Call the original panic hook to print the panic message
        original_hook(panic_info);
    }));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn vibes_terminal_type_alias_compiles() {
        // This test verifies the type alias is correctly defined.
        // We can't actually create a terminal in tests without a real TTY.
        fn _accepts_terminal(_t: &VibesTerminal) {}
    }

    #[test]
    fn setup_and_restore_functions_have_correct_signatures() {
        // Verify function signatures compile correctly.
        // Actual terminal tests require integration testing with a real TTY.
        fn _check_setup() -> io::Result<VibesTerminal> {
            setup_terminal()
        }

        fn _check_restore(t: &mut VibesTerminal) -> io::Result<()> {
            restore_terminal(t)
        }
    }

    #[test]
    fn install_panic_hook_function_exists() {
        // Verify the function compiles. We don't actually call it in tests
        // because it modifies global state (the panic hook).
        fn _check_install() {
            install_panic_hook()
        }
    }
}
