/// Diagnostic tool for smoke system testing
/// Runs frames and outputs particle state clearly
use sl::terminal::Terminal;
use sl::config::Config;
use sl::render::render_frame;
use sl::debug::debug_smoke_state;
use sl::smoke::clear_smoke;

fn main() -> std::io::Result<()> {
    let terminal = Terminal::new()?;
    let config = Config {
        accident: false,
        c51: false,
        logo: false,
        flying: false,
    };

    println!("=== SMOKE SYSTEM DIAGNOSTIC ===");
    println!("Terminal: {}x{}", terminal.width(), terminal.height());
    println!("Mode: D51 (default)");
    println!();

    // Test frames and capture clean output
    for frame_num in 0..24 {
        let x = 50 - frame_num;
        let pattern = frame_num % 6;
        let gate = x % 4 == 0;

        // Render frame (this updates particles and adds new ones)
        render_frame(&terminal, x as i32, pattern, &config)?;

        // Get clean particle info
        let smoke_info = debug_smoke_state();
        
        // Print summary
        println!("Frame {} | x={} | gate={} | {}", 
                 frame_num, x, if gate { "Y" } else { "N" }, smoke_info.lines().next().unwrap_or(""));
    }

    clear_smoke();
    println!("=== END DIAGNOSTIC ===");
    Ok(())
}
