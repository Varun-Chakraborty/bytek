#[derive(Default)]
pub struct Args {
    pub input_filepath: Option<String>,
    pub debug: bool,
    pub pretty: bool,
    pub log_file_path: String,
    pub log_to: Option<String>,
    pub log_filename: Option<String>,
    pub output_filepath: Option<String>,
}

#[derive(Debug, thiserror::Error)]
pub enum ArgsError {
    #[error("Invalid flag: {0}")]
    InvalidFlag(String),
}

impl Args {
    pub fn parse() -> Result<Self, ArgsError> {
        let args: Vec<String> = std::env::args().collect();
        Self::parse_from(args)
    }

    fn parse_from(args: Vec<String>) -> Result<Self, ArgsError> {
        if args.len() < 2 {
            return Ok(Self {
                debug: false,
                pretty: false,
                log_file_path: String::from("/logs/"),
                log_to: None,
                log_filename: None,
                input_filepath: None,
                output_filepath: None,
            });
        }
        let debug = args.contains(&String::from("--debug"));
        let pretty = args.contains(&String::from("--pretty"));
        let log_to = args.iter().fold(None, |acc, x| {
            if x.contains("--log=") {
                Some(x[6..].to_string())
            } else {
                acc
            }
        });
        let path = args.iter().fold("", |acc, x| {
            if x.contains("--log_path=") {
                &x[11..]
            } else {
                acc
            }
        });
        let log_filename = args.iter().fold(None, |acc, x| {
            if x.contains("--log_filename=") {
                Some(x[15..].to_string())
            } else {
                acc
            }
        });

        let output_filepath = args.iter().fold(None, |acc, x| {
            if x.contains("--out=") {
                Some(x[6..].to_string())
            } else {
                acc
            }
        });

        Ok(Self {
            input_filepath: args.get(1).cloned(),
            debug,
            pretty,
            log_to,
            log_file_path: path.to_string(),
            log_filename,
            output_filepath,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_defaults_without_arguments() {
        let args = Args::parse_from(vec![String::from("assembler")]).unwrap();

        assert_eq!(args.input_filepath, None);
        assert!(!args.debug);
        assert!(!args.pretty);
        assert_eq!(args.log_file_path, "/logs/");
        assert_eq!(args.log_to, None);
        assert_eq!(args.log_filename, None);
        assert_eq!(args.output_filepath, None);
    }

    #[test]
    fn parse_known_flags() {
        let args = Args::parse_from(vec![
            String::from("assembler"),
            String::from("programs/kernel.asm"),
            String::from("--debug"),
            String::from("--pretty"),
            String::from("--log=file"),
            String::from("--log_path=/tmp/bytek-logs/"),
            String::from("--log_filename=vm.txt"),
            String::from("--out=kernel.bin"),
        ])
        .unwrap();

        assert_eq!(args.input_filepath, Some(String::from("programs/kernel.asm")));
        assert!(args.debug);
        assert!(args.pretty);
        assert_eq!(args.log_to, Some(String::from("file")));
        assert_eq!(args.log_file_path, "/tmp/bytek-logs/");
        assert_eq!(args.log_filename, Some(String::from("vm.txt")));
        assert_eq!(args.output_filepath, Some(String::from("kernel.bin")));
    }
}
