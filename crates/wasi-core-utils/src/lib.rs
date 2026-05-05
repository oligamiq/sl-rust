pub mod utils;
#[cfg(test)]
mod tests;

use std::ffi::OsString;
use std::path::Path;
use std::io::{Read, Write};

/// Context for utility IO, allowing redirection of stdin, stdout, and stderr.
pub struct IoContext {
    pub stdin: Box<dyn Read + Send>,
    pub stdout: Box<dyn Write + Send>,
    pub stderr: Box<dyn Write + Send>,
}

impl IoContext {
    pub fn new(
        stdin: Box<dyn Read + Send>,
        stdout: Box<dyn Write + Send>,
        stderr: Box<dyn Write + Send>,
    ) -> Self {
        Self { stdin, stdout, stderr }
    }
}

impl Default for IoContext {
    fn default() -> Self {
        Self {
            stdin: Box::new(std::io::stdin()),
            stdout: Box::new(std::io::stdout()),
            stderr: Box::new(std::io::stderr()),
        }
    }
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
    let args_vec: Vec<OsString> = args.into_iter().map(|a| a.into()).collect();
    if args_vec.is_empty() {
        return Err("No utility specified".to_string());
    }

    let prog_name_raw = Path::new(&args_vec[0])
        .file_name()
        .unwrap_or_default()
        .to_string_lossy();
    let prog_name = prog_name_raw
        .trim_end_matches(".exe")
        .trim_end_matches(".wasm");

    let (util_name, util_args) = if prog_name == "core" || prog_name == "core-rust" || prog_name == "wasi-core-utils" {
        if args_vec.len() < 2 {
            return Err("Usage: core <utility> [args...]".to_string());
        }
        (args_vec[1].to_string_lossy().to_string(), &args_vec[1..])
    } else {
        (prog_name.to_string(), &args_vec[..])
    };

    match util_name.as_str() {
        #[cfg(feature = "arch")]
        "arch" => utils::arch::execute_with_context(util_args, ctx),
        #[cfg(feature = "basename")]
        "basename" => utils::basename::execute(util_args),
        #[cfg(feature = "cat")]
        "cat" => utils::cat::execute_with_context(util_args, ctx),
        #[cfg(feature = "cp")]
        "cp" => utils::cp::execute(util_args),
        #[cfg(feature = "dir")]
        "dir" => utils::dir::execute(util_args),
        #[cfg(feature = "dirname")]
        "dirname" => utils::dirname::execute(util_args),
        #[cfg(feature = "echo")]
        "echo" => utils::echo::execute_with_context(util_args, ctx),
        #[cfg(feature = "env")]
        "env" => utils::env::execute(util_args),
        #[cfg(feature = "false")]
        "false" => utils::r#false::execute(util_args),
        #[cfg(feature = "grep")]
        "grep" => utils::grep::execute_with_context(util_args, ctx),
        #[cfg(feature = "head")]
        "head" => utils::head::execute(util_args),
        #[cfg(feature = "link")]
        "link" => utils::link::execute(util_args),
        #[cfg(feature = "ln")]
        "ln" => utils::ln::execute(util_args),
        #[cfg(feature = "ls")]
        "ls" => utils::ls::execute_with_context(util_args, ctx),
        #[cfg(feature = "mkdir")]
        "mkdir" => utils::mkdir::execute(util_args),
        #[cfg(feature = "mv")]
        "mv" => utils::mv::execute(util_args),
        #[cfg(feature = "pwd")]
        "pwd" => utils::pwd::execute(util_args),
        #[cfg(feature = "rm")]
        "rm" => utils::rm::execute(util_args),
        #[cfg(feature = "rmdir")]
        "rmdir" => utils::rmdir::execute(util_args),
        #[cfg(feature = "sleep")]
        "sleep" => utils::sleep::execute(util_args),
        #[cfg(feature = "tail")]
        "tail" => utils::tail::execute(util_args),
        #[cfg(feature = "tee")]
        "tee" => utils::tee::execute(util_args),
        #[cfg(feature = "touch")]
        "touch" => utils::touch::execute(util_args),
        #[cfg(feature = "tree")]
        "tree" => utils::tree::execute(util_args),
        #[cfg(feature = "true")]
        "true" => utils::r#true::execute(util_args),
        #[cfg(feature = "uname")]
        "uname" => utils::uname::execute(util_args),
        #[cfg(feature = "unlink")]
        "unlink" => utils::unlink::execute(util_args),
        #[cfg(feature = "wc")]
        "wc" => utils::wc::execute_with_context(util_args, ctx),
        #[cfg(feature = "whoami")]
        "whoami" => utils::whoami::execute(util_args),
        #[cfg(feature = "yes")]
        "yes" => utils::yes::execute(util_args),
        _ => Err(format!("Unknown utility: {}", util_name)),
    }
}
