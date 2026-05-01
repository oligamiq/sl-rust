/// Test suite for smoke system
/// Verifies pattern definitions and C version compatibility

#[cfg(test)]
mod smoke_tests {
    use sl::train::ascii::{SMOKE_PATTERN, SMOKE_DY, SMOKE_DX, SMOKE_ERASER};

    #[test]
    fn test_smoke_pattern_count() {
        // Must have 2 kinds (normal and accident) with 16 patterns each
        assert_eq!(SMOKE_PATTERN.len(), 2, "Must have 2 smoke kinds");
        
        for kind in 0..2 {
            assert_eq!(
                SMOKE_PATTERN[kind].len(),
                16,
                "Kind {} must have 16 patterns",
                kind
            );
        }
    }

    #[test]
    fn test_normal_smoke_patterns() {
        let kind = 0;
        let patterns = &SMOKE_PATTERN[kind];
        
        // Verify progression: expand -> contract -> dot -> disappear
        assert_eq!(patterns[0], "(   )", "Pattern 0: expanding");
        assert_eq!(patterns[1], "(    )", "Pattern 1: expanded");
        assert_eq!(patterns[3], "(   )", "Pattern 3: contracting");
        assert_eq!(patterns[8], "()", "Pattern 8: small parens");
        assert_eq!(patterns[10], "O", "Pattern 10: dot");
        assert_eq!(patterns[15], " ", "Pattern 15: disappear");
    }

    #[test]
    fn test_accident_smoke_patterns() {
        let kind = 1;
        let patterns = &SMOKE_PATTERN[kind];
        
        // Same structure but with @ instead of ()
        assert_eq!(patterns[0], "(@@@)", "Pattern 0: expanding");
        assert_eq!(patterns[8], "@@", "Pattern 8: small @");
        assert_eq!(patterns[10], "@", "Pattern 10: dot");
        assert_eq!(patterns[15], " ", "Pattern 15: disappear");
    }

    #[test]
    fn test_dy_array() {
        let expected_dy: [i32; 16] = [
            2, 1, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0
        ];
        
        for i in 0..16 {
            assert_eq!(
                SMOKE_DY[i], expected_dy[i],
                "DY[{}] mismatch: expected {}, got {}",
                i, expected_dy[i], SMOKE_DY[i]
            );
        }
    }

    #[test]
    fn test_dx_array() {
        let expected_dx: [i32; 16] = [
            -2, -1, 0, 1, 1, 1, 1, 1, 2, 2, 2, 2, 2, 3, 3, 3
        ];
        
        for i in 0..16 {
            assert_eq!(
                SMOKE_DX[i], expected_dx[i],
                "DX[{}] mismatch: expected {}, got {}",
                i, expected_dx[i], SMOKE_DX[i]
            );
        }
    }

    #[test]
    fn test_eraser_patterns() {
        assert_eq!(SMOKE_ERASER.len(), 16, "Must have 16 eraser patterns");
        
        // Verify key erasers
        assert_eq!(SMOKE_ERASER[0], "     ", "Eraser for pattern 0");
        assert_eq!(SMOKE_ERASER[8], "  ", "Eraser for pattern 8");
        assert_eq!(SMOKE_ERASER[10], " ", "Eraser for pattern 10");
        assert_eq!(SMOKE_ERASER[15], " ", "Eraser for pattern 15");
    }

    #[test]
    fn test_smoke_lifecycle() {
        // Verify the lifecycle matches C version
        // Patterns 0-4: expanding phase (dy decreases from 2 to 0)
        for i in 0..5 {
            assert!(
                SMOKE_DY[i] >= 0,
                "DY for expand phase (0-4) should be >= 0"
            );
        }

        // Patterns 5-9: contraction phase (no vertical movement)
        for i in 5..10 {
            assert_eq!(
                SMOKE_DY[i], 0,
                "DY for contract phase (5-9) should be 0"
            );
        }

        // Patterns 10-14: dot phase (no vertical movement)
        for i in 10..15 {
            assert_eq!(
                SMOKE_DY[i], 0,
                "DY for dot phase (10-14) should be 0"
            );
        }

        // Pattern 15: disappear
        assert_eq!(SMOKE_DY[15], 0, "Pattern 15 should have no movement");
    }

    #[test]
    fn test_dx_progression() {
        // Verify horizontal drift progressively increases
        // Should move left initially, then right
        assert!(SMOKE_DX[0] < 0, "Initial movement should be leftward");
        assert!(SMOKE_DX[14] > 0, "Later movement should be rightward");
    }
}
