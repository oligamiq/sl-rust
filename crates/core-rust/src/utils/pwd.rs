use clap::Parser;
use std::env;
use std::ffi::OsString;

#[derive(Parser)]
#[command(name = "pwd", about = "Print working directory")]
struct Args {}

pub fn execute<I, T>(args: I) -> Result<(), String>
where
    I: IntoIterator<Item = T>,
    T: Into<OsString> + Clone,
{
    let _ = Args::try_parse_from(args).map_err(|e| e.to_string())?;
    let path = env::current_dir().map_err(|e| format!("pwd: {}", e))?;
    println!("{}", path.display());
    Ok(())
}
