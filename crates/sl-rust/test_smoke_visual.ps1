# Visual test of smoke system
# Run sl command in a terminal with specific size and capture output

Write-Host "Building release binary..."
cd F:\sl-rust
cargo build --release --quiet

Write-Host "Running SL animation for 10 frames..."
Write-Host ""

# Create a test mode that shows the animation
# We'll capture the output from smoke_debug which shows particles

$output = & cargo run --release --bin smoke_debug 2>&1

# Parse output and show particle information
$lines = $output -split "`n"
$frame_num = -1
$in_smoke_state = $false

foreach ($line in $lines) {
    if ($line -match "Frame (\d+)") {
        $frame_num = [int]$matches[1]
        Write-Host ""
        Write-Host "═════════════════════════════════════════"
        Write-Host "Frame $frame_num"
        Write-Host "═════════════════════════════════════════"
    }
    elseif ($line -match "Total particles: (\d+)") {
        $count = [int]$matches[1]
        Write-Host "Total particles: $count"
        $in_smoke_state = $true
    }
    elseif ($line -match "x=\s*(\d+)\s*y=\s*(\d+)\s*pattern=\s*(\d+)\s*kind=(\d+)") {
        $x = [int]$matches[1]
        $y = [int]$matches[2]
        $pat = [int]$matches[3]
        $kind = [int]$matches[4]
        $kind_str = if ($kind -eq 0) { "normal" } else { "accident" }
        Write-Host "  Particle: x=$x, y=$y, pattern=$pat ($kind_str)"
    }
    elseif ($line -match "^={2,}$") {
        $in_smoke_state = $false
    }
}
