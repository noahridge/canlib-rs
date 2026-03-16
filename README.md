# canlib-rs

Safe, idiomatic Rust bindings for the [Kvaser CANLib SDK](https://kvaser.com/developer/canlib-sdk/). Provides access to Kvaser CAN hardware for sending and receiving CAN and CAN FD messages.

## Setup

### 1. Install the Kvaser CANLib SDK

**Windows:**
Download and run the installer from [kvaser.com/developer/canlib-sdk](https://kvaser.com/developer/canlib-sdk/). This installs the SDK, drivers, and the Virtual CAN driver.

**Linux:**
Install the [linuxcan](https://www.kvaser.com/linux-drivers-and-sdk-2/) package.

### 2. Verify the SDK is found

The library loads `canlib32.dll` (Windows) or `libcanlib.so` (Linux) at runtime. The standard SDK installer places this on the system library path automatically.

If the SDK is installed in a non-standard location, set the `CANLIB_SDK_DIR` environment variable:

```sh
# Windows (PowerShell)
$env:CANLIB_SDK_DIR = "C:\path\to\Kvaser\Canlib"

# Windows (cmd)
set CANLIB_SDK_DIR=C:\path\to\Kvaser\Canlib

# Linux
export CANLIB_SDK_DIR=/opt/kvaser/canlib
```

The library will look for the shared library inside this directory first, then fall back to the system search path.

### 3. Add the dependency

```toml
[dependencies]
canlib = { path = "path/to/canlib-rs/canlib" }
```

### 4. Hardware

You need either:
- **Kvaser CAN hardware** (USB interfaces, PCIe cards, etc.)
- **Kvaser Virtual CAN Driver** (included with the Windows SDK installer) — provides two virtual channels connected via a virtual bus, no hardware required

## Quick start

```rust
use canlib::{Bitrate, CanMessage, Channel, OpenFlags};
use std::time::Duration;

fn main() -> canlib::Result<()> {
    let mut ch = Channel::open(0, OpenFlags::ACCEPT_VIRTUAL)?;
    ch.set_bitrate(Bitrate::Rate500K)?;
    ch.bus_on()?;

    // Send
    let msg = CanMessage::new(0x123, &[0xDE, 0xAD, 0xBE, 0xEF])?;
    ch.write(&msg)?;

    // Receive (2s timeout)
    match ch.read_wait(Duration::from_secs(2)) {
        Ok(rx) => println!("id=0x{:X} data={:02X?}", rx.id(), rx.data()),
        Err(canlib::CanError::Timeout) => println!("No message"),
        Err(e) => return Err(e),
    }

    Ok(())
    // Channel goes off-bus and closes on drop
}
```

## CAN FD

```rust
use canlib::{Bitrate, CanMessage, Channel, DriverType, FdBitrate, OpenFlags};

let flags = OpenFlags::CAN_FD | OpenFlags::ACCEPT_VIRTUAL | OpenFlags::REQUIRE_INIT_ACCESS;
let mut ch = Channel::open(0, flags)?;

ch.set_bitrate(Bitrate::Rate500K)?;          // Arbitration phase
ch.set_fd_bitrate(FdBitrate::Rate2M80P)?;    // Data phase
ch.set_output_control(DriverType::Normal)?;
ch.bus_on()?;

let msg = CanMessage::new_fd(0x456, &[0u8; 64], true)?; // 64 bytes, BRS enabled
ch.write(&msg)?;
```

## Examples

```sh
cargo run --example list_channels       # List available CAN channels
cargo run --example send_receive        # Send/receive on channel 0
cargo run --example canfd               # Send a CAN FD message
cargo run --example virtual_loopback    # Full test using virtual channels (no hardware)
cargo run --example can_fd_channel_setup # CAN FD setup walkthrough
```

## Troubleshooting

**"Failed to load canlib32.dll"** — The SDK is not installed or the DLL is not on the system PATH. Install the SDK, or set `CANLIB_SDK_DIR` to point to the SDK root directory.

**"No channels available"** — No Kvaser hardware connected and the Virtual CAN driver is not installed. Install the Kvaser Virtual CAN driver (included with the SDK on Windows).

## Architecture

The project is a Cargo workspace with two crates:

- **`canlib-sys`** — Low-level FFI layer. Loads `canlib32.dll`/`libcanlib.so` at runtime via `libloading`. Contains all type definitions, constants, and function pointer signatures.
- **`canlib`** — Safe wrapper. Provides `Channel`, `CanMessage`, error handling, and RAII resource management.

See [ARCHITECTURE.md](ARCHITECTURE.md) for details. Full API documentation is in [USER_GUIDE.md](USER_GUIDE.md).

## License

MIT
