pub trait Device {
    fn read_byte(&mut self) -> Result<u8, std::io::Error>;
    fn write_byte(&mut self, byte: u8) -> Result<(), std::io::Error>;
}

use std::io::{Read, Write};

pub struct ConsoleDevice;

impl Device for ConsoleDevice {
    fn read_byte(&mut self) -> Result<u8, std::io::Error> {
        let mut buf = [0u8];
        std::io::stdin().read_exact(&mut buf)?;
        Ok(buf[0])
    }

    fn write_byte(&mut self, byte: u8) -> Result<(), std::io::Error> {
        print!("{}", byte as char);
        std::io::stdout().flush()?;
        Ok(())
    }
}
