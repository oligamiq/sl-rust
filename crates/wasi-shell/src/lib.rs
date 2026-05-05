use std::env;
use std::fs::File;
use std::io::{self, Read, Write};
use std::sync::{Arc, Mutex};
use colored::*;
pub use wasibox_core::IoContext;

pub fn handle_pipeline<R, W, F>(
    line: &str, 
    initial_stdin: Box<R>, 
    final_stdout: Box<W>,
    handler: &mut F,
) -> Result<(), String> 
where 
    R: Read + Send + 'static,
    W: Write + Send + 'static,
    F: FnMut(&[String], &mut IoContext) -> Option<Result<(), String>> + Send,
{
    let stages_str: Vec<&str> = line.split('|').collect();
    let handler = Arc::new(Mutex::new(handler));

    let mut final_stdout = Some(final_stdout);

    std::thread::scope(|s| {
        let mut prev_reader: Box<dyn Read + Send> = Box::new(initial_stdin);
        let mut threads = Vec::new();

        for (i, stage_str) in stages_str.iter().enumerate() {
            let is_last = i == stages_str.len() - 1;
            let mut tokens = shlex::split(stage_str.trim())
                .ok_or_else(|| "Error: Invalid input (quoting error)".to_string())?;

            let stdin = std::mem::replace(&mut prev_reader, Box::new(io::empty()));
            let (stdout, next_reader): (Box<dyn Write + Send>, Option<Box<dyn Read + Send>>) = if is_last {
                let mut out: Box<dyn Write + Send> = Box::new(final_stdout.take().unwrap());
                if let Some(pos) = tokens.iter().position(|t| t == ">" || t == ">>") {
                    let append = tokens[pos] == ">>";
                    let filename = tokens.get(pos + 1).ok_or("Error: Missing file for redirection")?;
                    let file = if append {
                        std::fs::OpenOptions::new().create(true).append(true).open(filename)
                    } else {
                        File::create(filename)
                    }.map_err(|e| format!("Error opening file: {}", e))?;
                    out = Box::new(file);
                    tokens.truncate(pos);
                }
                (out, None)
            } else {
                let (reader, writer) = create_pipe();
                (Box::new(writer), Some(Box::new(reader)))
            };

            if let Some(r) = next_reader {
                prev_reader = r;
            }

            let handler = Arc::clone(&handler);
            let thread = s.spawn(move || {
                let mut ctx = IoContext::new(stdin, stdout, Box::new(io::stderr()));
                let mut h = |args: &[String], ctx: &mut IoContext| {
                    let mut h_lock = handler.lock().unwrap();
                    (h_lock)(args, ctx)
                };
                execute_command(tokens, &mut ctx, &mut h)
            });
            threads.push(thread);
        }

        let mut final_res = Ok(());
        for thread in threads {
            if let Ok(res) = thread.join() {
                if res.is_err() && final_res.is_ok() {
                    final_res = res;
                }
            }
        }
        final_res
    })
}

struct PipeReader {
    rx: std::sync::mpsc::Receiver<Vec<u8>>,
    buffer: Vec<u8>,
    pos: usize,
}
impl Read for PipeReader {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        if self.pos >= self.buffer.len() {
            match self.rx.recv() {
                Ok(data) => {
                    self.buffer = data;
                    self.pos = 0;
                }
                Err(_) => return Ok(0),
            }
        }
        let available = self.buffer.len() - self.pos;
        let to_copy = std::cmp::min(available, buf.len());
        buf[..to_copy].copy_from_slice(&self.buffer[self.pos..self.pos + to_copy]);
        self.pos += to_copy;
        Ok(to_copy)
    }
}

struct PipeWriter {
    tx: std::sync::mpsc::Sender<Vec<u8>>,
}
impl Write for PipeWriter {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.tx.send(buf.to_vec()).map_err(|e| io::Error::new(io::ErrorKind::BrokenPipe, e))?;
        Ok(buf.len())
    }
    fn flush(&mut self) -> io::Result<()> { Ok(()) }
}

fn create_pipe() -> (PipeReader, PipeWriter) {
    let (tx, rx) = std::sync::mpsc::channel();
    (PipeReader { rx, buffer: Vec::new(), pos: 0 }, PipeWriter { tx })
}

pub struct ArcVecWriter {
    pub inner: Arc<Mutex<Vec<u8>>>,
}
impl Write for ArcVecWriter {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let mut inner = self.inner.lock().map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))?;
        inner.extend_from_slice(buf);
        Ok(buf.len())
    }
    fn flush(&mut self) -> io::Result<()> { Ok(()) }
}

pub fn execute_command<F>(
    args: Vec<String>, 
    ctx: &mut IoContext,
    handler: &mut F,
) -> Result<(), String> 
where
    F: FnMut(&[String], &mut IoContext) -> Option<Result<(), String>>,
{
    if args.is_empty() {
        return Ok(());
    }

    if let Some(res) = handler(&args, ctx) {
        return res;
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

pub fn handle_parallel<F>(
    lines: Vec<String>,
    handler: Arc<F>,
) -> Vec<Result<(), String>>
where
    F: Fn(&[String], &mut IoContext) -> Option<Result<(), String>> + Send + Sync + 'static,
{
    let mut handles = Vec::new();
    for line in lines {
        let handler = Arc::clone(&handler);
        let handle = std::thread::spawn(move || {
            let mut h = |args: &[String], ctx: &mut IoContext| handler(args, ctx);
            handle_pipeline(
                &line, 
                Box::new(io::empty()), 
                Box::new(io::sink()), 
                &mut h
            )
        });
        handles.push(handle);
    }

    handles.into_iter()
        .map(|h| h.join().unwrap_or(Err("Thread panicked".to_string())))
        .collect()
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
        handle_pipeline("echo hello", Box::new(Cursor::new("")), Box::new(ArcVecWriter { inner: Arc::clone(&out) }), &mut |_, _| None).unwrap();
        let buf = out.lock().unwrap();
        assert_eq!(String::from_utf8_lossy(&buf).trim(), "hello");
    }

    #[test]
    fn test_pipe_echo_cat() {
        let out = Arc::new(Mutex::new(Vec::new()));
        handle_pipeline("echo hello | cat", Box::new(Cursor::new("")), Box::new(ArcVecWriter { inner: Arc::clone(&out) }), &mut |_, _| None).unwrap();
        let buf = out.lock().unwrap();
        assert_eq!(String::from_utf8_lossy(&buf).trim(), "hello");
    }

    #[test]
    fn test_pipe_grep() {
        let out = Arc::new(Mutex::new(Vec::new()));
        // Our 'echo' likely doesn't support -e. Let's use a simpler check.
        handle_pipeline("echo world | grep world", Box::new(Cursor::new("")), Box::new(ArcVecWriter { inner: Arc::clone(&out) }), &mut |_, _| None).unwrap();
        let buf = out.lock().unwrap();
        assert_eq!(String::from_utf8_lossy(&buf).trim(), "world");
    }

    #[test]
    fn test_redirection_create() {
        let dir = get_temp_dir();
        let file_path = dir.path().join("test_create.txt");
        let cmd = format!("echo hello > \"{}\"", file_path.display());
        
        handle_pipeline(&cmd, Box::new(Cursor::new("")), Box::new(io::sink()), &mut |_, _| None).unwrap();
        
        let content = std::fs::read_to_string(file_path).unwrap();
        assert_eq!(content.trim(), "hello");
    }

    #[test]
    fn test_redirection_append() {
        let dir = get_temp_dir();
        let file_path = dir.path().join("test_append.txt");
        
        let cmd1 = format!("echo hello > \"{}\"", file_path.display());
        handle_pipeline(&cmd1, Box::new(Cursor::new("")), Box::new(io::sink()), &mut |_, _| None).unwrap();
        
        let cmd2 = format!("echo world >> \"{}\"", file_path.display());
        handle_pipeline(&cmd2, Box::new(Cursor::new("")), Box::new(io::sink()), &mut |_, _| None).unwrap();
        
        let content = std::fs::read_to_string(file_path).unwrap();
        assert!(content.contains("hello"));
        assert!(content.contains("world"));
    }

    #[test]
    fn test_complex_pipeline() {
        let out = Arc::new(Mutex::new(Vec::new()));
        handle_pipeline("echo hello | grep h | wc -c", Box::new(Cursor::new("")), Box::new(ArcVecWriter { inner: Arc::clone(&out) }), &mut |_, _| None).unwrap();
        let buf = out.lock().unwrap();
        let result = String::from_utf8_lossy(&buf).trim().to_string();
        assert_eq!(result, "6");
    }
    #[test]
    fn test_custom_handler() {
        let out = Arc::new(Mutex::new(Vec::new()));
        let mut handler = |args: &[String], ctx: &mut IoContext| {
            if args[0] == "magic" {
                write!(ctx.stdout, "magic happen").unwrap();
                return Some(Ok(()));
            }
            None
        };
        handle_pipeline("magic", Box::new(io::empty()), Box::new(ArcVecWriter { inner: Arc::clone(&out) }), &mut handler).unwrap();
        let buf = out.lock().unwrap();
        assert_eq!(String::from_utf8_lossy(&buf), "magic happen");
    }

    #[test]
    fn test_parallel_execution() {
        let handler = Arc::new(|args: &[String], _ctx: &mut IoContext| {
            if args[0] == "slow" {
                std::thread::sleep(std::time::Duration::from_millis(100));
                return Some(Ok(()));
            }
            None
        });
        let lines = vec!["slow".to_string(), "slow".to_string(), "slow".to_string()];
        let start = std::time::Instant::now();
        let results = handle_parallel(lines, handler);
        let duration = start.elapsed();
        
        assert_eq!(results.len(), 3);
        for res in results {
            assert!(res.is_ok());
        }
        // If parallel, it should take less than 300ms.
        assert!(duration < std::time::Duration::from_millis(250));
    }

    #[test]
    fn test_streaming_pipeline() {
        let out = Arc::new(Mutex::new(Vec::new()));
        handle_pipeline("yes | head -n 2", Box::new(io::empty()), Box::new(ArcVecWriter { inner: Arc::clone(&out) }), &mut |_, _| None).unwrap();
        let buf = out.lock().unwrap();
        let result = String::from_utf8_lossy(&buf);
        assert_eq!(result.trim(), "y\ny");
    }
}
