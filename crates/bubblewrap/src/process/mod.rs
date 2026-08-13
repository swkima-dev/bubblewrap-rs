mod init;
mod intermediate;
pub(crate) mod parent_death;
mod protocol;

use crate::config::SandboxConfig;
use crate::namespace;
use crate::{Error, Result};
use nix::errno::Errno;
use nix::sys::wait::{WaitStatus, waitpid};
use nix::unistd::{ForkResult, Pid, fork, getpid};
use protocol::{Message, Protocol};
use std::os::unix::process::ExitStatusExt;
use std::process::ExitStatus;

pub(crate) fn spawn(config: SandboxConfig) -> Result<Pid> {
    let (mut main_channel, intermediate_channel) = Protocol::pair()?;
    let main_pid = getpid();

    // SAFETY: the child enters the dedicated launcher path and never returns
    // to the caller's control flow. See `Command::spawn` for its threading
    // restriction.
    let child = match unsafe { fork()? } {
        ForkResult::Parent { child } => child,
        ForkResult::Child => {
            drop(main_channel);
            let result = intermediate::run(config, intermediate_channel, main_pid);
            if let Err(ref error) = result {
                eprintln!("bubblewrap: intermediate setup failed: {error}");
            }
            exit_immediately(if result.is_ok() { 0 } else { 125 });
        }
    };

    drop(intermediate_channel);

    match main_channel.receive() {
        Ok(Message::UserNamespaceReady) => {}
        _ => return failed_setup(child),
    }

    if let Err(error) = namespace::write_id_maps(child) {
        let _ = main_channel.send(Message::Abort);
        let _ = wait(child);
        return Err(error);
    }
    main_channel.send(Message::IdMappingsInstalled)?;

    match main_channel.receive() {
        Ok(Message::InitReady) => Ok(child),
        _ => failed_setup(child),
    }
}

fn failed_setup<T>(pid: Pid) -> Result<T> {
    let _ = wait(pid);
    Err(Error::ChildSetup)
}

pub(crate) fn wait(pid: Pid) -> Result<ExitStatus> {
    loop {
        match waitpid(pid, None) {
            Ok(WaitStatus::Exited(_, code)) => {
                return Ok(ExitStatus::from_raw(code << 8));
            }
            Ok(WaitStatus::Signaled(_, signal, dumped_core)) => {
                let core_dump_flag = if dumped_core { 0x80 } else { 0 };
                return Ok(ExitStatus::from_raw(signal as i32 | core_dump_flag));
            }
            Ok(_) | Err(Errno::EINTR) => continue,
            Err(error) => return Err(error.into()),
        }
    }
}

pub(crate) fn exit_code(status: ExitStatus) -> i32 {
    status
        .code()
        .unwrap_or_else(|| 128 + status.signal().unwrap_or(127))
}

pub(crate) fn exit_immediately(code: i32) -> ! {
    // nix deliberately does not wrap `_exit`; its own `fork` documentation
    // recommends this re-export for terminating a post-fork child safely.
    unsafe { nix::libc::_exit(code) }
}
