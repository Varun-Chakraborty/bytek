use std::{
    fs::File,
    io::{self, Write},
};

use super::encoder::delimiter::DelimiterTable;

#[derive(Debug, thiserror::Error)]
pub enum WriterError {
    #[error("I/O error: {0}")]
    Io(#[from] io::Error),
}

pub struct Writer {
    debug: bool,
    pretty: bool,
    bin_file_name: String,
    bin_file: File,
    debug_file_name: String,
    debug_file: Option<File>,
}

impl Writer {
    pub fn new(
        debug: bool,
        pretty: bool,
        bin_file_name: Option<String>,
        debug_file_name: Option<String>,
    ) -> Result<Self, WriterError> {
        Ok(Self {
            debug,
            pretty,
            bin_file: File::create(match &bin_file_name {
                Some(name) => name,
                None => "output.bin",
            })?,
            bin_file_name: match &bin_file_name {
                Some(name) => name.clone(),
                None => String::from("output.bin"),
            },
            debug_file: if debug {
                Some(File::create(match &debug_file_name {
                    Some(name) => name,
                    None => "debug.txt",
                })?)
            } else {
                None
            },
            debug_file_name: match &debug_file_name {
                Some(name) => name.clone(),
                None => String::from("debug.txt"),
            },
        })
    }

    pub fn write(
        &mut self,
        bytes_stream: Vec<u8>,
        delimiter_table: &mut DelimiterTable,
    ) -> Result<(), WriterError> {
        let mut bits_written = 0 as usize;
        delimiter_table.next();

        for byte in &bytes_stream {
            let byte = *byte;
            self.bin_file.write_all(&[byte])?;

            if self.debug {
                let mut debug_file = match &self.debug_file {
                    Some(file) => file,
                    None => continue,
                };

                for bit in format!("{:0>8b}", byte).chars() {
                    if self.pretty {
                        while let Some(current) = delimiter_table.get_current().cloned() {
                            if bits_written != current.address {
                                break;
                            }

                            debug_file.write_all(current.symbol.as_bytes())?;
                            delimiter_table.next();
                        }
                        bits_written += 1;
                    }
                    debug_file.write_all(bit.to_string().as_bytes())?;
                }
            }
        }

        println!(
            "Binary file `{}` written with {} bytes",
            self.bin_file_name,
            bytes_stream.len()
        );
        if self.debug {
            println!("Debug file `{}` written successfully", self.debug_file_name);
        }
        Ok(())
    }
}
