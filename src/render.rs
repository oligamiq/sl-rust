use crate::terminal::Terminal;
use crate::config::Config;
use crate::train::ascii::*;
use crate::smoke::{add_smoke, update_smoke, get_smoke_particles};
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
    
    if config.logo {
        build_logo(&mut frame, terminal, x, pattern, config);
    } else if config.c51 {
        build_c51(&mut frame, terminal, x, pattern, config);
    } else {
        build_d51(&mut frame, terminal, x, pattern, config);
    }

    build_smoke(&mut frame, terminal);

    if config.accident {
        build_man(&mut frame, terminal, x, config);
    }

    frame
}

fn build_d51(frame: &mut String, terminal: &Terminal, x: i32, pattern: usize, config: &Config) {
    let y_base = if config.flying {
        ((terminal.height() as i32 - 10) / 2) - (x / 4)
    } else {
        terminal.height() as i32 - 10
    };

    let pattern = pattern % D51_PATTERNS;

    // Draw main body
    for (i, line) in D51_STR.iter().enumerate() {
        let y = y_base + i as i32;
        if y >= 0 && y < terminal.height() as i32 {
            add_line_to_frame(frame, x, y, line, terminal);
        }
        if i == D51_FUNNEL {
            add_smoke(x + 8, y - 1);
        }
    }

    // Draw wheels
    for (i, line) in D51_WHL[pattern].iter().enumerate() {
        let y = y_base + (D51_STR.len() as i32) + i as i32;
        if y >= 0 && y < terminal.height() as i32 {
            add_line_to_frame(frame, x, y, line, terminal);
        }
    }

    // Draw coal
    let coal_y_offset = D51_STR.len() as i32 + D51_WHL[pattern].len() as i32;
    for (i, line) in D51_COAL.iter().enumerate() {
        let y = y_base + coal_y_offset + i as i32;
        if y >= 0 && y < terminal.height() as i32 {
            add_line_to_frame(frame, x + D51_LENGTH as i32, y, line, terminal);
        }
    }
}

fn build_c51(frame: &mut String, terminal: &Terminal, x: i32, pattern: usize, config: &Config) {
    let y_base = if config.flying {
        ((terminal.height() as i32 - 10) / 2) - (x / 4)
    } else {
        terminal.height() as i32 - 10
    };

    let pattern = pattern % C51_PATTERNS;

    // Draw main body
    for (i, line) in C51_STR.iter().enumerate() {
        let y = y_base + i as i32;
        if y >= 0 && y < terminal.height() as i32 {
            add_line_to_frame(frame, x, y, line, terminal);
        }
        if i == C51_FUNNEL {
            add_smoke(x + 8, y - 1);
        }
    }

    // Draw wheels
    for (i, line) in C51_WHL[pattern].iter().enumerate() {
        let y = y_base + (C51_STR.len() as i32) + i as i32;
        if y >= 0 && y < terminal.height() as i32 {
            add_line_to_frame(frame, x, y, line, terminal);
        }
    }

    // Draw coal
    let coal_y_offset = C51_STR.len() as i32 + C51_WHL[pattern].len() as i32;
    for (i, line) in C51_COAL.iter().enumerate() {
        let y = y_base + coal_y_offset + i as i32;
        if y >= 0 && y < terminal.height() as i32 {
            add_line_to_frame(frame, x + C51_LENGTH as i32, y, line, terminal);
        }
    }
}

fn build_logo(frame: &mut String, terminal: &Terminal, x: i32, pattern: usize, config: &Config) {
    let y_base = if config.flying {
        ((terminal.height() as i32 - 10) / 2) - (x / 4)
    } else {
        terminal.height() as i32 - 10
    };

    let pattern = pattern % 6;

    // Draw SL logo
    for (i, line) in LOGO_STR.iter().enumerate() {
        let y = y_base + i as i32;
        if y >= 0 && y < terminal.height() as i32 {
            add_line_to_frame(frame, x, y, line, terminal);
        }
    }

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
            add_line_to_frame(frame, x + 20, y, line, terminal);
        }
    }

    // Draw car
    for (i, line) in LOGO_CAR.iter().enumerate() {
        let y = y_base + LOGO_STR.len() as i32 + 2 + i as i32;
        if y >= 0 && y < terminal.height() as i32 {
            add_line_to_frame(frame, x + 40, y, line, terminal);
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
    update_smoke();
    for particle in get_smoke_particles() {
        if particle.x >= 0 && particle.x < terminal.width() as i32
            && particle.y >= 0 && particle.y < terminal.height() as i32 {
            let pattern = particle.pattern.min(4);
            let kind = particle.kind % 5;
            let line = SMOKE_PATTERN[pattern];
            if kind < line.len() {
                if let Some(ch) = line.chars().nth(kind) {
                    if ch != ' ' {
                        add_char_to_frame(frame, particle.x as u16, particle.y as u16, ch);
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
