mod user;

pub(crate) use user::write_id_maps;

use crate::Result;
use nix::sched::{CloneFlags, unshare as nix_unshare};

pub(crate) fn unshare(flags: CloneFlags) -> Result<()> {
    nix_unshare(flags)?;
    Ok(())
}
