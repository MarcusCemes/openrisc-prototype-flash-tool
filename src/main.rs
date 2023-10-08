mod device;
mod log;

use std::{
    fs::File,
    io::{stdin, Read},
};

use clap::Parser;
use device::{Command, VirtualPrototype};
use eyre::Result;
use log::with_status;

use crate::device::Sequence;

const DEFAULT_BAUD_RATE: u32 = 115_200;

#[derive(Parser, Debug)]
#[clap(name = "OpenRISC Prototype Flash Tool", author, version)]
struct Args {
    #[arg(help = "Serial port to use (e.g. /dev/ttyUSB0 or COM4)")]
    port: String,

    #[arg(help = r#"Path of file to send, or "-" for stdin"#)]
    file: String,

    #[arg(short, long, default_value_t = DEFAULT_BAUD_RATE)]
    baud_rate: u32,
}

fn main() -> Result<()> {
    let args = Args::parse();

    eprintln!(
        "\n  Port:      {}\n  Baud rate: {}\n",
        args.port, args.baud_rate
    );

    let mut device = with_status("Connecting to port", || {
        VirtualPrototype::open(&args.port, args.baud_rate)
    })?;

    let in_bios = with_status("Verifying device state", || device.in_bios())?;

    if !in_bios {
        with_status("Not in BIOS, waiting for manual device reset", || {
            device.wait_for_reset()
        })?;
    }

    // Request the device to enter program mode, expecting the device to respond
    // with a programming ready sequence to indicate that it is ready to receive
    with_status("Requesting program write", || {
        device.send_command(Command::Program)?;
        device.wait_for_sequence(device::Sequence::Programming)
    })?;

    // Stream the complete program (.mem) to the device. The device
    // expects the program to terminate with a special sequence
    write_program(&mut device, &args)?;

    // Finally, request the device to run the program
    with_status("Requesting run", || device.send_command(Command::Run))
}

/// Stream the correct program source to the device. The device
/// should recognise that the transfer is complete once it receives
/// a special termination sequence.
fn write_program(device: &mut VirtualPrototype, args: &Args) -> Result<()> {
    // Select the correct message and stream to read from
    let (msg, mut stream): (&str, Box<dyn Read>) = match args.file.as_str() {
        "-" => ("Streaming stdin", Box::new(stdin())),
        path => ("Sending file", Box::new(File::open(path)?)),
    };

    let bytes_written = with_status(msg, || {
        let bytes_written = device.write_stream(&mut *stream)?;
        device.wait_for_sequence(Sequence::UploadComplete)?;
        Ok(bytes_written)
    })?;

    eprintln!("  ({} bytes written)", bytes_written);
    Ok(())
}
