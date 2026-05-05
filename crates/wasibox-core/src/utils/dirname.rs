use clap::Parser;
use std::ffi::OsString;
use std::path::Path;

use std::io::Write;
use crate::IoContext;

#[derive(Parser)]
#[command(name = "dirname", about = "Strip last component from file name")]
pub struct Args {
    /// Filename
    pub name: String,
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
    let path = Path::new(&args.name);
    let dir = path.parent()
        .map(|p| {
            let s = p.to_string_lossy();
            if s.is_empty() { ".".to_string() } else { s.into_owned() }
        })
        .unwrap_or_else(|| ".".to_string());

    writeln!(ctx.stdout, "{}", dir).map_err(|e| e.to_string())?;
    Ok(())
}
