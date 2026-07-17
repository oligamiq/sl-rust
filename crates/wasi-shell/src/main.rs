use colored::*;
use std::env;
use std::io;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use wasi_shell::{CommandRegistry, LineEditor, LoopAction, handle_parallel};

fn main() {
    let registry = CommandRegistry::with_builtins();
    let cancel_token = wasibox_core::CancellationToken::new();

    #[cfg(not(target_family = "wasm"))]
    {
        let ctrlc_token = cancel_token.clone();
        ctrlc::set_handler(move || {
            ctrlc_token.cancel();
        })
        .expect("Error setting Ctrl-C handler");
    }

    #[cfg(target_family = "wasm")]
    let mux = wasi_shell::StdinMultiplexer::new(cancel_token.clone());

    let args: Vec<String> = env::args().skip(1).collect();
    if !args.is_empty() {
        let arc_registry = Arc::new(registry);

        #[cfg(target_family = "wasm")]
        let stdin = Box::new(mux.subscribe());
        #[cfg(not(target_family = "wasm"))]
        let stdin = Box::new(io::stdin());

        let finished = Arc::new(AtomicBool::new(false));
        let finished_clone = Arc::clone(&finished);
        let timeout_token = cancel_token.clone();
        std::thread::spawn(move || {
            std::thread::sleep(std::time::Duration::from_secs(10));
            if !finished_clone.load(Ordering::SeqCst) {
                timeout_token.cancel();
            }
        });

        let results = handle_parallel(
            args,
            stdin,
            Box::new(io::stdout()),
            arc_registry,
            cancel_token,
        );
        finished.store(true, Ordering::SeqCst);
        // ... (rest of main)

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
    let mut reader = LineEditor::new(1000);

    println!("{}", "Welcome to WASI-Shell!".green().bold());
    println!("Type 'help' for available commands or 'exit' to quit.");

    let cancel_clone_repl = cancel_token.clone();
    let reg = Arc::clone(&arc_registry);

    #[cfg(target_family = "wasm")]
    let mux_repl = Arc::clone(&mux);

    let handler = move |line: &str| -> Result<LoopAction, String> {
        if line == "exit" {
            println!("Goodbye!");
            return Ok(LoopAction::Break);
        }
        cancel_clone_repl.reset();

        let finished = Arc::new(AtomicBool::new(false));
        let finished_clone = Arc::clone(&finished);
        let timeout_token = cancel_clone_repl.clone();
        std::thread::spawn(move || {
            std::thread::sleep(std::time::Duration::from_secs(10));
            if !finished_clone.load(Ordering::SeqCst) {
                timeout_token.cancel();
            }
        });

        #[cfg(target_family = "wasm")]
        let stdin = Box::new(mux_repl.subscribe());
        #[cfg(not(target_family = "wasm"))]
        let stdin = Box::new(io::empty());

        let results = handle_parallel(
            vec![line.to_string()],
            stdin,
            Box::new(io::stdout()),
            Arc::clone(&reg),
            cancel_clone_repl.clone(),
        );
        finished.store(true, Ordering::SeqCst);
        for res in results {
            if let Err(e) = res {
                eprintln!("{}", e.red());
            }
        }
        Ok(LoopAction::Continue)
    };

    let prompt_fn = || {
        let cwd = env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        format!("{} $ ", cwd.display().to_string().cyan())
    };

    #[cfg(target_family = "wasm")]
    let loop_res =
        reader.run_loop_with_stdin(prompt_fn, &handler, cancel_token, Box::new(mux.subscribe()));
    #[cfg(not(target_family = "wasm"))]
    let loop_res = reader.run_loop(prompt_fn, &handler, cancel_token);

    if let Err(e) = loop_res {
        eprintln!("{}", format!("Input error: {}", e).red());
    }
}
