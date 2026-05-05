use clap::Parser;
use std::ffi::OsString;
use std::fs;

#[derive(Parser)]
#[command(name = "ln", about = "Create links")]
struct Args {
    /// Target file
    #[arg(required = true)]
    target: String,

    /// Link name
    #[arg(required = true)]
    link: String,

    /// Create symbolic link instead of hard link
    #[arg(short = 's', long)]
    symbolic: bool,
}

pub fn execute<I, T>(args: I) -> Result<(), String>
where
    I: IntoIterator<Item = T>,
    T: Into<OsString> + Clone,
{
    let args = Args::try_parse_from(args).map_err(|e| e.to_string())?;
    if args.symbolic {
        #[cfg(windows)]
        {
            // On Windows, symbolic links require special privileges or Dev Mode
            // and separate functions for files and directories.
            let path = std::path::Path::new(&args.target);
            if path.is_dir() {
                std::os::windows::fs::symlink_dir(&args.target, &args.link)
                    .map_err(|e| format!("ln: symlink_dir: {}", e))?;
            } else {
                std::os::windows::fs::symlink_file(&args.target, &args.link)
                    .map_err(|e| format!("ln: symlink_file: {}", e))?;
            }
        }
        #[cfg(unix)]
        {
            std::os::unix::fs::symlink(&args.target, &args.link)
                .map_err(|e| format!("ln: symlink: {}", e))?;
        }
        #[cfg(target_os = "wasi")]
        {
             // WASI support for symlinks depends on the host
             return Err("ln -s is not fully supported on standard WASIp1".to_string());
        }
    } else {
        fs::hard_link(&args.target, &args.link)
            .map_err(|e| format!("ln: hard_link: {}", e))?;
    }
    Ok(())
}
