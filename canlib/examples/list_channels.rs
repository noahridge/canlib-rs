//! List all available CAN channels.

fn main() {
    let (major, minor) = canlib::get_version();
    println!("CANLib version: {}.{}", major, minor);

    match canlib::enumerate_channels() {
        Ok(channels) => {
            println!("Found {} channel(s):", channels.len());
            for ch in &channels {
                println!(
                    "  [{}] {} - {} (serial: {})",
                    ch.index, ch.name, ch.device_description, ch.serial_number
                );
            }
        }
        Err(e) => eprintln!("Error enumerating channels: {}", e),
    }
}
