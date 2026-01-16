//! Setup wizard infrastructure.
//!
//! This module provides reusable prompt helpers for interactive CLI wizards.
//! Used by tunnel setup, auth setup, and other configuration wizards.

mod prompts;

pub mod auth_wizard;
pub mod cloudflared;
pub mod connectivity;
pub mod tunnel_wizard;

#[allow(unused_imports)]
pub use prompts::{print_error, print_header, print_step, print_success};
