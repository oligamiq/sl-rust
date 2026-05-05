use std::ffi::OsString;
use std::io::{self, Write};

pub fn execute<I, T>(args: I) -> Result<(), String>
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

    let stdout = io::stdout();
    let mut handle = stdout.lock();
    loop {
        if writeln!(handle, "{}", message).is_err() {
            break;
        }
    }
    Ok(())
}
