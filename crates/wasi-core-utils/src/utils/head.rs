use clap::Parser;
use std::ffi::OsString;
use std::fs::File;
use std::io::{self, BufRead, BufReader};
use std::path::PathBuf;

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
    let args = Args::try_parse_from(args).map_err(|e| e.to_string())?;

    if args.files.is_empty() {
        head_stream(io::stdin().lock(), args.lines)?;
    } else {
        for file_path in &args.files {
            if args.files.len() > 1 {
                println!("==> {} <==", file_path.display());
            }
            let file = File::open(file_path).map_err(|e| format!("head: {}: {}", file_path.display(), e))?;
            head_stream(BufReader::new(file), args.lines)?;
        }
    }
    Ok(())
}

fn head_stream<R: BufRead>(reader: R, lines: usize) -> Result<(), String> {
    for (i, line) in reader.lines().enumerate() {
        if i >= lines {
            break;
        }
        let line = line.map_err(|e| e.to_string())?;
        println!("{}", line);
    }
    Ok(())
}
