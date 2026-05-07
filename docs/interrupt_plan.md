# handle_parallel 外部中断(Ctrl+C)対応

## 概要

`handle_parallel` 実行中のコマンドを外部から中断できるようにする。`CancellationToken`（`Arc<AtomicBool>` ラッパー）を導入し、シグナルが立つとパイプライン内の I/O が即座に `Interrupted` エラーを返す仕組み。

## Proposed Changes

### wasibox-core — IoContext にキャンセルトークン追加

#### [MODIFY] [lib.rs](file:///f:/sl-rust/crates/wasibox-core/src/lib.rs)

- `CancellationToken` 構造体を追加（`Arc<AtomicBool>` ラッパー、`Clone`/`Send`/`Sync`）
  - `fn new() -> Self`
  - `fn cancel(&self)` — フラグをセット
  - `fn is_cancelled(&self) -> bool` — フラグ確認
  - `fn reset(&self)` — フラグリセット
- `IoContext` に `pub cancel_token: CancellationToken` フィールドを追加
- `IoContext::new()` と `Default` にデフォルトトークンを追加
- `IoContext::with_cancel(stdin, stdout, stderr, token)` コンストラクタ追加

---

### wasi-shell — パイプラインにキャンセル伝播

#### [MODIFY] [lib.rs](file:///f:/sl-rust/crates/wasi-shell/src/lib.rs)

- `handle_parallel` のシグネチャに `cancel_token: CancellationToken` パラメータ追加
- `handle_pipeline` にも同様に追加
- パイプライン各ステージの `IoContext` 生成時にトークンを共有
- 各ステージのスレッドで、コマンド実行前に `is_cancelled()` チェック
- キャンセル時は `Err("Interrupted".to_string())` を返す
- `run_loop` 内で `ctrlc` 的なシグナルハンドリング（ネイティブ: signal hook / WASI: トークン公開）

#### [MODIFY] [main.rs](file:///f:/sl-rust/crates/wasi-shell/src/main.rs)

- REPL ループ内で `CancellationToken` を作成し `handle_parallel` に渡す

#### [MODIFY] [readline.rs](file:///f:/sl-rust/crates/wasi-shell/src/readline.rs)

- `run_loop` に `cancel_token` パラメータ追加
- ネイティブ: raw mode 中に Ctrl-C (byte 3) 受信でトークンをキャンセル → handle_parallel のスレッドが中断

> [!IMPORTANT]
> raw mode ではCtrl+CはOSシグナルとして処理されず、byte `0x03` として `read_byte` に届きます。readline の read_line 中のCtrl-Cは「行破棄」として処理済みですが、`handle_parallel` 実行中にCtrl-Cを受信するには、実行中に別スレッドでstdinを監視するか、あるいは `CancellationToken` を公開APIとして外部から `cancel()` を呼べるようにする方針が必要です。

> [!NOTE]  
> WASI環境では OS シグナルが存在しないため、`CancellationToken` を公開し、ホストランタイム（JS等）から `cancel()` を呼べる設計にします。ネイティブ環境では `ctrlc` crateを使わず、raw mode を解除した状態では OS の SIGINT がデフォルトで効くため、handle_parallel 実行中のみ token ベースの中断を行います。

## 設計方針

実行中のコマンドの中断は、**パイプの書き込み側を閉じる**ことで自然に実現する：

1. `CancellationToken::cancel()` が呼ばれる
2. パイプラインの各ステージの I/O が次の write/read 時にエラーを返す
3. 各スレッドが終了する

これにより既存のユーティリティコードを変更せずに中断が可能。

## Verification Plan

### Automated Tests
- `test_cancel_token_basic` — トークンの set/check
- `test_handle_parallel_cancel` — 無限コマンド (`yes`) を `handle_parallel` で実行中にトークンでキャンセル → 正常終了
- `test_pipeline_cancel` — パイプライン実行中のキャンセル
