use clap::Parser;
use std::ffi::OsString;
use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use crate::IoContext;

#[derive(Parser)]
#[command(name = "tail", about = "Output the last part of files")]
struct Args {
    /// Files to output
    pub files: Vec<String>,

    /// Number of lines to output
    #[arg(short = 'n', long, default_value = "10")]
    lines: usize,
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
        tail_stream(BufReader::new(&mut ctx.stdin), &mut ctx.stdout, args.lines)?;
    } else {
        for file_path in &args.files {
            if args.files.len() > 1 {
                writeln!(ctx.stdout, "==> {} <==", file_path).map_err(|e| e.to_string())?;
            }
            let f = File::open(file_path).map_err(|e| format!("tail: {}: {}", file_path, e))?;
            tail_stream(BufReader::new(f), &mut ctx.stdout, args.lines)?;
        }
    }
    Ok(())
}

fn tail_stream<R: BufRead, W: Write>(reader: R, writer: &mut W, lines_count: usize) -> Result<(), String> {
    // Minimal implementation: read all lines and keep last N
    let lines: Vec<String> = reader.lines().filter_map(|l| l.ok()).collect();
    let start = if lines.len() > lines_count {
        lines.len() - lines_count
    } else {
        0
    };

    for line in &lines[start..] {
        writeln!(writer, "{}", line).map_err(|e| e.to_string())?;
    }
    Ok(())
}
