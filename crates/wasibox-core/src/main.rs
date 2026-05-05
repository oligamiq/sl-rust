use std::env;
use std::process;

fn main() {
    let args = env::args_os();
    
    if let Err(e) = wasibox_core::execute(args) {
        eprintln!("{}", e);
        process::exit(1);
    }
}
