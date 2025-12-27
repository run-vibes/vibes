mod loader;
mod types;

pub use loader::ConfigLoader;
// VibesConfig is used by tests in commands/claude.rs
#[allow(unused_imports)]
pub use types::VibesConfig;
