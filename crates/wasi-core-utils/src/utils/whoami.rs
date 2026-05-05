use std::ffi::OsString;
use std::env;

pub fn execute<I, T>(_args: I) -> Result<(), String>
where
    I: IntoIterator<Item = T>,
    T: Into<OsString> + Clone,
{
    // On WASI/Windows, getting actual user can be tricky.
    // Try environment variables first.
    let user = env::var("USER")
        .or_else(|_| env::var("USERNAME"))
        .unwrap_or_else(|_| "wasi-user".to_string());
    
    println!("{}", user);
    Ok(())
}
