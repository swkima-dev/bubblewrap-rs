use anyhow::Result;
use nix::unistd::{ForkResult, fork};

pub struct Sandbox {}

impl Sandbox {
    pub fn new() -> Self {
        Self {}
    }
    pub fn create(&self) -> Result<()> {
        match unsafe { fork()? } {
            ForkResult::Parent { child, .. } => {
                self.monitor(child);
            }
            ForkResult::Child => {
                self.intermediate();
            }
        }

        Ok(())
    }
}
