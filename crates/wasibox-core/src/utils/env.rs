use std::ffi::OsString;
use std::env;

use std::io::Write;
use crate::IoContext;

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
    let args_vec: Vec<OsString> = args.into_iter().map(|a| a.into()).collect();
    
    // For now, only support printing the environment
    if args_vec.len() <= 1 {
        for (key, value) in env::vars() {
            writeln!(ctx.stdout, "{}={}", key, value).map_err(|e| e.to_string())?;
        }
    } else {
        // Running a program in a modified environment is complex for a library function.
        // We could implement basic key=value setting here.
        return Err("env: command execution not yet supported".to_string());
    }
    Ok(())
}
