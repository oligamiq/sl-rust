# SL Rust Implementation

SL locomotive animation in Rust using crossterm.

## Build

```bash
cargo build --release
```

Binary: `target/release/sl.exe` (215 KB)

## Usage

```bash
sl.exe          # D51 locomotive (default)
sl.exe -c       # C51 locomotive
sl.exe -l       # Logo/SL
sl.exe -a       # Accident mode
sl.exe -F       # Flying mode
sl.exe -ac      # C51 in accident mode
sl.exe -lF      # Logo flying
```

## Architecture

### Key Design Principles

1. **Full-screen redraw**: Clears and redraws entire screen each frame (vs. delta/dirty region)
   - Eliminates flickering (no partial updates)
   - Matches original C implementation behavior
   - Simpler mental model

2. **crossterm-only**: No extra dependencies
   - Uses: `MoveTo`, `Clear`, `write!`, `execute!`
   - Avoids complex state management

3. **Simple rendering flow**:
   ```
   for each frame:
     1. clear_screen()
     2. draw_train(x, pattern)
     3. draw_smoke()
     4. draw_man() if accident
     5. flush()
     6. sleep(40ms)
   ```

### Module Structure

```
terminal.rs
  ├─ Terminal struct (wrapper around crossterm)
  ├─ move_to()
  ├─ clear()
  ├─ write_str()
  └─ cleanup()

config.rs
  └─ Config struct (parse CLI flags)

train/
  ├─ ascii.rs (ASCII art constants from original sl.h)
  ├─ d51.rs, c51.rs, logo.rs (stubs)

smoke.rs
  ├─ Particle struct
  ├─ thread_local SMOKE queue
  ├─ add_smoke()
  ├─ update_smoke()
  └─ get_smoke_particles()

render.rs
  ├─ render_frame()
  ├─ render_d51/c51/logo/man()
  ├─ render_smoke()
  └─ draw_line()

main.rs
  └─ parse args → loop → render → sleep
```

## Implementation Details

### Rendering Strategy

**Character-by-character placement**:
```rust
for y in train_lines {
  terminal.move_to(x, y)?;
  terminal.write_str(line)?;
}
```

This avoids:
- Buffer management complexity
- Synchronization issues between buffer and output
- Character loss at boundaries

### Smoke System

- `thread_local!` particle queue (state persists across frames)
- Particles created at locomotive funnel position
- Pattern cycles (0-4) and disappear after 5 frames
- 5 smoke styles determined by creation time counter

### Flying Mode

Y position varies by x:
```rust
y = (height/2) - (x/4)  // parabolic arc
```

## Performance

- **Frame time**: 40ms (25 FPS, matching original)
- **CPU**: Minimal (one-shot I/O per frame)
- **Memory**: ~2-3 KB (small particle queue)

## Differences from C Version

| Aspect | C | Rust |
|--------|---|------|
| Terminal library | ncurses | crossterm |
| Dependencies | None | crossterm 0.27 |
| Code lines | ~300 | ~650 (incl. ASCII data) |
| Smoke | Global static array | thread_local queue |
| Pattern timing | Hardcoded loop | Atomic counter |

## Testing

All combinations verified:
- ✓ Default (D51)
- ✓ C51 (-c)
- ✓ Logo (-l)
- ✓ Accident (-a)
- ✓ Flying (-F)
- ✓ Multi-flag combinations

No flickering, no shape corruption, no cut-off characters.
