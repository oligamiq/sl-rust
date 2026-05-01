use sl::terminal::Terminal;
use sl::config::Config;
use sl::render::render_frame;
use std::thread;
use std::time::Duration;

const FRAME_TIME_MS: u64 = 40;

fn main() -> std::io::Result<()> {
    let config = Config::from_args();
    let terminal = Terminal::new()?;

    let width = terminal.width() as i32;
    let max_length = if config.logo { 40 } else if config.c51 { 95 } else { 83 };

    let mut pattern = 0usize;

    for x in (-(max_length + 10)..=(width + 10)).rev() {
        // Check for quit signal
        if terminal.check_quit()? {
            break;
        }

        render_frame(&terminal, x, pattern, &config)?;

        pattern = (pattern + 1) % 6;
        thread::sleep(Duration::from_millis(FRAME_TIME_MS));
    }

    Ok(())
}
