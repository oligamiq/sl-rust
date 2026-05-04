use clap::Parser;
use std::ffi::OsString;

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
    let args = Args::try_parse_from(args).map_err(|e| e.to_string())?;
    let output = args.text.join(" ");
    if args.no_newline {
        print!("{}", output);
    } else {
        println!("{}", output);
    }
    Ok(())
}
