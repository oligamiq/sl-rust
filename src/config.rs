use std::env;

#[derive(Clone, Debug)]
pub struct Config {
    pub accident: bool,      // -a
    pub c51: bool,          // -c
    pub logo: bool,         // -l
    pub flying: bool,       // -F
}

impl Config {
    pub fn from_args() -> Self {
        let mut config = Config {
            accident: false,
            c51: false,
            logo: false,
            flying: false,
        };

        for arg in env::args().skip(1) {
            if arg.starts_with('-') {
                for ch in arg.chars().skip(1) {
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
