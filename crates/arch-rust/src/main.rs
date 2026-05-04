use std::env;
use std::process;

fn main() {
    // Collect arguments including the program name.
    let args = env::args_os();
    
    match arch::execute(args) {
        Ok(output) => {
            println!("{}", output);
        }
        Err(err) => {
            // Check if it's a help/version message (usually starts with 'Print machine architecture.' or 'arch')
            // or a real error.
            // Clap's Error::to_string() includes the message and exits with 0 for help/version if called via print.
            // But since we returned it as Err, we just print it.
            
            // If it contains "Usage:", it's likely a help message or a parsing error.
            // GNU arch prints help to stdout on --help.
            if err.contains("Usage:") || err.contains("arch ") {
                print!("{}", err);
                process::exit(0);
            } else {
                eprintln!("{}", err);
                process::exit(1);
            }
        }
    }
}
