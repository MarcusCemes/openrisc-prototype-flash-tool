use std::{
    io::{self, Read},
    time::Duration,
};

use eyre::{Result, WrapErr};
use serialport::SerialPort;

const CMD_PROGRAM: &[u8] = b"*p";
const CMD_RUN: &[u8] = b"$";

const SEQ_READY: &[u8] = b"Openrisc based virtual Prototype.\n";
const SEQ_PROGRAM: &[u8] = b"Setting prog. mode\n";

pub struct VirtualPrototype {
    port: Box<dyn SerialPort>,
}

impl VirtualPrototype {
    pub fn open(port: &str, baud_rate: u32) -> Result<Self> {
        serialport::new(port, baud_rate)
            .open()
            .map(|port| Self { port })
            .wrap_err("Failed to open serial port")
    }

    /* == High-level == */

    pub fn wait_for_reset(&mut self) -> Result<()> {
        self.read_until_seq(SEQ_READY)
    }

    pub fn wait_for_program_ready(&mut self) -> Result<()> {
        self.read_until_seq(SEQ_PROGRAM)
    }

    pub fn request_program_write(&mut self) -> Result<()> {
        self.write_bytes(CMD_PROGRAM)
    }

    pub fn run_program(&mut self) -> Result<()> {
        self.write_bytes(CMD_RUN)
    }

    /* == IO ==  */

    pub fn set_timeout(&mut self, timeout: Duration) -> Result<()> {
        self.port.set_timeout(timeout)?;
        Ok(())
    }

    /// Reads from the serial port until the specified sequence is found.
    pub fn read_until_seq(&mut self, sequence: &[u8]) -> Result<()> {
        let mut i = 0;

        // Iterate over bytes as they are received, resetting the
        // sequence index if the byte does not match
        for byte in self.port.as_mut().bytes() {
            if byte? == sequence[i] {
                i += 1;
            } else {
                i = 0;
            }

            if i == sequence.len() {
                return Ok(());
            }
        }

        // In the unlikely event that the device disconnects,
        // the port may return an EOF
        Err(io::Error::new(
            io::ErrorKind::UnexpectedEof,
            "EOF before sequence was found",
        )
        .into())
    }

    /// Writes the specified buffer to the serial port.
    pub fn write_bytes(&mut self, buffer: &[u8]) -> Result<()> {
        self.port.write_all(buffer)?;
        Ok(())
    }

    /// Streams the specified reader to the serial port.
    pub fn write_stream<R: Read + ?Sized>(&mut self, reader: &mut R) -> Result<()> {
        io::copy(reader, &mut *self.port)?;
        Ok(())
    }
}
