pub mod init;
pub mod intermediate;
pub mod monitor;

use std::{io::Result, os::unix::process::ExitStatusExt, process::ExitStatus};

use crate::{config::Config, constant::EXIT_INTERNAL_FAILURE};
use nix::{
    sys::wait::WaitStatus,
    unistd::{ForkResult, fork},
};

pub struct Sandbox {
    config: Config,
}

impl Sandbox {
    pub fn new(config: Config) -> Self {
        Self { config }
    }
    pub fn create(&self) -> Result<ExitStatus> {
        match unsafe { fork()? } {
            ForkResult::Parent { child, .. } => self.monitor(child),
            ForkResult::Child => self.intermediate(),
        }
    }

    fn waitstatus_to_exitcode(status: WaitStatus) -> i32 {
        // Following the example of bash's man page,
        // the return value of command is just exit status, or 128 + n
        // if the command is terminated by signal n.
        match status {
            WaitStatus::Exited(_, code) => code,
            WaitStatus::Signaled(_, signal, _) => 128 + (signal as i32),
            _ => EXIT_INTERNAL_FAILURE,
        }
    }

    fn waitstatus_to_exitstatus(status: WaitStatus) -> ExitStatus {
        ExitStatus::from_raw(match status {
            WaitStatus::Exited(_, code) => (code & 0xff) << 8,
            WaitStatus::Signaled(_, signal, _) => signal as i32,
            _ => (EXIT_INTERNAL_FAILURE & 0xff) << 8,
        })
    }
}
