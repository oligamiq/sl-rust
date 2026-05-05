use clap::Parser;
use std::ffi::OsString;
use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;
use crate::IoContext;

#[derive(Parser)]
#[command(name = "head", about = "Output the first part of files")]
pub struct Args {
    /// Files to read
    pub files: Vec<PathBuf>,

    /// Print the first K lines instead of the first 10
    #[arg(short = 'n', long, default_value_t = 10)]
    pub lines: usize,
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

    if args.files.is_empty() {
        head_stream(BufReader::new(&mut ctx.stdin), &mut ctx.stdout, args.lines)?;
    } else {
        for file_path in &args.files {
            if args.files.len() > 1 {
                writeln!(ctx.stdout, "==> {} <==", file_path.display()).map_err(|e| e.to_string())?;
            }
            let file = File::open(file_path).map_err(|e| format!("head: {}: {}", file_path.display(), e))?;
            head_stream(BufReader::new(file), &mut ctx.stdout, args.lines)?;
        }
    }
    Ok(())
}

fn head_stream<R: BufRead, W: Write>(reader: R, writer: &mut W, lines: usize) -> Result<(), String> {
    for (i, line) in reader.lines().enumerate() {
        if i >= lines {
            break;
        }
        let line = match line {
            Ok(l) => l,
            Err(e) if e.kind() == std::io::ErrorKind::BrokenPipe => break,
            Err(e) => return Err(e.to_string()),
        };
        if writeln!(writer, "{}", line).is_err() { break; }
    }
    Ok(())
}
