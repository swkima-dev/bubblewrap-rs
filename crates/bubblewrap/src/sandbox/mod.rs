pub mod init;
pub mod intermediate;
pub mod monitor;

use std::io::Result;

use crate::config::Config;
use nix::unistd::{ForkResult, fork};

pub struct Sandbox {
    config: Config,
}

impl Sandbox {
    pub fn new(config: Config) -> Self {
        Self { config }
    }
    pub fn create(&self) -> Result<()> {
        match unsafe { fork()? } {
            ForkResult::Parent { child, .. } => {
                self.monitor(child);
            }
            ForkResult::Child => self.intermediate()?,
        }

        Ok(())
    }
}
