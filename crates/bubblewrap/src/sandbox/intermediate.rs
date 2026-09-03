use crate::constant::EXIT_DOMAIN_FAILURE;
use crate::sandbox::Sandbox;
use nix::libc::{_exit, EXIT_FAILURE};
use nix::sys::wait::waitpid;
use nix::unistd::write;
use nix::unistd::{ForkResult, fork};

impl Sandbox {
    pub fn intermediate(&self) -> ! {
        write(
            std::io::stdout(),
            "I'm a new intermediate process\n".as_bytes(),
        )
        .ok();
        match unsafe { fork() } {
            Ok(ForkResult::Parent { child, .. }) => {
                write(
                    std::io::stdout(),
                    "Continuing execution in intermediate process\n".as_bytes(),
                )
                .ok();
                waitpid(child, None).unwrap();
                unsafe { _exit(0) };
            }

            Ok(ForkResult::Child) => {
                if let Err(e) = self.init() {
                    eprintln!("bwrap: init failed: {e}");
                }
                unsafe { _exit(EXIT_FAILURE) };
            }

            Err(_) => unsafe { _exit(EXIT_DOMAIN_FAILURE) },
        }
    }
}
