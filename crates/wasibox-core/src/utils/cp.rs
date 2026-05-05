use clap::Parser;
use std::ffi::OsString;
use std::fs;

use crate::IoContext;

#[derive(Parser)]
#[command(name = "cp", about = "Copy files and directories")]
struct Args {
    /// Source file(s)
    #[arg(required = true)]
    source: Vec<String>,

    /// Destination directory or file
    #[arg(required = true)]
    dest: String,

    /// Copy directories recursively
    #[arg(short = 'r', short_alias = 'R', long)]
    recursive: bool,
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

    if args.source.len() > 1 {
        let dest_path = std::path::Path::new(&args.dest);
        if !dest_path.is_dir() {
            return Err(format!("cp: target {} is not a directory", args.dest));
        }
    }

    for src in args.source {
        let src_path = std::path::Path::new(&src);
        let mut dest = std::path::PathBuf::from(&args.dest);
        if dest.is_dir() {
            if let Some(name) = src_path.file_name() {
                dest.push(name);
            }
        }

        if src_path.is_dir() {
            if args.recursive {
                copy_dir_recursive(src_path, &dest)?;
            } else {
                return Err(format!("cp: -r not specified; omitting directory {}", src));
            }
        } else {
            fs::copy(src_path, &dest).map_err(|e| format!("cp: {}: {}", src, e))?;
        }
    }
    Ok(())
}

fn copy_dir_recursive(src: &std::path::Path, dest: &std::path::Path) -> Result<(), String> {
    if !dest.exists() {
        fs::create_dir(dest).map_err(|e| format!("cp: mkdir {}: {}", dest.display(), e))?;
    }
    for entry in fs::read_dir(src).map_err(|e| format!("cp: readdir {}: {}", src.display(), e))? {
        let entry = entry.map_err(|e| e.to_string())?;
        let path = entry.path();
        let mut new_dest = std::path::PathBuf::from(dest);
        new_dest.push(entry.file_name());
        if path.is_dir() {
            copy_dir_recursive(&path, &new_dest)?;
        } else {
            fs::copy(&path, &new_dest).map_err(|e| format!("cp: {}: {}", path.display(), e))?;
        }
    }
    Ok(())
}
