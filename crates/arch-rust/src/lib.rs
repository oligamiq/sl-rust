use clap::Parser;
use std::ffi::OsString;

/// Print machine architecture.
#[derive(Parser, Debug)]
#[command(
    name = "arch",
    version,
    about = "Print machine architecture.",
    override_usage = "arch [OPTION]...",
    disable_help_flag = true,
    disable_version_flag = true
)]
pub struct Args {
    /// display this help and exit
    #[arg(long, action = clap::ArgAction::Help)]
    pub help: Option<bool>,

    /// output version information and exit
    #[arg(long, action = clap::ArgAction::Version)]
    pub version: Option<bool>,
}

/// Executes the arch logic with the given arguments.
/// Returns the architecture string on success, or an error message on failure.
pub fn execute<I, T>(args: I) -> Result<String, String>
where
    I: IntoIterator<Item = T>,
    T: Into<OsString> + Clone,
{
    // Try to parse arguments. 
    // Note: clap::Parser::try_parse_from will handle --help and --version 
    // by returning an Err that contains the help/version message if configured.
    match Args::try_parse_from(args) {
        Ok(_) => {
            get_arch().map_err(|e| format!("arch: {}", e))
        }
        Err(e) => {
            // Return the help or version message as the "result" if it's a help/version event,
            // otherwise return the error message.
            Err(e.to_string())
        }
    }
}

/// Returns the machine hardware name (architecture).
pub fn get_arch() -> Result<String, String> {
    #[cfg(all(not(windows), not(target_os = "wasi")))]
    {
        use std::ffi::CStr;
        use std::mem::MaybeUninit;

        unsafe {
            let mut name = MaybeUninit::<libc::utsname>::uninit();
            if libc::uname(name.as_mut_ptr()) == -1 {
                return Err("cannot get system name".to_string());
            }
            let name = name.assume_init();
            Ok(CStr::from_ptr(name.machine.as_ptr())
                .to_string_lossy()
                .into_owned())
        }
    }

    #[cfg(any(windows, target_os = "wasi"))]
    {
        Ok(std::env::consts::ARCH.to_string())
    }
}
