use clap::Parser;

/// Print machine architecture.
#[derive(Parser)]
#[command(
    name = "arch",
    version,
    about = "Print machine architecture.",
    override_usage = "arch [OPTION]...",
    disable_help_flag = true,
    disable_version_flag = true
)]
struct Args {
    /// display this help and exit
    #[arg(long, action = clap::ArgAction::Help)]
    help: Option<bool>,

    /// output version information and exit
    #[arg(long, action = clap::ArgAction::Version)]
    version: Option<bool>,
}

fn main() {
    // Parse arguments. Clap handles --help and --version automatically.
    let _ = Args::parse();

    let arch = get_arch();
    println!("{}", arch);
}

#[cfg(all(not(windows), not(target_os = "wasi")))]
fn get_arch() -> String {
    use std::ffi::CStr;
    use std::mem::MaybeUninit;

    unsafe {
        let mut name = MaybeUninit::<libc::utsname>::uninit();
        if libc::uname(name.as_mut_ptr()) == -1 {
            // GNU arch prints an error to stderr and exits with 1
            eprintln!("arch: cannot get system name");
            std::process::exit(1);
        }
        let name = name.assume_init();
        CStr::from_ptr(name.machine.as_ptr())
            .to_string_lossy()
            .into_owned()
    }
}

#[cfg(any(windows, target_os = "wasi"))]
fn get_arch() -> String {
    // On Windows and WASI, we use the architecture the binary was compiled for.
    // For WASI, this is typically "wasm32".
    std::env::consts::ARCH.to_string()
}
