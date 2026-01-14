//! Setup wizard infrastructure.
//!
//! This module provides reusable prompt helpers for interactive CLI wizards.
//! Used by tunnel setup, auth setup, and other configuration wizards.

mod prompts;

pub mod cloudflared;
pub mod tunnel_wizard;

pub use prompts::{print_error, print_header, print_step, print_success};
