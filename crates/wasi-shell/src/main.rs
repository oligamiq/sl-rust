use std::env;
use std::io::{self, Write};
use std::path::PathBuf;
use std::sync::Arc;
use colored::*;
use wasi_shell::{CommandRegistry, handle_command_line, handle_parallel};

fn main() {
    let registry = CommandRegistry::with_builtins();

    let args: Vec<String> = env::args().skip(1).collect();
    if !args.is_empty() {
        let arc_registry = Arc::new(registry);
        let results = handle_parallel(
            args,
            Box::new(io::stdin()),
            Box::new(io::stdout()),
            arc_registry,
        );

        let mut has_error = false;
        for res in results {
            if let Err(e) = res {
                eprintln!("{}", e.red());
                has_error = true;
            }
        }
        if has_error {
            std::process::exit(1);
        }
        return;
    }

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

        if let Err(e) = handle_command_line(line, &registry) {
            eprintln!("{}", e.red());
        }
    }
}
