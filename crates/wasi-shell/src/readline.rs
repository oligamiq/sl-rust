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
        raw.c_lflag &= !(libc::ICANON | libc::ECHO);
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
            & !(win32::ENABLE_LINE_INPUT | win32::ENABLE_ECHO_INPUT))
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
// LineReader
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
pub struct LineReader {
    history: Vec<String>,
    max_history: usize,
}

impl LineReader {
    /// Create a new `LineReader` with the given maximum history size.
    pub fn new(max_history: usize) -> Self {
        Self {
            history: Vec::new(),
            max_history,
        }
    }

    /// Add a line to the history (skipping duplicates of the last entry and empty lines).
    fn push_history(&mut self, line: &str) {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            return;
        }
        if self.history.last().map(|s| s.as_str()) == Some(trimmed) {
            return;
        }
        self.history.push(trimmed.to_string());
        if self.history.len() > self.max_history {
            self.history.remove(0);
        }
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

    /// Run an interactive REPL loop, delegating each line to `handler`.
    ///
    /// The loop ends when:
    /// - The handler returns `Ok(LoopAction::Break)`
    /// - EOF is reached (Ctrl-D)
    /// - An I/O error occurs
    pub fn run_loop<P, H>(&mut self, prompt_fn: P, handler: &H, cancel_token: wasibox_core::CancellationToken) -> io::Result<()>
    where
        P: Fn() -> String,
        H: LineHandler,
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

    /// Testable REPL loop that reads from `reader` and writes to `writer`.
    #[cfg(test)]
    fn run_loop_from<R: Read, W: Write, H: LineHandler>(
        &mut self,
        reader: &mut R,
        writer: &mut W,
        prompt: &str,
        handler: &H,
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
    fn read_line_from<R: Read, W: Write>(
        &mut self,
        reader: &mut R,
        writer: &mut W,
        prompt: &str,
        cancel_token: Option<wasibox_core::CancellationToken>,
    ) -> io::Result<Option<String>> {
        let mut line = String::new();
        let mut cursor_pos: usize = 0;
        let mut history_idx: usize = self.history.len();
        let mut saved_input = String::new();

        loop {
            let b = {
                let mut buf = [0u8; 1];
                reader.read_exact(&mut buf)?;
                buf[0]
            };
            match b {
                // Ctrl-D on empty line => EOF
                4 => {
                    if line.is_empty() {
                        write!(writer, "\r\n")?;
                        writer.flush()?;
                        return Ok(None);
                    }
                }
                // Ctrl-C => discard line
                3 => {
                    if let Some(token) = &cancel_token {
                        token.cancel();
                    }
                    write!(writer, "^C\r\n")?;
                    writer.flush()?;
                    return Ok(Some(String::new()));
                }
                // Enter (CR or LF)
                b'\r' | b'\n' => {
                    write!(writer, "\r\n")?;
                    writer.flush()?;
                    self.push_history(&line);
                    return Ok(Some(line));
                }
                // Backspace (127 = DEL on most terminals, 8 = BS)
                127 | 8 => {
                    if cursor_pos > 0 {
                        cursor_pos -= 1;
                        line.remove(cursor_pos);
                        Self::redraw_line(writer, prompt, &line, cursor_pos)?;
                    }
                }
                // Ctrl-A => Home
                1 => {
                    cursor_pos = 0;
                    Self::redraw_line(writer, prompt, &line, cursor_pos)?;
                }
                // Ctrl-E => End
                5 => {
                    cursor_pos = line.len();
                    Self::redraw_line(writer, prompt, &line, cursor_pos)?;
                }
                // Ctrl-U => clear line
                21 => {
                    line.clear();
                    cursor_pos = 0;
                    Self::redraw_line(writer, prompt, &line, cursor_pos)?;
                }
                // Ctrl-K => kill to end of line
                11 => {
                    line.truncate(cursor_pos);
                    Self::redraw_line(writer, prompt, &line, cursor_pos)?;
                }
                // Ctrl-W => delete word backwards
                23 => {
                    if cursor_pos > 0 {
                        let mut new_pos = cursor_pos;
                        while new_pos > 0 && line.as_bytes()[new_pos - 1] == b' ' {
                            new_pos -= 1;
                        }
                        while new_pos > 0 && line.as_bytes()[new_pos - 1] != b' ' {
                            new_pos -= 1;
                        }
                        line.drain(new_pos..cursor_pos);
                        cursor_pos = new_pos;
                        Self::redraw_line(writer, prompt, &line, cursor_pos)?;
                    }
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
                            // Up arrow
                            b'A' => {
                                if !self.history.is_empty() && history_idx > 0 {
                                    if history_idx == self.history.len() {
                                        saved_input = line.clone();
                                    }
                                    history_idx -= 1;
                                    line = self.history[history_idx].clone();
                                    cursor_pos = line.len();
                                    Self::redraw_line(writer, prompt, &line, cursor_pos)?;
                                }
                            }
                            // Down arrow
                            b'B' => {
                                if history_idx < self.history.len() {
                                    history_idx += 1;
                                    if history_idx == self.history.len() {
                                        line = saved_input.clone();
                                    } else {
                                        line = self.history[history_idx].clone();
                                    }
                                    cursor_pos = line.len();
                                    Self::redraw_line(writer, prompt, &line, cursor_pos)?;
                                }
                            }
                            // Right arrow
                            b'C' => {
                                if cursor_pos < line.len() {
                                    cursor_pos += 1;
                                    write!(writer, "\x1b[C")?;
                                    writer.flush()?;
                                }
                            }
                            // Left arrow
                            b'D' => {
                                if cursor_pos > 0 {
                                    cursor_pos -= 1;
                                    write!(writer, "\x1b[D")?;
                                    writer.flush()?;
                                }
                            }
                            // Home
                            b'H' => {
                                cursor_pos = 0;
                                Self::redraw_line(writer, prompt, &line, cursor_pos)?;
                            }
                            // End
                            b'F' => {
                                cursor_pos = line.len();
                                Self::redraw_line(writer, prompt, &line, cursor_pos)?;
                            }
                            // Delete: ESC [ 3 ~
                            b'3' => {
                                let seq3 = {
                                    let mut buf = [0u8; 1];
                                    reader.read_exact(&mut buf)?;
                                    buf[0]
                                };
                                if seq3 == b'~' && cursor_pos < line.len() {
                                    line.remove(cursor_pos);
                                    Self::redraw_line(writer, prompt, &line, cursor_pos)?;
                                }
                            }
                            _ => {
                                // Unknown escape sequence — ignore
                            }
                        }
                    }
                    // Else: lone ESC or ESC + unknown — ignore
                }
                // Printable ASCII
                b if b >= 0x20 => {
                    line.insert(cursor_pos, b as char);
                    cursor_pos += 1;
                    if cursor_pos == line.len() {
                        write!(writer, "{}", b as char)?;
                        writer.flush()?;
                    } else {
                        Self::redraw_line(writer, prompt, &line, cursor_pos)?;
                    }
                }
                _ => {
                    // Ignore other control characters
                }
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
    fn test_simple_input() {
        let mut reader = LineReader::new(100);
        let mut input = keys(&[b"hello", ENTER]);
        let mut out = Vec::new();
        let result = reader.read_line_from(&mut input, &mut out, "$ ", None).unwrap();
        assert_eq!(result, Some("hello".to_string()));
    }

    #[test]
    fn test_eof_on_empty() {
        let mut reader = LineReader::new(100);
        let mut input = Cursor::new(vec![4u8]); // Ctrl-D
        let mut out = Vec::new();
        let result = reader.read_line_from(&mut input, &mut out, "$ ", None).unwrap();
        assert_eq!(result, None);
    }

    #[test]
    fn test_history_up_arrow() {
        let mut reader = LineReader::new(100);
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
        let mut reader = LineReader::new(100);
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
    fn test_history_down_restores_current_input() {
        let mut reader = LineReader::new(100);
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
        let mut reader = LineReader::new(100);
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
        let mut reader = LineReader::new(3);
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
        let mut reader = LineReader::new(100);
        let mut input = keys(&[b"helloo", &[127], ENTER]);
        let mut out = Vec::new();
        let result = reader.read_line_from(&mut input, &mut out, "$ ", None).unwrap();
        assert_eq!(result, Some("hello".to_string()));
    }

    #[test]
    fn test_ctrl_u_clears_line() {
        let mut reader = LineReader::new(100);
        let mut input = keys(&[b"garbage", &[21], b"clean", ENTER]); // Ctrl-U = 21
        let mut out = Vec::new();
        let result = reader.read_line_from(&mut input, &mut out, "$ ", None).unwrap();
        assert_eq!(result, Some("clean".to_string()));
    }

    #[test]
    fn test_empty_line_not_in_history() {
        let mut reader = LineReader::new(100);
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

        let mut reader = LineReader::new(100);
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

        let mut reader = LineReader::new(100);
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

        let mut reader = LineReader::new(100);
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

        let mut reader = LineReader::new(100);
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

        let mut reader = LineReader::new(100);
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
}
