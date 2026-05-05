use clap::Parser;
use colored::*;
use std::ffi::OsString;
use std::fs;
use std::path::{Path, PathBuf};
use chrono::{DateTime, Local};
use crate::IoContext;
use std::io::Write;

#[derive(Parser, Debug)]
#[command(name = "ls", about = "List directory contents")]
pub struct Args {
    /// Directory to list
    #[arg(default_value = ".")]
    pub paths: Vec<PathBuf>,

    /// List all entries (including hidden and . ..)
    #[arg(short = 'a', long)]
    pub all: bool,

    /// Use a long listing format
    #[arg(short = 'l', long)]
    pub long: bool,

    /// List directories recursively
    #[arg(short = 'R', long)]
    pub recursive: bool,
}

pub fn execute<I, T>(args: I) -> Result<(), String>
where
    I: IntoIterator<Item = T>,
    T: Into<OsString> + Clone,
{
    execute_with_context(args, &mut IoContext::default())
}

pub fn execute_with_context<I, T>(args: I, ctx: &mut IoContext) -> Result<(), String>
where
    I: IntoIterator<Item = T>,
    T: Into<OsString> + Clone,
{
    let args = Args::try_parse_from(args).map_err(|e| e.to_string())?;
    
    let paths = if args.paths.is_empty() {
        vec![PathBuf::from(".")]
    } else {
        args.paths.clone()
    };

    for (i, path) in paths.iter().enumerate() {
        if paths.len() > 1 {
            writeln!(ctx.stdout, "{}:", path.display()).map_err(|e| e.to_string())?;
        }
        render_ls(path, &args, ctx)?;
        if i < paths.len() - 1 {
            writeln!(ctx.stdout).map_err(|e| e.to_string())?;
        }
    }
    Ok(())
}

struct EntryInfo {
    name: String,
    metadata: fs::Metadata,
    is_dir: bool,
}

fn render_ls(path: &Path, args: &Args, ctx: &mut IoContext) -> Result<(), String> {
    let mut entries_vec = Vec::new();

    if args.all {
        if let Ok(meta) = fs::metadata(path) {
            entries_vec.push(EntryInfo {
                name: ".".to_string(),
                metadata: meta,
                is_dir: true,
            });
        }
        let parent = if path.as_os_str() == "." {
            Path::new("..")
        } else {
            path.parent().unwrap_or(path)
        };
        if let Ok(meta) = fs::metadata(parent) {
            entries_vec.push(EntryInfo {
                name: "..".to_string(),
                metadata: meta,
                is_dir: true,
            });
        }
    }

    let read_entries = match fs::read_dir(path) {
        Ok(e) => e,
        Err(e) => return Err(format!("ls: cannot access '{}': {}", path.display(), e)),
    };

    for entry in read_entries.filter_map(|e| e.ok()) {
        let name = entry.file_name().to_string_lossy().into_owned();
        if !args.all && name.starts_with('.') {
            continue;
        }
        let metadata = entry.metadata().map_err(|e| e.to_string())?;
        entries_vec.push(EntryInfo {
            is_dir: metadata.is_dir(),
            name,
            metadata,
        });
    }

    entries_vec.sort_by(|a, b| a.name.cmp(&b.name));

    if args.long {
        let total_blocks = entries_vec.iter().map(|e| {
            #[cfg(unix)]
            {
                use std::os::unix::fs::MetadataExt;
                e.metadata.blocks() / 2
            }
            #[cfg(not(unix))]
            {
                (e.metadata.len() + 1023) / 1024
            }
        }).sum::<u64>();
        writeln!(ctx.stdout, "total {}", total_blocks).map_err(|e| e.to_string())?;

        for entry in &entries_vec {
            print_long(entry, &mut ctx.stdout)?;
        }
    } else {
        for entry in &entries_vec {
            if entry.is_dir {
                write!(ctx.stdout, "{}  ", entry.name.blue().bold()).map_err(|e| e.to_string())?;
            } else {
                write!(ctx.stdout, "{}  ", entry.name.normal()).map_err(|e| e.to_string())?;
            }
        }
        writeln!(ctx.stdout).map_err(|e| e.to_string())?;
    }

    if args.recursive {
        for entry in entries_vec {
            if entry.is_dir && entry.name != "." && entry.name != ".." {
                let mut new_path = PathBuf::from(path);
                new_path.push(&entry.name);
                writeln!(ctx.stdout, "\n{}:", new_path.display()).map_err(|e| e.to_string())?;
                render_ls(&new_path, args, ctx)?;
            }
        }
    }

    Ok(())
}

fn print_long(entry: &EntryInfo, out: &mut Box<dyn Write + Send>) -> Result<(), String> {
    let mode = get_mode_string(&entry.metadata);
    let nlink = get_nlink(&entry.metadata);
    let owner = get_owner(&entry.metadata);
    let group = get_group(&entry.metadata);
    let size = entry.metadata.len();
    let time = get_time_string(&entry.metadata);
    let name = if entry.is_dir {
        entry.name.blue().bold()
    } else {
        entry.name.normal()
    };

    writeln!(
        out,
        "{} {:>3} {:<8} {:<8} {:>8} {} {}",
        mode, nlink, owner, group, size, time, name
    ).map_err(|e| e.to_string())
}

fn get_mode_string(meta: &fs::Metadata) -> String {
    let mut s = String::with_capacity(10);
    s.push(if meta.is_dir() { 'd' } else if meta.is_symlink() { 'l' } else { '-' });
    
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mode = meta.permissions().mode();
        let chars = ['-', 'x', 'w', 'x', 'r', 'x', 'w', 'x', 'r'];
        for i in (0..9).rev() {
            if mode & (1 << i) != 0 {
                s.push(match i % 3 {
                    0 => 'x',
                    1 => 'w',
                    2 => 'r',
                    _ => unreachable!(),
                });
            } else {
                s.push('-');
            }
        }
    }
    #[cfg(not(unix))]
    {
        if meta.permissions().readonly() {
            s.push_str("r--r--r--");
        } else {
            s.push_str("rw-r--r--");
        }
    }
    s
}

fn get_nlink(_meta: &fs::Metadata) -> u64 {
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        _meta.nlink()
    }
    #[cfg(not(unix))]
    {
        1
    }
}

fn get_owner(_meta: &fs::Metadata) -> String {
    "user".to_string() 
}

fn get_group(_meta: &fs::Metadata) -> String {
    "user".to_string() 
}

fn get_time_string(meta: &fs::Metadata) -> String {
    if let Ok(mtime) = meta.modified() {
        let dt: DateTime<Local> = mtime.into();
        dt.format("%b %d %H:%M").to_string()
    } else {
        "Jan 01 00:00".to_string()
    }
}
