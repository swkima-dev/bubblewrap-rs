use std::{convert::Infallible, ffi::CString, io::Result, os::unix::ffi::OsStrExt};

use crate::{constant::EXIT_INTERNAL_FAILURE, sandbox::Sandbox};
use nix::{
    errno::Errno,
    libc::_exit,
    sched::CloneFlags,
    sys::wait::{WaitStatus, waitpid},
    unistd::{ForkResult, Pid, execve, fork},
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
            Ok(ForkResult::Parent { child, .. }) => {
                let exit_code = if self.config.namespaces.contains(CloneFlags::CLONE_NEWPID) {
                    Self::reap_orphan(child)
                } else {
                    Self::wait_child(child)
                };
                match exit_code {
                    Ok(status) => {
                        let exit_code = Self::waitstatus_to_exitcode(status);
                        unsafe { _exit(exit_code) }
                    }
                    Err(_) => unsafe { _exit(EXIT_INTERNAL_FAILURE) },
                }
            }
            Ok(ForkResult::Child) => {
                execve(&program, &args, &env)?;
                unsafe { _exit(EXIT_INTERNAL_FAILURE) }
            }
            Err(_) => unsafe {
                _exit(EXIT_INTERNAL_FAILURE);
            },
        }
    }

    fn reap_orphan(child: Pid) -> nix::Result<WaitStatus> {
        let mut main_child_status: Option<WaitStatus> = None;
        loop {
            match waitpid(Pid::from_raw(-1), None) {
                Ok(status @ WaitStatus::Exited(pid, _))
                | Ok(status @ WaitStatus::Signaled(pid, _, _)) => {
                    if pid == child {
                        main_child_status = Some(status)
                    }
                }
                Ok(_) => continue,
                // Errors other than interruption by a signal (EINTR) and the absence of child processes to monitor (ECHILD) are treated
                // as errors preventing the continued reaping of orphaned processes, so an early return is performed.
                // When the init process of a newly created PID namespace terminates,
                // the kernel terminates all remaining processes in that namespace by sending them a SIGKILL signal.
                // Consequently, processes within a newly created PID namespace do not persist as zombie processes.
                // For details, see pid_namespaces(7) man page.
                Err(Errno::EINTR) => continue,
                Err(Errno::ECHILD) => break,
                Err(e) => return Err(e),
            }
        }
        main_child_status.ok_or(Errno::ECHILD)
    }

    fn wait_child(child: Pid) -> nix::Result<WaitStatus> {
        waitpid(child, None)
    }
}
