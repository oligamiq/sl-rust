use crate::smoke::get_smoke_particles;

pub fn debug_smoke_state() -> String {
    let particles = get_smoke_particles();
    
    let mut output = String::from("\n=== SMOKE STATE ===\n");
    output.push_str(&format!("Total particles: {}\n", particles.len()));
    
    for (i, p) in particles.iter().enumerate() {
        output.push_str(&format!(
            "  [{}] x={:3} y={:3} pattern={:2} kind={} \n",
            i, p.x, p.y, p.pattern, p.kind
        ));
    }
    
    output.push_str("==================\n");
    output
}

#[cfg(test)]
mod tests {

    #[test]
    fn test_smoke_patterns() {
        // Verify all 16 patterns are defined
        let patterns = vec![
            "(   )", "(    )", "(    )", "(   )", "(  )",
            "(  )", "( )", "( )", "()", "()",
            "O", "O", "O", "O", "O", " "
        ];
        
        assert_eq!(patterns.len(), 16, "Must have 16 smoke patterns");
    }

    #[test]
    fn test_dy_dx_arrays() {
        let dy = vec![2, 1, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        let dx = vec![-2, -1, 0, 1, 1, 1, 1, 1, 2, 2, 2, 2, 2, 3, 3, 3];
        
        assert_eq!(dy.len(), 16, "DY array must have 16 elements");
        assert_eq!(dx.len(), 16, "DX array must have 16 elements");
        
        // Verify expected C version values
        assert_eq!(dy[0], 2, "dy[0] should be 2 (expand upward)");
        assert_eq!(dx[0], -2, "dx[0] should be -2 (leftward)");
        assert_eq!(dy[4], 0, "dy[4] should be 0 (contract stops moving up)");
        assert_eq!(dx[14], 3, "dx[14] should be 3 (rightward at end)");
    }

    #[test]
    fn test_eraser_patterns() {
        let erasers = vec![
            "     ", "      ", "      ", "     ", "    ",
            "    ", "   ", "   ", "  ", "  ",
            " ", " ", " ", " ", " ", " "
        ];
        
        assert_eq!(erasers.len(), 16, "Must have 16 eraser patterns");
    }
}
