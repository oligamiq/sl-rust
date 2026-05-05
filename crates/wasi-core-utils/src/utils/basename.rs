use clap::Parser;
use std::ffi::OsString;
use std::path::Path;

#[derive(Parser)]
#[command(name = "basename", about = "Strip directory and suffix from filenames")]
pub struct Args {
    /// Filename
    pub name: String,

    /// Suffix to remove
    pub suffix: Option<String>,
}

pub fn execute<I, T>(args: I) -> Result<(), String>
where
    I: IntoIterator<Item = T>,
    T: Into<OsString> + Clone,
{
    let args = Args::try_parse_from(args).map_err(|e| e.to_string())?;
    let path = Path::new(&args.name);
    let mut base = path.file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_else(|| "".to_string());

    if let Some(suffix) = args.suffix {
        if base.ends_with(&suffix) && base != suffix {
            base = base.trim_end_matches(&suffix).to_string();
        }
    }

    println!("{}", base);
    Ok(())
}
