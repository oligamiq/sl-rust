use clap::Parser;
use std::ffi::OsString;
use std::fs::File;
use std::io;

#[derive(Parser)]
#[command(name = "cat", about = "Concatenate and print files")]
struct Args {
    /// Files to concatenate (use - for stdin)
    #[arg(default_value = "-")]
    files: Vec<String>,
}

pub fn execute<I, T>(args: I) -> Result<(), String>
where
    I: IntoIterator<Item = T>,
    T: Into<OsString> + Clone,
{
    let args = Args::try_parse_from(args).map_err(|e| e.to_string())?;
    let stdout = io::stdout();
    let mut handle = stdout.lock();

    for file in args.files {
        if file == "-" {
            io::copy(&mut io::stdin(), &mut handle).map_err(|e| e.to_string())?;
        } else {
            let mut f = File::open(&file).map_err(|e| format!("cat: {}: {}", file, e))?;
            io::copy(&mut f, &mut handle).map_err(|e| format!("cat: {}: {}", file, e))?;
        }
    }
    Ok(())
}
