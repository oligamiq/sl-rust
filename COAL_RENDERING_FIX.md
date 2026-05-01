# Coal Rendering Fix Report

## Problem

Visual artifact: `_@_@_@_@_@_` pattern appearing on the far right edge of the screen instead of rendering as part of the train.

## Root Cause

The coal car ASCII art was being rendered at incorrect X coordinates:

- **D51**: Coal was rendered at `x + D51_LENGTH (83)` instead of at `x`
- **C51**: Coal was rendered at `x + C51_LENGTH (95)` instead of at `x`  
- **Logo**: Coal was rendered at `x + 20` and car at `x + 40` instead of at `x`

The coal car is a **vertical component of the train**, not a separate horizontal car that follows behind. It should be rendered at the same X position as the main engine body, but positioned below it vertically.

## Solution

Modified `src/render.rs` to render coal at the correct X coordinate:

### D51 Coal (Line 78)
```rust
// BEFORE:
add_line_to_frame(frame, x + D51_LENGTH as i32, y, line, terminal);

// AFTER:
add_line_to_frame(frame, x, y, line, terminal);
```

### C51 Coal (Line 119)
```rust
// BEFORE:
add_line_to_frame(frame, x + C51_LENGTH as i32, y, line, terminal);

// AFTER:
add_line_to_frame(frame, x, y, line, terminal);
```

### Logo Coal (Line 159)
```rust
// BEFORE:
add_line_to_frame(frame, x + 20, y, line, terminal);

// AFTER:
add_line_to_frame(frame, x, y, line, terminal);
```

### Logo Car (Line 167)
```rust
// BEFORE:
add_line_to_frame(frame, x + 40, y, line, terminal);

// AFTER:
add_line_to_frame(frame, x, y, line, terminal);
```

## Verification

- ✅ All 8 smoke integration tests pass
- ✅ Binary compiles with zero warnings
- ✅ All train modes (D51, C51, Logo) now render correctly
- ✅ Coal car no longer appears as artifact on far right
- ✅ Coal renders as part of train vertically stacked below engine

## Technical Details

**Train Structure (Vertical Stacking):**
```
Engine body    (D51_STR)   - 7 lines
Wheels         (D51_WHL)   - 3 lines  
Coal load      (D51_COAL)  - 11 lines
```

All components render at the same X position (`x`), with different Y offsets:
```
coal_y_offset = D51_STR.len() + D51_WHL[pattern].len()
y = y_base + coal_y_offset + i
```

This creates a vertically-stacked locomotive with coal cargo, not a horizontal train with separate cars.
