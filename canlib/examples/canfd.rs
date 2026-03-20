//! Send a CAN FD message.

use canlib::{Bitrate, CanMessage, Channel, DriverType, OpenFlags};
use std::time::Duration;

fn main() -> canlib::Result<()> {
    let mut ch = Channel::open(
        0,
        OpenFlags::CAN_FD | OpenFlags::ACCEPT_VIRTUAL | OpenFlags::REQUIRE_INIT_ACCESS,
    )?;

    // Set arbitration phase bitrate
    ch.set_bitrate(Bitrate::Rate500K)?;
    // Set data phase bitrate (2 Mbit/s)
    ch.set_bus_params_fd(2_000_000, 5, 2, 1)?;
    ch.set_output_control(DriverType::Normal)?;

    ch.bus_on()?;
    println!("On bus with CAN FD.");

    // Send a CAN FD message with BRS (Bit Rate Switch)
    let data: Vec<u8> = (0..24).collect();
    let msg = CanMessage::new_fd(0x456, &data, true, false)?;
    ch.write(&msg)?;
    ch.write_sync(Duration::from_secs(1))?;
    println!("Sent FD message: id=0x{:03X} {} bytes", msg.id(), msg.data().len());

    Ok(())
}
