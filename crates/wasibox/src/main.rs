use std::env;
use std::path::Path;
use std::process;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.is_empty() {
        eprintln!("No utility specified");
        process::exit(1);
    }

    let prog_name_raw = Path::new(&args[0])
        .file_name()
        .unwrap_or_default()
        .to_string_lossy();
    let prog_name = prog_name_raw
        .trim_end_matches(".exe")
        .trim_end_matches(".wasm");

    let (util_name, util_args) = if prog_name == "wasibox" || prog_name == "core" {
        if args.len() < 2 {
            eprintln!("Usage: wasibox <utility> [args...]");
            process::exit(1);
        }
        (args[1].clone(), &args[1..])
    } else {
        (prog_name.to_string(), &args[..])
    };

    let result: Result<(), String> = match util_name.as_str() {
        #[cfg(feature = "sl")]
        "sl" => {
            let _ = sl::run(util_args.iter().cloned());
            Ok(())
        }
        #[cfg(feature = "arch")]
        "arch" => wasi_core_utils::utils::arch::execute(util_args),
        #[cfg(feature = "cat")]
        "cat" => wasi_core_utils::utils::cat::execute(util_args),
        #[cfg(feature = "cp")]
        "cp" => wasi_core_utils::utils::cp::execute(util_args),
        #[cfg(feature = "dir")]
        "dir" => wasi_core_utils::utils::dir::execute(util_args),
        #[cfg(feature = "echo")]
        "echo" => wasi_core_utils::utils::echo::execute(util_args),
        #[cfg(feature = "link")]
        "link" => wasi_core_utils::utils::link::execute(util_args),
        #[cfg(feature = "ln")]
        "ln" => wasi_core_utils::utils::ln::execute(util_args),
        #[cfg(feature = "ls")]
        "ls" => wasi_core_utils::utils::ls::execute(util_args),
        #[cfg(feature = "mkdir")]
        "mkdir" => wasi_core_utils::utils::mkdir::execute(util_args),
        #[cfg(feature = "mv")]
        "mv" => wasi_core_utils::utils::mv::execute(util_args),
        #[cfg(feature = "pwd")]
        "pwd" => wasi_core_utils::utils::pwd::execute(util_args),
        #[cfg(feature = "rm")]
        "rm" => wasi_core_utils::utils::rm::execute(util_args),
        #[cfg(feature = "rmdir")]
        "rmdir" => wasi_core_utils::utils::rmdir::execute(util_args),
        #[cfg(feature = "sleep")]
        "sleep" => wasi_core_utils::utils::sleep::execute(util_args),
        #[cfg(feature = "tail")]
        "tail" => wasi_core_utils::utils::tail::execute(util_args),
        #[cfg(feature = "tee")]
        "tee" => wasi_core_utils::utils::tee::execute(util_args),
        #[cfg(feature = "touch")]
        "touch" => wasi_core_utils::utils::touch::execute(util_args),
        #[cfg(feature = "tree")]
        "tree" => wasi_core_utils::utils::tree::execute(util_args),
        #[cfg(feature = "uname")]
        "uname" => wasi_core_utils::utils::uname::execute(util_args),
        #[cfg(feature = "unlink")]
        "unlink" => wasi_core_utils::utils::unlink::execute(util_args),
        _ => {
            Err(format!("Unknown utility: {}", util_name))
        }
    };

    if let Err(e) = result {
        eprintln!("{}", e);
        process::exit(1);
    }
}
