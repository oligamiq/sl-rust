use clap::Parser;
use std::process;

fn main() {
    let args = tree_lib::Args::parse();
    
    if let Err(e) = tree_lib::execute(args) {
        eprintln!("tree error: {}", e);
        process::exit(1);
    }
}
