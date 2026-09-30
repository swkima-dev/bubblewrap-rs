mod common;

use common::run;

use crate::common::describe;

#[test]
fn pid_namespace_are_valid() {
    let output = run(&["--unshare-pid", "/bin/sh", "-c", r#"[ "$$" -eq 2 ]"#]);

    assert!(output.status.success(), "{}", describe(&output));
}

#[test]
fn orphaned_process_is_reaped() {
    let output = run(&[
        "--unshare-pid",
        "/bin/sh",
        "-c",
        r#"
        # The inner shell exits, leaving sleep to be adopted by init.
        orphan=$(/bin/sh -c '/bin/sleep 30 >/dev/null 2>&1 & echo $!')
        kill -KILL "$orphan" || exit 1

        # A zombie still responds to kill -0; wait briefly for init to reap it.
        attempts=0
        while [ "$attempts" -lt 100 ]; do
            if ! kill -0 "$orphan" 2>/dev/null; then
                exit 0
            fi
            /bin/sleep 0.01
            attempts=$((attempts + 1))
        done
        echo "orphan was not reaped" >&2
        exit 1
        "#,
    ]);

    assert!(output.status.success(), "{}", describe(&output));
}
