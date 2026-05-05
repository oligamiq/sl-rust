use clap::Parser;
use std::ffi::OsString;
use std::fs;

use crate::IoContext;

#[derive(Parser)]
#[command(name = "link", about = "Create a link to a file")]
struct Args {
    /// Target file
    #[arg(required = true)]
    target: String,

    /// Link name
    #[arg(required = true)]
    link: String,
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
    fs::hard_link(&args.target, &args.link)
        .map_err(|e| format!("link: cannot create link {} to {}: {}", args.link, args.target, e))?;
    Ok(())
}
