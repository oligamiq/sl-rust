use clap::Parser;
use std::ffi::OsString;
use std::fs::OpenOptions;

#[derive(Parser)]
#[command(name = "touch", about = "Update timestamps or create empty files")]
struct Args {
    /// Files to update/create
    #[arg(required = true)]
    files: Vec<String>,
}

pub fn execute<I, T>(args: I) -> Result<(), String>
where
    I: IntoIterator<Item = T>,
    T: Into<OsString> + Clone,
{
    let args = Args::try_parse_from(args).map_err(|e| e.to_string())?;
    for file in args.files {
        // Simple touch: open with create/append or just open if exists
        OpenOptions::new()
            .create(true)
            .append(true)
            .open(&file)
            .map_err(|e| format!("touch: {}: {}", file, e))?;
        
        // Note: For full GNU touch, we should update atime/mtime using filetime crate
        // but for a minimal implementation, just opening for append works for creating.
    }
    Ok(())
}
