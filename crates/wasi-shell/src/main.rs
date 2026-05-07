use std::env;
use std::io;
use std::path::PathBuf;
use std::sync::Arc;
use colored::*;
use wasi_shell::{CommandRegistry, LineReader, handle_parallel};

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

    let arc_registry = Arc::new(registry);
    let mut reader = LineReader::new(1000);

    println!("{}", "Welcome to WASI-Shell!".green().bold());
    println!("Type 'help' for available commands or 'exit' to quit.");

    loop {
        let cwd = env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        let prompt = format!("{} $ ", cwd.display().to_string().cyan());

        let line = match reader.read_line(&prompt) {
            Ok(Some(line)) => line,
            Ok(None) => break,            // EOF (Ctrl-D)
            Err(e) => {
                eprintln!("{}", format!("Input error: {}", e).red());
                break;
            }
        };

        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        if trimmed == "exit" {
            println!("Goodbye!");
            break;
        }

        let results = handle_parallel(
            vec![trimmed.to_string()],
            Box::new(io::empty()),
            Box::new(io::stdout()),
            Arc::clone(&arc_registry),
        );

        for res in results {
            if let Err(e) = res {
                eprintln!("{}", e.red());
            }
        }
    }
}
