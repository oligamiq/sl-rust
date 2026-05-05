use clap::Parser;
use std::ffi::OsString;
use std::fs;

use crate::IoContext;

#[derive(Parser)]
#[command(name = "rm", about = "Remove files or directories")]
struct Args {
    /// Files/directories to remove
    #[arg(required = true)]
    files: Vec<String>,

    /// Remove directories and their contents recursively
    #[arg(short = 'r', short_alias = 'R', long)]
    recursive: bool,

    /// Force removal (ignore non-existent files)
    #[arg(short = 'f', long)]
    force: bool,
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
    for file in args.files {
        let path = std::path::Path::new(&file);
        if !path.exists() {
            if args.force {
                continue;
            } else {
                return Err(format!("rm: {}: No such file or directory", file));
            }
        }

        if path.is_dir() {
            if args.recursive {
                fs::remove_dir_all(path).map_err(|e| format!("rm: {}: {}", file, e))?;
            } else {
                return Err(format!("rm: {}: is a directory", file));
            }
        } else {
            fs::remove_file(path).map_err(|e| format!("rm: {}: {}", file, e))?;
        }
    }
    Ok(())
}
