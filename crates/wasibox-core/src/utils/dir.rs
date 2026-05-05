use std::ffi::OsString;
use crate::utils::ls;

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
    // GNU dir is essentially ls with some different defaults, 
    // but in many implementations it's just an alias.
    ls::execute_with_context(args, ctx)
}
