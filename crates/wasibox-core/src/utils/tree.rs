use clap::Parser;
use colored::*;
use std::ffi::OsString;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Parser, Debug)]
#[command(name = "tree", version, about = "Recursive directory listing.", long_about = None)]
pub struct Args {
    /// Directory to list
    #[arg(default_value = ".")]
    pub path: PathBuf,

    /// List all files (including hidden)
    #[arg(short = 'a', long)]
    pub all: bool,

    /// List directories only
    #[arg(short = 'd', long)]
    pub dirs_only: bool,

    /// Max display depth of the directory tree
    #[arg(short = 'L', long)]
    pub level: Option<usize>,

    /// Do not output color
    #[arg(short = 'n', long)]
    pub no_color: bool,
}

pub struct TreeStats {
    pub directories: usize,
    pub files: usize,
}

pub fn execute<I, T>(args: I) -> Result<(), String>
where
    I: IntoIterator<Item = T>,
    T: Into<OsString> + Clone,
{
    let args = Args::try_parse_from(args).map_err(|e| e.to_string())?;

    if args.no_color {
        control::set_override(false);
    }

    let root = &args.path;
    println!("{}", root.display().to_string().blue().bold());

    let mut stats = TreeStats {
        directories: 0,
        files: 0,
    };

    render_recursive(root, "", &args, 0, &mut stats).map_err(|e| e.to_string())?;

    println!(
        "\n{} directories, {} files",
        stats.directories, stats.files
    );

    Ok(())
}

fn render_recursive(
    path: &Path,
    prefix: &str,
    args: &Args,
    depth: usize,
    stats: &mut TreeStats,
) -> Result<(), std::io::Error> {
    if let Some(max_depth) = args.level {
        if depth >= max_depth {
            return Ok(());
        }
    }

    let entries = match fs::read_dir(path) {
        Ok(e) => e,
        Err(_) => return Ok(()),
    };

    let mut valid_entries: Vec<_> = entries
        .filter_map(|e| e.ok())
        .filter(|e| {
            let name = e.file_name().to_string_lossy().into_owned();
            if !args.all && name.starts_with('.') {
                return false;
            }
            if args.dirs_only && !e.path().is_dir() {
                return false;
            }
            true
        })
        .collect();

    valid_entries.sort_by_key(|e| e.file_name());

    let count = valid_entries.len();
    for (i, entry) in valid_entries.iter().enumerate() {
        let is_last = i == count - 1;
        let path = entry.path();
        let metadata = entry.metadata()?;
        let is_dir = metadata.is_dir();
        let name = entry.file_name().to_string_lossy().into_owned();

        let branch = if is_last { "└── " } else { "├── " };
        let display_name = if is_dir {
            name.blue().bold()
        } else {
            name.normal()
        };

        println!("{}{}{}", prefix, branch, display_name);

        if is_dir {
            stats.directories += 1;
            let new_prefix = format!("{}{}", prefix, if is_last { "    " } else { "│   " });
            render_recursive(&path, &new_prefix, args, depth + 1, stats)?;
        } else {
            stats.files += 1;
        }
    }

    Ok(())
}
