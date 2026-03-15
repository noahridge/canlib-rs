//! Safe, idiomatic Rust wrapper for the Kvaser CANLib SDK.
//!
//! # Getting started
//!
//! ```no_run
//! use canlib::{Channel, OpenFlags, Bitrate, CanMessage};
//! use std::time::Duration;
//!
//! // Library is initialized automatically on first use.
//! let num = canlib::get_number_of_channels().unwrap();
//! println!("Found {} CAN channels", num);
//!
//! // Open channel 0
//! let mut ch = Channel::open(0, OpenFlags::ACCEPT_VIRTUAL).unwrap();
//! ch.set_bitrate(Bitrate::Rate500K).unwrap();
//! ch.bus_on().unwrap();
//!
//! // Send a message
//! let msg = CanMessage::new(0x123, &[0xDE, 0xAD, 0xBE, 0xEF]).unwrap();
//! ch.write(&msg).unwrap();
//!
//! // Receive a message
//! match ch.read_wait(Duration::from_secs(1)) {
//!     Ok(rx) => println!("Received: id=0x{:X} data={:?}", rx.id(), rx.data()),
//!     Err(canlib::CanError::Timeout) => println!("No message received"),
//!     Err(e) => eprintln!("Error: {}", e),
//! }
//! // Channel is automatically taken off-bus and closed on drop.
//! ```

pub mod bus_params;
pub mod channel;
pub mod error;
pub mod message;
pub mod status;

// Re-export primary types at crate root for convenience.
pub use bus_params::{Bitrate, BusParams, BusParamsTq, DriverType, FdBitrate};
pub use channel::{CanBusControl, CanChannel, CanDiagnostics, CanRead, CanWrite, Channel, OpenFlags};
pub use error::{CanError, Result};
pub use message::{CanMessage, MessageFlags, CAN_STD_ID_MAX, CAN_EXT_ID_MAX};
pub use status::{BusStatistics, BusStatus, ErrorCounters};

use std::sync::Once;

/// Buffer size for channel data string queries and error text lookups.
const CHANNEL_DATA_BUF_SIZE: usize = 256;

static INIT: Once = Once::new();

/// Ensure the CANLib library is initialized. Called automatically by [`Channel::open`].
pub fn ensure_initialized() -> Result<()> {
    let lib = error::lib()?;
    INIT.call_once(|| unsafe {
        (lib.canInitializeLibrary)();
    });
    Ok(())
}

/// Get the CANLib version as a (major, minor) tuple.
pub fn get_version() -> Result<(u8, u8)> {
    ensure_initialized()?;
    let lib = error::lib()?;
    let v = unsafe { (lib.canGetVersion)() };
    Ok(((v >> 8) as u8, (v & 0xFF) as u8))
}

/// Get the number of available CAN channels.
pub fn get_number_of_channels() -> Result<i32> {
    ensure_initialized()?;
    let lib = error::lib()?;
    let mut count: i32 = 0;
    error::check_status(unsafe { (lib.canGetNumberOfChannels)(&mut count) })?;
    Ok(count)
}

/// Information about a CAN channel.
#[derive(Debug, Clone)]
pub struct ChannelInfo {
    /// Channel index.
    pub index: i32,
    /// Channel name.
    pub name: String,
    /// Device description.
    pub device_description: String,
    /// Card serial number.
    pub serial_number: u64,
}

/// Enumerate all available CAN channels.
pub fn enumerate_channels() -> Result<Vec<ChannelInfo>> {
    let count = get_number_of_channels()?;
    let mut channels = Vec::with_capacity(count as usize);

    for i in 0..count {
        let name = get_channel_string(i, canlib_sys::canCHANNELDATA_CHANNEL_NAME)?;
        let desc = get_channel_string(i, canlib_sys::canCHANNELDATA_DEVDESCR_ASCII)?;

        let lib = error::lib()?;
        let mut serial: u64 = 0;
        error::check_status(unsafe {
            (lib.canGetChannelData)(
                i,
                canlib_sys::canCHANNELDATA_CARD_SERIAL_NO,
                &mut serial as *mut u64 as *mut std::os::raw::c_void,
                std::mem::size_of::<u64>(),
            )
        })?;

        channels.push(ChannelInfo {
            index: i,
            name,
            device_description: desc,
            serial_number: serial,
        });
    }

    Ok(channels)
}

/// Get the error text for a status code from the CANLib SDK.
pub fn get_error_text(err: CanError) -> String {
    let code = err.to_status_code();
    let lib = match error::lib() {
        Ok(lib) => lib,
        Err(_) => return format!("Unknown error ({})", code),
    };
    let mut buf = [0u8; CHANNEL_DATA_BUF_SIZE];
    let status = unsafe {
        (lib.canGetErrorText)(
            code,
            buf.as_mut_ptr() as *mut std::os::raw::c_char,
            buf.len() as u32,
        )
    };
    if status >= 0 {
        let len = buf.iter().position(|&b| b == 0).unwrap_or(buf.len());
        String::from_utf8_lossy(&buf[..len]).to_string()
    } else {
        format!("Unknown error ({})", code)
    }
}

fn get_channel_string(channel: i32, item: i32) -> Result<String> {
    let lib = error::lib()?;
    let mut buf = [0u8; CHANNEL_DATA_BUF_SIZE];
    error::check_status(unsafe {
        (lib.canGetChannelData)(
            channel,
            item,
            buf.as_mut_ptr() as *mut std::os::raw::c_void,
            buf.len(),
        )
    })?;
    let len = buf.iter().position(|&b| b == 0).unwrap_or(buf.len());
    Ok(String::from_utf8_lossy(&buf[..len]).to_string())
}
