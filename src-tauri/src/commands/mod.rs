// Cron-Insta — Tauri command modules
//
// Each submodule groups related Tauri commands by domain.
// pub use re-exports make all command functions available
// at the crate::commands level for the test module and handler.

pub mod project;
pub mod chapters;
pub mod characters;
pub mod notes;
pub mod places;
pub mod timeline;
pub mod media;
pub mod tramas;
pub mod git;
pub mod config;
pub mod stats;
pub mod export;
pub mod repair;
pub mod shortcuts;
// Re-export all public items so crate::commands::* gives access to commands
pub use project::*;
pub use chapters::*;
pub use characters::*;
pub use notes::*;
pub use places::*;
pub use timeline::*;
pub use media::*;
pub use tramas::*;
pub use git::*;
pub use config::*;
pub use stats::*;
pub use export::*;
pub use repair::*;
pub use shortcuts::*;
