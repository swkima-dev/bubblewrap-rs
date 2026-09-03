use std::{io::Result, process::ExitStatus};

use nix::{sys::wait::waitpid, unistd::Pid};

use crate::sandbox::Sandbox;

impl Sandbox {
    pub fn monitor(&self, child: Pid) -> Result<ExitStatus> {
        println!(
            "Continuing execution in parent process, new child has pid: {}",
            child
        );
        match waitpid(child, None) {
            Ok(status) => Ok(Self::waitstatus_to_exitstatus(status)),
            Err(e) => Err(e.into()),
        }
    }
}
