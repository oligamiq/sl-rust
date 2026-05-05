use clap::Parser;
use std::ffi::OsString;
use std::fs::File;
use std::io;
use crate::IoContext;

#[derive(Parser)]
#[command(name = "cat", about = "Concatenate and print files")]
struct Args {
    /// Files to concatenate (use - for stdin)
    #[arg(default_value = "-")]
    files: Vec<String>,
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

    for file in args.files {
        let result = if file == "-" {
            io::copy(&mut ctx.stdin, &mut ctx.stdout)
        } else {
            let mut f = File::open(&file).map_err(|e| format!("cat: {}: {}", file, e))?;
            io::copy(&mut f, &mut ctx.stdout)
        };
        match result {
            Ok(_) => {}
            Err(e) if e.kind() == io::ErrorKind::BrokenPipe => break,
            Err(e) => return Err(format!("cat: {}: {}", file, e)),
        }
    }
    Ok(())
}
