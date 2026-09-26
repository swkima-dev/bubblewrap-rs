use nix::sched::{CloneFlags, unshare};
use nix::unistd::{Gid, Uid, getgid, getuid};
use std::fs::File;
use std::io::{Result, Write};

pub(crate) fn apply_user_namespace(internal_uid: &Uid, internal_gid: &Gid) -> Result<()> {
    let real_uid_in_parental_ns = getuid();
    let real_gid_in_parental_ns = getgid();

    unshare(CloneFlags::CLONE_NEWUSER)?;

    write_file(
        "/proc/self/uid_map",
        &format!("{} {} 1", internal_uid, real_uid_in_parental_ns),
    )?;

    write_file("/proc/self/setgroups", "deny")?;

    write_file(
        "/proc/self/gid_map",
        &format!("{} {} 1", internal_gid, real_gid_in_parental_ns),
    )?;

    Ok(())
}

fn write_file(path: &str, content: &str) -> std::io::Result<()> {
    let mut file = File::create(path)?;

    file.write_all(content.as_bytes())
}
