/// Simple smoke test without rendering
use sl::smoke::{add_smoke, update_smoke, get_smoke_particles, set_generation_gate, clear_smoke};

fn main() {
    println!("=== SIMPLE SMOKE TEST ===");
    println!();

    clear_smoke();
    
    // Simulate 20 frames
    for frame in 0..20 {
        let x = 50 - frame;
        let gate_open = x % 4 == 0;
        
        // Set generation gate
        set_generation_gate(gate_open);
        
        // Add smoke particles (only if gate is open)
        if gate_open {
            add_smoke(x as i32 + sl::train::ascii::D51_FUNNEL as i32, 40);
        }
        
        // Update all particles (move them)
        update_smoke();
        
        // Get current particle state
        let particles = get_smoke_particles();
        
        println!("Frame {:2} | x={} | gate={} | particles={} |", 
                 frame, x, if gate_open { "Y" } else { "N" }, particles.len());
        
        for (i, p) in particles.iter().enumerate() {
            if i < 3 {  // Show first 3 particles
                println!("    [{}: x={}, y={}, pat={}, kind={}]", i, p.x, p.y, p.pattern, p.kind);
            }
        }
        
        if particles.len() > 3 {
            println!("    ... and {} more", particles.len() - 3);
        }
    }
    
    clear_smoke();
    println!();
    println!("=== END TEST ===");
}
