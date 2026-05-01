use sl::terminal::Terminal;
use sl::config::Config;
use std::io::Write;

fn main() -> std::io::Result<()> {
    // Test D51 train rendering
    let config = Config {
        accident: false,
        c51: false,
        logo: false,
        flying: false,
    };
    
    let terminal = Terminal::new()?;
    let width = terminal.width();
    let height = terminal.height();
    
    eprintln!("Terminal size: {}x{}", width, height);
    eprintln!("Testing D51 train rendering...\n");
    
    // Manually build frame without using render_frame to have full control
    let y_base = (height as i32 - 10) / 2;
    let x = 20;
    let pattern = 0;
    
    // Expected output for verification
    let expected_train = [
        "      ====        ________                ___________ ",
        "  _D _|  |_______/        \\__I_I_____===__|_________| ",
        "   |(_)---  |   H\\________/ |   |        =|___ ___|   ",
        "   /     |  |   H  |  |     |   |         ||_| |_||   ",
        "  |      |  |   H  |__--------------------| [___] |   ",
        "  | ________|___H__/__|_____/[][]~\\_______|       |   ",
        "  |/ |   |-----------I_____I [][] []  D   |=======|__ ",
    ];
    
    eprintln!("Expected train body:");
    for (i, line) in expected_train.iter().enumerate() {
        eprintln!("{}: {}", i, line);
    }
    
    eprintln!("\nExpected wheels (pattern 0):");
    let expected_wheels = [
        "__/ =| o |=-~~\\  /~~\\  /~~\\  /~~\\ ____Y___________|__ ",
        " |/-=|___|=    ||    ||    ||    |_____/~\\___/        ",
        "  \\_/      \\O=====O=====O=====O_/      \\_/            ",
    ];
    for (i, line) in expected_wheels.iter().enumerate() {
        eprintln!("{}: {}", i, line);
    }
    
    eprintln!("\nExpected coal:");
    let expected_coal = [
        "                              ",
        "    _@_@_@_@_@_@_@_@_@_@_@_@_  ",
        "   (_)-(_)-(_)-(_)-(_)-(_)-(_) ",
        "    _@_@_@_@_@_@_@_@_@_@_@_@_  ",
        "   (_)-(_)-(_)-(_)-(_)-(_)-(_) ",
        "    _@_@_@_@_@_@_@_@_@_@_@_@_  ",
        "   (_)-(_)-(_)-(_)-(_)-(_)-(_) ",
        "    _@_@_@_@_@_@_@_@_@_@_@_@_  ",
        "   (_)-(_)-(_)-(_)-(_)-(_)-(_) ",
        "    _@_@_@_@_@_@_@_@_@_@_@_@_  ",
        "   (_)-(_)-(_)-(_)-(_)-(_)-(_) ",
    ];
    for (i, line) in expected_coal.iter().enumerate() {
        eprintln!("{}: {}", i, line);
    }
    
    eprintln!("\n\nTrack positions:");
    eprintln!("x base: {}", x);
    eprintln!("y base: {}", y_base);
    eprintln!("Train height: 10 lines (body 7 + wheels 3)");
    eprintln!("Coal starts at y: {}", y_base + 10);
    eprintln!("Coal length: 11 lines");
    
    eprintln!("\nExpected complete visual layout:");
    eprintln!("Train body lines: y {} to {}", y_base, y_base + 6);
    eprintln!("Wheels lines:     y {} to {}", y_base + 7, y_base + 9);
    eprintln!("Coal lines:       y {} to {}", y_base + 10, y_base + 20);
    
    Ok(())
}
