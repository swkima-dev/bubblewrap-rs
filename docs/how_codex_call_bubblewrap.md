## Overview

- `codex-rs/linux-sandbox`
    - `launcher.rs/exec_bwrap()` bubblewrapを実行する場所を確定させる
        - ホストOS上のbwrap
        - プロジェクトにバンドルされているbwrap
        - 実行不可能
    - `launcher.rs/exec_system_bwrap()`
        - ホストOS上のパスを変数に格納→ `libc::execv` でargsも渡して実行
    - `linux_run_main.rs/run_bwrap_with_proc_fallback()`
        - bubblewrapに渡すオプション（引数）を構築している
    - `bwrap.rs/create_bwrap_flags()`
        - `--unshare-user` などのコマンドライン引数を、内部的に持っている `Option` 型から必要なものだけ選別して作っている
        - `args.push("--new-session".to_string());` のような形で `args = Vec::new();` に文字列をたくさんpushしている！←きれいじゃない！
    - `bwrap.rs/create_filesystem_args`
        - ファイルシステム系のオプションが指定されている
        - やり方は`create_bwrap_flags()` とおなじ←よくない！
        - `append_mount_target_parent_dir_args()` を呼び出している
- `codex-rs/sandboxing`
    - `bwrap.rs`

### 利用されているコマンドライン引数

- 必ず指定される
    - `--new-sssion` : 新しいターミナルセッションを立ち上げる
        - https://security.sios.jp/vulnerability/selinux-security-20161116/
        - 子プロセスから親プロセスの標準入力にアクセスできてしまう脆弱性を防止する
    - `--die-with-parent` : 親プロセスが死んだ際に子プロセスが死ぬことを保証する
    - `--unshare-user`
    - `--unshare-pid`
- オプションで指定されれば
    - `--unshare-net`
    - `--chdir DIR` : DIRにディレクトリ遷移
- ファイルシステム系（どういったタイミングで呼ばれるかは詳しくは把握していない）
    - `--ro-bind　SRC DEST` : ホストのSRCをDESTにRead Onlyでバインドマウント
    - `--tmpfs DEST` : 新しいtmpfsをDESTにマウントする
        - `--perms` : `--tempfs` の前に置くことで、tmpfsのモードを指定できる
        - 何もしていない時は 0775（Owner: rwx, Group: rx, Other:rx）
    - `--dev DEST` : 新しいdevtmpfsをDESTにマウント
        - 同様
    - `--proc DEST` : procfsをDESTにマウント
    - `--bind`
    - `--dir DEST` : ディレクトリを作る
    - `--ro-bind-data FD DEST` : FDのファイルディスクリプタのファイルをDESTにバインドマウントする
        - `--perms` でモードを指定できる
        - 何も指定していない時は0600
    - `--remount-ro DEST` : パスDESTをRead Onlyでリマウントする
- `--argv0`
