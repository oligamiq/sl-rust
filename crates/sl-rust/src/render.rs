use crate::terminal::Terminal;
use crate::config::Config;
use crate::train::ascii::*;
use crate::smoke::{add_smoke, update_smoke, get_smoke_particles, set_generation_gate};
use std::io;

pub fn render_frame(terminal: &Terminal, x: i32, pattern: usize, config: &Config) -> io::Result<()> {
    let frame = build_frame(terminal, x, pattern, config);
    terminal.render_frame(frame)?;
    Ok(())
}

fn build_frame(terminal: &Terminal, x: i32, pattern: usize, config: &Config) -> String {
    let mut frame = String::new();
    
    // Clear screen (ANSI command)
    frame.push_str("\x1B[2J\x1B[H");
    
    // Set generation gate: only generate smoke every 4 frames
    set_generation_gate(x % 4 == 0);
    
    // Update smoke particles (applies movement and increments pattern)
    update_smoke();
    
    // Draw smoke BEFORE train so train renders on top (prevents smoke from overwriting train)
    build_smoke(&mut frame, terminal);

    if config.logo {
        build_logo(&mut frame, terminal, x, pattern, config);
    } else if config.c51 {
        build_c51(&mut frame, terminal, x, pattern, config);
    } else {
        build_d51(&mut frame, terminal, x, pattern, config);
    }

    if config.accident {
        build_man(&mut frame, terminal, x, config);
    }

    frame
}

fn build_d51(frame: &mut String, terminal: &Terminal, x: i32, pattern: usize, config: &Config) {
    let train_height = 10i32;
    let y_center = (terminal.height() as i32 - train_height) / 2;
    let y_base = if config.flying {
        y_center - (x / 4)
    } else {
        y_center
    };

    let pattern = pattern % D51_PATTERNS;

    // Draw main body
    for (i, line) in D51_STR.iter().enumerate() {
        let y = y_base + i as i32;
        if y >= 0 && y < terminal.height() as i32 {
            add_line_to_frame(frame, x, y, line, terminal);
        }
    }

    // Add smoke from funnel (generation gated in set_generation_gate)
    let funnel_y = y_base - 1;
    add_smoke(x + D51_FUNNEL as i32, funnel_y);

    // Draw wheels
    for (i, line) in D51_WHL[pattern].iter().enumerate() {
        let y = y_base + (D51_STR.len() as i32) + i as i32;
        if y >= 0 && y < terminal.height() as i32 {
            add_line_to_frame(frame, x, y, line, terminal);
        }
    }

    // Draw coal
    for (i, line) in D51_COAL.iter().enumerate() {
        let y = y_base + i as i32;
        if y >= 0 && y < terminal.height() as i32 {
            add_line_to_frame(frame, x + 53, y, line, terminal);
        }
    }
}

fn build_c51(frame: &mut String, terminal: &Terminal, x: i32, pattern: usize, config: &Config) {
    let train_height = 10i32;
    let y_center = (terminal.height() as i32 - train_height) / 2;
    let y_base = if config.flying {
        y_center - (x / 4)
    } else {
        y_center
    };

    let pattern = pattern % C51_PATTERNS;

    // Draw main body
    for (i, line) in C51_STR.iter().enumerate() {
        let y = y_base + i as i32;
        if y >= 0 && y < terminal.height() as i32 {
            add_line_to_frame(frame, x, y, line, terminal);
        }
    }

    // Add smoke from funnel (generation gated in set_generation_gate)
    let funnel_y = y_base - 1;
    add_smoke(x + C51_FUNNEL as i32, funnel_y);

    // Draw wheels
    for (i, line) in C51_WHL[pattern].iter().enumerate() {
        let y = y_base + (C51_STR.len() as i32) + i as i32;
        if y >= 0 && y < terminal.height() as i32 {
            add_line_to_frame(frame, x, y, line, terminal);
        }
    }

    // Draw coal
    for (i, line) in C51_COAL.iter().enumerate() {
        let y = y_base + i as i32;
        if y >= 0 && y < terminal.height() as i32 {
            add_line_to_frame(frame, x + 55, y, line, terminal);
        }
    }
}

fn build_logo(frame: &mut String, terminal: &Terminal, x: i32, pattern: usize, config: &Config) {
    let train_height = 10i32;
    let y_center = (terminal.height() as i32 - train_height) / 2;
    let y_base = if config.flying {
        y_center - (x / 4)
    } else {
        y_center
    };

    let pattern = pattern % 6;

    // Draw SL logo
    for (i, line) in LOGO_STR.iter().enumerate() {
        let y = y_base + i as i32;
        if y >= 0 && y < terminal.height() as i32 {
            add_line_to_frame(frame, x, y, line, terminal);
        }
    }

    // Add smoke from funnel (generation gated in set_generation_gate)
    let funnel_y = y_base - 1;
    add_smoke(x + LOGO_FUNNEL as i32, funnel_y);

    // Draw wheels
    for (i, line) in LOGO_WHL[pattern].iter().enumerate() {
        let y = y_base + LOGO_STR.len() as i32 + i as i32;
        if y >= 0 && y < terminal.height() as i32 {
            add_line_to_frame(frame, x, y, line, terminal);
        }
    }

    // Draw coal
    for (i, line) in LOGO_COAL.iter().enumerate() {
        let y = y_base + LOGO_STR.len() as i32 + 2 + i as i32;
        if y >= 0 && y < terminal.height() as i32 {
            add_line_to_frame(frame, x, y, line, terminal);
        }
    }

    // Draw car
    for (i, line) in LOGO_CAR.iter().enumerate() {
        let y = y_base + LOGO_STR.len() as i32 + 2 + i as i32;
        if y >= 0 && y < terminal.height() as i32 {
            add_line_to_frame(frame, x, y, line, terminal);
        }
    }
}

fn build_man(frame: &mut String, terminal: &Terminal, x: i32, _config: &Config) {
    let y = terminal.height() as i32 - 3;
    for (i, line) in MAN.iter().enumerate() {
        let y = y + i as i32;
        if y >= 0 && y < terminal.height() as i32 {
            add_line_to_frame(frame, x + 15, y, line, terminal);
        }
    }
}

fn build_smoke(frame: &mut String, terminal: &Terminal) {
    let particles = get_smoke_particles();
    
    for particle in particles {
        // Particle coordinates have already been updated in build_frame() before this function
        // Only draw if within screen bounds
        if particle.x >= 0 && particle.x < terminal.width() as i32
            && particle.y >= 0 && particle.y < terminal.height() as i32 {
            
            let pattern_idx = particle.pattern.min(15) as usize;
            let kind = particle.kind % 2;
            
            if let Some(smoke_str) = SMOKE_PATTERN.get(kind) {
                if let Some(ch_str) = smoke_str.get(pattern_idx) {
                    // Draw all characters from the smoke string (e.g., "(   )")
                    let mut x_offset = 0;
                    for ch in ch_str.chars() {
                        let draw_x = (particle.x as i32 + x_offset) as u16;
                        if (particle.x as i32 + x_offset) >= 0 
                            && (particle.x as i32 + x_offset) < terminal.width() as i32
                            && ch != ' ' {
                            add_char_to_frame(frame, draw_x, particle.y as u16, ch);
                        }
                        x_offset += 1;
                    }
                }
            }
        }
    }
}

fn add_line_to_frame(frame: &mut String, start_x: i32, y: i32, line: &str, terminal: &Terminal) {
    if y < 0 || y >= terminal.height() as i32 {
        return;
    }

    for (i, ch) in line.chars().enumerate() {
        let x = start_x + i as i32;
        if x >= 0 && x < terminal.width() as i32 {
            add_char_to_frame(frame, x as u16, y as u16, ch);
        }
    }
}

fn add_char_to_frame(frame: &mut String, x: u16, y: u16, ch: char) {
    // ANSI escape sequence: CSI Py ; Px H (MoveTo)
    frame.push_str(&format!("\x1B[{};{}H{}", y + 1, x + 1, ch));
}
