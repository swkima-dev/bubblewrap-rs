mod common;

use common::run;

use crate::common::describe;

#[test]
fn pid_namespace_are_valid() {
    let output = run(&["--unshare-pid", "/bin/sh", "-c", r#"[ "$$" -eq 2 ]"#]);

    assert!(output.status.success(), "{}", describe(&output));
}
