use clap::Parser;
use std::ffi::OsString;

#[derive(Parser)]
#[command(name = "uname", about = "Print system information")]
struct Args {
    /// print the machine hardware name
    #[arg(short = 'm')]
    machine: bool,

    /// print all information
    #[arg(short = 'a')]
    all: bool,
}

pub fn execute<I, T>(args: I) -> Result<(), String>
where
    I: IntoIterator<Item = T>,
    T: Into<OsString> + Clone,
{
    let args = Args::try_parse_from(args).map_err(|e| e.to_string())?;

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
            
            let mut output = Vec::new();
            if args.all || (!args.machine) {
                output.push(CStr::from_ptr(name.sysname.as_ptr()).to_string_lossy());
            }
            if args.all || args.machine {
                output.push(CStr::from_ptr(name.machine.as_ptr()).to_string_lossy());
            }
            println!("{}", output.join(" "));
        }
    }

    #[cfg(any(windows, target_os = "wasi"))]
    {
        let sysname = if cfg!(windows) { "Windows" } else { "WASI" };
        let mut output = Vec::new();
        if args.all || (!args.machine) {
            output.push(sysname.to_string());
        }
        if args.all || args.machine {
            output.push(std::env::consts::ARCH.to_string());
        }
        println!("{}", output.join(" "));
    }

    Ok(())
}
