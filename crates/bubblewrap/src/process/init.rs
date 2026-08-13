use super::exit_code;
use super::parent_death;
use crate::Result;
use crate::config::SandboxConfig;
use crate::namespace;
use crate::rootfs;
use nix::errno::Errno;
use nix::mount::{MsFlags, mount};
use nix::sched::CloneFlags;
use nix::sys::wait::{WaitStatus, wait};
use nix::unistd::{Pid, setsid};
use std::io;
use std::os::unix::process::CommandExt;
use std::process::{Command, Stdio};

pub(crate) fn run(config: &SandboxConfig, expected_parent: Pid) -> Result<i32> {
    if config.die_with_parent {
        parent_death::set_for_parent(expected_parent)?;
    }

    let mut flags = CloneFlags::CLONE_NEWNS;
    if config.unshare_net {
        flags |= CloneFlags::CLONE_NEWNET;
    }
    namespace::unshare(flags)?;
    make_mounts_private()?;
    if !config.filesystem.is_empty() {
        rootfs::setup(&config.filesystem, config.unshare_pid)?;
    }

    let mut command = Command::new(&config.program);
    command
        .args(&config.args)
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit());
    if let Some(directory) = &config.current_dir {
        command.current_dir(directory);
    }
    if config.new_session {
        // SAFETY: `setsid` is async-signal-safe and no allocation or locking is
        // performed in this pre-exec hook.
        unsafe {
            command.pre_exec(|| setsid().map(drop).map_err(errno_to_io));
        }
    }

    let mut child = command.spawn()?;
    let command_status = child.wait()?;

    // PID 1 owns all orphaned descendants in the namespace. Keep reaping until
    // none remain, preventing zombies from escaping the sandbox lifecycle.
    reap_remaining_children()?;
    Ok(exit_code(command_status))
}

fn make_mounts_private() -> Result<()> {
    mount::<str, str, str, str>(None, "/", None, MsFlags::MS_REC | MsFlags::MS_PRIVATE, None)?;
    Ok(())
}

fn reap_remaining_children() -> Result<()> {
    loop {
        match wait() {
            Ok(WaitStatus::Exited(..) | WaitStatus::Signaled(..)) | Err(Errno::EINTR) => continue,
            Err(Errno::ECHILD) => return Ok(()),
            Ok(_) => continue,
            Err(error) => return Err(error.into()),
        }
    }
}

fn errno_to_io(error: Errno) -> io::Error {
    io::Error::from_raw_os_error(error as i32)
}
