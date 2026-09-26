use crate::constant::EXIT_INTERNAL_FAILURE;
use crate::namespaces;
use crate::sandbox::Sandbox;
use nix::libc::{_exit, EXIT_FAILURE};
use nix::sys::wait::waitpid;
use nix::unistd::{ForkResult, fork, getgid};
use nix::unistd::{getuid, write};

impl Sandbox {
    pub fn intermediate(&self) -> ! {
        write(
            std::io::stdout(),
            "I'm a new intermediate process\n".as_bytes(),
        )
        .ok();

        if !self.config.share_user
            && namespaces::user::apply_user_namespace(
                &self.config.internal_uid.unwrap_or(getuid()),
                &self.config.internal_gid.unwrap_or(getgid()),
            )
            .is_err()
        {
            // TODO: You should use a pipe to propagate Errno errors as std::io::Error.
            unsafe { _exit(EXIT_INTERNAL_FAILURE) }
        }

        match unsafe { fork() } {
            Ok(ForkResult::Parent { child, .. }) => {
                write(
                    std::io::stdout(),
                    "Continuing execution in intermediate process\n".as_bytes(),
                )
                .ok();
                match waitpid(child, None) {
                    Ok(status) => {
                        let exit_code = Self::waitstatus_to_exitcode(status);
                        unsafe { _exit(exit_code) }
                    }
                    Err(_) => unsafe { _exit(EXIT_INTERNAL_FAILURE) },
                }
            }

            Ok(ForkResult::Child) => {
                if let Err(e) = self.init() {
                    eprintln!("bwrap: init failed: {e}");
                }
                unsafe { _exit(EXIT_FAILURE) };
            }

            Err(_) => unsafe { _exit(EXIT_INTERNAL_FAILURE) },
        }
    }
}
