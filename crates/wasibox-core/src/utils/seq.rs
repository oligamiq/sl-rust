use clap::Parser;
use std::ffi::OsString;
use std::io::Write;
use crate::IoContext;

#[derive(Parser)]
#[command(name = "seq", about = "Print a sequence of numbers")]
pub struct Args {
    /// First number (default: 1)
    #[arg()]
    pub first: Option<f64>,

    /// Increment (when three arguments are given)
    #[arg()]
    pub increment: Option<f64>,

    /// Last number (omit for infinite sequence)
    #[arg()]
    pub last: Option<f64>,
}

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
    let raw_args: Vec<String> = args.into_iter()
        .map(|a| a.into().to_string_lossy().into_owned())
        .collect();

    // Parse positional arguments manually for flexibility:
    //   seq           -> 1 to infinity, step 1
    //   seq LAST      -> 1 to LAST, step 1
    //   seq FIRST LAST -> FIRST to LAST, step 1
    //   seq FIRST INCREMENT LAST -> FIRST to LAST, step INCREMENT
    //   seq -inf      -> 1 to infinity, step 1 (explicit)
    let positional: Vec<&str> = raw_args.iter()
        .skip(1) // skip "seq"
        .filter(|a| !a.starts_with('-') || a.parse::<f64>().is_ok())
        .map(|s| s.as_str())
        .collect();

    let (first, increment, last): (f64, f64, Option<f64>) = match positional.len() {
        0 => (1.0, 1.0, None), // infinite from 1
        1 => {
            let val = positional[0].parse::<f64>()
                .map_err(|_| format!("seq: invalid argument: '{}'", positional[0]))?;
            (1.0, 1.0, Some(val))
        }
        2 => {
            let f = positional[0].parse::<f64>()
                .map_err(|_| format!("seq: invalid argument: '{}'", positional[0]))?;
            let l = positional[1].parse::<f64>()
                .map_err(|_| format!("seq: invalid argument: '{}'", positional[1]))?;
            (f, 1.0, Some(l))
        }
        _ => {
            let f = positional[0].parse::<f64>()
                .map_err(|_| format!("seq: invalid argument: '{}'", positional[0]))?;
            let inc = positional[1].parse::<f64>()
                .map_err(|_| format!("seq: invalid argument: '{}'", positional[1]))?;
            let l = positional[2].parse::<f64>()
                .map_err(|_| format!("seq: invalid argument: '{}'", positional[2]))?;
            (f, inc, Some(l))
        }
    };

    if increment == 0.0 {
        return Err("seq: zero increment".to_string());
    }

    let mut current = first;
    loop {
        if let Some(last_val) = last {
            if increment > 0.0 && current > last_val {
                break;
            }
            if increment < 0.0 && current < last_val {
                break;
            }
        }

        // Format: use integer display when value is integral
        let display = if current == current.floor() && current.abs() < 1e15 {
            format!("{}", current as i64)
        } else {
            format!("{}", current)
        };

        if writeln!(ctx.stdout, "{}", display).is_err() {
            break; // BrokenPipe — downstream closed
        }

        current += increment;
    }
    Ok(())
}
