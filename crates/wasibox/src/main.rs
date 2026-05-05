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
        "arch" => wasibox_core::utils::arch::execute(util_args),
        #[cfg(feature = "basename")]
        "basename" => wasibox_core::utils::basename::execute(util_args),
        #[cfg(feature = "cat")]
        "cat" => wasibox_core::utils::cat::execute(util_args),
        #[cfg(feature = "cp")]
        "cp" => wasibox_core::utils::cp::execute(util_args),
        #[cfg(feature = "dir")]
        "dir" => wasibox_core::utils::dir::execute(util_args),
        #[cfg(feature = "dirname")]
        "dirname" => wasibox_core::utils::dirname::execute(util_args),
        #[cfg(feature = "echo")]
        "echo" => wasibox_core::utils::echo::execute(util_args),
        #[cfg(feature = "env")]
        "env" => wasibox_core::utils::env::execute(util_args),
        #[cfg(feature = "false")]
        "false" => wasibox_core::utils::r#false::execute(util_args),
        #[cfg(feature = "grep")]
        "grep" => wasibox_core::utils::grep::execute(util_args),
        #[cfg(feature = "head")]
        "head" => wasibox_core::utils::head::execute(util_args),
        #[cfg(feature = "link")]
        "link" => wasibox_core::utils::link::execute(util_args),
        #[cfg(feature = "ln")]
        "ln" => wasibox_core::utils::ln::execute(util_args),
        #[cfg(feature = "ls")]
        "ls" => wasibox_core::utils::ls::execute(util_args),
        #[cfg(feature = "mkdir")]
        "mkdir" => wasibox_core::utils::mkdir::execute(util_args),
        #[cfg(feature = "mv")]
        "mv" => wasibox_core::utils::mv::execute(util_args),
        #[cfg(feature = "pwd")]
        "pwd" => wasibox_core::utils::pwd::execute(util_args),
        #[cfg(feature = "rm")]
        "rm" => wasibox_core::utils::rm::execute(util_args),
        #[cfg(feature = "rmdir")]
        "rmdir" => wasibox_core::utils::rmdir::execute(util_args),
        #[cfg(feature = "sleep")]
        "sleep" => wasibox_core::utils::sleep::execute(util_args),
        #[cfg(feature = "tail")]
        "tail" => wasibox_core::utils::tail::execute(util_args),
        #[cfg(feature = "tee")]
        "tee" => wasibox_core::utils::tee::execute(util_args),
        #[cfg(feature = "touch")]
        "touch" => wasibox_core::utils::touch::execute(util_args),
        #[cfg(feature = "tree")]
        "tree" => wasibox_core::utils::tree::execute(util_args),
        #[cfg(feature = "true")]
        "true" => wasibox_core::utils::r#true::execute(util_args),
        #[cfg(feature = "uname")]
        "uname" => wasibox_core::utils::uname::execute(util_args),
        #[cfg(feature = "unlink")]
        "unlink" => wasibox_core::utils::unlink::execute(util_args),
        #[cfg(feature = "wc")]
        "wc" => wasibox_core::utils::wc::execute(util_args),
        #[cfg(feature = "whoami")]
        "whoami" => wasibox_core::utils::whoami::execute(util_args),
        #[cfg(feature = "yes")]
        "yes" => wasibox_core::utils::yes::execute(util_args),
        _ => {
            Err(format!("Unknown utility: {}", util_name))
        }
    };

    if let Err(e) = result {
        eprintln!("{}", e);
        process::exit(1);
    }
}
