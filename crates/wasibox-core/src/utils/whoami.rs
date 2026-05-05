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

pub fn execute_with_context<I, T>(_args: I, ctx: &mut IoContext) -> Result<(), String>
where
    I: IntoIterator<Item = T>,
    T: Into<OsString> + Clone,
{
    // On WASI/Windows, getting actual user can be tricky.
    // Try environment variables first.
    let user = env::var("USER")
        .or_else(|_| env::var("USERNAME"))
        .unwrap_or_else(|_| "wasi-user".to_string());
    
    writeln!(ctx.stdout, "{}", user).map_err(|e| e.to_string())?;
    Ok(())
}
