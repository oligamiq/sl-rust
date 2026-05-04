pub mod utils;

use std::ffi::OsString;

pub fn execute<I, T>(args: I) -> Result<(), String>
where
    I: IntoIterator<Item = T>,
    T: Into<OsString> + Clone,
{
    let args_vec: Vec<OsString> = args.into_iter().map(|a| a.into()).collect();
    if args_vec.is_empty() {
        return Err("No utility specified".to_string());
    }

    // Determine the utility name. 
    // It can be the first argument if run as 'core <util> ...' 
    // or the program name if run as '<util> ...'.
    let prog_name_raw = Path::new(&args_vec[0])
        .file_name()
        .unwrap_or_default()
        .to_string_lossy();
    let prog_name = prog_name_raw.trim_end_matches(".exe");

    let (util_name, util_args) = if prog_name == "core" || prog_name == "core-rust" {
        if args_vec.len() < 2 {
            return Err("Usage: core <utility> [args...]".to_string());
        }
        (args_vec[1].to_string_lossy().to_string(), &args_vec[1..])
    } else {
        (prog_name.to_string(), &args_vec[..])
    };

    match util_name.as_str() {
        "arch" => utils::arch::execute(util_args),
        "cat" => utils::cat::execute(util_args),
        "cp" => utils::cp::execute(util_args),
        "dir" => utils::dir::execute(util_args),
        "echo" => utils::echo::execute(util_args),
        "link" => utils::link::execute(util_args),
        "ln" => utils::ln::execute(util_args),
        "ls" => utils::ls::execute(util_args),
        "mkdir" => utils::mkdir::execute(util_args),
        "mv" => utils::mv::execute(util_args),
        "pwd" => utils::pwd::execute(util_args),
        "rm" => utils::rm::execute(util_args),
        "rmdir" => utils::rmdir::execute(util_args),
        "sleep" => utils::sleep::execute(util_args),
        "tail" => utils::tail::execute(util_args),
        "tee" => utils::tee::execute(util_args),
        "touch" => utils::touch::execute(util_args),
        "tree" => utils::tree::execute(util_args),
        "uname" => utils::uname::execute(util_args),
        "unlink" => utils::unlink::execute(util_args),
        _ => Err(format!("Unknown utility: {}", util_name)),
    }
}

use std::path::Path;
