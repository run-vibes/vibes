mod loader;
mod types;

#[allow(unused_imports)] // SaveReport will be used by setup wizards
pub use loader::{ConfigLoader, SaveReport};
pub use types::{DEFAULT_HOST, DEFAULT_PORT, IggyClientConfig, OllamaConfigSection};
// VibesConfig is used by tests in commands/claude.rs
#[allow(unused_imports)]
pub use types::VibesConfig;
