use std::env;
use std::io::{self, Write};
use std::path::PathBuf;
use colored::*;
use wasi_shell::handle_pipeline;

fn main() {
    let mut input = String::new();
    let stdin = io::stdin();
    
    println!("{}", "Welcome to WASI-Shell!".green().bold());
    println!("Type 'help' for available commands or 'exit' to quit.");

    loop {
        let cwd = env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        print!("{} $ ", cwd.display().to_string().cyan());
        io::stdout().flush().unwrap();

        input.clear();
        let n = stdin.read_line(&mut input).unwrap_or(0);
        if n == 0 || input.trim() == "exit" {
            if n != 0 { println!("Goodbye!"); }
            break;
        }

        let line = input.trim();
        if line.is_empty() {
            continue;
        }

        if let Err(e) = handle_pipeline(line, Box::new(io::stdin()), Box::new(io::stdout())) {
            eprintln!("{}", e.red());
        }
    }
}
