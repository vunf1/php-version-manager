pub mod config;
pub mod download;
pub mod install;
pub mod logging;
pub mod manager;
pub mod platform;
pub mod provider;
pub mod state;
pub mod version;

pub use manager::PhpManager;
pub use provider::VersionInfo;
pub use state::{PhpState, InstallMetadata};
pub use version::PhpVersion;

// Re-export platform functions for easier access
pub use platform::{get_php_executable_path, get_current_path, get_path_env_var};
