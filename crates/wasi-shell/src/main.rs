use std::env;
use std::fs::File;
use std::io::{self, Cursor, Read, Write};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use colored::*;
use wasi_core_utils::IoContext;

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

        if let Err(e) = handle_pipeline(line) {
            eprintln!("{}", e.red());
        }
    }
}

fn handle_pipeline(line: &str) -> Result<(), String> {
    let stages: Vec<&str> = line.split('|').collect();
    
    if stages.len() == 1 {
        let mut tokens = shlex::split(stages[0].trim())
            .ok_or_else(|| "Error: Invalid input (quoting error)".to_string())?;
        
        let mut stdout: Box<dyn Write + Send> = Box::new(io::stdout());
        if let Some(pos) = tokens.iter().position(|t| t == ">" || t == ">>") {
            let append = tokens[pos] == ">>";
            let filename = tokens.get(pos + 1).ok_or("Error: Missing file for redirection")?;
            let file = if append {
                std::fs::OpenOptions::new().create(true).append(true).open(filename)
            } else {
                File::create(filename)
            }.map_err(|e| format!("Error opening file: {}", e))?;
            stdout = Box::new(file);
            tokens.truncate(pos);
        }

        let mut ctx = IoContext::new(
            Box::new(io::stdin()),
            stdout,
            Box::new(io::stderr()),
        );
        return execute_command(tokens, &mut ctx);
    }

    let mut last_output = Vec::new();

    for (i, stage) in stages.iter().enumerate() {
        let is_first = i == 0;
        let is_last = i == stages.len() - 1;
        
        let mut tokens = shlex::split(stage.trim())
            .ok_or_else(|| "Error: Invalid input (quoting error)".to_string())?;

        let stdin: Box<dyn Read + Send> = if is_first {
            Box::new(io::stdin())
        } else {
            Box::new(Cursor::new(last_output.clone()))
        };

        let pipe_buf = Arc::new(Mutex::new(Vec::new()));
        let stdout: Box<dyn Write + Send> = if is_last {
            if let Some(pos) = tokens.iter().position(|t| t == ">" || t == ">>") {
                let append = tokens[pos] == ">>";
                let filename = tokens.get(pos + 1).ok_or("Error: Missing file for redirection")?;
                let file = if append {
                    std::fs::OpenOptions::new().create(true).append(true).open(filename)
                } else {
                    File::create(filename)
                }.map_err(|e| format!("Error opening file: {}", e))?;
                tokens.truncate(pos);
                Box::new(file)
            } else {
                Box::new(io::stdout())
            }
        } else {
            Box::new(ArcVecWriter { inner: Arc::clone(&pipe_buf) })
        };

        let mut ctx = IoContext::new(
            stdin,
            stdout,
            Box::new(io::stderr()),
        );

        execute_command(tokens, &mut ctx)?;
        
        if !is_last {
            // Retrieve the data from the Arc
            let buf = pipe_buf.lock().map_err(|_| "Internal error: pipe lock poisoned")?;
            last_output = buf.clone();
        }
    }

    Ok(())
}

struct ArcVecWriter {
    inner: Arc<Mutex<Vec<u8>>>,
}
impl Write for ArcVecWriter {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let mut inner = self.inner.lock().map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))?;
        inner.write(buf)
    }
    fn flush(&mut self) -> io::Result<()> {
        let mut inner = self.inner.lock().map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))?;
        inner.flush()
    }
}

fn execute_command(args: Vec<String>, ctx: &mut IoContext) -> Result<(), String> {
    if args.is_empty() {
        return Ok(());
    }

    let cmd = &args[0];

    match cmd.as_str() {
        "help" => {
            writeln!(ctx.stdout, "{}", "Available Commands:".yellow().bold()).map_err(|e| e.to_string())?;
            writeln!(ctx.stdout, "  Shell Built-ins: cd, help, exit").map_err(|e| e.to_string())?;
            writeln!(ctx.stdout, "  Animations: sl").map_err(|e| e.to_string())?;
            writeln!(ctx.stdout, "  Core Utilities: arch, cat, cp, dir, echo, grep, head, ls, mkdir, mv, pwd, rm, rmdir, sleep, tail, tee, touch, tree, uname, wc, whoami, yes, etc.").map_err(|e| e.to_string())?;
            Ok(())
        }
        "cd" => {
            let new_dir = args.get(1).map(|s| s.as_str()).unwrap_or(".");
            env::set_current_dir(new_dir).map_err(|e| format!("cd: {}", e))
        }
        "exit" => Ok(()),
        "sl" => {
            let _ = sl::run(args.iter().cloned());
            Ok(())
        }
        _ => wasi_core_utils::execute_with_context(args.iter().cloned(), ctx),
    }
}
