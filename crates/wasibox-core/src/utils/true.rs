use std::ffi::OsString;

pub fn execute<I, T>(_args: I) -> Result<(), String>
where
    I: IntoIterator<Item = T>,
    T: Into<OsString> + Clone,
{
    Ok(())
}
