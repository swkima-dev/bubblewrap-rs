use bubblewrap::add;

use clap::Parser;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Ensures child process (COMMAND) dies when bwrap's parent dies. Kills
    /// (SIGKILL) all bwrap sandbox processes in sequence from parent to child
    /// including COMMAND process when bwrap or bwrap's parent dies. See
    /// prctl, PR_SET_PDEATHSIG.
    #[arg(long, action = clap::ArgAction::SetTrue)]
    die_with_parent: bool,

    /// Create a new user namespace
    #[arg(long, action = clap::ArgAction::SetTrue)]
    unshare_user: bool,

    /// Create a new user namespace
    #[arg(long, action = clap::ArgAction::SetTrue)]
    unshare_pid: bool,

    /// Create a new network namespace
    #[arg(long, action = clap::ArgAction::SetTrue)]
    unshare_net: bool,

    /// Change directory to DIR
    #[arg(long, value_name = "DIR")]
    chdir: Option<String>,

    /// Bind mount the host path SRC readonly on DEST
    #[arg(long, num_args = 2, value_names = ["SRC", "DEST"])]
    ro_bind: Option<Vec<String>>,

    /// Mount new tmpfs on DEST. If the previous option was --perms, it sets
    /// the mode of the tmpfs. Otherwise, the tmpfs has mode 0755.
    #[arg(long, value_name = "DEST")]
    tempfs: Option<String>,

    /// Bind mount the host path SRC on DEST
    #[arg(long, num_args = 2, value_names = ["SRC", "DEST"])]
    bind: Option<Vec<String>>,

    /// Create a directory at DEST. If the directory already exists, its
    /// permissions are unmodified, ignoring --perms (use --chmod if the
    /// permissions of an existing directory need to be changed). If the
    /// directory is newly created and the previous option was --perms, it
    /// sets the mode of the directory. Otherwise, newly-created directories
    /// have mode 0755.
    #[arg(long, value_name = "DEST")]
    dir: Option<String>,

    /// Remount the path DEST as readonly. It works only on the specified
    /// mount point, without changing any other mount point under the
    /// specified path
    #[arg(long, value_name = "DEST")]
    remount_ro: Option<String>,
}

fn main() {
    let args = Args::parse();
    println!("{:#?}", args);
}
