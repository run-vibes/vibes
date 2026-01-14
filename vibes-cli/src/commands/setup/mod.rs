//! Setup wizard infrastructure.
//!
//! This module provides reusable prompt helpers for interactive CLI wizards.
//! Used by tunnel setup, auth setup, and other configuration wizards.

// These functions are infrastructure for future wizard commands.
// They will be used once tunnel/auth setup wizards are implemented.
#[allow(unused)]
mod prompts;

#[allow(unused_imports)]
pub use prompts::{
    print_error, print_error_to, print_header, print_header_to, print_step, print_step_to,
    print_success, print_success_to,
};
