use std::io::{self, Write};
use crossterm::{
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, Clear, ClearType, enable_raw_mode, disable_raw_mode},
    cursor::{Hide, Show},
    event::{poll, read, Event},
};
#[cfg(feature = "debug")]
use crossterm::event::KeyCode;
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

impl Terminal {
    pub fn new() -> io::Result<Self> {
        enable_raw_mode()?;
        let (w, h) = crossterm::terminal::size()?;
        let mut stdout = io::stdout();
        execute!(
            stdout,
            EnterAlternateScreen,
            Hide,
            Clear(ClearType::All)
        )?;
        Ok(Terminal {
            width: w,
            height: h,
        })
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
        let mut stdout = io::stdout();
        execute!(
            stdout,
            Show,
            LeaveAlternateScreen
        )?;
        disable_raw_mode()?;
        Ok(())
    }

    pub fn poll_event(&self, timeout: Duration) -> io::Result<bool> {
        poll(timeout)
    }

    pub fn read_event(&self) -> io::Result<Event> {
        read()
    }

    pub fn check_input(&self) -> io::Result<InputAction> {
        #[allow(unused_mut)]
        let mut action = InputAction::None;
        while self.poll_event(Duration::from_millis(0))? {
            if let Event::Key(key_event) = self.read_event()? {
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

impl Drop for Terminal {
    fn drop(&mut self) {
        let _ = self.cleanup();
    }
}
