use canlib_sys as sys;

bitflags::bitflags! {
    /// Flags associated with a CAN message.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct MessageFlags: u32 {
        /// Remote Transmission Request.
        const RTR = sys::canMSG_RTR;
        /// Standard (11-bit) identifier.
        const STD = sys::canMSG_STD;
        /// Extended (29-bit) identifier.
        const EXT = sys::canMSG_EXT;
        /// Wakeup message.
        const WAKEUP = sys::canMSG_WAKEUP;
        /// NERR was active during the message.
        const NERR = sys::canMSG_NERR;
        /// Error frame.
        const ERROR_FRAME = sys::canMSG_ERROR_FRAME;
        /// TX acknowledgement.
        const TX_ACK = sys::canMSG_TXACK;
        /// TX request.
        const TX_RQ = sys::canMSG_TXRQ;
        /// Delayed message.
        const DELAY_MSG = sys::canMSG_DELAY_MSG;
        /// CAN FD frame.
        const FD = sys::canFDMSG_FDF;
        /// CAN FD Bit Rate Switch.
        const BRS = sys::canFDMSG_BRS;
        /// CAN FD Error State Indicator.
        const ESI = sys::canFDMSG_ESI;
    }
}

impl Default for MessageFlags {
    fn default() -> Self {
        MessageFlags::empty()
    }
}

/// Maximum data length for classic CAN.
pub const CAN_MAX_DLC: usize = 8;

/// Maximum data length for CAN FD.
pub const CANFD_MAX_DLC: usize = 64;

/// A CAN message (classic or FD).
#[derive(Debug, Clone)]
pub struct CanMessage {
    /// CAN identifier (11-bit or 29-bit).
    pub id: u32,
    /// Message data payload (up to 8 bytes for classic CAN, up to 64 for FD).
    pub data: Vec<u8>,
    /// Data Length Code.
    pub dlc: u8,
    /// Message flags.
    pub flags: MessageFlags,
    /// Hardware timestamp in microseconds (populated on receive, `None` for transmit).
    pub timestamp: Option<u64>,
}

impl CanMessage {
    /// Create a new standard (11-bit ID) CAN message.
    pub fn new(id: u32, data: &[u8]) -> Self {
        Self {
            id,
            data: data.to_vec(),
            dlc: data.len() as u8,
            flags: MessageFlags::STD,
            timestamp: None,
        }
    }

    /// Create a new extended (29-bit ID) CAN message.
    pub fn new_extended(id: u32, data: &[u8]) -> Self {
        Self {
            id,
            data: data.to_vec(),
            dlc: data.len() as u8,
            flags: MessageFlags::EXT,
            timestamp: None,
        }
    }

    /// Create a new CAN FD message.
    pub fn new_fd(id: u32, data: &[u8], brs: bool) -> Self {
        let mut flags = MessageFlags::FD;
        if brs {
            flags |= MessageFlags::BRS;
        }
        Self {
            id,
            data: data.to_vec(),
            dlc: data.len() as u8,
            flags,
            timestamp: None,
        }
    }

    /// Returns true if this is a CAN FD message.
    pub fn is_fd(&self) -> bool {
        self.flags.contains(MessageFlags::FD)
    }

    /// Returns true if this is an extended (29-bit) frame.
    pub fn is_extended(&self) -> bool {
        self.flags.contains(MessageFlags::EXT)
    }

    /// Returns true if this is a Remote Transmission Request.
    pub fn is_rtr(&self) -> bool {
        self.flags.contains(MessageFlags::RTR)
    }

    /// Returns true if this is an error frame.
    pub fn is_error_frame(&self) -> bool {
        self.flags.contains(MessageFlags::ERROR_FRAME)
    }
}
