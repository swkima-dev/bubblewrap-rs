mod common;

use common::{describe, run};
use std::fs;
use std::os::unix::fs::{PermissionsExt, chown};

#[test]
fn uid_and_gid_are_mapped() {
    let output = run(&[
        "--uid",
        "1234",
        "--gid",
        "5678",
        "--",
        "/bin/sh",
        "-c",
        r#"[ "$(id -u):$(id -g)" = "1234:5678" ]"#,
    ]);

    assert!(output.status.success(), "{}", describe(&output));
}

/// Root in a new user namespace loses CAP_DAC_OVERRIDE over files whose owner
/// is not mapped; with `--share-user` it stays in the host namespace and keeps it.
/// Run with `sudo -E env "PATH=$PATH" cargo test -- --ignored`.
/// This behavior makes it possible to indirectly test whether the current User Namespace is being used.
/// For details regarding user namespaces and file permissions, see man of user_namespaces(7) "Operation of file-related capabilities"
#[test]
#[ignore = "requires root"]
fn root_user_namespace_depends_on_share_user() {
    let path = std::env::temp_dir().join(format!("bwrap-rs-{}", std::process::id()));
    fs::write(&path, "secret").expect("failed to write file");
    fs::set_permissions(&path, fs::Permissions::from_mode(0o600)).expect("failed to chmod file");
    chown(&path, Some(65533), Some(65533)).expect("failed to chown file (requires root)");
    let path = path.to_str().expect("temp path is not UTF-8");

    let default = run(&["--", "/bin/cat", path]);
    let shared = run(&["--share-user", "--", "/bin/cat", path]);
    let _ = fs::remove_file(path);

    assert!(!default.status.success(), "{}", describe(&default));
    assert!(shared.status.success(), "{}", describe(&shared));
}
