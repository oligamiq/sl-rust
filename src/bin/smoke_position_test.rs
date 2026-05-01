// Frame capture and analysis
use sl::terminal::Terminal;
use sl::config::Config;
use sl::train::ascii::*;
use sl::smoke::{add_smoke, update_smoke, get_smoke_particles, set_generation_gate, clear_smoke};
use std::io::Write;

fn main() -> std::io::Result<()> {
    let config = Config {
        accident: false,
        c51: false,
        logo: false,
        flying: false,
    };
    
    let terminal = Terminal::new()?;
    let width = terminal.width() as i32;
    let height = terminal.height() as i32;
    
    eprintln!("Terminal: {}x{}", width, height);
    
    // Simulate what build_frame does
    let x = 40;
    let y_base = (height - 10) / 2;
    let funnel_y = y_base - 1;
    
    eprintln!("Train position: x={}, y_base={}", x, y_base);
    eprintln!("Funnel position: y={}", funnel_y);
    
    // Simulate 2 frames of smoke generation (gate opens every 4 frames, x%4==0)
    // Frame 0: x=40, gate should be open (40%4==0)
    eprintln!("\n--- Simulating Frame 0 (x=40, gate=true) ---");
    set_generation_gate(40 % 4 == 0);
    add_smoke(x + D51_FUNNEL as i32, funnel_y);
    update_smoke();
    let particles = get_smoke_particles();
    eprintln!("Particles after frame 0:");
    for (i, p) in particles.iter().enumerate() {
        eprintln!("  [{}] x={}, y={}, pattern={}, kind={}", i, p.x, p.y, p.pattern, p.kind);
    }
    
    // Frame 1: x=39, gate should be closed (39%4!=0)
    eprintln!("\n--- Simulating Frame 1 (x=39, gate=false) ---");
    set_generation_gate(39 % 4 == 0);
    add_smoke(x + D51_FUNNEL as i32, funnel_y);  // should NOT add due to gate
    update_smoke();
    let particles = get_smoke_particles();
    eprintln!("Particles after frame 1:");
    for (i, p) in particles.iter().enumerate() {
        eprintln!("  [{}] x={}, y={}, pattern={}, kind={}", i, p.x, p.y, p.pattern, p.kind);
    }
    
    // Frame 2-15: Continue updating
    eprintln!("\n--- Simulating Frames 2-15 ---");
    for frame in 2..=15 {
        let gate = ((40 - frame) as i32 % 4) == 0;
        set_generation_gate(gate);
        add_smoke(x + D51_FUNNEL as i32, funnel_y);
        update_smoke();
    }
    
    let particles = get_smoke_particles();
    eprintln!("Particles after frame 15:");
    for (i, p) in particles.iter().enumerate() {
        eprintln!("  [{}] x={}, y={}, pattern={}, kind={}", i, p.x, p.y, p.pattern, p.kind);
    }
    
    // Check if any particles are still at engine Y level
    let engine_y_range = y_base..(y_base + 10);
    let mut bad_particles = 0;
    for p in &particles {
        if engine_y_range.contains(&p.y) {
            eprintln!("WARNING: Particle at y={} (engine Y range: {}-{})", 
                     p.y, y_base, y_base + 9);
            bad_particles += 1;
        }
    }
    
    if bad_particles > 0 {
        eprintln!("\nERROR: {} particles are rendering on top of engine!", bad_particles);
    } else {
        eprintln!("\nOK: No particles at engine level");
    }
    
    clear_smoke();
    Ok(())
}
