mod loader;
mod types;

pub use loader::ConfigLoader;
pub use types::{DEFAULT_HOST, DEFAULT_PORT, IggyClientConfig, OllamaConfigSection};
// VibesConfig is used by tests in commands/claude.rs
#[allow(unused_imports)]
pub use types::VibesConfig;
