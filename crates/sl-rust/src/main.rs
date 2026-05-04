use std::env;
use std::io;

fn main() -> io::Result<()> {
    // Skip the first argument (program name)
    let args: Vec<String> = env::args().skip(1).collect();
    sl::run(args)
}
