use std::io::{self, Read, Write};

// ---------------------------------------------------------------------------
// Platform-specific raw terminal mode
// ---------------------------------------------------------------------------

#[cfg(unix)]
struct RawModeGuard {
    original: libc::termios,
}

#[cfg(unix)]
impl RawModeGuard {
    fn enter() -> io::Result<Self> {
        let mut original: libc::termios = unsafe { std::mem::zeroed() };
        if unsafe { libc::tcgetattr(libc::STDIN_FILENO, &mut original) } != 0 {
            return Err(io::Error::last_os_error());
        }
        let mut raw = original;
        raw.c_lflag &= !(libc::ICANON | libc::ECHO | libc::ISIG);
        raw.c_cc[libc::VMIN] = 1;
        raw.c_cc[libc::VTIME] = 0;
        if unsafe { libc::tcsetattr(libc::STDIN_FILENO, libc::TCSANOW, &raw) } != 0 {
            return Err(io::Error::last_os_error());
        }
        Ok(Self { original })
    }
}

#[cfg(unix)]
impl Drop for RawModeGuard {
    fn drop(&mut self) {
        unsafe {
            libc::tcsetattr(libc::STDIN_FILENO, libc::TCSANOW, &self.original);
        }
    }
}

// ---- Windows ----

#[cfg(windows)]
mod win32 {
    #[link(name = "kernel32")]
    unsafe extern "system" {
        pub fn GetStdHandle(nStdHandle: u32) -> isize;
        pub fn GetConsoleMode(hConsoleHandle: isize, lpMode: *mut u32) -> i32;
        pub fn SetConsoleMode(hConsoleHandle: isize, dwMode: u32) -> i32;
    }

    pub const STD_INPUT_HANDLE: u32 = 0xFFFF_FFF6; // (DWORD)-10
    pub const ENABLE_PROCESSED_INPUT: u32 = 0x0001;
    pub const ENABLE_LINE_INPUT: u32 = 0x0002;
    pub const ENABLE_ECHO_INPUT: u32 = 0x0004;
    pub const ENABLE_VIRTUAL_TERMINAL_INPUT: u32 = 0x0200;
}

#[cfg(windows)]
struct RawModeGuard {
    handle: isize,
    original_mode: u32,
}

#[cfg(windows)]
impl RawModeGuard {
    fn enter() -> io::Result<Self> {
        let handle = unsafe { win32::GetStdHandle(win32::STD_INPUT_HANDLE) };
        if handle == -1 {
            return Err(io::Error::last_os_error());
        }
        let mut original_mode: u32 = 0;
        if unsafe { win32::GetConsoleMode(handle, &mut original_mode) } == 0 {
            return Err(io::Error::last_os_error());
        }
        let new_mode = (original_mode
            & !(win32::ENABLE_LINE_INPUT | win32::ENABLE_ECHO_INPUT | win32::ENABLE_PROCESSED_INPUT))
            | win32::ENABLE_VIRTUAL_TERMINAL_INPUT;
        if unsafe { win32::SetConsoleMode(handle, new_mode) } == 0 {
            return Err(io::Error::last_os_error());
        }
        Ok(Self {
            handle,
            original_mode,
        })
    }
}

#[cfg(windows)]
impl Drop for RawModeGuard {
    fn drop(&mut self) {
        unsafe {
            win32::SetConsoleMode(self.handle, self.original_mode);
        }
    }
}

// ---- WASI ----

#[cfg(target_os = "wasi")]
struct RawModeGuard;

#[cfg(target_os = "wasi")]
impl RawModeGuard {
    fn enter() -> io::Result<Self> {
        Ok(Self)
    }
}

// ---------------------------------------------------------------------------
// LineHandler trait
// ---------------------------------------------------------------------------

/// Trait for handling lines read by [`LineReader`].
///
/// Implement this to inject command-execution logic into the REPL loop.
///
/// # Return values
///
/// - `Ok(LoopAction::Continue)` — prompt for the next line.
/// - `Ok(LoopAction::Break)` — exit the loop normally.
/// - `Err(msg)` — print the error and continue.
pub trait LineHandler {
    fn handle_line(&self, line: &str) -> Result<LoopAction, String>;
}

/// Controls the REPL loop flow after a line is handled.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LoopAction {
    /// Continue reading the next line.
    Continue,
    /// Exit the REPL loop.
    Break,
}

// Blanket impl: closures that return Result<LoopAction, String>
impl<F> LineHandler for F
where
    F: Fn(&str) -> Result<LoopAction, String>,
{
    fn handle_line(&self, line: &str) -> Result<LoopAction, String> {
        self(line)
    }
}

// ---------------------------------------------------------------------------
// KeyEvent, KeyEventHandler, and LineBuffer
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyEvent {
    Char(char),
    Enter,
    Backspace,
    Delete,
    Up,
    Down,
    Left,
    Right,
    Home,
    End,
    CtrlA,
    CtrlE,
    CtrlU,
    CtrlK,
    CtrlW,
    CtrlD,
    CtrlC,
    Esc,
}

pub trait KeyEventHandler {
    fn on_key_event(&mut self, key: KeyEvent);
}

struct NoopKeyEventHandler;

impl KeyEventHandler for NoopKeyEventHandler {
    fn on_key_event(&mut self, _key: KeyEvent) {}
}

/// A stateful editor for a single line of text buffer.
pub struct LineBuffer {
    pub buffer: String,
    pub cursor_pos: usize,
}

impl LineBuffer {
    /// Create a new `LineBuffer` initialized for a new line.
    pub const fn new() -> Self {
        Self {
            buffer: String::new(),
            cursor_pos: 0,
        }
    }

    pub fn set_buffer(&mut self, text: String) {
        self.buffer = text;
        self.cursor_pos = self.buffer.len();
    }

    pub fn apply_key(&mut self, key: KeyEvent) {
        match key {
            KeyEvent::Char(ch) => {
                self.buffer.insert(self.cursor_pos, ch);
                self.cursor_pos += 1;
            }
            KeyEvent::Backspace => {
                if self.cursor_pos > 0 {
                    self.cursor_pos -= 1;
                    self.buffer.remove(self.cursor_pos);
                }
            }
            KeyEvent::Delete => {
                if self.cursor_pos < self.buffer.len() {
                    self.buffer.remove(self.cursor_pos);
                }
            }
            KeyEvent::Left => {
                if self.cursor_pos > 0 {
                    self.cursor_pos -= 1;
                }
            }
            KeyEvent::Right => {
                if self.cursor_pos < self.buffer.len() {
                    self.cursor_pos += 1;
                }
            }
            KeyEvent::Home | KeyEvent::CtrlA => {
                self.cursor_pos = 0;
            }
            KeyEvent::End | KeyEvent::CtrlE => {
                self.cursor_pos = self.buffer.len();
            }
            KeyEvent::CtrlU => {
                self.buffer.clear();
                self.cursor_pos = 0;
            }
            KeyEvent::CtrlK => {
                self.buffer.truncate(self.cursor_pos);
            }
            KeyEvent::CtrlW => {
                if self.cursor_pos > 0 {
                    let mut new_pos = self.cursor_pos;
                    while new_pos > 0 && self.buffer.as_bytes().get(new_pos - 1) == Some(&b' ') {
                        new_pos -= 1;
                    }
                    while new_pos > 0 && self.buffer.as_bytes().get(new_pos - 1) != Some(&b' ') {
                        new_pos -= 1;
                    }
                    self.buffer.drain(new_pos..self.cursor_pos);
                    self.cursor_pos = new_pos;
                }
            }
            KeyEvent::Esc => {}
            _ => {}
        }
    }
}

// ---------------------------------------------------------------------------
// History
// ---------------------------------------------------------------------------

pub trait History {
    fn push(&mut self, line: &str);
    fn len(&self) -> usize;
    fn get(&self, index: usize) -> Option<String>;
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

pub struct InMemoryHistory {
    entries: Vec<String>,
    max_len: usize,
}

impl InMemoryHistory {
    pub const fn new(max_len: usize) -> Self {
        Self {
            entries: Vec::new(),
            max_len,
        }
    }
}

impl History for InMemoryHistory {
    fn push(&mut self, line: &str) {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            return;
        }
        if self.entries.last().map(|s| s.as_str()) == Some(trimmed) {
            return;
        }
        self.entries.push(trimmed.to_string());
        if self.entries.len() > self.max_len {
            self.entries.remove(0);
        }
    }

    fn len(&self) -> usize {
        self.entries.len()
    }

    fn get(&self, index: usize) -> Option<String> {
        self.entries.get(index).cloned()
    }
}

// ---------------------------------------------------------------------------
// LineEditor
// ---------------------------------------------------------------------------

/// A minimal line editor with command history.
///
/// Supports:
/// - Up/Down arrow keys to navigate command history
/// - Left/Right arrow keys to move the cursor within the line
/// - Home/End to jump to the beginning/end of the line
/// - Delete to remove the character under the cursor
/// - Backspace to remove the character before the cursor
/// - Ctrl-U to clear the line, Ctrl-K to kill to end of line
/// - Ctrl-W to delete the previous word
/// - Ctrl-A / Ctrl-E for Home / End
pub struct LineEditor<H: History = InMemoryHistory> {
    line_buffer: LineBuffer,
    history: H,
    history_idx: usize,
    saved_input: String,
    esc_buf: smallvec::SmallVec<[u8; 4]>,
}

impl LineEditor<InMemoryHistory> {
    /// Create a new `LineEditor` with the given maximum history size.
    pub const fn new(max_history: usize) -> Self {
        Self::with_history_and_len(InMemoryHistory::new(max_history), 0)
    }
}

impl<H: History> LineEditor<H> {
    pub fn with_history(history: H) -> Self {
        let history_idx = history.len();
        Self {
            line_buffer: LineBuffer::new(),
            history,
            history_idx,
            saved_input: String::new(),
            esc_buf: smallvec::SmallVec::new_const(),
        }
    }

    pub const fn with_history_and_len(history: H, len: usize) -> Self {
        Self {
            line_buffer: LineBuffer::new(),
            history,
            history_idx: len,
            saved_input: String::new(),
            esc_buf: smallvec::SmallVec::new_const(),
        }
    }

    pub fn buffer(&self) -> &str {
        &self.line_buffer.buffer
    }

    pub fn cursor_pos(&self) -> usize {
        self.line_buffer.cursor_pos
    }

    /// Prepares the internal state for a new line of input.
    pub fn start_new_line(&mut self) {
        self.line_buffer.buffer.clear();
        self.line_buffer.cursor_pos = 0;
        self.history_idx = self.history.len();
        self.saved_input.clear();
        self.esc_buf.clear();
    }

    pub fn input_char(&mut self, code: u32) -> Option<String> {
        self.input_char_with_handler(code, &mut NoopKeyEventHandler)
    }

    pub fn input_char_with_handler<K: KeyEventHandler>(&mut self, code: u32, handler: &mut K) -> Option<String> {
        // 1. Handle escape sequence state machine
        if code == 27 {
            self.esc_buf.clear();
            self.esc_buf.push(27);
            return None;
        }

        if !self.esc_buf.is_empty() {
            self.esc_buf.push(code as u8);
            let seq = self.esc_buf.as_slice();

            let key = match seq {
                [27, b'[', b'A'] => Some(KeyEvent::Up),
                [27, b'[', b'B'] => Some(KeyEvent::Down),
                [27, b'[', b'C'] => Some(KeyEvent::Right),
                [27, b'[', b'D'] => Some(KeyEvent::Left),
                [27, b'[', b'H'] => Some(KeyEvent::Home),
                [27, b'[', b'F'] => Some(KeyEvent::End),
                [27, b'[', b'3', b'~'] => Some(KeyEvent::Delete),
                _ => {
                    // Check if it's still potentially a valid prefix
                    if seq.len() >= 4 || (seq.len() == 2 && seq[1] != b'[') {
                        // Invalid or unsupported sequence
                        self.esc_buf.clear();
                        None
                    } else {
                        // Keep waiting for more bytes
                        return None;
                    }
                }
            };

            if let Some(k) = key {
                self.esc_buf.clear();
                return self.handle_key_event(k, handler);
            }
            // If the sequence was invalid, we fall through to process the current 'code'
        }

        // 2. Map single code to KeyEvent
        let key = match code {
            // Control characters
            1 => KeyEvent::CtrlA,
            3 => KeyEvent::CtrlC,
            4 => KeyEvent::CtrlD,
            5 => KeyEvent::CtrlE,
            8 | 127 => KeyEvent::Backspace,
            11 => KeyEvent::CtrlK,
            13 | 10 => KeyEvent::Enter,
            21 => KeyEvent::CtrlU,
            23 => KeyEvent::CtrlW,
            27 => KeyEvent::Esc,

            // Custom codes for special keys (defined by the caller/LineEditor)
            1001 => KeyEvent::Up,
            1002 => KeyEvent::Down,
            1003 => KeyEvent::Right,
            1004 => KeyEvent::Left,
            1005 => KeyEvent::Home,
            1006 => KeyEvent::End,
            1007 => KeyEvent::Delete,

            // Printable characters
            c if c >= 0x20 && c < 1000 => KeyEvent::Char(char::from_u32(c).unwrap_or(' ')),
            _ => return None,
        };

        self.handle_key_event(key, handler)
    }

    fn handle_key_event<K: KeyEventHandler>(&mut self, key: KeyEvent, handler: &mut K) -> Option<String> {
        handler.on_key_event(key);

        if key == KeyEvent::Enter {
            let final_line = self.line_buffer.buffer.clone();
            self.history.push(&final_line);
            self.start_new_line();
            return Some(final_line);
        }

        match key {
            KeyEvent::Up => {
                if !self.history.is_empty() && self.history_idx > 0 {
                    if self.history_idx == self.history.len() {
                        self.saved_input = self.line_buffer.buffer.clone();
                    }
                    self.history_idx -= 1;
                    if let Some(hist_line) = self.history.get(self.history_idx) {
                        self.line_buffer.set_buffer(hist_line);
                    }
                }
            }
            KeyEvent::Down => {
                if self.history_idx < self.history.len() {
                    self.history_idx += 1;
                    if self.history_idx == self.history.len() {
                        self.line_buffer.set_buffer(self.saved_input.clone());
                    } else if let Some(hist_line) = self.history.get(self.history_idx) {
                        self.line_buffer.set_buffer(hist_line);
                    }
                }
            }
            _ => {
                self.line_buffer.apply_key(key);
            }
        }

        None
    }

    /// Read a line interactively with arrow-key history navigation.
    ///
    /// Returns `Ok(Some(line))` on success, `Ok(None)` on EOF (Ctrl-D).
    pub fn read_line(&mut self, prompt: &str, cancel_token: Option<wasibox_core::CancellationToken>) -> io::Result<Option<String>> {
        let mut stdout = io::stdout();
        write!(stdout, "{}", prompt)?;
        stdout.flush()?;

        let _guard = RawModeGuard::enter()?;

        let mut reader = io::stdin();
        self.read_line_from(&mut reader, &mut stdout, prompt, cancel_token)
    }

    /// Read a line interactively using a provided reader.
    pub fn read_line_with_stdin(&mut self, prompt: &str, cancel_token: Option<wasibox_core::CancellationToken>, mut reader: Box<dyn Read>) -> io::Result<Option<String>> {
        let mut stdout = io::stdout();
        write!(stdout, "{}", prompt)?;
        stdout.flush()?;

        let _guard = RawModeGuard::enter()?;

        self.read_line_from(&mut reader, &mut stdout, prompt, cancel_token)
    }

    /// Run an interactive REPL loop, delegating each line to `handler`.
    ///
    /// The loop ends when:
    /// - The handler returns `Ok(LoopAction::Break)`
    /// - EOF is reached (Ctrl-D)
    /// - An I/O error occurs
    pub fn run_loop<P, L>(&mut self, prompt_fn: P, handler: &L, cancel_token: wasibox_core::CancellationToken) -> io::Result<()>
    where
        P: Fn() -> String,
        L: LineHandler,
    {
        loop {
            let prompt = prompt_fn();
            match self.read_line(&prompt, Some(cancel_token.clone()))? {
                None => break,
                Some(line) => {
                    let trimmed = line.trim();
                    if trimmed.is_empty() {
                        continue;
                    }
                    match handler.handle_line(trimmed) {
                        Ok(LoopAction::Continue) => {}
                        Ok(LoopAction::Break) => break,
                        Err(e) => {
                            eprintln!("{}", e);
                        }
                    }
                }
            }
        }
        Ok(())
    }

    /// Run an interactive REPL loop using a provided reader.
    pub fn run_loop_with_stdin<P, L>(&mut self, prompt_fn: P, handler: &L, cancel_token: wasibox_core::CancellationToken, mut reader: Box<dyn Read>) -> io::Result<()>
    where
        P: Fn() -> String,
        L: LineHandler,
    {
        loop {
            let prompt = prompt_fn();
            let _guard = RawModeGuard::enter()?;
            match self.read_line_from(&mut reader, &mut io::stdout(), &prompt, Some(cancel_token.clone()))? {
                None => break,
                Some(line) => {
                    let trimmed = line.trim();
                    if trimmed.is_empty() {
                        continue;
                    }
                    match handler.handle_line(trimmed) {
                        Ok(LoopAction::Continue) => {}
                        Ok(LoopAction::Break) => break,
                        Err(e) => {
                            eprintln!("{}", e);
                        }
                    }
                }
            }
        }
        Ok(())
    }

    /// Testable REPL loop that reads from `reader` and writes to `writer`.
    #[cfg(test)]
    fn run_loop_from<R: Read, W: Write, L: LineHandler>(
        &mut self,
        reader: &mut R,
        writer: &mut W,
        prompt: &str,
        handler: &L,
        cancel_token: Option<wasibox_core::CancellationToken>,
    ) -> io::Result<()> {
        loop {
            write!(writer, "{}", prompt)?;
            writer.flush()?;
            match self.read_line_from(reader, writer, prompt, cancel_token.clone())? {
                None => break,
                Some(line) => {
                    let trimmed = line.trim();
                    if trimmed.is_empty() {
                        continue;
                    }
                    match handler.handle_line(trimmed) {
                        Ok(LoopAction::Continue) => {}
                        Ok(LoopAction::Break) => break,
                        Err(e) => {
                            writeln!(writer, "Error: {}", e)?;
                        }
                    }
                }
            }
        }
        Ok(())
    }

    /// Core line-editing logic, reading bytes from `reader` and writing to `writer`.
    /// Separated from `read_line` so it can be tested with synthetic input.
    pub fn read_line_from<R: Read, W: Write>(
        &mut self,
        reader: &mut R,
        writer: &mut W,
        prompt: &str,
        cancel_token: Option<wasibox_core::CancellationToken>,
    ) -> io::Result<Option<String>> {
        self.start_new_line();

        loop {
            let b = {
                let mut buf = [0u8; 1];
                reader.read_exact(&mut buf)?;
                buf[0]
            };

            let code = match b {
                // Ctrl-D on empty line => EOF
                4 => {
                    if self.buffer().is_empty() {
                        write!(writer, "\r\n")?;
                        writer.flush()?;
                        return Ok(None);
                    }
                    4
                }
                // Ctrl-C => discard line
                3 => {
                    if let Some(token) = &cancel_token {
                        token.cancel();
                    }
                    write!(writer, "^C\r\n")?;
                    writer.flush()?;
                    self.start_new_line(); // Reset on ctrl-c
                    return Ok(Some(String::new()));
                }
                // ESC => start of escape sequence
                27 => {
                    let seq1 = {
                        let mut buf = [0u8; 1];
                        reader.read_exact(&mut buf)?;
                        buf[0]
                    };
                    if seq1 == b'[' {
                        let seq2 = {
                            let mut buf = [0u8; 1];
                            reader.read_exact(&mut buf)?;
                            buf[0]
                        };
                        match seq2 {
                            b'A' => 1001, // Up
                            b'B' => 1002, // Down
                            b'C' => 1003, // Right
                            b'D' => 1004, // Left
                            b'H' => 1005, // Home
                            b'F' => 1006, // End
                            b'3' => {
                                let seq3 = {
                                    let mut buf = [0u8; 1];
                                    reader.read_exact(&mut buf)?;
                                    buf[0]
                                };
                                if seq3 == b'~' {
                                    1007 // Delete
                                } else {
                                    continue;
                                }
                            }
                            _ => continue,
                        }
                    } else {
                        continue;
                    }
                }
                other => other as u32,
            };

            let old_pos = self.cursor_pos();
            let old_len = self.buffer().len();

            if let Some(completed_line) = self.input_char(code) {
                write!(writer, "\r\n")?;
                writer.flush()?;
                return Ok(Some(completed_line));
            }

            // Redraw optimization
            if code >= 0x20 && code < 1000 && old_pos == old_len && self.cursor_pos() == self.buffer().len() {
                write!(writer, "{}", char::from_u32(code).unwrap())?;
                writer.flush()?;
            } else if code == 1004 && old_pos > self.cursor_pos() && old_pos > 0 { // Left
                write!(writer, "\x1b[D")?;
                writer.flush()?;
            } else if code == 1003 && old_pos < self.cursor_pos() && old_pos < old_len { // Right
                write!(writer, "\x1b[C")?;
                writer.flush()?;
            } else {
                Self::redraw_line(writer, prompt, self.buffer(), self.cursor_pos())?;
            }
        }
    }

    /// Redraw the current line (clear and rewrite).
    fn redraw_line<W: Write>(
        writer: &mut W,
        prompt: &str,
        line: &str,
        cursor_pos: usize,
    ) -> io::Result<()> {
        write!(writer, "\r\x1b[K{}{}", prompt, line)?;
        let total_len = prompt.len() + line.len();
        let target = prompt.len() + cursor_pos;
        if target < total_len {
            write!(writer, "\x1b[{}D", total_len - target)?;
        }
        writer.flush()
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    /// Helper: build a byte sequence from a list of key inputs.
    fn keys(parts: &[&[u8]]) -> Cursor<Vec<u8>> {
        let mut buf = Vec::new();
        for part in parts {
            buf.extend_from_slice(part);
        }
        Cursor::new(buf)
    }

    const UP: &[u8] = b"\x1b[A";
    const DOWN: &[u8] = b"\x1b[B";
    const ENTER: &[u8] = b"\r";

    #[test]
    fn test_line_editor_basic() {
        let mut editor = LineEditor::new(0);

        assert!(editor.input_char('a' as u32).is_none());
        assert!(editor.input_char('b' as u32).is_none());
        assert_eq!(editor.buffer(), "ab");
        assert_eq!(editor.cursor_pos(), 2);

        assert!(editor.input_char(1004).is_none()); // Left
        assert_eq!(editor.cursor_pos(), 1);

        assert!(editor.input_char('c' as u32).is_none());
        assert_eq!(editor.buffer(), "acb");
        assert_eq!(editor.cursor_pos(), 2);

        assert!(editor.input_char(127).is_none()); // Backspace
        assert_eq!(editor.buffer(), "ab");
        assert_eq!(editor.cursor_pos(), 1);

        let result = editor.input_char(13); // Enter
        assert_eq!(result, Some("ab".to_string()));
    }

    #[test]
    fn test_line_editor_history() {
        let mut editor = LineEditor::new(10);
        editor.input_char('f' as u32);
        editor.input_char('i' as u32);
        editor.input_char('r' as u32);
        editor.input_char('s' as u32);
        editor.input_char('t' as u32);
        editor.input_char(13); // Enter saves "first"

        editor.input_char('s' as u32);
        editor.input_char('e' as u32);
        editor.input_char('c' as u32);
        editor.input_char('o' as u32);
        editor.input_char('n' as u32);
        editor.input_char('d' as u32);
        editor.input_char(13); // Enter saves "second"

        editor.input_char(1001); // Up
        assert_eq!(editor.buffer(), "second");

        editor.input_char(1001); // Up
        assert_eq!(editor.buffer(), "first");

        editor.input_char(1002); // Down
        assert_eq!(editor.buffer(), "second");

        editor.input_char(1002); // Down
        assert_eq!(editor.buffer(), ""); // Back to current
    }

    #[test]
    fn test_simple_input() {
        let mut reader = LineEditor::new(100);
        let mut input = keys(&[b"hello", ENTER]);
        let mut out = Vec::new();
        let result = reader.read_line_from(&mut input, &mut out, "$ ", None).unwrap();
        assert_eq!(result, Some("hello".to_string()));
    }

    #[test]
    fn test_eof_on_empty() {
        let mut reader = LineEditor::new(100);
        let mut input = Cursor::new(vec![4u8]); // Ctrl-D
        let mut out = Vec::new();
        let result = reader.read_line_from(&mut input, &mut out, "$ ", None).unwrap();
        assert_eq!(result, None);
    }

    #[test]
    fn test_history_up_arrow() {
        let mut reader = LineEditor::new(100);
        let mut out = Vec::new();

        // First command
        let mut input = keys(&[b"echo hello", ENTER]);
        reader.read_line_from(&mut input, &mut out, "$ ", None).unwrap();

        // Second command: press Up then Enter (should recall "echo hello")
        let mut input = keys(&[UP, ENTER]);
        out.clear();
        let result = reader.read_line_from(&mut input, &mut out, "$ ", None).unwrap();
        assert_eq!(result, Some("echo hello".to_string()));
    }

    #[test]
    fn test_history_up_down_arrow() {
        let mut reader = LineEditor::new(100);
        let mut out = Vec::new();

        // Enter two commands
        let mut input = keys(&[b"first", ENTER]);
        reader.read_line_from(&mut input, &mut out, "$ ", None).unwrap();
        let mut input = keys(&[b"second", ENTER]);
        reader.read_line_from(&mut input, &mut out, "$ ", None).unwrap();

        // Up twice => "first", Down once => "second", Enter
        let mut input = keys(&[UP, UP, DOWN, ENTER]);
        out.clear();
        let result = reader.read_line_from(&mut input, &mut out, "$ ", None).unwrap();
        assert_eq!(result, Some("second".to_string()));
    }

    #[test]
    fn test_input_char_sequence() {
        let mut editor = LineEditor::new(10);
        editor.input_char('f' as u32);
        editor.input_char('i' as u32);
        editor.input_char('r' as u32);
        editor.input_char('s' as u32);
        editor.input_char('t' as u32);
        editor.input_char(13); // Enter saves "first"

        editor.input_char('s' as u32);
        editor.input_char('e' as u32);
        editor.input_char('c' as u32);
        editor.input_char('o' as u32);
        editor.input_char('n' as u32);
        editor.input_char('d' as u32);
        editor.input_char(13); // Enter saves "second"

        // Send Down arrow as [27, 91, 66]
        // History: ["first", "second"], idx starts at 2
        // Press Up twice to get to "first"
        editor.input_char(1001); // Up -> "second"
        editor.input_char(1001); // Up -> "first"
        assert_eq!(editor.buffer(), "first");

        // Now Down via sequence
        editor.input_char(27); // ESC
        editor.input_char(91); // '['
        editor.input_char(66); // 'B' -> Down
        assert_eq!(editor.buffer(), "second");
    }

    #[test]
    fn test_history_down_restores_current_input() {
        let mut reader = LineEditor::new(100);
        let mut out = Vec::new();

        // Enter a command into history
        let mut input = keys(&[b"old", ENTER]);
        reader.read_line_from(&mut input, &mut out, "$ ", None).unwrap();

        // Type "new", press Up (recalls "old"), press Down (restores "new"), Enter
        let mut input = keys(&[b"new", UP, DOWN, ENTER]);
        out.clear();
        let result = reader.read_line_from(&mut input, &mut out, "$ ", None).unwrap();
        assert_eq!(result, Some("new".to_string()));
    }

    #[test]
    fn test_history_dedup() {
        let mut reader = LineEditor::new(100);
        let mut out = Vec::new();

        // Enter same command twice
        let mut input = keys(&[b"dup", ENTER]);
        reader.read_line_from(&mut input, &mut out, "$ ", None).unwrap();
        let mut input = keys(&[b"dup", ENTER]);
        reader.read_line_from(&mut input, &mut out, "$ ", None).unwrap();

        // Up should recall "dup", another Up should NOT go further
        // (only one entry in history)
        let mut input = keys(&[UP, UP, ENTER]);
        out.clear();
        let result = reader.read_line_from(&mut input, &mut out, "$ ", None).unwrap();
        assert_eq!(result, Some("dup".to_string()));
    }

    #[test]
    fn test_history_max_size() {
        let mut reader = LineEditor::new(3);
        let mut out = Vec::new();

        for cmd in &["aaa", "bbb", "ccc", "ddd"] {
            let mut input = keys(&[cmd.as_bytes(), ENTER]);
            reader.read_line_from(&mut input, &mut out, "$ ", None).unwrap();
        }

        // Up 3 times should stop at "bbb" (oldest "aaa" was evicted)
        let mut input = keys(&[UP, UP, UP, ENTER]);
        out.clear();
        let result = reader.read_line_from(&mut input, &mut out, "$ ", None).unwrap();
        assert_eq!(result, Some("bbb".to_string()));
    }

    #[test]
    fn test_backspace() {
        let mut reader = LineEditor::new(100);
        let mut input = keys(&[b"helloo", &[127], ENTER]);
        let mut out = Vec::new();
        let result = reader.read_line_from(&mut input, &mut out, "$ ", None).unwrap();
        assert_eq!(result, Some("hello".to_string()));
    }

    #[test]
    fn test_ctrl_u_clears_line() {
        let mut reader = LineEditor::new(100);
        let mut input = keys(&[b"garbage", &[21], b"clean", ENTER]); // Ctrl-U = 21
        let mut out = Vec::new();
        let result = reader.read_line_from(&mut input, &mut out, "$ ", None).unwrap();
        assert_eq!(result, Some("clean".to_string()));
    }

    #[test]
    fn test_empty_line_not_in_history() {
        let mut reader = LineEditor::new(100);
        let mut out = Vec::new();

        // Enter a real command
        let mut input = keys(&[b"real", ENTER]);
        reader.read_line_from(&mut input, &mut out, "$ ", None).unwrap();

        // Enter an empty line
        let mut input = keys(&[ENTER]);
        reader.read_line_from(&mut input, &mut out, "$ ", None).unwrap();

        // Up should still recall "real", not empty
        let mut input = keys(&[UP, ENTER]);
        out.clear();
        let result = reader.read_line_from(&mut input, &mut out, "$ ", None).unwrap();
        assert_eq!(result, Some("real".to_string()));
    }

    // ── LineHandler / run_loop tests ─────────────────────────────────────

    #[test]
    fn test_run_loop_with_handler() {
        use std::sync::{Arc, Mutex};

        let executed = Arc::new(Mutex::new(Vec::new()));
        let exec_clone = Arc::clone(&executed);

        let handler = move |line: &str| -> Result<LoopAction, String> {
            exec_clone.lock().unwrap().push(line.to_string());
            Ok(LoopAction::Continue)
        };

        let mut reader = LineEditor::new(100);
        // Type two commands then Ctrl-D
        let mut input = keys(&[b"echo hello", ENTER, b"ls", ENTER, &[4]]);
        let mut out = Vec::new();
        reader.run_loop_from(&mut input, &mut out, "$ ", &handler, None).unwrap();

        let cmds = executed.lock().unwrap();
        assert_eq!(cmds.len(), 2);
        assert_eq!(cmds[0], "echo hello");
        assert_eq!(cmds[1], "ls");
    }

    #[test]
    fn test_run_loop_break_on_exit() {
        let handler = |line: &str| -> Result<LoopAction, String> {
            if line == "exit" {
                Ok(LoopAction::Break)
            } else {
                Ok(LoopAction::Continue)
            }
        };

        let mut reader = LineEditor::new(100);
        let mut input = keys(&[b"cmd1", ENTER, b"exit", ENTER, b"cmd2", ENTER]);
        let mut out = Vec::new();
        reader.run_loop_from(&mut input, &mut out, "$ ", &handler, None).unwrap();
        // Loop should have stopped after "exit"; "cmd2" is never processed.
    }

    #[test]
    fn test_run_loop_error_continues() {
        use std::sync::{Arc, Mutex};

        let count = Arc::new(Mutex::new(0u32));
        let count_clone = Arc::clone(&count);

        let handler = move |line: &str| -> Result<LoopAction, String> {
            *count_clone.lock().unwrap() += 1;
            if line == "fail" {
                Err("simulated error".to_string())
            } else {
                Ok(LoopAction::Continue)
            }
        };

        let mut reader = LineEditor::new(100);
        let mut input = keys(&[b"ok", ENTER, b"fail", ENTER, b"ok2", ENTER, &[4]]);
        let mut out = Vec::new();
        reader.run_loop_from(&mut input, &mut out, "$ ", &handler, None).unwrap();

        // All three commands should have been processed (error doesn't stop loop)
        assert_eq!(*count.lock().unwrap(), 3);
    }

    #[test]
    fn test_run_loop_with_history_navigation() {
        use std::sync::{Arc, Mutex};

        let executed = Arc::new(Mutex::new(Vec::new()));
        let exec_clone = Arc::clone(&executed);

        let handler = move |line: &str| -> Result<LoopAction, String> {
            exec_clone.lock().unwrap().push(line.to_string());
            Ok(LoopAction::Continue)
        };

        let mut reader = LineEditor::new(100);
        // Enter "echo hello", then press Up+Enter to replay it
        let mut input = keys(&[
            b"echo hello", ENTER,
            UP, ENTER,  // replay from history
            &[4],       // EOF
        ]);
        let mut out = Vec::new();
        reader.run_loop_from(&mut input, &mut out, "$ ", &handler, None).unwrap();

        let cmds = executed.lock().unwrap();
        assert_eq!(cmds.len(), 2);
        assert_eq!(cmds[0], "echo hello");
        assert_eq!(cmds[1], "echo hello"); // replayed from history
    }

    #[test]
    fn test_run_loop_with_handle_parallel() {
        use std::sync::{Arc, Mutex};
        use crate::{CommandRegistry, handle_parallel, ArcVecWriter};

        let registry = Arc::new(CommandRegistry::with_builtins());
        let output = Arc::new(Mutex::new(Vec::<u8>::new()));

        let reg = Arc::clone(&registry);
        let out_ref = Arc::clone(&output);

        let handler = move |line: &str| -> Result<LoopAction, String> {
            if line == "exit" {
                return Ok(LoopAction::Break);
            }
            let results = handle_parallel(
                vec![line.to_string()],
                Box::new(std::io::empty()),
                Box::new(ArcVecWriter { inner: Arc::clone(&out_ref) }),
                Arc::clone(&reg),
                wasibox_core::CancellationToken::new(),
            );
            for res in results {
                res?;
            }
            Ok(LoopAction::Continue)
        };

        let mut reader = LineEditor::new(100);
        // Run "echo hello", then Up+Enter to replay, then "exit"
        let mut input = keys(&[
            b"echo hello", ENTER,
            UP, ENTER,  // replay "echo hello" via history
            b"exit", ENTER,
        ]);
        let mut term_out = Vec::new();
        reader.run_loop_from(&mut input, &mut term_out, "$ ", &handler, None).unwrap();

        let buf = output.lock().unwrap();
        let result = String::from_utf8_lossy(&buf);
        let lines: Vec<&str> = result.trim().lines().collect();
        assert_eq!(lines.len(), 2);
        assert_eq!(lines[0], "hello");
        assert_eq!(lines[1], "hello"); // replayed from history
    }

    #[test]
    fn test_key_event_handler() {
        struct MockHandler {
            events: Vec<KeyEvent>,
        }
        impl KeyEventHandler for MockHandler {
            fn on_key_event(&mut self, key: KeyEvent) {
                self.events.push(key);
            }
        }

        let mut editor = LineEditor::new(10);
        let mut handler = MockHandler { events: Vec::new() };

        editor.input_char_with_handler('a' as u32, &mut handler);
        editor.input_char_with_handler('b' as u32, &mut handler);
        editor.input_char_with_handler(13, &mut handler);

        assert_eq!(handler.events, vec![
            KeyEvent::Char('a'),
            KeyEvent::Char('b'),
            KeyEvent::Enter,
        ]);
    }
}
