use std::env;
use std::fs::File;
use std::io::{self, Cursor, Read, Write};
use std::sync::{Arc, Mutex};
use colored::*;
pub use wasibox_core::IoContext;

pub fn handle_pipeline<R, W>(line: &str, initial_stdin: Box<R>, final_stdout: Box<W>) -> Result<(), String> 
where 
    R: Read + Send + 'static,
    W: Write + Send + 'static,
{
    let stages: Vec<&str> = line.split('|').collect();
    
    if stages.len() == 1 {
        let mut tokens = shlex::split(stages[0].trim())
            .ok_or_else(|| "Error: Invalid input (quoting error)".to_string())?;
        
        let mut stdout: Box<dyn Write + Send> = final_stdout;
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
            initial_stdin,
            stdout,
            Box::new(io::stderr()),
        );
        return execute_command(tokens, &mut ctx);
    }

    let mut last_output = Vec::new();
    let mut initial_stdin = Some(initial_stdin);
    let mut final_stdout = Some(final_stdout);

    for (i, stage) in stages.iter().enumerate() {
        let is_first = i == 0;
        let is_last = i == stages.len() - 1;
        
        let mut tokens = shlex::split(stage.trim())
            .ok_or_else(|| "Error: Invalid input (quoting error)".to_string())?;

        let stdin: Box<dyn Read + Send> = if is_first {
            initial_stdin.take().ok_or("Internal error: initial_stdin missing")?
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
                final_stdout.take().ok_or("Internal error: final_stdout missing")?
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

pub fn execute_command(args: Vec<String>, ctx: &mut IoContext) -> Result<(), String> {
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
        _ => wasibox_core::execute_with_context(args.iter().cloned(), ctx),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    fn get_temp_dir() -> tempfile::TempDir {
        #[cfg(target_os = "wasi")]
        {
            // Avoid any logic that might indirectly call env::temp_dir()
            tempfile::Builder::new()
                .prefix("test_")
                .tempdir_in(".")
                .expect("Failed to create temp dir in current directory")
        }
        #[cfg(not(target_os = "wasi"))]
        {
            tempfile::tempdir().expect("Failed to create system temp dir")
        }
    }

    #[test]
    fn test_simple_echo() {
        let out = Arc::new(Mutex::new(Vec::new()));
        handle_pipeline("echo hello", Box::new(Cursor::new("")), Box::new(ArcVecWriter { inner: Arc::clone(&out) })).unwrap();
        let buf = out.lock().unwrap();
        assert_eq!(String::from_utf8_lossy(&buf).trim(), "hello");
    }

    #[test]
    fn test_pipe_echo_cat() {
        let out = Arc::new(Mutex::new(Vec::new()));
        handle_pipeline("echo hello | cat", Box::new(Cursor::new("")), Box::new(ArcVecWriter { inner: Arc::clone(&out) })).unwrap();
        let buf = out.lock().unwrap();
        assert_eq!(String::from_utf8_lossy(&buf).trim(), "hello");
    }

    #[test]
    fn test_pipe_grep() {
        let out = Arc::new(Mutex::new(Vec::new()));
        // Our 'echo' likely doesn't support -e. Let's use a simpler check.
        handle_pipeline("echo world | grep world", Box::new(Cursor::new("")), Box::new(ArcVecWriter { inner: Arc::clone(&out) })).unwrap();
        let buf = out.lock().unwrap();
        assert_eq!(String::from_utf8_lossy(&buf).trim(), "world");
    }

    #[test]
    fn test_redirection_create() {
        let dir = get_temp_dir();
        let file_path = dir.path().join("test_create.txt");
        let cmd = format!("echo hello > \"{}\"", file_path.display());
        
        handle_pipeline(&cmd, Box::new(Cursor::new("")), Box::new(io::sink())).unwrap();
        
        let content = std::fs::read_to_string(file_path).unwrap();
        assert_eq!(content.trim(), "hello");
    }

    #[test]
    fn test_redirection_append() {
        let dir = get_temp_dir();
        let file_path = dir.path().join("test_append.txt");
        
        let cmd1 = format!("echo hello > \"{}\"", file_path.display());
        handle_pipeline(&cmd1, Box::new(Cursor::new("")), Box::new(io::sink())).unwrap();
        
        let cmd2 = format!("echo world >> \"{}\"", file_path.display());
        handle_pipeline(&cmd2, Box::new(Cursor::new("")), Box::new(io::sink())).unwrap();
        
        let content = std::fs::read_to_string(file_path).unwrap();
        assert!(content.contains("hello"));
        assert!(content.contains("world"));
    }

    #[test]
    fn test_complex_pipeline() {
        let out = Arc::new(Mutex::new(Vec::new()));
        handle_pipeline("echo hello | grep h | wc -c", Box::new(Cursor::new("")), Box::new(ArcVecWriter { inner: Arc::clone(&out) })).unwrap();
        let buf = out.lock().unwrap();
        let result = String::from_utf8_lossy(&buf).trim().to_string();
        assert_eq!(result, "6");
    }
}
