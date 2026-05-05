use clap::Parser;
use std::ffi::OsString;
use std::fs;

use crate::IoContext;

#[derive(Parser)]
#[command(name = "mkdir", about = "Create directories")]
struct Args {
    /// Directories to create
    #[arg(required = true)]
    dirs: Vec<String>,

    /// Create parent directories as needed
    #[arg(short = 'p', long)]
    parents: bool,
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
        if args.parents {
            fs::create_dir_all(&dir).map_err(|e| format!("mkdir: {}: {}", dir, e))?;
        } else {
            fs::create_dir(&dir).map_err(|e| format!("mkdir: {}: {}", dir, e))?;
        }
    }
    Ok(())
}
