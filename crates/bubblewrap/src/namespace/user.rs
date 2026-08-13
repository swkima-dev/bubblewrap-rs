use crate::Result;
use nix::unistd::{Pid, getegid, geteuid};
use std::fs;

pub(crate) fn write_id_maps(pid: Pid) -> Result<()> {
    let proc_dir = format!("/proc/{pid}");
    let uid = geteuid().as_raw();
    let gid = getegid().as_raw();

    fs::write(format!("{proc_dir}/uid_map"), format!("0 {uid} 1\n"))?;

    match fs::write(format!("{proc_dir}/setgroups"), "deny\n") {
        Ok(()) => {}
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(error) => return Err(error.into()),
    }

    fs::write(format!("{proc_dir}/gid_map"), format!("0 {gid} 1\n"))?;
    Ok(())
}
