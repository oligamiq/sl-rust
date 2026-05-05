use clap::Parser;
use std::ffi::OsString;
use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;
use crate::IoContext;

#[derive(Parser)]
#[command(name = "wc", about = "Print newline, word, and byte counts for each file")]
pub struct Args {
    /// Files to read
    pub files: Vec<PathBuf>,

    /// Print the newline counts
    #[arg(short = 'l', long)]
    pub lines: bool,

    /// Print the word counts
    #[arg(short = 'w', long)]
    pub words: bool,

    /// Print the byte counts
    #[arg(short = 'c', long)]
    pub bytes: bool,
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
    
    let mut total_lines = 0;
    let mut total_words = 0;
    let mut total_bytes = 0;

    let show_all = !args.lines && !args.words && !args.bytes;

    if args.files.is_empty() {
        let (l, w, c) = count_stream(BufReader::new(&mut ctx.stdin))?;
        print_counts(l, w, c, "", &args, show_all, &mut ctx.stdout)?;
    } else {
        for file_path in &args.files {
            let file = File::open(file_path).map_err(|e| format!("wc: {}: {}", file_path.display(), e))?;
            let (l, w, c) = count_stream(BufReader::new(file))?;
            print_counts(l, w, c, &file_path.to_string_lossy(), &args, show_all, &mut ctx.stdout)?;
            total_lines += l;
            total_words += w;
            total_bytes += c;
        }
        if args.files.len() > 1 {
            print_counts(total_lines, total_words, total_bytes, "total", &args, show_all, &mut ctx.stdout)?;
        }
    }
    Ok(())
}

fn count_stream<R: BufRead>(mut reader: R) -> Result<(usize, usize, usize), String> {
    let mut lines = 0;
    let mut words = 0;
    let mut bytes = 0;
    let mut buf = String::new();

    loop {
        buf.clear();
        let n = reader.read_line(&mut buf).map_err(|e| e.to_string())?;
        if n == 0 {
            break;
        }
        bytes += n;
        lines += 1;
        words += buf.split_whitespace().count();
    }
    Ok((lines, words, bytes))
}

fn print_counts<W: Write>(l: usize, w: usize, c: usize, name: &str, args: &Args, all: bool, mut out: W) -> Result<(), String> {
    if args.lines || all { write!(out, "{:>8} ", l).map_err(|e| e.to_string())?; }
    if args.words || all { write!(out, "{:>8} ", w).map_err(|e| e.to_string())?; }
    if args.bytes || all { write!(out, "{:>8} ", c).map_err(|e| e.to_string())?; }
    writeln!(out, "{}", name).map_err(|e| e.to_string())?;
    Ok(())
}
