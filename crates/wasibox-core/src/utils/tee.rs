use clap::Parser;
use std::ffi::OsString;
use std::io::{self, Read, Write};

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
        let bytes_read = io::stdin().read(&mut buffer).map_err(|e| e.to_string())?;
        if bytes_read == 0 {
            break;
        }
        io::stdout().write_all(&buffer[..bytes_read]).map_err(|e| e.to_string())?;
        for f in &mut files {
            f.write_all(&buffer[..bytes_read]).map_err(|e| e.to_string())?;
        }
    }
    Ok(())
}
