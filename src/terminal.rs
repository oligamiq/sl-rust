use std::io::{self, Write};
use crossterm::{
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, Clear, ClearType, enable_raw_mode, disable_raw_mode},
    cursor::{Hide, Show},
    event::{poll, read, Event, KeyEvent, KeyCode},
};
use std::time::Duration;

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

    pub fn check_quit(&self) -> io::Result<bool> {
        if self.poll_event(Duration::from_millis(0))? {
            if let Event::Key(KeyEvent { code: KeyCode::Char('c'), .. }) = self.read_event()? {
                return Ok(true);
            }
        }
        Ok(false)
    }
}

impl Drop for Terminal {
    fn drop(&mut self) {
        let _ = self.cleanup();
    }
}
