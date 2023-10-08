use std::{
    io::{self, Read},
    time::Duration,
};

use eyre::{eyre, Result, WrapErr};
use serialport::{ClearBuffer, SerialPort};

const DEFAULT_TIMEOUT: Duration = Duration::from_millis(500);
const INFINITE_TIMEOUT: Duration = Duration::ZERO;

#[derive(Copy, Clone, Debug)]
pub enum Command {
    ShowHelp,
    Program,
    Run,
}

#[derive(Copy, Clone, Debug)]
pub enum Sequence {
    HelpScreen,
    Programming,
    UploadComplete,
}

pub struct VirtualPrototype {
    port: Box<dyn SerialPort>,
}

impl VirtualPrototype {
    pub fn open(port: &str, baud_rate: u32) -> Result<Self> {
        serialport::new(port, baud_rate)
            .timeout(DEFAULT_TIMEOUT)
            .open()
            .map(|port| Self { port })
            .wrap_err("Failed to open serial port")
    }

    /* == High-level == */

    pub fn in_bios(&mut self) -> Result<bool> {
        self.send_command(Command::ShowHelp)?;

        // Expect the device to promptly respond with the help screen
        match self.wait_for_sequence(Sequence::HelpScreen) {
            Ok(_) => Ok(true),
            Err(error) => match error.downcast_ref::<io::Error>() {
                Some(e) => match e.kind() {
                    io::ErrorKind::TimedOut => Ok(false),
                    _ => Err(error),
                },
                None => Err(error),
            },
        }
    }

    pub fn wait_for_reset(&mut self) -> Result<()> {
        self.port.set_timeout(INFINITE_TIMEOUT)?;
        self.clear_read_buffer()?;
        let result = self.wait_for_sequence(Sequence::HelpScreen);
        self.port.set_timeout(DEFAULT_TIMEOUT)?;
        result
    }

    pub fn send_command(&mut self, cmd: Command) -> Result<()> {
        self.write_bytes(cmd.as_bytes())
    }

    pub fn wait_for_sequence(&mut self, sequence: Sequence) -> Result<()> {
        self.read_until_sequence(sequence.as_bytes())
    }

    /* == IO ==  */

    /// Reads from the serial port until the specified sequence is found.
    pub fn read_until_sequence(&mut self, sequence: &[u8]) -> Result<()> {
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

        // If the stream ends, return an EOF error
        Err(eyre!("EOF before sequence was found"))
    }

    /// Streams the specified reader to the serial port.
    pub fn write_stream<R: Read + ?Sized>(&mut self, reader: &mut R) -> Result<u64> {
        io::copy(reader, &mut *self.port).wrap_err("Failed to write stream")
    }

    fn clear_read_buffer(&mut self) -> Result<()> {
        self.port
            .clear(ClearBuffer::Input)
            .wrap_err("Failed to clear read buffer")
    }

    /// Writes the specified buffer to the serial port.
    fn write_bytes(&mut self, buffer: &[u8]) -> Result<()> {
        self.port
            .write_all(buffer)
            .wrap_err("Failed to write bytes")
    }
}

impl Command {
    fn as_bytes(&self) -> &[u8] {
        match self {
            Self::ShowHelp => b"*h",
            Self::Program => b"*p",
            Self::Run => b"$",
        }
    }
}

impl Sequence {
    fn as_bytes(&self) -> &[u8] {
        match self {
            Self::HelpScreen => b"Openrisc based virtual Prototype.\n",
            Self::Programming => b"Setting prog. mode\n",
            Self::UploadComplete => b"Upload done\n",
        }
    }
}
