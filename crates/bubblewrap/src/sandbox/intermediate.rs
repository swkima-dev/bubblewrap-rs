use std::fs::File;
use std::io::Write;

use crate::constant::EXIT_INTERNAL_FAILURE;
use crate::sandbox::Sandbox;
use nix::libc::{_exit, EXIT_FAILURE};
use nix::sched::{CloneFlags, unshare};
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

        if let Err(_) = unshare(CloneFlags::CLONE_NEWUSER) {
            unsafe { _exit(EXIT_INTERNAL_FAILURE) }
        }

        let uid_map_path = &format!("/proc/self/uid_map");
        let mut uid_map_file = match File::create(&uid_map_path) {
            Err(_) => unsafe { _exit(EXIT_INTERNAL_FAILURE) },
            Ok(file) => file,
        };

        let uid_mapping = String::from("0 1000 1");
        if let Err(_) = uid_map_file.write_all(&uid_mapping.as_bytes()) {
            unsafe { _exit(EXIT_INTERNAL_FAILURE) }
        }

        let setgroups_path = &format!("/proc/self/setgroups");
        let mut setgroups_file = match File::create(&setgroups_path) {
            Err(_) => unsafe { _exit(EXIT_INTERNAL_FAILURE) },
            Ok(file) => file,
        };

        let setgroups_status_deny = String::from("deny");
        if let Err(_) = setgroups_file.write_all(&setgroups_status_deny.as_bytes()) {
            unsafe { _exit(EXIT_INTERNAL_FAILURE) }
        }

        let gid_map_path = &format!("/proc/self/gid_map");
        let mut gid_map_file = match File::create(&gid_map_path) {
            Err(_) => unsafe { _exit(EXIT_INTERNAL_FAILURE) },
            Ok(file) => file,
        };

        let gid_mapping = String::from("0 1000 1");
        if let Err(_) = gid_map_file.write_all(&gid_mapping.as_bytes()) {
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
