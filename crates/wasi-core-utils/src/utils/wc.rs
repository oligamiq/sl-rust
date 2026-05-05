use clap::Parser;
use std::ffi::OsString;
use std::fs::File;
use std::io::{self, BufRead, BufReader};
use std::path::PathBuf;

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
    let args = Args::try_parse_from(args).map_err(|e| e.to_string())?;
    
    let mut total_lines = 0;
    let mut total_words = 0;
    let mut total_bytes = 0;

    let show_all = !args.lines && !args.words && !args.bytes;

    if args.files.is_empty() {
        let (l, w, c) = count_stream(io::stdin().lock())?;
        print_counts(l, w, c, "", &args, show_all);
    } else {
        for file_path in &args.files {
            let file = File::open(file_path).map_err(|e| format!("wc: {}: {}", file_path.display(), e))?;
            let (l, w, c) = count_stream(BufReader::new(file))?;
            print_counts(l, w, c, &file_path.to_string_lossy(), &args, show_all);
            total_lines += l;
            total_words += w;
            total_bytes += c;
        }
        if args.files.len() > 1 {
            print_counts(total_lines, total_words, total_bytes, "total", &args, show_all);
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

fn print_counts(l: usize, w: usize, c: usize, name: &str, args: &Args, all: bool) {
    if args.lines || all { print!("{:>8} ", l); }
    if args.words || all { print!("{:>8} ", w); }
    if args.bytes || all { print!("{:>8} ", c); }
    println!("{}", name);
}
