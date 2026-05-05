use clap::Parser;
use std::ffi::OsString;
use std::io::Write;
use crate::IoContext;

#[derive(Parser)]
#[command(name = "arch", about = "Print machine architecture")]
pub struct Args {}

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
    let _ = Args::try_parse_from(_args).map_err(|e| e.to_string())?;
    writeln!(ctx.stdout, "{}", get_arch()).map_err(|e| e.to_string())
}

pub fn get_arch() -> &'static str {
    #[cfg(target_os = "wasi")]
    { "wasm32" }
    #[cfg(all(not(target_os = "wasi"), target_arch = "x86_64"))]
    { "x86_64" }
    #[cfg(all(not(target_os = "wasi"), target_arch = "aarch64"))]
    { "aarch64" }
    #[cfg(all(not(target_os = "wasi"), not(target_arch = "x86_64"), not(target_arch = "aarch64")))]
    { "unknown" }
}
