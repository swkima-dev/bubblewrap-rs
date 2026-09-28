use std::{convert::Infallible, ffi::CString, io::Result, os::unix::ffi::OsStrExt};

use crate::{constant::EXIT_INTERNAL_FAILURE, sandbox::Sandbox};
use nix::{
    libc::_exit,
    sys::wait::waitpid,
    unistd::{ForkResult, execve, fork},
};

impl Sandbox {
    pub fn init(&self) -> ! {
        match self.init_inner() {
            Ok(never) => match never {},
            Err(e) => {
                eprintln!("bwrap: execve failed: {e}");
                unsafe { _exit(EXIT_INTERNAL_FAILURE) }
            }
        }
    }

    fn init_inner(&self) -> Result<Infallible> {
        let program = CString::new(self.config.program.as_bytes())?;
        let mut args = vec![program.clone()];
        for arg in &self.config.args {
            args.push(CString::new(arg.as_bytes())?);
        }
        let env: Vec<CString> = Vec::new();
        // TODO: Drop capabilities before execve. Upstream bwrap drops all of them
        // by default (unless --cap-add is given), but here the program keeps every
        // capability in the new user namespace when mapped to uid 0 (e.g. --uid 0),
        // and root keeps its capabilities in the host namespace with share_user.
        // Clear the bounding and ambient sets as well and set PR_SET_NO_NEW_PRIVS.
        // `execve` returns `Result<Infallible>`: on success the process image is
        // replaced, so the only way it returns is with an error.
        match unsafe { fork() } {
            Ok(ForkResult::Parent { child, .. }) => match waitpid(child, None) {
                Ok(status) => {
                    let exit_code = Self::waitstatus_to_exitcode(status);
                    unsafe { _exit(exit_code) }
                }
                Err(_) => unsafe { _exit(EXIT_INTERNAL_FAILURE) },
            },
            Ok(ForkResult::Child) => {
                execve(&program, &args, &env)?;
                unsafe { _exit(EXIT_INTERNAL_FAILURE) }
            }
            Err(_) => unsafe {
                _exit(EXIT_INTERNAL_FAILURE);
            },
        }
    }
}
