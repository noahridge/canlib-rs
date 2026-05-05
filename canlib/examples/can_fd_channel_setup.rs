//! CAN FD channel setup example.
//!
//! Demonstrates how to open and configure a CAN FD channel, then exchange
//! FD frames over a virtual loopback using two virtual channels.
//!
//! # Prerequisites
//!
//! The Kvaser Virtual CAN driver must be installed. It provides at least two
//! virtual channels that are wired together, so frames sent on channel 0
//! appear on channel 1 (and vice versa) without any physical hardware.
//!
//! # Run
//!
//! ```
//! cargo run --example can_fd_channel_setup
//! ```

use canlib::{
    Bitrate, Brs, CanError, CanMessage, Channel, DriverType, ExtendedId, FdBitrate, MessageFlags,
    OpenFlags, StandardId,
};
use std::time::Duration;

// Arbitration phase: 500 kbit/s (classic CAN speed used for ID/control fields)
const ARB_BITRATE: Bitrate = Bitrate::Rate500K;

// Data phase: 2 Mbit/s at 80% sample point (used for the payload when BRS is set)
const DATA_BITRATE: FdBitrate = FdBitrate::Rate2M80P;

// Timeout for receive calls
const RX_TIMEOUT: Duration = Duration::from_secs(2);

fn main() -> canlib::Result<()> {
    let (major, minor) = canlib::get_version()?;
    println!("CANlib version: {}.{}", major, minor);

    // -----------------------------------------------------------------------
    // Step 1: Open two virtual CAN FD channels.
    //
    // OpenFlags::CAN_FD        – enable ISO CAN FD mode on the channel
    // OpenFlags::ACCEPT_VIRTUAL – allow opening Kvaser virtual channels
    // OpenFlags::REQUIRE_INIT_ACCESS – needed to call set_bitrate /
    //                                  set_bus_params_fd on this handle
    // -----------------------------------------------------------------------
    let fd_flags = OpenFlags::CAN_FD | OpenFlags::ACCEPT_VIRTUAL | OpenFlags::REQUIRE_INIT_ACCESS;

    let mut sender = Channel::open(0, fd_flags)?;
    let mut receiver = Channel::open(1, fd_flags)?;

    // -----------------------------------------------------------------------
    // Step 2: Configure bus parameters.
    //
    // CAN FD has two independent bit-rate phases:
    //
    //   Arbitration phase – used for the SOF, ID, control fields and EOF.
    //     Configured with set_bitrate(), same as classic CAN.
    //
    //   Data phase – used for the payload (only when BRS flag is set in the
    //     frame). Configured with set_bus_params_fd() or by passing a
    //     predefined FdBitrate constant.
    //
    // Option A – predefined FdBitrate enum via set_fd_bitrate() (simplest):
    //   Mirrors set_bitrate() for classic CAN; the driver resolves timing
    //   values automatically.
    //
    // Option B – explicit timing values:
    //   set_bus_params_fd(freq_hz, tseg1, tseg2, sjw)
    //   Use when you need a non-standard configuration.
    //
    // Option C – time-quanta format:
    //   set_bus_params_fd_tq(&arb_tq, &data_tq)
    //   Full control over both phases in one call.
    // -----------------------------------------------------------------------
    for ch in [&sender, &receiver] {
        // Arbitration phase
        ch.set_bitrate(ARB_BITRATE)?;

        // Data phase – Option A: predefined FdBitrate constant
        ch.set_fd_bitrate(DATA_BITRATE)?;

        // Data phase – Option B (uncomment to use instead):
        // ch.set_bus_params_fd(2_000_000, 5, 2, 1)?;

        // Data phase – Option C (uncomment to use instead):
        // use canlib::BusParamsTq;
        // let arb_tq  = BusParamsTq::new(80, 63, 16, 16, 0, 1);
        // let data_tq = BusParamsTq::new(20, 15,  4,  4, 0, 1);
        // ch.set_bus_params_fd_tq(&arb_tq, &data_tq)?;

        ch.set_output_control(DriverType::Normal)?;
    }

    // -----------------------------------------------------------------------
    // Step 3: Go on-bus.
    //
    // bus_on() must be called after all bus parameter configuration is done.
    // The channel will automatically call bus_off() when dropped.
    // -----------------------------------------------------------------------
    sender.bus_on()?;
    receiver.bus_on()?;
    println!("Both channels on bus (CAN FD, arb=500K, data=2M).\n");

    // Flush any residual frames from previous runs
    sender.flush_rx()?;
    receiver.flush_rx()?;

    // -----------------------------------------------------------------------
    // Step 4: Send and receive CAN FD frames.
    //
    // CanMessage::new_fd(id, data, brs)
    //   id   – typed identifier (StandardId or ExtendedId); std/ext flag is
    //          chosen by which type you pass.
    //   data – payload slice; up to 64 bytes for CAN FD.
    //   brs  – Brs::On switches to DATA_BITRATE during the data phase;
    //          Brs::Off keeps the entire frame at the arbitration bitrate.
    // -----------------------------------------------------------------------

    // 4a. Short FD frame (8 bytes) without BRS, standard ID
    let msg_8 = CanMessage::new_fd(
        StandardId::new(0x100).unwrap(),
        &[0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08],
        Brs::Off,
    )?;
    send_and_receive(&sender, &receiver, &msg_8, "8-byte FD, no BRS")?;

    // 4b. Medium FD frame (32 bytes) with BRS – data phase at 2 Mbit/s
    let data_32: Vec<u8> = (0u8..32).collect();
    let msg_32 = CanMessage::new_fd(StandardId::new(0x200).unwrap(), &data_32, Brs::On)?;
    send_and_receive(&sender, &receiver, &msg_32, "32-byte FD, BRS")?;

    // 4c. Maximum FD frame (64 bytes) with BRS, extended ID
    let data_64: Vec<u8> = (0u8..64).collect();
    let msg_64 = CanMessage::new_fd(ExtendedId::new(0x1ABC_0300).unwrap(), &data_64, Brs::On)?;
    send_and_receive(&sender, &receiver, &msg_64, "64-byte FD, BRS, extended ID")?;

    // -----------------------------------------------------------------------
    // Step 5: Non-blocking receive loop – drain remaining frames.
    // -----------------------------------------------------------------------
    println!("\nDraining receive queue (non-blocking)...");
    let mut extra = 0u32;
    loop {
        match receiver.read() {
            Ok(msg) => {
                extra += 1;
                println!("  extra frame: id=0x{:08X} {} bytes", msg.id(), msg.data().len());
            }
            Err(CanError::NoMsg) => break,
            Err(e) => return Err(e),
        }
    }
    if extra == 0 {
        println!("  (queue empty)");
    }

    // Channels go off-bus and close automatically when dropped at end of scope.
    println!("\nDone.");
    Ok(())
}

/// Send `msg` on `tx`, wait for it on `rx`, and print a summary.
fn send_and_receive(
    tx: &Channel,
    rx: &Channel,
    msg: &CanMessage,
    label: &str,
) -> canlib::Result<()> {
    tx.write(msg)?;
    tx.write_sync(RX_TIMEOUT)?;

    let received = rx.read_wait(RX_TIMEOUT)?;

    println!(
        "[{}]  id=0x{:08X}  bytes={}  brs={}  fd={}",
        label,
        received.id(),
        received.data().len(),
        received.flags().contains(MessageFlags::BRS),
        received.is_fd(),
    );

    assert_eq!(received.id(), msg.id(), "ID mismatch");
    assert_eq!(received.data(), msg.data(), "Data mismatch");

    Ok(())
}
