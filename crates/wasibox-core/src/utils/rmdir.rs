use clap::Parser;
use std::ffi::OsString;
use std::fs;

use crate::IoContext;

#[derive(Parser)]
#[command(name = "rmdir", about = "Remove empty directories")]
struct Args {
    /// Directories to remove
    #[arg(required = true)]
    dirs: Vec<String>,
}

pub fn execute<I, T>(args: I) -> Result<(), String>
where
    I: IntoIterator<Item = T>,
    T: Into<OsString> + Clone,
{
    execute_with_context(args, &mut IoContext::default())
}

pub fn execute_with_context<I, T>(args: I, _ctx: &mut IoContext) -> Result<(), String>
where
    I: IntoIterator<Item = T>,
    T: Into<OsString> + Clone,
{
    let args = Args::try_parse_from(args).map_err(|e| e.to_string())?;
    for dir in args.dirs {
        fs::remove_dir(&dir).map_err(|e| format!("rmdir: {}: {}", dir, e))?;
    }
    Ok(())
}
