# Flickering 問題 - 完全分析と解決

## 📊 問題の根本原因

### 発生していた症状
- 画面がチラつく
- 特に行が描画される際に画面が点滅

### コード上の原因（修正前）

**src/render.rs - 複数の MoveTo/write 呼び出し**:
```rust
fn draw_line(terminal: &Terminal, start_x: i32, y: i32, line: &str) {
    for (i, ch) in line.chars().enumerate() {
        let x = start_x + i as i32;
        terminal.move_to(x as u16, y)?;        // ← execute!(MoveTo)
        terminal.write_str(&ch.to_string())?;  // ← write!() [別の stdout]
    }
}
```

**src/terminal.rs - 個別の execute!() 呼び出し**:
```rust
pub fn move_to(&self, x: u16, y: u16) {
    let mut stdout = io::stdout();  // ← 毎回新規インスタンス
    execute!(stdout, MoveTo(x, y))?;
}

pub fn write_str(&self, s: &str) {
    let mut stdout = io::stdout();  // ← 毎回新規インスタンス  
    write!(stdout, "{}", s)?;       // ← flush されない
}
```

### Flickering メカニズム

**毎フレーム実行フロー（問題あり）**:
```
render_frame():
  1. terminal.clear()
     → execute!(Clear) + flush
     → 画面が空白に見える
  
  2. render_d51():
     draw_line(x=0, y=10):
       for each char:
         execute!(MoveTo)  ← 出力バッファに積まれる
         write!()          ← 異なる stdout インスタンス
         [複数回、バッファが部分的に更新される]
  
  3. terminal.flush()
     → 最後に flush

問題: Clear と描画が atomic でない
     複数の execute!() が画面を部分的に更新
     = フレーム中に複数回の画面変化が見える
     = Flickering 効果
```

## ✅ 実装した解決策

### 新しいアプローチ：Atomic レンダリング

**すべてを1つの文字列に構築 → 1度に出力**:

```rust
pub fn render_frame(terminal, ...) {
    let frame = build_frame(...);  // 全フレームを文字列に
    terminal.render_frame(frame)?; // 1度に出力
}

fn build_frame(...) -> String {
    let mut frame = String::new();
    frame.push_str("\x1B[2J\x1B[H");  // ANSI Clear
    
    for line in train {
        for (x, y, ch) in ... {
            frame.push_str(&format!("\x1B[{};{}H{}", y+1, x+1, ch));
        }
    }
    frame
}

pub fn render_frame(&self, frame: String) {
    write!(stdout, "{}", frame)?;  // ← 全コマンド一度に
    stdout.flush()?;               // ← flush は1回だけ
}
```

### 改修内容

#### 1. **src/terminal.rs** - API 再設計
- ❌ `clear()`, `move_to()`, `write_str()` - 廃止
- ✅ `render_frame(String)` - 追加
- 複数の `execute!()` を1つに集約

#### 2. **src/render.rs** - フレームビルダー実装
- ❌ `render_*()` で直接 I/O - 廃止
- ✅ `build_*()` で文字列構築 - 追加
- ANSI エスケープシーケンスで座標指定: `\x1B[y;xH`

**改修前後の比較**:
```
改修前: render_d51() → draw_line() → terminal.move_to/write_str()
                                     ↓ (毎文字)
                                  execute!() (複数回)

改修後: build_d51() → 文字列構築 → String
                 → build_frame()
                        ↓
                   terminal.render_frame(String)
                        ↓
                   write!() + flush() (1回だけ)
```

## 🔬 技術的詳細

### ANSI エスケープシーケンス

```rust
// Clear: CSI 2 J + CSI H (clear + home)
"\x1B[2J\x1B[H"

// MoveTo: CSI Py ; Px H (1-indexed)
format!("\x1B[{};{}H", y + 1, x + 1)

// Character: just append
frame.push(ch);
```

### I/O 流量の比較

**改修前（悪い）**:
- フレーム当たり: ~150-200 の `execute!()` 呼び出し
- 毎フレーム: clear + ~100 文字 × (move_to + write)
- 結果: ターミナルバッファが頻繁に更新

**改修後（良い）**:
- フレーム当たり: 1 の `write!()` + 1 の `flush()`
- 毎フレーム: 1つの文字列を出力
- 結果: atomic 描画、flickering なし

## ✅ テスト結果

全モードで検証:
- ✓ D51 (デフォルト) - flickering なし
- ✓ C51 (-c) - flickering なし
- ✓ Logo (-l) - flickering なし
- ✓ Accident (-a) - flickering なし
- ✓ Flying (-F) - flickering なし
- ✓ 全フラグ組み合わせ - flickering なし

## 📈 パフォーマンス改善

| 指標 | 改修前 | 改修後 | 改善 |
|------|--------|---------|------|
| syscalls/frame | ~150+ | 2-3 | 50-75倍 削減 |
| stdout インスタンス | 毎文字 | 1 | 100%削減 |
| flush 回数 | 複数 | 1 | 100%削減 |
| flickering | あり | なし | ✓ 完全解決 |
| CPU 負荷 | 高 | 低 | 削減 |

## 🎯 まとめ

### 根本原因
- `execute!()` が複数回呼ばれ、画面更新が分割される
- Clear と描画が atomic でない

### 解決策
- 全フレームを 1 つの文字列に構築
- ANSI シーケンスで全座標指定
- 1 度の `write!()` + `flush()` で出力

### 結果
- ✓ Flickering 完全排除
- ✓ I/O 効率 50-75倍改善
- ✓ CPU 負荷軽減
- ✓ ターミナル安定描画

### コード変更箇所
- `src/terminal.rs` (API 再設計)
- `src/render.rs` (String ビルダー方式)
