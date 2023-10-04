mod device;
mod log;

use std::{fs::File, io::stdin, time::Duration};

use clap::Parser;
use device::VirtualPrototype;
use eyre::Result;
use log::with_status;

#[derive(Parser, Debug)]
#[clap(name = "OpenRISC Prototype Flash Tool", author, version)]
struct Args {
    #[arg(help = "Serial port to use (e.g. /dev/ttyUSB0 or COM4)")]
    port: String,

    #[arg(help = r#"Path of file to send, or "-" for stdin"#)]
    file: String,

    #[arg(short, long, default_value_t = 115_200)]
    baud_rate: u32,

    #[arg(
        short,
        long,
        help = "Wait for the device to reset before sending the program"
    )]
    wait_reset: bool,
}

fn main() -> Result<()> {
    let args = Args::parse();

    let mut device = with_status(
        &format!("Opening port {} ({})", args.port, args.baud_rate),
        || VirtualPrototype::open(&args.port, args.baud_rate),
    )?;

    if args.wait_reset {
        with_status("Waiting for device reset", || device.wait_for_reset())?;
    }

    // The device should now be in the bootloader, set a short
    // timeout in case the device does not respond as expected
    device.set_timeout(Duration::from_millis(500))?;

    // Request the device to enter program mode
    with_status("Requesting program write", || {
        device.request_program_write()
    })?;

    // Wait until the device is ready to receive the program
    with_status("Waiting for ready response", || {
        device.wait_for_program_ready()
    })?;

    // Stream the complete program (.mem) to the device. The device
    // expects the program to terminate with a special sequence
    write_program(&mut device, &args)?;

    // Finally, request the device to run the program
    with_status("Requesting run", || device.run_program())?;

    Ok(())
}

/// Stream the correct program source to the device. The device
/// should recognise that the transfer is complete once it receives
/// a special termination sequence.
fn write_program(device: &mut VirtualPrototype, args: &Args) -> Result<()> {
    match args.file.as_str() {
        "-" => with_status("Streaming stdin", || {
            let mut stream = stdin();
            device.write_stream(&mut stream)
        }),

        _ => with_status("Sending file", || {
            let mut file = File::open(&args.file)?;
            device.write_stream(&mut file)
        }),
    }
}
