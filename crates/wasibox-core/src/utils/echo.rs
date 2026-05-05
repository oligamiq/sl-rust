use clap::Parser;
use std::ffi::OsString;
use std::io::Write;
use crate::IoContext;

#[derive(Parser)]
#[command(name = "echo", about = "Display a line of text")]
struct Args {
    /// String to print
    #[arg(trailing_var_arg = true)]
    text: Vec<String>,

    /// Do not output the trailing newline
    #[arg(short = 'n')]
    no_newline: bool,
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
    let output = args.text.join(" ");
    if args.no_newline {
        write!(ctx.stdout, "{}", output).map_err(|e| e.to_string())?;
    } else {
        writeln!(ctx.stdout, "{}", output).map_err(|e| e.to_string())?;
    }
    ctx.stdout.flush().map_err(|e| e.to_string())?;
    Ok(())
}
