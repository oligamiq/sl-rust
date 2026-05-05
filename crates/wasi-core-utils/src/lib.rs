pub mod utils;
#[cfg(test)]
mod tests;

use std::ffi::OsString;
use std::path::Path;

pub fn execute<I, T>(args: I) -> Result<(), String>
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
        "arch" => utils::arch::execute(util_args),
        #[cfg(feature = "cat")]
        "cat" => utils::cat::execute(util_args),
        #[cfg(feature = "cp")]
        "cp" => utils::cp::execute(util_args),
        #[cfg(feature = "dir")]
        "dir" => utils::dir::execute(util_args),
        #[cfg(feature = "echo")]
        "echo" => utils::echo::execute(util_args),
        #[cfg(feature = "link")]
        "link" => utils::link::execute(util_args),
        #[cfg(feature = "ln")]
        "ln" => utils::ln::execute(util_args),
        #[cfg(feature = "ls")]
        "ls" => utils::ls::execute(util_args),
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
        #[cfg(feature = "uname")]
        "uname" => utils::uname::execute(util_args),
        #[cfg(feature = "unlink")]
        "unlink" => utils::unlink::execute(util_args),
        _ => Err(format!("Unknown utility: {}", util_name)),
    }
}
