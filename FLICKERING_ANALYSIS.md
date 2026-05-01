# Flickering 問題分析

## 🔍 根本原因

### 問題の症状
- 画面が点滅する
- 特に行が描画される際に見える

### コード上の原因

**src/render.rs の draw_line() 関数**:
```rust
fn draw_line(terminal: &Terminal, start_x: i32, y: i32, line: &str) -> io::Result<()> {
    let y = y as u16;
    for (i, ch) in line.chars().enumerate() {
        let x = start_x + i as i32;
        if x >= 0 && x < terminal.width() as i32 {
            terminal.move_to(x as u16, y)?;  // ← execute!(MoveTo) 呼び出し
            terminal.write_str(&ch.to_string())?;  // ← write!() 呼び出し
        }
    }
    Ok(())
}
```

**src/terminal.rs の実装**:
```rust
pub fn move_to(&self, x: u16, y: u16) -> io::Result<()> {
    let mut stdout = io::stdout();
    execute!(stdout, MoveTo(x, y))?;  // ← 毎回新しい stdout インスタンス
    Ok(())
}

pub fn write_str(&self, s: &str) -> io::Result<()> {
    let mut stdout = io::stdout();
    write!(stdout, "{}", s)?;  // ← flush されない
    Ok(())
}
```

### Flickering メカニズム

**フレーム内での実行順序**:
```
1. clear() → execute!(Clear) + flush    # 画面が空白に
2. render_d51():
   - draw_line() で最初の行：
     * move_to() → execute!(MoveTo)
     * write_str() → write!() [flush されない]
     * move_to() → execute!(MoveTo)
     * write_str() → write!() [flush されない]
     ...
   - 複数の execute!() で部分的に画面が更新される
3. flush() → 最終的に flush

問題: Clear と初描画の間に「空白画面」が見える
     + 複数の execute!() で描画が分割される
     = チラつき効果
```

## ✅ 解決策

### 方針：Atomic 描画

**1つの execute!() で全コマンドを一括実行し、flush は最後に1回だけ**

```rust
render_frame() {
  // 1. 全コマンドを集めておく
  let commands = [
    Clear(ClearType::All),
    MoveTo(0, 10), Print("train_line_1"),
    MoveTo(0, 11), Print("train_line_2"),
    ...
  ];
  
  // 2. 1つの execute!() で一括実行
  execute!(stdout, commands)?;
  
  // 3. 最後に flush（これまで見えていなかった出力が確定）
  stdout.flush()?;
}
```

この方式だと：
- Clear と描画が atomic（分割されない）
- Flickering がない

## 📋 実装予定

### Step 1: Terminal API 再設計
- 複数の `move_to/write_str` 呼び出しの廃止
- 代わりに `render_atomic()` メソッド追加

### Step 2: Command ビルダーパターン
```rust
pub struct FrameBuilder {
  commands: Vec<...>,
}

impl FrameBuilder {
  fn add_text(&mut self, x, y, text) { ... }
  fn build(self) -> Vec<...> { self.commands }
}
```

### Step 3: Render ロジック改修
- `render_d51/c51/logo()` を文字列構築に変更
- MoveTo/Print コマンドをビルド

### Step 4: テスト
- Flickering 確認
- CPU/Memory チェック
