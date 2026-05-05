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
use std::io::Write;
use crate::IoContext;

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
    let args = Args::try_parse_from(args).map_err(|e| e.to_string())?;

    let mut output = Vec::new();

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
            
            if args.all || (!args.machine) {
                output.push(CStr::from_ptr(name.sysname.as_ptr()).to_string_lossy().into_owned());
            }
            if args.all || args.machine {
                output.push(CStr::from_ptr(name.machine.as_ptr()).to_string_lossy().into_owned());
            }
        }
    }

    #[cfg(any(windows, target_os = "wasi"))]
    {
        let sysname = if cfg!(windows) { "Windows" } else { "WASI" };
        if args.all || (!args.machine) {
            output.push(sysname.to_string());
        }
        if args.all || args.machine {
            output.push(std::env::consts::ARCH.to_string());
        }
    }

    writeln!(ctx.stdout, "{}", output.join(" ")).map_err(|e| e.to_string())?;
    Ok(())
}
