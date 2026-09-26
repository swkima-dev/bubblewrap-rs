# Handling of User Namespace in root

## Context

本家Bubblewrapでは、非root環境において`--unshare-user`を省略した際にUser Namespaceを新規で作成するという仕様が存在する。
つまり、Bubblewrapでは非root環境とroot環境、`--unshare-user`の有無によって以下の表のように挙動が変化する。

|  | --unshare-userなし | --unshare-userあり |
| --- | --- | --- |
| 非root | 新たなUser Namespaceを作る | 新たなUser Namespaceを作る |
| root | 現在のUser Namespaceのまま | 新たなUser Namespaceを作る |

この挙動は、非root環境では`--unshare-user`の省略が「新たなUser Namespaceを作らない」を意味せず、
Bubblewrapの仕様によって自動でUser Namespaceを作成してしまうという点で問題である。

なお、2026/9/26時点では、bubblewrap.c:2931にて以下のような実装が見られる。
```
  /* We have to do this if we we're not root, so let's just DWIM */
  if (getuid () != 0 && opt_userns_fd == -1)
    opt_unshare_user = true;
```

この実装ではUIDが非rootつまり0以外である場合を条件式の要素としているが、
本来User Namespaceを利用するか否かはUIDが0であるかではなく以降の処理で必要なCapabilityがあるか否かで判断すべきである。

これらは解決すべき設計上の問題である。

## Decision

実行者のUIDによらず新しいUser Namespaceを作成することを既定とし、現在のUser Namespaceを引き継ぐ場合は`share_user`で明示する。
本家のroot実行時との互換性より、同じ設定が同じNamespace選択を意味することと、rootless実行の利便性を優先する。明示された選択は、権限不足を理由に自動で切り替えない。

`share_user`はroot専用の機能とは位置付けない。実行可能性は各操作に必要な権限に依存し、設定時の権限が実行時にも維持されるとは限らないため、Builderでは権限を検査せず、実行時のシステムコールに判定を委ねる。

UID/GID指定は新しいUser Namespaceのマッピングを要求するものとし、`share_user`との併用を許容しない。
設定順序に依存しない検証のため、設定間の矛盾は実行前に検査する。エラー型は`std::process::Command`に合わせて`std::io::Error`を維持し、設定の矛盾には`InvalidInput`と説明文を用いる。

以下の選択肢は採用しない。

- 本家との完全な互換を保つ挙動：互換性は保てるが、実行者によって非自明なNamespaceの作成選択が発生するという問題が残る。
- 非rootで`--unshare-user`相当の指定を必須とする：通常の非rootでの実行での互換性が保てなくなる。非特権コンテナの主なユースケースは非rootユーザーによるものだと考えられるため、非rootユーザーの挙動の互換性が保てないこの方針は採用しない。
- Builderパターンの設定時点でcapabilityの不足を検知しエラーを発生する：Builderの時点では設定の妥当性のみを検証すべきである。実行時の成功可否は実行時のエラーとすべき。

## Status

approval

## Consequences

- 本家でUser Namespaceを作らずに実行していたroot利用者は、`share_user`を明示する必要がある。既定の実行はrootでもUser Namespaceの作成制限やマッピングの影響を受ける。
- `share_user`は実行成功を保証しない。必要な権限を持つ実行環境の用意は呼び出し側の責務となる。
- 子プロセスであるintemediateプロセスやinitプロセスにて権限の不足などのエラーが発生した場合、pipeによってNamespace構築の失敗を`exec()`の`Err`として親プロセスへと伝播する必要がある。

## References

- [プロジェクトの互換性とライブラリ利用に関する方針](../README.md)
