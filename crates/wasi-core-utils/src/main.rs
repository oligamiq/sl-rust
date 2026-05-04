use std::env;
use std::process;

fn main() {
    let args = env::args_os();
    
    if let Err(e) = wasi_core_utils::execute(args) {
        eprintln!("{}", e);
        process::exit(1);
    }
}
