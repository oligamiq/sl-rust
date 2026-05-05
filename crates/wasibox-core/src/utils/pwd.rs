use clap::Parser;
use std::env;
use std::ffi::OsString;

use std::io::Write;
use crate::IoContext;

#[derive(Parser)]
#[command(name = "pwd", about = "Print working directory")]
struct Args {}

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
    let _ = Args::try_parse_from(args).map_err(|e| e.to_string())?;
    let path = env::current_dir().map_err(|e| format!("pwd: {}", e))?;
    writeln!(ctx.stdout, "{}", path.display()).map_err(|e| e.to_string())?;
    Ok(())
}
