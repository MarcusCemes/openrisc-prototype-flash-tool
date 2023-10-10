# OpenRISC Prototype Flash Tool

_A utility tool for CS-473 as part of MA-RO3 at EPFL, October 2023._

Communicates with the OpenRISC Virtual Prototype BIOS over a serial port (UART).

Written in Rust to be portable, fast and simple to use to automate the re-programming of the FPGA board's SRAM. Also happens to be a great companion when developing inside of WSL2.

### Features

- 💻 Runs on Windows, macOS and Linux
- 🚦 Supports streaming over stdin to allow integration with WSL2
- 🦀 Single executable, no dependencies

## Prerequisites

Check the [releases page](https://github.com/MarcusCemes/openrisc-prototype-flash-tool/releases) to see if there's a pre-compiled binary. If not, you will need to compile the tool yourself.

You will need to install the [Rust toolchain](https://www.rust-lang.org/).
If you are on Windows, you will also need the [Microsoft C++ Build Tools](https://visualstudio.microsoft.com/visual-cpp-build-tools/). On Linux, you will need GCC.

Grab a copy of the source code:

```bash
git clone https://github.com/MarcusCemse/openrisc-protoype-flash-tool.git
```

## Building

```bash
cargo build --release
```

Cargo will download all dependencies and compile the tool.

The executable will be located at `target/release/openrisc-prototype-flash-tool` (on Windows, this will have a `.exe` extension).

This should "just work". If not, you may need to install additional libraries, such as `libudev` on Linux for interfacing with local devices. Try running `apt install libudev-dev`.

## Usage

Ensure that the device is connected to the computer, and ensure that it is in the BIOS
by pressing the reset button. Run the OPFT executable, specifying the serial port to use
and the source of the program to flash:

```bat
openrisc-prototype-flash-tool /dev/ttyUSB0 path/to/program.mem
```

The negotiation and flashing process should begin automatically and roughly takes a second.
If the device is detected to be in the BIOS, the tool will display "waiting for device reset".
Press the device reset button and the device should continue once it receives the
post-reset BIOS welcome message.

To view all available options, run the following command:

```bat
openrisc-prototype-flash-tool --help
```

### Integration with WSL2

It's not currently possible to directly access serial ports on the host system from
within the WSL2 environment. It is, however, possible to run a Windows executable from
WSL2 that runs on the host system, which can communicate with the serial port, while
streaming data over stdin.

Ensure that the Windows executable `openrisc-prototype-flash-tool.exe` is placed somewhere
on the Windows filesystem, ideally somewhere that is on the Windows PATH (remember to restart WSL if you modify your Windows PATH).

Then, from within WSL2 you can run the tool as part of your compilation script, using "-" as
the source to indicate that the program should be read from stdin:

```bash
#!/bin/bash
set -eo pipefail

or1k-elf-gcc -Os  -nostartfiles -o program main.c
convert_or32 program

# If the tool is on the Windows PATH:
openrisc-prototype-flash-tool.exe COM4 - < program.mem

# Otherwise, specify the full path:
/mnt/c/path/to/openrisc-prototype-flash-tool.exe COM4 - < program.mem

```

## Author

- Marcus Cemes ([GitHub](https://github.com/MarcusCemes))

## License

This project is released under the MIT licence.

See [LICENCE](LICENCE).
