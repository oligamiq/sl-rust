use std::env;
use std::io;
use std::path::PathBuf;
use std::sync::Arc;
use colored::*;
use wasi_shell::{CommandRegistry, LineReader, LoopAction, handle_parallel};

fn main() {
    let registry = CommandRegistry::with_builtins();

    let args: Vec<String> = env::args().skip(1).collect();
    if !args.is_empty() {
        let arc_registry = Arc::new(registry);
        let cancel_token = wasibox_core::CancellationToken::new();
        let results = handle_parallel(
            args,
            Box::new(io::stdin()),
            Box::new(io::stdout()),
            arc_registry,
            cancel_token,
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

    let cancel_token = wasibox_core::CancellationToken::new();
    let cancel_clone_repl = cancel_token.clone();
    let reg = Arc::clone(&arc_registry);

    let handler = move |line: &str| -> Result<LoopAction, String> {
        if line == "exit" {
            println!("Goodbye!");
            return Ok(LoopAction::Break);
        }
        cancel_clone_repl.reset();
        let results = handle_parallel(
            vec![line.to_string()],
            Box::new(io::empty()),
            Box::new(io::stdout()),
            Arc::clone(&reg),
            cancel_clone_repl.clone(),
        );
        for res in results {
            if let Err(e) = res {
                eprintln!("{}", e.red());
            }
        }
        Ok(LoopAction::Continue)
    };

    if let Err(e) = reader.run_loop(
        || {
            let cwd = env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
            format!("{} $ ", cwd.display().to_string().cyan())
        },
        &handler,
        cancel_token,
    ) {
        eprintln!("{}", format!("Input error: {}", e).red());
    }
}
