use std::{
    fs::{File, create_dir_all},
    io::Write,
};

#[derive(Debug, thiserror::Error)]
pub enum LoggerError {
    #[error("Failed to create log file: {0}")]
    FileCreation(#[from] std::io::Error),
}

pub struct Logger {
    log_file: Option<File>,
}

#[derive(PartialEq)]
pub enum LogTo {
    File,
    Console,
}

impl Logger {
    pub fn new(file_name: String, path: String, log_to: LogTo) -> Result<Self, LoggerError> {
        Ok(Self {
            log_file: if log_to == LogTo::File {
                create_dir_all(path.clone())?;
                Some(File::create(format!("{}{}", path, file_name))?)
            } else {
                None
            },
        })
    }

    pub fn log(&mut self, message: String) -> Result<(), LoggerError> {
        if let Some(log_file) = &mut self.log_file {
            let message = format!("{}\n", message);
            log_file.write_all(message.as_bytes())?;
        } else {
            println!("{}", message);
        }
        Ok(())
    }
}
