use crate::sandbox::create::Sandbox;
use anyhow::Result;
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

            ForkResult::Child => self.init().unwrap(),
        }
        Ok(())
    }
}
