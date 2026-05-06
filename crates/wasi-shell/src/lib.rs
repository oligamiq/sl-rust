use std::collections::HashMap;
use std::env;
use std::fs::File;
use std::io::{self, Read, Write};
use std::sync::{Arc, Mutex};
use colored::*;
pub use wasibox_core::IoContext;

// ---------------------------------------------------------------------------
// CommandRegistry
// ---------------------------------------------------------------------------

/// A function that handles a shell command.
///
/// Receives the full argument list (argv\[0\] = command name) and an I/O context
/// whose stdin/stdout/stderr are already wired to the pipeline.
pub type CommandFn = Arc<dyn Fn(&[String], &mut IoContext) -> Result<(), String> + Send + Sync>;

/// Registry of named commands.
///
/// Commands registered here are used by [`handle_pipeline`] and
/// [`handle_parallel`] to dispatch each stage of a pipeline.
///
/// # Examples
///
/// ```ignore
/// use wasi_shell::{CommandRegistry, handle_pipeline, IoContext};
/// use std::io::{self, Write};
///
/// let mut reg = CommandRegistry::with_builtins();
///
/// // Add a custom command
/// reg.register("greet", |args, ctx| {
///     let name = args.get(1).map(|s| s.as_str()).unwrap_or("world");
///     writeln!(ctx.stdout, "Hello, {}!", name).map_err(|e| e.to_string())
/// });
///
/// handle_pipeline("greet Rust", Box::new(io::empty()), Box::new(io::stdout()), &reg).unwrap();
/// ```
pub struct CommandRegistry {
    commands: HashMap<String, CommandFn>,
    fallback: Option<CommandFn>,
}

impl CommandRegistry {
    /// Create an empty registry **with** a `wasibox_core` fallback.
    ///
    /// Any command name not explicitly registered will be forwarded to
    /// `wasibox_core::execute_with_context`.
    pub fn new() -> Self {
        Self {
            commands: HashMap::new(),
            fallback: Some(Arc::new(|args: &[String], ctx: &mut IoContext| {
                wasibox_core::execute_with_context(args.iter().cloned(), ctx)
            })),
        }
    }

    /// Create a registry pre-loaded with shell built-in commands
    /// (`cd`, `help`, `exit`, `sl`) and a `wasibox_core` fallback for all
    /// core utilities (echo, cat, grep, seq, head, …).
    pub fn with_builtins() -> Self {
        let mut reg = Self::new();

        reg.register("help", |_args, ctx| {
            writeln!(ctx.stdout, "{}", "Available Commands:".yellow().bold()).map_err(|e| e.to_string())?;
            
            let mut shell_builtins = vec!["cd", "help", "exit"];
            #[cfg(feature = "clear")] shell_builtins.push("clear");
            shell_builtins.sort();
            writeln!(ctx.stdout, "  Shell Built-ins: {}", shell_builtins.join(", ")).map_err(|e| e.to_string())?;

            #[cfg(feature = "sl")]
            writeln!(ctx.stdout, "  Animations: sl").map_err(|e| e.to_string())?;
            
            let mut utils = Vec::new();
            #[cfg(feature = "arch")] utils.push("arch");
            #[cfg(feature = "basename")] utils.push("basename");
            #[cfg(feature = "cat")] utils.push("cat");
            #[cfg(feature = "cp")] utils.push("cp");
            #[cfg(feature = "dir")] utils.push("dir");
            #[cfg(feature = "dirname")] utils.push("dirname");
            #[cfg(feature = "echo")] utils.push("echo");
            #[cfg(feature = "env")] utils.push("env");
            #[cfg(feature = "false")] utils.push("false");
            #[cfg(feature = "grep")] utils.push("grep");
            #[cfg(feature = "head")] utils.push("head");
            #[cfg(feature = "link")] utils.push("link");
            #[cfg(feature = "ln")] utils.push("ln");
            #[cfg(feature = "ls")] utils.push("ls");
            #[cfg(feature = "mkdir")] utils.push("mkdir");
            #[cfg(feature = "mv")] utils.push("mv");
            #[cfg(feature = "pwd")] utils.push("pwd");
            #[cfg(feature = "rm")] utils.push("rm");
            #[cfg(feature = "rmdir")] utils.push("rmdir");
            #[cfg(feature = "seq")] utils.push("seq");
            #[cfg(feature = "sleep")] utils.push("sleep");
            #[cfg(feature = "tail")] utils.push("tail");
            #[cfg(feature = "tee")] utils.push("tee");
            #[cfg(feature = "touch")] utils.push("touch");
            #[cfg(feature = "tree")] utils.push("tree");
            #[cfg(feature = "true")] utils.push("true");
            #[cfg(feature = "uname")] utils.push("uname");
            #[cfg(feature = "unlink")] utils.push("unlink");
            #[cfg(feature = "wc")] utils.push("wc");
            #[cfg(feature = "whoami")] utils.push("whoami");
            #[cfg(feature = "yes")] utils.push("yes");
            
            if !utils.is_empty() {
                utils.sort();
                writeln!(ctx.stdout, "  Core Utilities: {}", utils.join(", ")).map_err(|e| e.to_string())?;
            }
            Ok(())
        });

        reg.register("cd", |args, _ctx| {
            let new_dir = args.get(1).map(|s| s.as_str()).unwrap_or(".");
            env::set_current_dir(new_dir).map_err(|e| format!("cd: {}", e))
        });

        reg.register("exit", |_args, _ctx| Ok(()));

        #[cfg(feature = "clear")]
        reg.register("clear", |_args, ctx| {
            write!(ctx.stdout, "\x1B[2J\x1B[1;1H").map_err(|e| e.to_string())?;
            ctx.stdout.flush().map_err(|e| e.to_string())
        });

        #[cfg(feature = "sl")]
        reg.register("sl", |args, _ctx| {
            let _ = sl::run(args.iter().cloned());
            Ok(())
        });

        // Register coreutils from wasibox-core
        #[cfg(feature = "arch")]
        reg.register("arch", |args, ctx| wasibox_core::utils::arch::execute_with_context(args.iter().cloned(), ctx));
        #[cfg(feature = "basename")]
        reg.register("basename", |args, ctx| wasibox_core::utils::basename::execute_with_context(args.iter().cloned(), ctx));
        #[cfg(feature = "cat")]
        reg.register("cat", |args, ctx| wasibox_core::utils::cat::execute_with_context(args.iter().cloned(), ctx));
        #[cfg(feature = "cp")]
        reg.register("cp", |args, ctx| wasibox_core::utils::cp::execute_with_context(args.iter().cloned(), ctx));
        #[cfg(feature = "dir")]
        reg.register("dir", |args, ctx| wasibox_core::utils::dir::execute_with_context(args.iter().cloned(), ctx));
        #[cfg(feature = "dirname")]
        reg.register("dirname", |args, ctx| wasibox_core::utils::dirname::execute_with_context(args.iter().cloned(), ctx));
        #[cfg(feature = "echo")]
        reg.register("echo", |args, ctx| wasibox_core::utils::echo::execute_with_context(args.iter().cloned(), ctx));
        #[cfg(feature = "env")]
        reg.register("env", |args, ctx| wasibox_core::utils::env::execute_with_context(args.iter().cloned(), ctx));
        #[cfg(feature = "false")]
        reg.register("false", |args, ctx| wasibox_core::utils::r#false::execute_with_context(args.iter().cloned(), ctx));
        #[cfg(feature = "grep")]
        reg.register("grep", |args, ctx| wasibox_core::utils::grep::execute_with_context(args.iter().cloned(), ctx));
        #[cfg(feature = "head")]
        reg.register("head", |args, ctx| wasibox_core::utils::head::execute_with_context(args.iter().cloned(), ctx));
        #[cfg(feature = "link")]
        reg.register("link", |args, ctx| wasibox_core::utils::link::execute_with_context(args.iter().cloned(), ctx));
        #[cfg(feature = "ln")]
        reg.register("ln", |args, ctx| wasibox_core::utils::ln::execute_with_context(args.iter().cloned(), ctx));
        #[cfg(feature = "ls")]
        reg.register("ls", |args, ctx| wasibox_core::utils::ls::execute_with_context(args.iter().cloned(), ctx));
        #[cfg(feature = "mkdir")]
        reg.register("mkdir", |args, ctx| wasibox_core::utils::mkdir::execute_with_context(args.iter().cloned(), ctx));
        #[cfg(feature = "mv")]
        reg.register("mv", |args, ctx| wasibox_core::utils::mv::execute_with_context(args.iter().cloned(), ctx));
        #[cfg(feature = "pwd")]
        reg.register("pwd", |args, ctx| wasibox_core::utils::pwd::execute_with_context(args.iter().cloned(), ctx));
        #[cfg(feature = "rm")]
        reg.register("rm", |args, ctx| wasibox_core::utils::rm::execute_with_context(args.iter().cloned(), ctx));
        #[cfg(feature = "rmdir")]
        reg.register("rmdir", |args, ctx| wasibox_core::utils::rmdir::execute_with_context(args.iter().cloned(), ctx));
        #[cfg(feature = "seq")]
        reg.register("seq", |args, ctx| wasibox_core::utils::seq::execute_with_context(args.iter().cloned(), ctx));
        #[cfg(feature = "sleep")]
        reg.register("sleep", |args, ctx| wasibox_core::utils::sleep::execute_with_context(args.iter().cloned(), ctx));
        #[cfg(feature = "tail")]
        reg.register("tail", |args, ctx| wasibox_core::utils::tail::execute_with_context(args.iter().cloned(), ctx));
        #[cfg(feature = "tee")]
        reg.register("tee", |args, ctx| wasibox_core::utils::tee::execute_with_context(args.iter().cloned(), ctx));
        #[cfg(feature = "touch")]
        reg.register("touch", |args, ctx| wasibox_core::utils::touch::execute_with_context(args.iter().cloned(), ctx));
        #[cfg(feature = "tree")]
        reg.register("tree", |args, ctx| wasibox_core::utils::tree::execute_with_context(args.iter().cloned(), ctx));
        #[cfg(feature = "true")]
        reg.register("true", |args, ctx| wasibox_core::utils::r#true::execute_with_context(args.iter().cloned(), ctx));
        #[cfg(feature = "uname")]
        reg.register("uname", |args, ctx| wasibox_core::utils::uname::execute_with_context(args.iter().cloned(), ctx));
        #[cfg(feature = "unlink")]
        reg.register("unlink", |args, ctx| wasibox_core::utils::unlink::execute_with_context(args.iter().cloned(), ctx));
        #[cfg(feature = "wc")]
        reg.register("wc", |args, ctx| wasibox_core::utils::wc::execute_with_context(args.iter().cloned(), ctx));
        #[cfg(feature = "whoami")]
        reg.register("whoami", |args, ctx| wasibox_core::utils::whoami::execute_with_context(args.iter().cloned(), ctx));
        #[cfg(feature = "yes")]
        reg.register("yes", |args, ctx| wasibox_core::utils::yes::execute_with_context(args.iter().cloned(), ctx));

        reg
    }

    /// Register (or replace) a command.
    pub fn register<F>(&mut self, name: impl Into<String>, handler: F)
    where
        F: Fn(&[String], &mut IoContext) -> Result<(), String> + Send + Sync + 'static,
    {
        self.commands.insert(name.into(), Arc::new(handler));
    }

    /// Set a fallback handler invoked when no explicit command matches.
    pub fn set_fallback<F>(&mut self, handler: F)
    where
        F: Fn(&[String], &mut IoContext) -> Result<(), String> + Send + Sync + 'static,
    {
        self.fallback = Some(Arc::new(handler));
    }

    /// Remove the fallback handler. Unknown commands will return an error.
    pub fn remove_fallback(&mut self) {
        self.fallback = None;
    }

    /// Look up and execute a command.
    pub fn execute(&self, args: &[String], ctx: &mut IoContext) -> Result<(), String> {
        if args.is_empty() {
            return Ok(());
        }
        let cmd = &args[0];

        if let Some(handler) = self.commands.get(cmd.as_str()) {
            return handler(args, ctx);
        }
        if let Some(ref fallback) = self.fallback {
            return fallback(args, ctx);
        }
        Err(format!("command not found: {}", cmd))
    }
}

impl Default for CommandRegistry {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// Pipeline execution
// ---------------------------------------------------------------------------

/// Check if an error string represents a BrokenPipe (normal pipeline termination).
fn is_broken_pipe(err: &str) -> bool {
    err.contains("Broken pipe")
        || err.contains("BrokenPipe")
        || err.contains("broken pipe")
}

/// Execute a shell pipeline (commands separated by `|`).
///
/// Each stage runs in its own thread; stages are connected by in-process
/// pipes. The last stage supports `>` / `>>` redirection.
pub fn handle_pipeline(
    line: &str,
    initial_stdin: Box<dyn Read + Send + 'static>,
    final_stdout: Box<dyn Write + Send + 'static>,
    registry: &CommandRegistry,
) -> Result<(), String> {
    let stages_str: Vec<&str> = line.split('|').collect();

    let mut final_stdout = Some(final_stdout);

    std::thread::scope(|s| {
        let mut prev_reader: Box<dyn Read + Send> = initial_stdin;
        let mut threads = Vec::new();

        for (i, stage_str) in stages_str.iter().enumerate() {
            let is_last = i == stages_str.len() - 1;
            let mut tokens = shlex::split(stage_str.trim())
                .ok_or_else(|| "Error: Invalid input (quoting error)".to_string())?;

            let stdin = std::mem::replace(&mut prev_reader, Box::new(io::empty()));
            let (stdout, next_reader): (Box<dyn Write + Send>, Option<Box<dyn Read + Send>>) = if is_last {
                let mut out: Box<dyn Write + Send> = final_stdout.take().unwrap();
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

            let thread = s.spawn(move || {
                let mut ctx = IoContext::new(stdin, stdout, Box::new(io::stderr()));
                registry.execute(&tokens, &mut ctx)
            });
            threads.push((i, thread));
        }

        // Collect results: BrokenPipe in non-last stages is normal
        // (upstream terminated by downstream closing the pipe).
        let last_idx = stages_str.len() - 1;
        let mut final_res = Ok(());
        for (idx, thread) in threads {
            match thread.join() {
                Ok(Ok(())) => {}
                Ok(Err(e)) => {
                    if idx != last_idx && is_broken_pipe(&e) {
                        continue;
                    }
                    if final_res.is_ok() {
                        final_res = Err(e);
                    }
                }
                Err(_) => {
                    if final_res.is_ok() {
                        final_res = Err("Thread panicked".to_string());
                    }
                }
            }
        }
        final_res
    })
}

/// Execute multiple independent command lines in parallel.
///
/// The first line receives `initial_stdin` / `final_stdout`; all subsequent
/// lines use `io::empty()` / `io::stdout()`.
pub fn handle_parallel(
    lines: Vec<String>,
    initial_stdin: Box<dyn Read + Send + 'static>,
    final_stdout: Box<dyn Write + Send + 'static>,
    registry: Arc<CommandRegistry>,
) -> Vec<Result<(), String>> {
    let mut handles = Vec::new();
    let mut stdin_opt: Option<Box<dyn Read + Send>> = Some(initial_stdin);
    let mut stdout_opt: Option<Box<dyn Write + Send>> = Some(final_stdout);

    for line in lines {
        let registry = Arc::clone(&registry);
        let stdin: Box<dyn Read + Send> = stdin_opt.take().unwrap_or_else(|| Box::new(io::empty()));
        let stdout: Box<dyn Write + Send> = stdout_opt.take().unwrap_or_else(|| Box::new(io::stdout()));
        let handle = std::thread::spawn(move || {
            handle_pipeline(&line, stdin, stdout, &*registry)
        });
        handles.push(handle);
    }

    handles.into_iter()
        .map(|h| h.join().unwrap_or(Err("Thread panicked".to_string())))
        .collect()
}

// ---------------------------------------------------------------------------
// Pipe implementation
// ---------------------------------------------------------------------------

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
                Err(_) => return Ok(0), // Writer dropped = EOF
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

// ---------------------------------------------------------------------------
// Utility types
// ---------------------------------------------------------------------------

/// A thread-safe writer backed by a shared `Vec<u8>`. Useful for tests.
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

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{BufRead, BufReader, Cursor};

    fn get_temp_dir() -> tempfile::TempDir {
        #[cfg(target_os = "wasi")]
        {
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

    /// Shorthand: registry with all builtins (shell + wasibox-core)
    fn builtins() -> CommandRegistry {
        CommandRegistry::with_builtins()
    }

    // ── basic pipeline tests ────────────────────────────────────────────

    #[test]
    fn test_simple_echo() {
        let out = Arc::new(Mutex::new(Vec::new()));
        handle_pipeline("echo hello", Box::new(Cursor::new("")), Box::new(ArcVecWriter { inner: Arc::clone(&out) }), &builtins()).unwrap();
        let buf = out.lock().unwrap();
        assert_eq!(String::from_utf8_lossy(&buf).trim(), "hello");
    }

    #[test]
    fn test_pipe_echo_cat() {
        let out = Arc::new(Mutex::new(Vec::new()));
        handle_pipeline("echo hello | cat", Box::new(Cursor::new("")), Box::new(ArcVecWriter { inner: Arc::clone(&out) }), &builtins()).unwrap();
        let buf = out.lock().unwrap();
        assert_eq!(String::from_utf8_lossy(&buf).trim(), "hello");
    }

    #[test]
    fn test_pipe_grep() {
        let out = Arc::new(Mutex::new(Vec::new()));
        handle_pipeline("echo world | grep world", Box::new(Cursor::new("")), Box::new(ArcVecWriter { inner: Arc::clone(&out) }), &builtins()).unwrap();
        let buf = out.lock().unwrap();
        assert_eq!(String::from_utf8_lossy(&buf).trim(), "world");
    }

    #[test]
    fn test_redirection_create() {
        let dir = get_temp_dir();
        let file_path = dir.path().join("test_create.txt");
        let cmd = format!("echo hello > \"{}\"", file_path.display());
        handle_pipeline(&cmd, Box::new(Cursor::new("")), Box::new(io::sink()), &builtins()).unwrap();
        let content = std::fs::read_to_string(file_path).unwrap();
        assert_eq!(content.trim(), "hello");
    }

    #[test]
    fn test_redirection_append() {
        let dir = get_temp_dir();
        let file_path = dir.path().join("test_append.txt");
        let cmd1 = format!("echo hello > \"{}\"", file_path.display());
        handle_pipeline(&cmd1, Box::new(Cursor::new("")), Box::new(io::sink()), &builtins()).unwrap();
        let cmd2 = format!("echo world >> \"{}\"", file_path.display());
        handle_pipeline(&cmd2, Box::new(Cursor::new("")), Box::new(io::sink()), &builtins()).unwrap();
        let content = std::fs::read_to_string(file_path).unwrap();
        assert!(content.contains("hello"));
        assert!(content.contains("world"));
    }

    #[test]
    fn test_complex_pipeline() {
        let out = Arc::new(Mutex::new(Vec::new()));
        handle_pipeline("echo hello | grep h | wc -c", Box::new(Cursor::new("")), Box::new(ArcVecWriter { inner: Arc::clone(&out) }), &builtins()).unwrap();
        let buf = out.lock().unwrap();
        assert_eq!(String::from_utf8_lossy(&buf).trim(), "6");
    }

    // ── custom command tests ────────────────────────────────────────────

    #[test]
    fn test_custom_command() {
        let mut reg = CommandRegistry::with_builtins();
        reg.register("magic", |_args, ctx| {
            write!(ctx.stdout, "magic happen").map_err(|e| e.to_string())
        });

        let out = Arc::new(Mutex::new(Vec::new()));
        handle_pipeline("magic", Box::new(io::empty()), Box::new(ArcVecWriter { inner: Arc::clone(&out) }), &reg).unwrap();
        let buf = out.lock().unwrap();
        assert_eq!(String::from_utf8_lossy(&buf), "magic happen");
    }

    #[test]
    fn test_custom_command_in_pipeline() {
        // Custom "double" command that duplicates each input line
        let mut reg = CommandRegistry::with_builtins();
        reg.register("double", |_args, ctx| {
            for line in BufReader::new(&mut ctx.stdin).lines() {
                let line = match line {
                    Ok(l) => l,
                    Err(_) => break,
                };
                if writeln!(ctx.stdout, "{}", line).is_err() { break; }
                if writeln!(ctx.stdout, "{}", line).is_err() { break; }
            }
            Ok(())
        });

        let out = Arc::new(Mutex::new(Vec::new()));
        handle_pipeline("echo hello | double", Box::new(io::empty()), Box::new(ArcVecWriter { inner: Arc::clone(&out) }), &reg).unwrap();
        let buf = out.lock().unwrap();
        assert_eq!(String::from_utf8_lossy(&buf).trim(), "hello\nhello");
    }

    #[test]
    fn test_override_builtin() {
        // Override "echo" with a custom version
        let mut reg = CommandRegistry::with_builtins();
        reg.register("echo", |args, ctx| {
            let msg = args[1..].join(" ").to_uppercase();
            writeln!(ctx.stdout, "{}", msg).map_err(|e| e.to_string())
        });

        let out = Arc::new(Mutex::new(Vec::new()));
        handle_pipeline("echo hello", Box::new(io::empty()), Box::new(ArcVecWriter { inner: Arc::clone(&out) }), &reg).unwrap();
        let buf = out.lock().unwrap();
        assert_eq!(String::from_utf8_lossy(&buf).trim(), "HELLO");
    }

    // ── fully external pipeline (no builtins at all) ────────────────────

    #[test]
    fn test_all_external_pipeline() {
        let mut reg = CommandRegistry::new();
        reg.remove_fallback(); // no wasibox-core fallback

        // "count" — infinite counter
        reg.register("count", |_args, ctx| {
            for i in 1u64.. {
                if writeln!(ctx.stdout, "{}", i).is_err() { break; }
            }
            Ok(())
        });

        // "filter2" — keep lines containing '2'
        reg.register("filter2", |_args, ctx| {
            for line in BufReader::new(&mut ctx.stdin).lines() {
                let line = match line {
                    Ok(l) => l,
                    Err(_) => break,
                };
                if line.contains('2') {
                    if writeln!(ctx.stdout, "{}", line).is_err() { break; }
                }
            }
            Ok(())
        });

        // "take N" — first N lines
        reg.register("take", |args, ctx| {
            let n: usize = args.get(1).and_then(|s| s.parse().ok()).unwrap_or(5);
            for (i, line) in BufReader::new(&mut ctx.stdin).lines().enumerate() {
                if i >= n { break; }
                let line = match line {
                    Ok(l) => l,
                    Err(_) => break,
                };
                if writeln!(ctx.stdout, "{}", line).is_err() { break; }
            }
            Ok(())
        });

        let out = Arc::new(Mutex::new(Vec::new()));
        handle_pipeline(
            "count | filter2 | take 3",
            Box::new(io::empty()),
            Box::new(ArcVecWriter { inner: Arc::clone(&out) }),
            &reg,
        ).unwrap();

        let buf = out.lock().unwrap();
        let result = String::from_utf8_lossy(&buf);
        let lines: Vec<&str> = result.trim().lines().collect();
        assert_eq!(lines.len(), 3);
        for line in &lines {
            assert!(line.contains('2'), "expected '2' in line, got: {}", line);
        }
        assert_eq!(lines[0], "2");
        assert_eq!(lines[1], "12");
        assert_eq!(lines[2], "20");
    }

    // ── parallel tests ──────────────────────────────────────────────────

    #[test]
    fn test_parallel_execution() {
        let mut reg = CommandRegistry::with_builtins();
        reg.register("slow", |_args, _ctx| {
            std::thread::sleep(std::time::Duration::from_millis(100));
            Ok(())
        });
        let registry = Arc::new(reg);

        let lines = vec!["slow".to_string(), "slow".to_string(), "slow".to_string()];
        let start = std::time::Instant::now();
        let results = handle_parallel(lines, Box::new(io::empty()), Box::new(io::sink()), registry);
        let duration = start.elapsed();

        assert_eq!(results.len(), 3);
        for res in results {
            assert!(res.is_ok());
        }
        assert!(duration < std::time::Duration::from_millis(250));
    }

    // ── streaming / infinite pipeline tests ─────────────────────────────

    #[test]
    fn test_streaming_pipeline() {
        let out = Arc::new(Mutex::new(Vec::new()));
        handle_pipeline("yes | head -n 2", Box::new(io::empty()), Box::new(ArcVecWriter { inner: Arc::clone(&out) }), &builtins()).unwrap();
        let buf = out.lock().unwrap();
        assert_eq!(String::from_utf8_lossy(&buf).trim(), "y\ny");
    }

    #[test]
    fn test_seq_pipeline_head() {
        let out = Arc::new(Mutex::new(Vec::new()));
        handle_pipeline("seq | head -n 5", Box::new(io::empty()), Box::new(ArcVecWriter { inner: Arc::clone(&out) }), &builtins()).unwrap();
        let buf = out.lock().unwrap();
        assert_eq!(String::from_utf8_lossy(&buf).trim(), "1\n2\n3\n4\n5");
    }

    #[test]
    fn test_seq_pipeline_grep_head() {
        let out = Arc::new(Mutex::new(Vec::new()));
        handle_pipeline("seq | grep 2 | head -n 3", Box::new(io::empty()), Box::new(ArcVecWriter { inner: Arc::clone(&out) }), &builtins()).unwrap();
        let buf = out.lock().unwrap();
        let result = String::from_utf8_lossy(&buf);
        let lines: Vec<&str> = result.trim().lines().collect();
        assert_eq!(lines.len(), 3);
        for line in &lines {
            assert!(line.contains('2'));
        }
        assert_eq!(lines[0], "2");
        assert_eq!(lines[1], "12");
        assert_eq!(lines[2], "20");
    }

    #[test]
    fn test_seq_finite() {
        let out = Arc::new(Mutex::new(Vec::new()));
        handle_pipeline("seq 3", Box::new(io::empty()), Box::new(ArcVecWriter { inner: Arc::clone(&out) }), &builtins()).unwrap();
        let buf = out.lock().unwrap();
        assert_eq!(String::from_utf8_lossy(&buf).trim(), "1\n2\n3");
    }

    #[test]
    fn test_seq_range() {
        let out = Arc::new(Mutex::new(Vec::new()));
        handle_pipeline("seq 5 8", Box::new(io::empty()), Box::new(ArcVecWriter { inner: Arc::clone(&out) }), &builtins()).unwrap();
        let buf = out.lock().unwrap();
        assert_eq!(String::from_utf8_lossy(&buf).trim(), "5\n6\n7\n8");
    }

    #[test]
    fn test_seq_step() {
        let out = Arc::new(Mutex::new(Vec::new()));
        handle_pipeline("seq 1 2 10", Box::new(io::empty()), Box::new(ArcVecWriter { inner: Arc::clone(&out) }), &builtins()).unwrap();
        let buf = out.lock().unwrap();
        assert_eq!(String::from_utf8_lossy(&buf).trim(), "1\n3\n5\n7\n9");
    }
    #[test]
    fn test_cd_and_ls() {
        let dir = get_temp_dir();
        // Create files inside the temp dir
        std::fs::write(dir.path().join("aaa.txt"), "hello").unwrap();
        std::fs::write(dir.path().join("bbb.txt"), "world").unwrap();

        let original_cwd = env::current_dir().unwrap();
        let reg = builtins();

        // cd into the temp directory
        let cd_cmd = format!("cd \"{}\"", dir.path().display());
        handle_pipeline(&cd_cmd, Box::new(io::empty()), Box::new(io::sink()), &reg).unwrap();

        // ls the current directory (should now be the temp dir)
        let out = Arc::new(Mutex::new(Vec::new()));
        handle_pipeline("ls", Box::new(io::empty()), Box::new(ArcVecWriter { inner: Arc::clone(&out) }), &reg).unwrap();

        let buf = out.lock().unwrap();
        let output = String::from_utf8_lossy(&buf);
        assert!(output.contains("aaa.txt"), "expected aaa.txt in ls output, got: {}", output);
        assert!(output.contains("bbb.txt"), "expected bbb.txt in ls output, got: {}", output);

        // Restore original cwd
        env::set_current_dir(original_cwd).unwrap();
    }
}
