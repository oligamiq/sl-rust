use clap::Parser;
use std::ffi::OsString;

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
pub struct Args {
    /// display this help and exit
    #[arg(long, action = clap::ArgAction::Help)]
    pub help: Option<bool>,

    /// output version information and exit
    #[arg(long, action = clap::ArgAction::Version)]
    pub version: Option<bool>,
}

pub fn execute<I, T>(args: I) -> Result<(), String>
where
    I: IntoIterator<Item = T>,
    T: Into<OsString> + Clone,
{
    // Parse arguments. 
    let _ = Args::try_parse_from(args).map_err(|e| e.to_string())?;

    let arch = get_arch();
    println!("{}", arch);
    Ok(())
}

fn get_arch() -> String {
    #[cfg(all(not(windows), not(target_os = "wasi")))]
    {
        use std::ffi::CStr;
        use std::mem::MaybeUninit;

        unsafe {
            let mut name = MaybeUninit::<libc::utsname>::uninit();
            if libc::uname(name.as_mut_ptr()) == -1 {
                return "unknown".to_string();
            }
            let name = name.assume_init();
            CStr::from_ptr(name.machine.as_ptr())
                .to_string_lossy()
                .into_owned()
        }
    }

    #[cfg(any(windows, target_os = "wasi"))]
    {
        std::env::consts::ARCH.to_string()
    }
}
