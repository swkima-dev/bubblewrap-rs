use super::init;
use super::parent_death;
use super::protocol::{Message, Protocol};
use super::{exit_code, exit_immediately, wait};
use crate::config::SandboxConfig;
use crate::namespace;
use crate::{Error, Result};
use nix::sched::CloneFlags;
use nix::unistd::{ForkResult, Gid, Pid, Uid, fork, getpid, setresgid, setresuid};

pub(crate) fn run(
    config: SandboxConfig,
    mut channel: Protocol,
    expected_parent: Pid,
) -> Result<()> {
    if config.die_with_parent {
        parent_death::set_for_parent(expected_parent)?;
    }

    namespace::unshare(CloneFlags::CLONE_NEWUSER)?;
    channel.send(Message::UserNamespaceReady)?;

    match channel.receive()? {
        Message::IdMappingsInstalled => {}
        Message::Abort => return Err(Error::ChildSetup),
        _ => return Err(Error::InvalidConfig("unexpected process protocol state")),
    }

    become_namespace_root()?;

    if config.unshare_pid {
        namespace::unshare(CloneFlags::CLONE_NEWPID)?;
    }

    // SAFETY: this is the single-threaded intermediate launcher. The child
    // enters `init::run` and exits without returning to inherited application code.
    let intermediate_pid = getpid();
    let init_pid = match unsafe { fork()? } {
        ForkResult::Parent { child } => child,
        ForkResult::Child => {
            drop(channel);
            let result = init::run(&config, intermediate_pid);
            if let Err(error) = &result {
                eprintln!("bubblewrap: init setup failed: {error}");
            }
            exit_immediately(result.map_or(125, |code| code));
        }
    };

    channel.send(Message::InitReady)?;
    drop(channel);
    let status = wait(init_pid)?;
    exit_immediately(exit_code(status));
}

fn become_namespace_root() -> Result<()> {
    // Supplementary groups cannot be changed after `setgroups` was denied by
    // the parent; the namespace starts with the caller's existing group list.
    // Drop GID before UID so losing UID-related privilege cannot block setgid.
    let root_gid = Gid::from_raw(0);
    let root_uid = Uid::from_raw(0);
    setresgid(root_gid, root_gid, root_gid)?;
    setresuid(root_uid, root_uid, root_uid)?;
    Ok(())
}
