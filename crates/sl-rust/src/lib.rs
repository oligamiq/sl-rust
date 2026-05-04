pub mod terminal;
pub mod train;
pub mod smoke;
pub mod config;
pub mod render;
pub mod debug;

use crate::terminal::{Terminal, InputAction};
use crate::config::Config;
use crate::render::render_frame;
use std::thread;
use std::time::Duration;
use std::io;

const FRAME_TIME_MS: u64 = 40;

/// Runs the SL animation with the given arguments.
pub fn run<I, T>(args: I) -> io::Result<()> 
where
    I: IntoIterator<Item = T>,
    T: AsRef<str>,
{
    let config = Config::from_args(args);
    let terminal = Terminal::new()?;

    let width = terminal.width() as i32;
    let max_length = if config.logo { 40 } else if config.c51 { 95 } else { 83 };

    let mut pattern = 0usize;
    let mut paused = false;

    for x in (-(max_length + 10)..=(width + 10)).rev() {
        // Unified input handler
        match terminal.check_input()? {
            InputAction::Quit => break,
            InputAction::Pause => {
                paused = !paused;
                if paused {
                    eprintln!("⏸️  PAUSED - Press Space or P to resume, Ctrl+C to quit");
                }
            }
            InputAction::None => {}
        }

        // If paused, wait for resume signal without advancing animation
        while paused {
            thread::sleep(Duration::from_millis(100));
            match terminal.check_input()? {
                InputAction::Pause => {
                    paused = false;
                    eprintln!("▶️  RESUMED");
                }
                InputAction::Quit => return Ok(()),
                InputAction::None => {}
            }
        }

        render_frame(&terminal, x, pattern, &config)?;

        pattern = (pattern + 1) % 6;
        thread::sleep(Duration::from_millis(FRAME_TIME_MS));
    }

    Ok(())
}
