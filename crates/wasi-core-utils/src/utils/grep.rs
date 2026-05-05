use clap::Parser;
use std::ffi::OsString;
use std::fs::File;
use std::io::{self, BufRead, BufReader};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "grep", about = "Search for patterns in files")]
pub struct Args {
    /// Pattern to search for
    pub pattern: String,

    /// Files to search
    pub files: Vec<PathBuf>,

    /// Ignore case distinctions
    #[arg(short = 'i', long)]
    pub ignore_case: bool,

    /// Invert match: select non-matching lines
    #[arg(short = 'v', long)]
    pub invert_match: bool,
}

pub fn execute<I, T>(args: I) -> Result<(), String>
where
    I: IntoIterator<Item = T>,
    T: Into<OsString> + Clone,
{
    let args = Args::try_parse_from(args).map_err(|e| e.to_string())?;

    #[cfg(feature = "grep")]
    {
        use regex::RegexBuilder;
        let re = RegexBuilder::new(&args.pattern)
            .case_insensitive(args.ignore_case)
            .build()
            .map_err(|e| format!("grep: invalid pattern: {}", e))?;

        if args.files.is_empty() {
            grep_stream(io::stdin().lock(), &re, args.invert_match, None)?;
        } else {
            for file_path in &args.files {
                let file = File::open(file_path).map_err(|e| format!("grep: {}: {}", file_path.display(), e))?;
                let label_str;
                let label = if args.files.len() > 1 { 
                    label_str = file_path.to_string_lossy();
                    Some(label_str.as_ref()) 
                } else { 
                    None 
                };
                grep_stream(BufReader::new(file), &re, args.invert_match, label)?;
            }
        }
        Ok(())
    }
    #[cfg(not(feature = "grep"))]
    {
        Err("grep feature is disabled".to_string())
    }
}

#[cfg(feature = "grep")]
fn grep_stream<R: BufRead>(reader: R, re: &regex::Regex, invert: bool, label: Option<&str>) -> Result<(), String> {
    for line in reader.lines() {
        let line = line.map_err(|e| e.to_string())?;
        let matched = re.is_match(&line);
        if matched ^ invert {
            if let Some(l) = label {
                print!("{}:", l);
            }
            println!("{}", line);
        }
    }
    Ok(())
}
