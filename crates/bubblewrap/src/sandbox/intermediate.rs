use crate::sandbox::Sandbox;
use anyhow::Result;
use nix::libc::{_exit, EXIT_FAILURE};
use nix::sys::wait::waitpid;
use nix::unistd::{ForkResult, fork};
use nix::{libc::exit, unistd::write};

impl Sandbox {
    pub fn intermediate(&self) -> Result<()> {
        write(
            std::io::stdout(),
            "I'm a new intermediate process\n".as_bytes(),
        )
        .ok();
        match unsafe { fork()? } {
            ForkResult::Parent { child, .. } => {
                write(
                    std::io::stdout(),
                    "Continuing execution in intermediate process\n".as_bytes(),
                )
                .ok();
                waitpid(child, None).unwrap();
                unsafe { exit(0) };
            }

            ForkResult::Child => {
                if let Err(e) = self.init() {
                    eprintln!("bwrap: init failed: {e}");
                    unsafe { _exit(EXIT_FAILURE) };
                }
            }
        }
        Ok(())
    }
}
