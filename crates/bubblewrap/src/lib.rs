//! A small, Linux-only library for launching processes in rootless namespaces.

#[cfg(not(target_os = "linux"))]
compile_error!("bubblewrap is only supported on Linux");

mod builder;
mod config;
mod error;
mod namespace;
mod process;
mod rootfs;

pub use builder::{Child, Command};
pub use error::{Error, Result};

/// Arrange for the current process to receive `SIGKILL` when its parent dies.
///
/// This mutates process-wide state and is primarily intended for the command-line
/// frontend. Library users normally only need [`Command::die_with_parent`],
/// which applies the setting to the newly-created sandbox processes.
pub fn die_with_parent() -> Result<()> {
    process::parent_death::set()
}
