
#[derive(Clone, Debug)]
pub struct Config {
    pub accident: bool,      // -a
    pub c51: bool,          // -c
    pub logo: bool,         // -l
    pub flying: bool,       // -F
}

impl Config {
    pub fn from_args<I, T>(args: I) -> Self 
    where
        I: IntoIterator<Item = T>,
        T: AsRef<str>,
    {
        let mut config = Config {
            accident: false,
            c51: false,
            logo: false,
            flying: false,
        };

        for arg in args.into_iter() {
            let arg_str = arg.as_ref();
            if arg_str.starts_with('-') {
                for ch in arg_str.chars().skip(1) {
                    match ch {
                        'a' => config.accident = true,
                        'c' => config.c51 = true,
                        'l' => config.logo = true,
                        'F' => config.flying = true,
                        _ => {}
                    }
                }
            }
        }

        config
    }
}
