use clap::Parser;
use std::ffi::OsString;
use std::io::Write;
use crate::IoContext;

#[derive(Parser)]
#[command(name = "tee", about = "Read from standard input and write to standard output and files")]
struct Args {
    /// Files to write to
    files: Vec<String>,

    /// Append to the given files, do not overwrite
    #[arg(short = 'a', long)]
    append: bool,
}

pub fn execute<I, T>(args: I) -> Result<(), String>
where
    I: IntoIterator<Item = T>,
    T: Into<OsString> + Clone,
{
    execute_with_context(args, &mut IoContext::default())
}

pub fn execute_with_context<I, T>(args: I, ctx: &mut IoContext) -> Result<(), String>
where
    I: IntoIterator<Item = T>,
    T: Into<OsString> + Clone,
{
    let args = Args::try_parse_from(args).map_err(|e| e.to_string())?;
    let mut files = Vec::new();
    for path in args.files {
        let f = std::fs::OpenOptions::new()
            .create(true)
            .write(true)
            .append(args.append)
            .truncate(!args.append)
            .open(&path)
            .map_err(|e| format!("tee: {}: {}", path, e))?;
        files.push(f);
    }

    let mut buffer = [0; 8192];
    loop {
        let bytes_read = match ctx.stdin.read(&mut buffer) {
            Ok(0) => break,
            Ok(n) => n,
            Err(e) if e.kind() == std::io::ErrorKind::BrokenPipe => break,
            Err(e) => return Err(e.to_string()),
        };
        if ctx.stdout.write_all(&buffer[..bytes_read]).is_err() { break; }
        for f in &mut files {
            let _ = f.write_all(&buffer[..bytes_read]);
        }
    }
    Ok(())
}
