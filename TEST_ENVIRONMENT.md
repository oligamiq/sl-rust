# Smoke System Test Environment

This document describes the test and diagnostic tools available for debugging the smoke system.

## Test Components

### 1. Integration Tests (`tests/smoke_test.rs`)

**Purpose**: Verify smoke pattern definitions and C version compatibility

**Run**:
```bash
cargo test --test smoke_test
cargo test --test smoke_test -- --nocapture  # with verbose output
```

**Tests**:
- `test_smoke_pattern_count` - Verify 2 kinds × 16 patterns
- `test_normal_smoke_patterns` - Check normal smoke progression
- `test_accident_smoke_patterns` - Check accident mode patterns
- `test_dy_array` - Verify vertical movement array
- `test_dx_array` - Verify horizontal movement array
- `test_eraser_patterns` - Verify eraser strings
- `test_smoke_lifecycle` - Verify movement phases
- `test_dx_progression` - Verify drift direction changes

All tests should PASS ✓

### 2. Unit Tests (`src/debug.rs`)

**Purpose**: Verify individual smoke component definitions

**Run**:
```bash
cargo test --lib
```

### 3. Diagnostic Binary (`src/bin/smoke_debug.rs`)

**Purpose**: Run a single frame with debug output

**Run**:
```bash
cargo run --release --bin smoke_debug
```

**Output**:
- Terminal dimensions
- Frame-by-frame particle state
- Total particles, position, pattern, kind for each

**Example output**:
```
=== SMOKE SYSTEM DIAGNOSTIC ===
Terminal: 80x24
Mode: D51 (default)

Frame 0: x=50
=== SMOKE STATE ===
Total particles: 1
  [0] x= 50 y= 12 pattern= 0 kind=0 
==================
```

### 4. Debug Module (`src/debug.rs`)

**Purpose**: Programmatic access to smoke state

**Function**:
```rust
pub fn debug_smoke_state() -> String
```

**Usage**:
```rust
use sl::debug::debug_smoke_state;

// In your code:
println!("{}", debug_smoke_state());
```

## Expected Behavior

### Particle Lifecycle

1. **Generation** (pattern 0)
   - Only every 4 frames (x % 4 == 0)
   - Position: (funnel_x, funnel_y)
   - Pattern: 0, Kind: counter % 2

2. **Expansion** (patterns 0-3)
   - Shape: "(   )" → "(    )" → "(    )" → "(   )"
   - Movement: up 2 → 1 → 1 → 1
   - Horizontal: left 2 → 1 → 0 → 1

3. **Contraction** (patterns 4-9)
   - Shape: "(  )" → "( )" → "()"
   - Movement: stationary horizontally drifting
   - Horizontal: progressively rightward

4. **Dot Phase** (patterns 10-14)
   - Shape: "O" → "O" → "O" → "O" → "O"
   - Movement: stationary with rightward drift
   - Horizontal: continuing rightward

5. **Disappear** (pattern 15)
   - Shape: " "
   - Deleted from particle list (pattern < 16 check)

### Movement Arrays

**DY** (vertical):
- Patterns 0-4: 2, 1, 1, 1, 0 (upward, then stops)
- Patterns 5-15: 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0 (no vertical)

**DX** (horizontal):
- Pattern 0: -2 (left)
- Patterns 1-3: -1, 0, 1 (leftward → rightward)
- Patterns 4-14: 1, 1, 1, 2, 2, 2, 2, 2, 3, 3, 3 (increasing rightward)

## Debugging Checklist

If smoke behavior is wrong, check:

- [ ] Generation gate: `x % 4 == 0` working?
- [ ] Pattern → character mapping correct?
- [ ] Movement arrays (dy/dx) applied correctly?
- [ ] Screen boundary handling?
- [ ] Kind value (0=normal, 1=accident) correct?
- [ ] Particle cleanup (pattern < 16)?
- [ ] Eraser patterns match smoke sizes?

## Files

- `src/debug.rs` - Debug utilities and unit tests
- `src/bin/smoke_debug.rs` - Diagnostic binary
- `tests/smoke_test.rs` - Integration tests
- This file - Test documentation
