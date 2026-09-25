use std::{os::unix::process::ExitStatusExt, process::ExitStatus};

use bubblewrap;

#[test]
fn apply_user_namespace() {
    let args: Vec<String> = vec![
        "/bin/sh".to_string(),
        "-c".to_string(),
        "[ \"$(id -u)\" = \"0\" ] && exit 0 || exit 1".to_string(),
    ];
    println!("{:?}", args);
    let exit_status = bubblewrap::builder::Command::new(args[0].clone().into())
        .args(args[1..].to_vec().iter().map(|s| s.into()).collect())
        .internal_uid(0)
        .internal_gid(0)
        .exec()
        .unwrap_or(ExitStatus::from_raw(255 << 8))
        .code();

    assert_eq!(exit_status.unwrap(), 0);
}
