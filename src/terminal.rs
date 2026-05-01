use std::io::{self, Write};
use std::time::Duration;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputAction {
    None,
    Pause,
    Quit,
}

pub struct Terminal {
    width: u16,
    height: u16,
}

#[cfg(not(target_os = "wasi"))]
mod sys {
    use super::*;
    use crossterm::{
        execute,
        terminal::{EnterAlternateScreen, LeaveAlternateScreen, Clear, ClearType, enable_raw_mode, disable_raw_mode},
        cursor::{Hide, Show},
        event::{poll, read, Event},
    };
    #[cfg(feature = "debug")]
    use crossterm::event::KeyCode;

    pub fn init() -> io::Result<(u16, u16)> {
        enable_raw_mode()?;
        let (w, h) = crossterm::terminal::size()?;
        let mut stdout = io::stdout();
        execute!(
            stdout,
            EnterAlternateScreen,
            Hide,
            Clear(ClearType::All)
        )?;
        Ok((w, h))
    }

    pub fn cleanup() -> io::Result<()> {
        let mut stdout = io::stdout();
        execute!(
            stdout,
            Show,
            LeaveAlternateScreen
        )?;
        disable_raw_mode()?;
        Ok(())
    }

    pub fn check_input() -> io::Result<InputAction> {
        #[allow(unused_mut)]
        let mut action = InputAction::None;
        while poll(Duration::from_millis(0))? {
            if let Event::Key(key_event) = read()? {
                if key_event.kind == crossterm::event::KeyEventKind::Press {
                    #[cfg(feature = "debug")]
                    match key_event.code {
                        KeyCode::Char(' ') | KeyCode::Char('p') | KeyCode::Char('P') => {
                            action = InputAction::Pause;
                        }
                        KeyCode::Char('q') | KeyCode::Char('Q') | KeyCode::Esc => {
                            action = InputAction::Quit;
                        }
                        KeyCode::Char('c') if key_event.modifiers.contains(crossterm::event::KeyModifiers::CONTROL) => {
                            action = InputAction::Quit;
                        }
                        _ => {}
                    }
                }
            }
        }
        Ok(action)
    }
}

#[cfg(target_os = "wasi")]
mod sys {
    use super::*;

    pub fn init() -> io::Result<(u16, u16)> {
        let w = std::env::var("COLUMNS").ok().and_then(|v| v.parse().ok()).unwrap_or(80);
        let h = std::env::var("LINES").ok().and_then(|v| v.parse().ok()).unwrap_or(24);
        
        let mut stdout = io::stdout();
        write!(stdout, "\x1B[?1049h\x1B[?25l\x1B[2J")?;
        stdout.flush()?;
        Ok((w, h))
    }

    pub fn cleanup() -> io::Result<()> {
        let mut stdout = io::stdout();
        write!(stdout, "\x1B[?25h\x1B[?1049l")?;
        stdout.flush()?;
        Ok(())
    }

    pub fn check_input() -> io::Result<InputAction> {
        Ok(InputAction::None)
    }
}

impl Terminal {
    pub fn new() -> io::Result<Self> {
        let (width, height) = sys::init()?;
        Ok(Terminal { width, height })
    }

    pub fn width(&self) -> u16 {
        self.width
    }

    pub fn height(&self) -> u16 {
        self.height
    }

    /// Render frame atomically - all commands in one execute!()
    pub fn render_frame(&self, frame: String) -> io::Result<()> {
        let mut stdout = io::stdout();
        write!(stdout, "{}", frame)?;
        stdout.flush()
    }

    pub fn cleanup(&self) -> io::Result<()> {
        sys::cleanup()
    }

    pub fn check_input(&self) -> io::Result<InputAction> {
        sys::check_input()
    }
}

impl Drop for Terminal {
    fn drop(&mut self) {
        let _ = self.cleanup();
    }
}
