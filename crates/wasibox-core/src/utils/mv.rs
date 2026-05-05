use clap::Parser;
use std::ffi::OsString;
use std::fs;

use crate::IoContext;

#[derive(Parser)]
#[command(name = "mv", about = "Move (rename) files")]
struct Args {
    /// Source file(s)
    #[arg(required = true)]
    source: Vec<String>,

    /// Destination directory or file
    #[arg(required = true)]
    dest: String,
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
    
    // If multiple sources, dest must be a directory
    if args.source.len() > 1 {
        let dest_path = std::path::Path::new(&args.dest);
        if !dest_path.is_dir() {
            return Err(format!("mv: target {} is not a directory", args.dest));
        }
    }

    for src in args.source {
        let mut dest = std::path::PathBuf::from(&args.dest);
        if dest.is_dir() {
            let src_path = std::path::Path::new(&src);
            if let Some(name) = src_path.file_name() {
                dest.push(name);
            }
        }
        fs::rename(&src, &dest).map_err(|e| format!("mv: cannot move {} to {}: {}", src, dest.display(), e))?;
    }
    Ok(())
}
