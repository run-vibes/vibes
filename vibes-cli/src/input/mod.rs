//! Input handling for CLI

mod history;

// TODO: Integrate `InputHistory` into the CLI input pipeline for arrow-key
// command history navigation. This will require crossterm for key detection
// and modification of the session input loop. Currently exported for API
// completeness but not yet wired up.
#[allow(unused_imports)]
pub use history::InputHistory;
