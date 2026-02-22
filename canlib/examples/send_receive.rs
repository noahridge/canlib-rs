//! Send and receive CAN messages on channel 0.

use canlib::{Bitrate, CanError, CanMessage, Channel, DriverType, OpenFlags};
use std::time::Duration;

fn main() -> canlib::Result<()> {
    // Open channel 0 with virtual channel support
    let mut ch = Channel::open(
        0,
        OpenFlags::ACCEPT_VIRTUAL | OpenFlags::REQUIRE_INIT_ACCESS,
    )?;

    // Configure bus parameters
    ch.set_bitrate(Bitrate::Rate500K)?;
    ch.set_output_control(DriverType::Normal)?;

    // Go on-bus
    ch.bus_on()?;
    println!("On bus. Sending test message...");

    // Send a message
    let msg = CanMessage::new(0x123, &[0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08]);
    ch.write(&msg)?;
    ch.write_sync(Duration::from_secs(1))?;
    println!("Sent: id=0x{:03X} data={:02X?}", msg.id, msg.data);

    // Try to receive messages for 5 seconds
    println!("Listening for messages (5 seconds)...");
    let deadline = std::time::Instant::now() + Duration::from_secs(5);

    while std::time::Instant::now() < deadline {
        match ch.read_wait(Duration::from_millis(100)) {
            Ok(rx) => {
                println!(
                    "  Received: id=0x{:03X} dlc={} data={:02X?} flags={:?} ts={}us",
                    rx.id,
                    rx.dlc,
                    rx.data,
                    rx.flags,
                    rx.timestamp.unwrap_or(0),
                );
            }
            Err(CanError::Timeout) => continue,
            Err(e) => {
                eprintln!("Read error: {}", e);
                break;
            }
        }
    }

    // Channel goes off-bus and closes automatically on drop
    println!("Done.");
    Ok(())
}
