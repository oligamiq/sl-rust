use clap::Parser;
use std::ffi::OsString;
use std::thread;
use std::time::Duration;

#[derive(Parser)]
#[command(name = "sleep", about = "Delay for a specified amount of time")]
struct Args {
    /// Number of seconds to sleep
    #[arg(required = true)]
    seconds: f64,
}

pub fn execute<I, T>(args: I) -> Result<(), String>
where
    I: IntoIterator<Item = T>,
    T: Into<OsString> + Clone,
{
    let args = Args::try_parse_from(args).map_err(|e| e.to_string())?;
    let duration = Duration::from_secs_f64(args.seconds);
    thread::sleep(duration);
    Ok(())
}
