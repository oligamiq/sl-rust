use std::ffi::OsString;
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
    let args_vec: Vec<String> = args.into_iter()
        .skip(1) // Skip "yes" itself
        .map(|a| a.into().to_string_lossy().into_owned())
        .collect();
    
    let message = if args_vec.is_empty() {
        "y".to_string()
    } else {
        args_vec.join(" ")
    };

    loop {
        if writeln!(ctx.stdout, "{}", message).is_err() {
            break;
        }
    }
    Ok(())
}
