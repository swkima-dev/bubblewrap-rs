use crate::Result;
use nix::sys::prctl::set_pdeathsig;
use nix::sys::signal::{Signal, kill};
use nix::unistd::{Pid, getpid, getppid};

pub(crate) fn set() -> Result<()> {
    let parent = getppid();
    set_for_parent(parent)
}

pub(crate) fn set_for_parent(expected_parent: Pid) -> Result<()> {
    set_pdeathsig(Signal::SIGKILL)?;
    // Check after prctl so a parent death racing with setup is not missed.
    if getppid() != expected_parent {
        kill(getpid(), Signal::SIGKILL)?;
    }
    Ok(())
}
