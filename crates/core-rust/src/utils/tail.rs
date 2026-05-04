use clap::Parser;
use std::ffi::OsString;
use std::fs::File;
use std::io::{BufRead, BufReader};

#[derive(Parser)]
#[command(name = "tail", about = "Output the last part of files")]
struct Args {
    /// File to output
    #[arg(required = true)]
    file: String,

    /// Number of lines to output
    #[arg(short = 'n', long, default_value = "10")]
    lines: usize,
}

pub fn execute<I, T>(args: I) -> Result<(), String>
where
    I: IntoIterator<Item = T>,
    T: Into<OsString> + Clone,
{
    let args = Args::try_parse_from(args).map_err(|e| e.to_string())?;
    let f = File::open(&args.file).map_err(|e| format!("tail: {}: {}", args.file, e))?;
    let reader = BufReader::new(f);
    
    // Minimal implementation: read all lines and keep last N
    // For large files, we should seek from the end.
    let lines: Vec<String> = reader.lines().filter_map(|l| l.ok()).collect();
    let start = if lines.len() > args.lines {
        lines.len() - args.lines
    } else {
        0
    };

    for line in &lines[start..] {
        println!("{}", line);
    }
    Ok(())
}
