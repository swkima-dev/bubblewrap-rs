use crate::Result;
use crate::config::FilesystemOperation;
use nix::fcntl::{OFlag, open};
use nix::mount::{MntFlags, MsFlags, mount, umount2};
use nix::sys::stat::Mode;
use nix::unistd::{chdir, fchdir, pivot_root};
use std::fs::{self, OpenOptions};
use std::os::fd::AsRawFd;
use std::os::unix::fs::OpenOptionsExt;
use std::path::{Path, PathBuf};

const STAGING_ROOT: &str = "/tmp";
const OLD_ROOT: &str = "/oldroot";
const NEW_ROOT: &str = "/newroot";

pub(crate) fn setup(operations: &[FilesystemOperation], unshare_pid: bool) -> Result<()> {
    mount_staging_root()?;
    first_pivot()?;
    apply_operations(operations, unshare_pid)?;
    detach_host_root()?;
    second_pivot()?;
    Ok(())
}

fn mount_staging_root() -> Result<()> {
    mount(
        Some("tmpfs"),
        STAGING_ROOT,
        Some("tmpfs"),
        MsFlags::MS_NODEV | MsFlags::MS_NOSUID,
        Some("mode=0755"),
    )?;
    chdir(STAGING_ROOT)?;
    fs::create_dir("newroot")?;
    mount(
        Some("newroot"),
        "newroot",
        None::<&str>,
        MsFlags::MS_BIND | MsFlags::MS_REC,
        None::<&str>,
    )?;
    fs::create_dir("oldroot")?;
    Ok(())
}

fn first_pivot() -> Result<()> {
    pivot_root(STAGING_ROOT, "oldroot")?;
    chdir("/")?;
    Ok(())
}

fn apply_operations(operations: &[FilesystemOperation], unshare_pid: bool) -> Result<()> {
    for operation in operations {
        match operation {
            FilesystemOperation::Bind {
                source,
                destination,
                readonly,
            } => bind(source, destination, *readonly)?,
            FilesystemOperation::Tmpfs { destination } => mount_tmpfs(destination)?,
            FilesystemOperation::Proc { destination } => mount_proc(destination, unshare_pid)?,
            FilesystemOperation::Dir { destination } => create_directory(destination)?,
            FilesystemOperation::RemountReadonly { destination } => {
                remount_readonly(&newroot_path(destination))?
            }
        }
    }
    Ok(())
}

fn mount_proc(destination: &Path, unshare_pid: bool) -> Result<()> {
    let destination = newroot_path(destination);
    fs::create_dir_all(&destination)?;

    if unshare_pid {
        // Mount while /oldroot/proc is still fully visible. Linux requires this
        // for an unprivileged user namespace to mount a new procfs safely.
        mount(
            Some("proc"),
            destination.as_path(),
            Some("proc"),
            MsFlags::MS_NOSUID | MsFlags::MS_NODEV | MsFlags::MS_NOEXEC,
            None::<&str>,
        )?;
    } else {
        mount(
            Some(Path::new("/oldroot/proc")),
            destination.as_path(),
            None::<&str>,
            MsFlags::MS_BIND | MsFlags::MS_REC,
            None::<&str>,
        )?;
    }
    Ok(())
}

fn bind(source: &Path, destination: &Path, readonly: bool) -> Result<()> {
    let source = oldroot_path(source);
    let destination = newroot_path(destination);
    let metadata = fs::metadata(&source)?;

    if metadata.is_dir() {
        fs::create_dir_all(&destination)?;
    } else {
        create_file_target(&destination)?;
    }

    mount(
        Some(source.as_path()),
        destination.as_path(),
        None::<&str>,
        MsFlags::MS_BIND | MsFlags::MS_REC,
        None::<&str>,
    )?;
    if readonly {
        remount_readonly(&destination)?;
    }
    Ok(())
}

fn mount_tmpfs(destination: &Path) -> Result<()> {
    let destination = newroot_path(destination);
    fs::create_dir_all(&destination)?;
    mount(
        Some("tmpfs"),
        destination.as_path(),
        Some("tmpfs"),
        MsFlags::MS_NODEV | MsFlags::MS_NOSUID,
        Some("mode=0755"),
    )?;
    Ok(())
}

fn create_directory(destination: &Path) -> Result<()> {
    fs::create_dir_all(newroot_path(destination))?;
    Ok(())
}

fn create_file_target(destination: &Path) -> Result<()> {
    if let Some(parent) = destination.parent() {
        fs::create_dir_all(parent)?;
    }
    OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(false)
        .mode(0o644)
        .open(destination)?;
    Ok(())
}

fn remount_readonly(destination: &Path) -> Result<()> {
    mount(
        Some(destination),
        destination,
        None::<&str>,
        MsFlags::MS_BIND | MsFlags::MS_REMOUNT | MsFlags::MS_RDONLY,
        None::<&str>,
    )?;
    Ok(())
}

fn detach_host_root() -> Result<()> {
    mount::<Path, Path, str, str>(
        None,
        Path::new(OLD_ROOT),
        None,
        MsFlags::MS_REC | MsFlags::MS_PRIVATE,
        None,
    )?;
    umount2(OLD_ROOT, MntFlags::MNT_DETACH)?;
    Ok(())
}

fn second_pivot() -> Result<()> {
    let staging_root = open(
        "/",
        OFlag::O_RDONLY | OFlag::O_DIRECTORY | OFlag::O_CLOEXEC,
        Mode::empty(),
    )?;
    chdir(NEW_ROOT)?;
    pivot_root(".", ".")?;
    fchdir(staging_root.as_raw_fd())?;
    umount2(".", MntFlags::MNT_DETACH)?;
    chdir("/")?;
    Ok(())
}

fn oldroot_path(path: &Path) -> PathBuf {
    join_under(OLD_ROOT, path)
}

fn newroot_path(path: &Path) -> PathBuf {
    join_under(NEW_ROOT, path)
}

fn join_under(root: &str, path: &Path) -> PathBuf {
    let relative = path.strip_prefix("/").expect("validated absolute path");
    Path::new(root).join(relative)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn paths_are_mapped_below_old_and_new_root() {
        assert_eq!(
            oldroot_path(Path::new("/usr/bin")),
            Path::new("/oldroot/usr/bin")
        );
        assert_eq!(newroot_path(Path::new("/")), Path::new("/newroot"));
    }
}
