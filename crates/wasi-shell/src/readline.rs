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
// On WASI the host runtime controls the terminal mode.
// We assume the host provides character-at-a-time input.

#[cfg(target_os = "wasi")]
struct RawModeGuard;

#[cfg(target_os = "wasi")]
impl RawModeGuard {
    fn enter() -> io::Result<Self> {
        Ok(Self)
    }
}

// ---------------------------------------------------------------------------
// Byte reader
// ---------------------------------------------------------------------------

fn read_byte() -> io::Result<u8> {
    let mut buf = [0u8; 1];
    io::stdin().read_exact(&mut buf)?;
    Ok(buf[0])
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
    pub fn read_line(&mut self, prompt: &str) -> io::Result<Option<String>> {
        let mut stdout = io::stdout();
        write!(stdout, "{}", prompt)?;
        stdout.flush()?;

        let _guard = RawModeGuard::enter()?;

        let mut line = String::new();
        let mut cursor_pos: usize = 0;
        let mut history_idx: usize = self.history.len();
        let mut saved_input = String::new();

        loop {
            let b = read_byte()?;
            match b {
                // Ctrl-D on empty line => EOF
                4 => {
                    if line.is_empty() {
                        write!(stdout, "\r\n")?;
                        stdout.flush()?;
                        return Ok(None);
                    }
                }
                // Ctrl-C => discard line
                3 => {
                    write!(stdout, "^C\r\n")?;
                    stdout.flush()?;
                    return Ok(Some(String::new()));
                }
                // Enter (CR or LF)
                b'\r' | b'\n' => {
                    write!(stdout, "\r\n")?;
                    stdout.flush()?;
                    self.push_history(&line);
                    return Ok(Some(line));
                }
                // Backspace (127 = DEL on most terminals, 8 = BS)
                127 | 8 => {
                    if cursor_pos > 0 {
                        cursor_pos -= 1;
                        line.remove(cursor_pos);
                        Self::redraw_line(&mut stdout, prompt, &line, cursor_pos)?;
                    }
                }
                // Ctrl-A => Home
                1 => {
                    cursor_pos = 0;
                    Self::redraw_line(&mut stdout, prompt, &line, cursor_pos)?;
                }
                // Ctrl-E => End
                5 => {
                    cursor_pos = line.len();
                    Self::redraw_line(&mut stdout, prompt, &line, cursor_pos)?;
                }
                // Ctrl-U => clear line
                21 => {
                    line.clear();
                    cursor_pos = 0;
                    Self::redraw_line(&mut stdout, prompt, &line, cursor_pos)?;
                }
                // Ctrl-K => kill to end of line
                11 => {
                    line.truncate(cursor_pos);
                    Self::redraw_line(&mut stdout, prompt, &line, cursor_pos)?;
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
                        Self::redraw_line(&mut stdout, prompt, &line, cursor_pos)?;
                    }
                }
                // ESC => start of escape sequence
                27 => {
                    let seq1 = read_byte()?;
                    if seq1 == b'[' {
                        let seq2 = read_byte()?;
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
                                    Self::redraw_line(&mut stdout, prompt, &line, cursor_pos)?;
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
                                    Self::redraw_line(&mut stdout, prompt, &line, cursor_pos)?;
                                }
                            }
                            // Right arrow
                            b'C' => {
                                if cursor_pos < line.len() {
                                    cursor_pos += 1;
                                    write!(stdout, "\x1b[C")?;
                                    stdout.flush()?;
                                }
                            }
                            // Left arrow
                            b'D' => {
                                if cursor_pos > 0 {
                                    cursor_pos -= 1;
                                    write!(stdout, "\x1b[D")?;
                                    stdout.flush()?;
                                }
                            }
                            // Home
                            b'H' => {
                                cursor_pos = 0;
                                Self::redraw_line(&mut stdout, prompt, &line, cursor_pos)?;
                            }
                            // End
                            b'F' => {
                                cursor_pos = line.len();
                                Self::redraw_line(&mut stdout, prompt, &line, cursor_pos)?;
                            }
                            // Delete: ESC [ 3 ~
                            b'3' => {
                                let seq3 = read_byte()?;
                                if seq3 == b'~' && cursor_pos < line.len() {
                                    line.remove(cursor_pos);
                                    Self::redraw_line(&mut stdout, prompt, &line, cursor_pos)?;
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
                        write!(stdout, "{}", b as char)?;
                        stdout.flush()?;
                    } else {
                        Self::redraw_line(&mut stdout, prompt, &line, cursor_pos)?;
                    }
                }
                _ => {
                    // Ignore other control characters
                }
            }
        }
    }

    /// Redraw the current line (clear and rewrite).
    fn redraw_line(
        stdout: &mut io::Stdout,
        prompt: &str,
        line: &str,
        cursor_pos: usize,
    ) -> io::Result<()> {
        // Move to beginning of line, clear it, rewrite prompt + line
        write!(stdout, "\r\x1b[K{}{}", prompt, line)?;
        // Move cursor to correct position
        let total_len = prompt.len() + line.len();
        let target = prompt.len() + cursor_pos;
        if target < total_len {
            write!(stdout, "\x1b[{}D", total_len - target)?;
        }
        stdout.flush()
    }
}
