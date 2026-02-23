# canlib-rs User Guide

A safe, idiomatic Rust wrapper for the [Kvaser CANLib SDK](https://kvaser.com/developer/canlib-sdk/), providing access to Kvaser CAN hardware for sending and receiving CAN and CAN FD messages.

## Prerequisites

- **Rust** 1.70+ (2021 edition)
- **Kvaser CANLib SDK** installed:
  - **Windows**: Download from [kvaser.com](https://kvaser.com/developer/canlib-sdk/) and run the installer
  - **Linux**: Install the [linuxcan](https://www.kvaser.com/linux-drivers-and-sdk-2/) package
- **Kvaser hardware** or the **Kvaser Virtual CAN Driver** (installed automatically with the SDK on Windows)

### Optional

- **LLVM/Clang**: If installed, `bindgen` will auto-generate FFI bindings from the SDK headers. Without it, the crate uses pre-written manual declarations — this is fully functional.

### SDK Path Configuration

The build script auto-detects the SDK at standard install locations. If your SDK is installed elsewhere, set the environment variable:

```sh
set CANLIB_SDK_DIR=C:\path\to\your\Kvaser\Canlib
```

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
canlib = { path = "path/to/canlib-rs/canlib" }
```

## Quick Start

```rust
use canlib::{Bitrate, CanMessage, Channel, OpenFlags};
use std::time::Duration;

fn main() -> canlib::Result<()> {
    // Open channel 0 (use ACCEPT_VIRTUAL for virtual channels)
    let mut ch = Channel::open(0, OpenFlags::ACCEPT_VIRTUAL)?;
    ch.set_bitrate(Bitrate::Rate500K)?;
    ch.bus_on()?;

    // Send a message
    let msg = CanMessage::new(0x123, &[0xDE, 0xAD, 0xBE, 0xEF]);
    ch.write(&msg)?;

    // Receive a message (2 second timeout)
    let received = ch.read_wait(Duration::from_secs(2))?;
    println!("Got: id=0x{:X} data={:02X?}", received.id, received.data);

    Ok(())
    // Channel automatically goes off-bus and closes when dropped
}
```

## API Reference

### Library Functions

These free functions are available at the crate root.

```rust
// Get CANLib version as (major, minor)
let (major, minor) = canlib::get_version();

// Get the number of available CAN channels
let count = canlib::get_number_of_channels()?;

// Enumerate all channels with metadata
let channels = canlib::enumerate_channels()?;
for ch in &channels {
    println!("[{}] {} - {} (serial: {})",
        ch.index, ch.name, ch.device_description, ch.serial_number);
}
```

### Opening a Channel

```rust
use canlib::{Channel, OpenFlags};

// Basic open
let mut ch = Channel::open(0, OpenFlags::empty())?;

// With flags (combinable with |)
let mut ch = Channel::open(0,
    OpenFlags::ACCEPT_VIRTUAL | OpenFlags::REQUIRE_INIT_ACCESS
)?;
```

**Available flags:**

| Flag | Description |
|---|---|
| `EXCLUSIVE` | Prevent other applications from using this channel |
| `REQUIRE_EXTENDED` | Require extended CAN (2.0B) support |
| `ACCEPT_VIRTUAL` | Allow opening virtual channels |
| `OVERRIDE_EXCLUSIVE` | Override another handle's exclusive access |
| `REQUIRE_INIT_ACCESS` | Require permission to set bus parameters |
| `NO_INIT_ACCESS` | Open without init access |
| `ACCEPT_LARGE_DLC` | Accept DLC values greater than 8 |
| `CAN_FD` | Open for CAN FD (ISO 11898-1:2015) |
| `CAN_FD_NONISO` | Open for non-ISO CAN FD |

### Configuring Bus Parameters

**Predefined bitrates (recommended):**

```rust
use canlib::Bitrate;

ch.set_bitrate(Bitrate::Rate500K)?;
```

Available: `Rate1M`, `Rate500K`, `Rate250K`, `Rate125K`, `Rate100K`, `Rate83K`, `Rate62K`, `Rate50K`, `Rate10K`.

**Custom timing parameters:**

```rust
use canlib::BusParams;

ch.set_bus_params(&BusParams {
    freq: 500_000,
    tseg1: 4,
    tseg2: 3,
    sjw: 1,
    no_samp: 1,
    sync_mode: 0,
})?;
```

**Time-quanta parameters:**

```rust
use canlib::BusParamsTq;

ch.set_bus_params_tq(&BusParamsTq {
    tq: 8,
    phase1: 2,
    phase2: 2,
    sjw: 1,
    prop: 2,
    prescaler: 4,
})?;
```

**Reading back current parameters:**

```rust
let params = ch.get_bus_params()?;
println!("Frequency: {} Hz", params.freq);
```

### Driver Output Mode

```rust
use canlib::DriverType;

ch.set_output_control(DriverType::Normal)?;     // Normal operation
ch.set_output_control(DriverType::Silent)?;      // Listen-only mode
```

Available: `Normal`, `Silent`, `SelfReception`, `Off`.

### Going On-Bus / Off-Bus

```rust
ch.bus_on()?;           // Start participating in CAN traffic
ch.bus_off()?;          // Stop participating
ch.reset_bus()?;        // Reset the CAN controller

if ch.is_on_bus() {
    println!("Channel is active");
}
```

### Creating Messages

**Standard CAN (11-bit ID):**

```rust
use canlib::CanMessage;

let msg = CanMessage::new(0x123, &[0x01, 0x02, 0x03]);
```

**Extended CAN (29-bit ID):**

```rust
let msg = CanMessage::new_extended(0x1ABCDEF, &[0x01, 0x02, 0x03]);
```

**CAN FD (up to 64 bytes, with optional Bit Rate Switch):**

```rust
let data: Vec<u8> = (0..32).collect();
let msg = CanMessage::new_fd(0x456, &data, true); // true = BRS enabled
```

**Inspecting a message:**

```rust
msg.is_extended();    // 29-bit ID?
msg.is_fd();          // CAN FD frame?
msg.is_rtr();         // Remote Transmission Request?
msg.is_error_frame(); // Error frame?
```

### Sending Messages

```rust
use std::time::Duration;

// Non-blocking send (queues the message)
ch.write(&msg)?;

// Send and wait for transmission (with timeout)
ch.write_wait(&msg, Duration::from_secs(1))?;

// Wait for all queued messages to be transmitted
ch.write_sync(Duration::from_secs(1))?;
```

### Receiving Messages

```rust
use canlib::CanError;
use std::time::Duration;

// Non-blocking read (returns Err(CanError::NoMsg) if queue is empty)
match ch.read() {
    Ok(msg) => println!("Got: 0x{:X}", msg.id),
    Err(CanError::NoMsg) => println!("No message"),
    Err(e) => return Err(e),
}

// Blocking read with timeout
let msg = ch.read_wait(Duration::from_secs(2))?;
println!("id=0x{:X} data={:02X?} timestamp={}us",
    msg.id, msg.data, msg.timestamp.unwrap_or(0));

// Wait for any message to arrive (doesn't consume it)
ch.read_sync(Duration::from_secs(5))?;
let msg = ch.read()?;

// Read only messages with a specific ID
let msg = ch.read_specific(0x123)?;

// Read specific ID, discarding non-matching messages from the queue
let msg = ch.read_specific_skip(0x123)?;
```

### Acceptance Filters

Limit which messages are received:

```rust
// Accept only messages with a specific standard (11-bit) ID
ch.set_acceptance_filter(0x123, 0x7FF, false)?;

// Accept only messages with a specific extended (29-bit) ID
ch.set_acceptance_filter(0x1ABCDEF, 0x1FFFFFFF, true)?;

// Accept a range of IDs (mask bits: 1 = must match, 0 = don't care)
ch.set_acceptance_filter(0x100, 0x700, false)?; // Accepts 0x100-0x1FF
```

### Bus Status and Diagnostics

```rust
// Read bus status flags
let status = ch.read_status()?;
if status.contains(canlib::BusStatus::BUS_OFF) {
    println!("Bus is OFF");
}
if status.contains(canlib::BusStatus::ERROR_PASSIVE) {
    println!("Error passive");
}

// Read error counters
let counters = ch.read_error_counters()?;
println!("TX errors: {}, RX errors: {}, Overruns: {}",
    counters.tx_errors, counters.rx_errors, counters.overrun_errors);

// Bus statistics (request first, then read)
ch.request_bus_statistics()?;
let stats = ch.get_bus_statistics()?;
println!("Bus load: {:.1}%", stats.bus_load_percent());
println!("Std frames: {}, Ext frames: {}", stats.std_data, stats.ext_data);
```

### Queue Management

```rust
ch.flush_rx()?;   // Discard all messages in the receive queue
ch.flush_tx()?;   // Discard all messages in the transmit queue
```

### Advanced: Raw Handle Access

For functionality not yet wrapped, you can access the raw CANLib handle and call `canlib-sys` functions directly:

```rust
let handle = ch.raw_handle();
// Use with canlib_sys functions:
// unsafe { canlib_sys::canSomeFunction(handle, ...) }
```

## CAN FD

To use CAN FD, open the channel with the `CAN_FD` flag and configure both arbitration and data phase bitrates:

```rust
use canlib::{Bitrate, CanMessage, Channel, DriverType, OpenFlags};

let mut ch = Channel::open(0,
    OpenFlags::CAN_FD | OpenFlags::ACCEPT_VIRTUAL | OpenFlags::REQUIRE_INIT_ACCESS
)?;

// Arbitration phase: 500 kbit/s
ch.set_bitrate(Bitrate::Rate500K)?;

// Data phase: custom timing (2 Mbit/s)
ch.set_bus_params_fd(2_000_000, 5, 2, 1)?;

ch.set_output_control(DriverType::Normal)?;
ch.bus_on()?;

// Send a CAN FD message with BRS
let data: Vec<u8> = (0..48).collect();
let msg = CanMessage::new_fd(0x456, &data, true);
ch.write(&msg)?;
```

## Error Handling

All fallible operations return `canlib::Result<T>`, which is `Result<T, CanError>`. The `CanError` enum has a variant for every CANLib status code:

```rust
use canlib::CanError;

match ch.read() {
    Ok(msg) => { /* process message */ }
    Err(CanError::NoMsg) => { /* no message available */ }
    Err(CanError::Timeout) => { /* operation timed out */ }
    Err(CanError::InvalidHandle) => { /* handle is invalid */ }
    Err(CanError::Hardware) => { /* hardware error */ }
    Err(e) => eprintln!("Error: {}", e),
}
```

All error variants implement `Display` with human-readable messages.

## Thread Safety

- The CANLib library is initialized once, thread-safely, using `std::sync::Once`.
- `Channel` is `Send` but **not** `Sync`. You can move a channel to another thread, but you cannot share it across threads. Each thread that needs CAN access should open its own channel.
- Multiple threads can open the same physical channel number independently.

## Resource Cleanup

`Channel` implements `Drop`. When a `Channel` goes out of scope:

1. If the channel is on-bus, `canBusOff` is called automatically.
2. `canClose` is called to release the handle.

You never need to manually close a channel.

## Examples

The crate includes four runnable examples:

```sh
# List all available CAN channels
cargo run --example list_channels

# Send and receive messages on channel 0
cargo run --example send_receive

# Send a CAN FD message
cargo run --example canfd

# Full test suite using virtual channels (no hardware needed)
cargo run --example virtual_loopback
```

### Virtual Channel Testing

The Kvaser Virtual CAN Driver (included with the SDK on Windows) provides two channels connected via a virtual bus. This allows full testing without physical hardware:

```sh
cargo run --example virtual_loopback
```

This runs 5 tests: single message, multiple messages, extended ID, bidirectional, and a 100-message burst.
