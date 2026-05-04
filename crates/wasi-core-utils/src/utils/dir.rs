use std::ffi::OsString;
use crate::utils::ls;

pub fn execute<I, T>(args: I) -> Result<(), String>
where
    I: IntoIterator<Item = T>,
    T: Into<OsString> + Clone,
{
    // GNU dir is essentially ls with some different defaults, 
    // but in many implementations it's just an alias.
    ls::execute(args)
}
