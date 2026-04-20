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
                &x[7..]
            } else {
                acc
            }
        });
        let log_filename = args.iter().fold(None, |acc, x| {
            if x.contains("--log_filename=") {
                Some(x[11..].to_string())
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
