// Capture what's actually being rendered to compare with expected
use sl::terminal::Terminal;
use sl::config::Config;
use sl::render::render_frame;
use std::fs::File;
use std::io::Write;

fn main() -> std::io::Result<()> {
    let config = Config {
        accident: false,
        c51: false,
        logo: false,
        flying: false,
    };
    
    let terminal = Terminal::new()?;
    
    // Render one frame
    render_frame(&terminal, 40, 0, &config)?;
    
    eprintln!("Frame rendered to terminal.");
    eprintln!("Train should be centered vertically with coal below.");
    eprintln!("Press any key to exit...");
    
    // Wait a moment to see the output
    std::thread::sleep(std::time::Duration::from_secs(2));
    
    eprintln!("\nIf you see corrupted characters like (@@), (||), or misplaced _@ patterns,");
    eprintln!("the rendering is still broken.");
    
    Ok(())
}
