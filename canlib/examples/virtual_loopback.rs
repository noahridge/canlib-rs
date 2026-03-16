//! Test script: send CAN messages between Kvaser virtual channels.
//!
//! Kvaser installs a virtual device with two channels (0 and 1) connected
//! via a virtual bus. This example opens both channels, sends messages from
//! one to the other, and verifies they arrive correctly.
//!
//! Run with: cargo run --example virtual_loopback

use canlib::{
    Bitrate, CanError, CanMessage, Channel, DriverType, OpenFlags,
};
use std::time::Duration;

const BITRATE: Bitrate = Bitrate::Rate500K;
const TIMEOUT: Duration = Duration::from_secs(2);

fn main() -> canlib::Result<()> {
    // Print library info
    let (major, minor) = canlib::get_version()?;
    println!("CANLib version: {}.{}", major, minor);

    // List available channels so we can find the virtual ones
    let channels = canlib::enumerate_channels()?;
    println!("\nAvailable channels:");
    for ch in &channels {
        println!(
            "  [{}] {} - {} (serial: {})",
            ch.index, ch.name, ch.device_description, ch.serial_number
        );
    }
    println!();

    // Find two virtual channels (look for "Virtual" in the name)
    let virtual_indices: Vec<i32> = channels
        .iter()
        .filter(|c| {
            c.name.to_lowercase().contains("virtual")
                || c.device_description.to_lowercase().contains("virtual")
        })
        .map(|c| c.index)
        .collect();

    let (ch_a_idx, ch_b_idx) = if virtual_indices.len() >= 2 {
        (virtual_indices[0], virtual_indices[1])
        } else {
        println!("Could not auto-detect virtual channels, using channels 0 and 1.");
        println!("Make sure the Kvaser Virtual CAN driver is installed.");
        (0, 1)
    };

    println!("Using channel {} (sender) and channel {} (receiver)", ch_a_idx, ch_b_idx);

    // Open both channels
    let flags = OpenFlags::ACCEPT_VIRTUAL | OpenFlags::REQUIRE_INIT_ACCESS;

    let mut sender = Channel::open(ch_a_idx, flags)?;
    sender.set_bitrate(BITRATE)?;
    sender.set_output_control(DriverType::Normal)?;
    sender.bus_on()?;
    println!("Sender (channel {}) is on bus.", ch_a_idx);

    let mut receiver = Channel::open(ch_b_idx, flags)?;
    receiver.set_bitrate(BITRATE)?;
    receiver.set_output_control(DriverType::Normal)?;
    receiver.bus_on()?;
    println!("Receiver (channel {}) is on bus.", ch_b_idx);

    // Flush any stale messages
    sender.flush_rx()?;
    receiver.flush_rx()?;

    println!("\n--- Test 1: Single standard CAN message ---");
    test_single_message(&sender, &receiver)?;

    println!("\n--- Test 2: Multiple messages with different IDs ---");
    test_multiple_messages(&sender, &receiver)?;

    println!("\n--- Test 3: Extended (29-bit) ID message ---");
    test_extended_message(&sender, &receiver)?;

    println!("\n--- Test 4: Bidirectional communication ---");
    test_bidirectional(&sender, &receiver)?;

    println!("\n--- Test 5: Burst of messages ---");
    test_burst(&sender, &receiver)?;

    println!("\n========================================");
    println!("All tests passed!");
    println!("========================================");

    // Channels go off-bus and close automatically on drop
    Ok(())
}

fn test_single_message(sender: &Channel, receiver: &Channel) -> canlib::Result<()> {
    let tx_msg = CanMessage::new(0x123, &[0xDE, 0xAD, 0xBE, 0xEF])?;
    sender.write(&tx_msg)?;
    sender.write_sync(TIMEOUT)?;

    let rx_msg = receiver.read_wait(TIMEOUT)?;

    println!("  TX: id=0x{:03X} data={:02X?}", tx_msg.id(), tx_msg.data());
    println!(
        "  RX: id=0x{:03X} data={:02X?} ts={}us",
        rx_msg.id(),
        rx_msg.data(),
        rx_msg.timestamp().unwrap_or(0)
    );

    assert_eq!(rx_msg.id(), tx_msg.id(), "ID mismatch");
    assert_eq!(rx_msg.data(), tx_msg.data(), "Data mismatch");
    assert_eq!(rx_msg.dlc(), tx_msg.dlc(), "DLC mismatch");
    println!("  PASS");
    Ok(())
}

fn test_multiple_messages(sender: &Channel, receiver: &Channel) -> canlib::Result<()> {
    let messages = vec![
        CanMessage::new(0x100, &[0x01])?,
        CanMessage::new(0x200, &[0x02, 0x03])?,
        CanMessage::new(0x300, &[0x04, 0x05, 0x06])?,
        CanMessage::new(0x7FF, &[0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08])?,
    ];

    // Send all
    for msg in &messages {
        sender.write(msg)?;
    }
    sender.write_sync(TIMEOUT)?;

    // Receive and verify in order
    for (i, tx_msg) in messages.iter().enumerate() {
        let rx_msg = receiver.read_wait(TIMEOUT)?;
        println!(
            "  [{}] TX: id=0x{:03X} data={:02X?}  ->  RX: id=0x{:03X} data={:02X?}",
            i, tx_msg.id(), tx_msg.data(), rx_msg.id(), rx_msg.data()
        );
        assert_eq!(rx_msg.id(), tx_msg.id(), "ID mismatch at message {}", i);
        assert_eq!(rx_msg.data(), tx_msg.data(), "Data mismatch at message {}", i);
    }
    println!("  PASS");
    Ok(())
}

fn test_extended_message(sender: &Channel, receiver: &Channel) -> canlib::Result<()> {
    let tx_msg = CanMessage::new_extended(0x1ABCDEF, &[0xCA, 0xFE, 0xBA, 0xBE])?;
    sender.write(&tx_msg)?;
    sender.write_sync(TIMEOUT)?;

    let rx_msg = receiver.read_wait(TIMEOUT)?;

    println!(
        "  TX: id=0x{:08X} ext={} data={:02X?}",
        tx_msg.id(),
        tx_msg.is_extended(),
        tx_msg.data()
    );
    println!(
        "  RX: id=0x{:08X} ext={} data={:02X?}",
        rx_msg.id(),
        rx_msg.is_extended(),
        rx_msg.data()
    );

    assert_eq!(rx_msg.id(), tx_msg.id(), "ID mismatch");
    assert_eq!(rx_msg.data(), tx_msg.data(), "Data mismatch");
    assert!(rx_msg.is_extended(), "Expected extended frame flag");
    println!("  PASS");
    Ok(())
}

fn test_bidirectional(sender: &Channel, receiver: &Channel) -> canlib::Result<()> {
    // sender -> receiver
    let msg_a = CanMessage::new(0x100, &[0xAA])?;
    sender.write(&msg_a)?;
    sender.write_sync(TIMEOUT)?;

    let rx_a = receiver.read_wait(TIMEOUT)?;
    assert_eq!(rx_a.id(), 0x100);
    assert_eq!(rx_a.data(), &[0xAA]);
    println!("  channel {} -> channel {}: OK", "sender", "receiver");

    // receiver -> sender
    let msg_b = CanMessage::new(0x200, &[0xBB])?;
    receiver.write(&msg_b)?;
    receiver.write_sync(TIMEOUT)?;

    let rx_b = sender.read_wait(TIMEOUT)?;
    assert_eq!(rx_b.id(), 0x200);
    assert_eq!(rx_b.data(), &[0xBB]);
    println!("  channel {} -> channel {}: OK", "receiver", "sender");

    println!("  PASS");
    Ok(())
}

fn test_burst(sender: &Channel, receiver: &Channel) -> canlib::Result<()> {
    let count = 100;

    // Send a burst of messages
    for i in 0..count {
        let msg = CanMessage::new(0x400, &[(i & 0xFF) as u8, ((i >> 8) & 0xFF) as u8])?;
        sender.write(&msg)?;
    }
    sender.write_sync(TIMEOUT)?;

    // Receive and verify all
    let mut received = 0u32;
    loop {
        match receiver.read_wait(Duration::from_millis(500)) {
            Ok(rx_msg) => {
                let expected_lo = (received & 0xFF) as u8;
                let expected_hi = ((received >> 8) & 0xFF) as u8;
                assert_eq!(rx_msg.data()[0], expected_lo, "Data mismatch at msg {}", received);
                assert_eq!(rx_msg.data()[1], expected_hi, "Data mismatch at msg {}", received);
                received += 1;
            }
            Err(CanError::Timeout) | Err(CanError::NoMsg) => break,
            Err(e) => return Err(e),
        }
    }

    println!("  Sent {} messages, received {} messages", count, received);
    assert_eq!(received, count, "Message count mismatch");
    println!("  PASS");
    Ok(())
}
