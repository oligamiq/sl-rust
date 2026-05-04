# SL Pause/Resume Feature

## Keyboard Shortcuts

### Pause/Resume Animation
- **Space Bar** - Pause or resume the animation
- **P / p** - Pause or resume the animation

### Control Animation
- **Ctrl+C** - Quit the animation (works in paused or running state)
- **Esc** - Quit the animation (works in paused or running state)

## Usage

```bash
# Run with default D51 locomotive
./target/release/sl.exe

# During animation:
# - Press Space or P to pause
# - Press Space or P again to resume
# - Press Ctrl+C or Esc to quit
```

## Examples

```bash
# Run with different modes
./target/release/sl.exe -c    # C51 locomotive
./target/release/sl.exe -l    # Logo/SL text
./target/release/sl.exe -a    # Accident mode
./target/release/sl.exe -F    # Flying mode

# All modes support pause/resume
```

## Messages

When you pause the animation, you'll see:
```
⏸️  PAUSED - Press Space or P to resume, Ctrl+C to quit
```

When you resume:
```
▶️  RESUMED
```

## Implementation Details

- Pause state is handled in the main event loop
- When paused, the animation frame stays on screen while waiting for input
- Pattern animation stops during pause
- Smoke particles remain in their current state during pause
- All smoke system state is preserved when pausing
