use canlib_sys as sys;

use crate::error::{CanError, Result};

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
        /// Hardware receive buffer overrun (set on error frames).
        const ERR_HW_OVERRUN = sys::canMSGERR_HW_OVERRUN;
        /// Software receive buffer overrun (set on error frames).
        const ERR_SW_OVERRUN = sys::canMSGERR_SW_OVERRUN;
        /// Stuff error (set on error frames).
        const ERR_STUFF = sys::canMSGERR_STUFF;
        /// Form error (set on error frames).
        const ERR_FORM = sys::canMSGERR_FORM;
        /// CRC error (set on error frames).
        const ERR_CRC = sys::canMSGERR_CRC;
        /// Bit-0 error (dominant transmitted, recessive monitored).
        const ERR_BIT0 = sys::canMSGERR_BIT0;
        /// Bit-1 error (recessive transmitted, dominant monitored).
        const ERR_BIT1 = sys::canMSGERR_BIT1;
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

/// Valid CAN FD payload sizes above 8 bytes.
const CANFD_VALID_SIZES: [usize; 7] = [12, 16, 20, 24, 32, 48, 64];

/// Round a data length up to the nearest valid CAN FD DLC.
/// For lengths 0-8, returns the length as-is. For lengths 9-64,
/// returns the next valid FD size (12, 16, 20, 24, 32, 48, or 64).
fn fd_dlc_for_len(len: usize) -> usize {
    if len <= CAN_MAX_DLC {
        return len;
    }
    for &valid in &CANFD_VALID_SIZES {
        if len <= valid {
            return valid;
        }
    }
    CANFD_MAX_DLC
}

/// Maximum standard (11-bit) CAN identifier.
pub const CAN_STD_ID_MAX: u32 = 0x7FF;

/// Maximum extended (29-bit) CAN identifier.
pub const CAN_EXT_ID_MAX: u32 = 0x1FFF_FFFF;

/// A CAN data frame (classic or FD).
#[derive(Debug, Clone)]
pub struct DataFrame {
    id: u32,
    data: Vec<u8>,
    dlc: u8,
    flags: MessageFlags,
    timestamp: Option<u64>,
}

/// A CAN Remote Transmission Request frame.
#[derive(Debug, Clone)]
pub struct RemoteFrame {
    id: u32,
    dlc: u8,
    flags: MessageFlags,
    timestamp: Option<u64>,
}

/// A CAN error frame.
#[derive(Debug, Clone)]
pub struct ErrorFrame {
    id: u32,
    data: Vec<u8>,
    dlc: u8,
    flags: MessageFlags,
    timestamp: Option<u64>,
}

/// A CAN message (classic or FD).
///
/// Each variant enforces which fields and flag combinations are valid:
/// - `Data` — standard data frames (classic CAN and FD)
/// - `Remote` — Remote Transmission Request frames (no payload)
/// - `Error` — error frames from the controller
#[derive(Debug, Clone)]
pub enum CanMessage {
    /// A data frame (classic CAN or CAN FD).
    Data(DataFrame),
    /// A Remote Transmission Request frame.
    Remote(RemoteFrame),
    /// An error frame.
    Error(ErrorFrame),
}

impl CanMessage {
    /// Create a new standard (11-bit ID) CAN message.
    ///
    /// Returns `Err(CanError::Param)` if `id > 0x7FF` or `data.len() > 8`.
    pub fn new(id: u32, data: &[u8]) -> Result<Self> {
        if id > CAN_STD_ID_MAX {
            return Err(CanError::Param);
        }
        if data.len() > CAN_MAX_DLC {
            return Err(CanError::Param);
        }
        Ok(CanMessage::Data(DataFrame {
            id,
            data: data.to_vec(),
            dlc: data.len() as u8,
            flags: MessageFlags::STD,
            timestamp: None,
        }))
    }

    /// Create a new extended (29-bit ID) CAN message.
    ///
    /// Returns `Err(CanError::Param)` if `id > 0x1FFF_FFFF` or `data.len() > 8`.
    pub fn new_extended(id: u32, data: &[u8]) -> Result<Self> {
        if id > CAN_EXT_ID_MAX {
            return Err(CanError::Param);
        }
        if data.len() > CAN_MAX_DLC {
            return Err(CanError::Param);
        }
        Ok(CanMessage::Data(DataFrame {
            id,
            data: data.to_vec(),
            dlc: data.len() as u8,
            flags: MessageFlags::EXT,
            timestamp: None,
        }))
    }

    /// Create a new CAN FD message.
    ///
    /// The DLC is rounded up to the nearest valid CAN FD size
    /// (0-8, 12, 16, 20, 24, 32, 48, or 64) and the data payload
    /// is zero-padded to match.
    ///
    /// If `extended` is true, the message uses a 29-bit extended ID
    /// (up to `0x1FFF_FFFF`). Otherwise, only standard 11-bit IDs
    /// (up to `0x7FF`) are accepted.
    ///
    /// Returns `Err(CanError::Param)` if the ID exceeds the allowed
    /// range for the selected mode, or if `data.len() > 64`.
    pub fn new_fd(id: u32, data: &[u8], brs: bool, extended: bool) -> Result<Self> {
        if extended {
            if id > CAN_EXT_ID_MAX {
                return Err(CanError::Param);
            }
        } else {
            if id > CAN_STD_ID_MAX {
                return Err(CanError::Param);
            }
        }
        if data.len() > CANFD_MAX_DLC {
            return Err(CanError::Param);
        }
        let dlc = fd_dlc_for_len(data.len());
        let mut padded = vec![0u8; dlc];
        padded[..data.len()].copy_from_slice(data);

        let mut flags = MessageFlags::FD;
        if extended {
            flags |= MessageFlags::EXT;
        } else {
            flags |= MessageFlags::STD;
        }
        if brs {
            flags |= MessageFlags::BRS;
        }
        Ok(CanMessage::Data(DataFrame {
            id,
            data: padded,
            dlc: dlc as u8,
            flags,
            timestamp: None,
        }))
    }

    /// Create a new standard (11-bit ID) Remote Transmission Request frame.
    ///
    /// RTR frames carry no data payload — only the requested DLC.
    /// Returns `Err(CanError::Param)` if `id > 0x7FF` or `dlc > 8`.
    pub fn new_rtr(id: u32, dlc: u8) -> Result<Self> {
        if id > CAN_STD_ID_MAX {
            return Err(CanError::Param);
        }
        if dlc as usize > CAN_MAX_DLC {
            return Err(CanError::Param);
        }
        Ok(CanMessage::Remote(RemoteFrame {
            id,
            dlc,
            flags: MessageFlags::STD | MessageFlags::RTR,
            timestamp: None,
        }))
    }

    /// Create a new extended (29-bit ID) Remote Transmission Request frame.
    ///
    /// RTR frames carry no data payload — only the requested DLC.
    /// Returns `Err(CanError::Param)` if `id > 0x1FFF_FFFF` or `dlc > 8`.
    pub fn new_rtr_extended(id: u32, dlc: u8) -> Result<Self> {
        if id > CAN_EXT_ID_MAX {
            return Err(CanError::Param);
        }
        if dlc as usize > CAN_MAX_DLC {
            return Err(CanError::Param);
        }
        Ok(CanMessage::Remote(RemoteFrame {
            id,
            dlc,
            flags: MessageFlags::EXT | MessageFlags::RTR,
            timestamp: None,
        }))
    }

    /// Create a message from raw FFI values (no validation).
    ///
    /// Dispatches to the appropriate variant based on flags:
    /// - `ERROR_FRAME` → `CanMessage::Error`
    /// - `RTR` → `CanMessage::Remote` (data discarded)
    /// - otherwise → `CanMessage::Data`
    pub(crate) fn from_raw(id: u32, data: Vec<u8>, dlc: u8, flags: MessageFlags, timestamp: u64) -> Self {
        if flags.contains(MessageFlags::ERROR_FRAME) {
            CanMessage::Error(ErrorFrame {
                id,
                data,
                dlc,
                flags,
                timestamp: Some(timestamp),
            })
        } else if flags.contains(MessageFlags::RTR) {
            CanMessage::Remote(RemoteFrame {
                id,
                dlc,
                flags,
                timestamp: Some(timestamp),
            })
        } else {
            CanMessage::Data(DataFrame {
                id,
                data,
                dlc,
                flags,
                timestamp: Some(timestamp),
            })
        }
    }

    /// CAN identifier (11-bit or 29-bit).
    pub fn id(&self) -> u32 {
        match self {
            CanMessage::Data(f) => f.id,
            CanMessage::Remote(f) => f.id,
            CanMessage::Error(f) => f.id,
        }
    }

    /// Message data payload. Returns an empty slice for RTR frames.
    pub fn data(&self) -> &[u8] {
        match self {
            CanMessage::Data(f) => &f.data,
            CanMessage::Remote(_) => &[],
            CanMessage::Error(f) => &f.data,
        }
    }

    /// Data Length Code.
    pub fn dlc(&self) -> u8 {
        match self {
            CanMessage::Data(f) => f.dlc,
            CanMessage::Remote(f) => f.dlc,
            CanMessage::Error(f) => f.dlc,
        }
    }

    /// Message flags.
    pub fn flags(&self) -> MessageFlags {
        match self {
            CanMessage::Data(f) => f.flags,
            CanMessage::Remote(f) => f.flags,
            CanMessage::Error(f) => f.flags,
        }
    }

    /// Hardware timestamp in microseconds (populated on receive).
    pub fn timestamp(&self) -> Option<u64> {
        match self {
            CanMessage::Data(f) => f.timestamp,
            CanMessage::Remote(f) => f.timestamp,
            CanMessage::Error(f) => f.timestamp,
        }
    }

    /// Returns true if this is a CAN FD message.
    pub fn is_fd(&self) -> bool {
        self.flags().contains(MessageFlags::FD)
    }

    /// Returns true if this is an extended (29-bit) frame.
    pub fn is_extended(&self) -> bool {
        self.flags().contains(MessageFlags::EXT)
    }

    /// Returns true if this is a Remote Transmission Request.
    pub fn is_rtr(&self) -> bool {
        matches!(self, CanMessage::Remote(_))
    }

    /// Returns true if this is an error frame.
    pub fn is_error_frame(&self) -> bool {
        matches!(self, CanMessage::Error(_))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_sets_std_flag_and_correct_dlc() {
        let msg = CanMessage::new(0x123, &[1, 2, 3, 4]).unwrap();
        assert_eq!(msg.id(), 0x123);
        assert_eq!(msg.data(), &[1, 2, 3, 4]);
        assert_eq!(msg.dlc(), 4);
        assert!(msg.flags().contains(MessageFlags::STD));
        assert!(!msg.is_extended());
        assert!(!msg.is_fd());
        assert!(msg.timestamp().is_none());
    }

    #[test]
    fn new_extended_sets_ext_flag() {
        let msg = CanMessage::new_extended(0x1FFFFFFF, &[0xAA, 0xBB]).unwrap();
        assert_eq!(msg.id(), 0x1FFFFFFF);
        assert!(msg.is_extended());
        assert!(msg.flags().contains(MessageFlags::EXT));
        assert!(!msg.flags().contains(MessageFlags::STD));
        assert_eq!(msg.dlc(), 2);
        assert!(msg.timestamp().is_none());
    }

    #[test]
    fn new_fd_sets_fd_flag_and_brs_when_requested() {
        let msg_no_brs = CanMessage::new_fd(0x100, &[1; 64], false, false).unwrap();
        assert!(msg_no_brs.is_fd());
        assert!(msg_no_brs.flags().contains(MessageFlags::FD));
        assert!(msg_no_brs.flags().contains(MessageFlags::STD));
        assert!(!msg_no_brs.flags().contains(MessageFlags::BRS));
        assert!(!msg_no_brs.is_extended());
        assert_eq!(msg_no_brs.dlc(), 64);

        let msg_brs = CanMessage::new_fd(0x100, &[2; 32], true, false).unwrap();
        assert!(msg_brs.is_fd());
        assert!(msg_brs.flags().contains(MessageFlags::FD));
        assert!(msg_brs.flags().contains(MessageFlags::STD));
        assert!(msg_brs.flags().contains(MessageFlags::BRS));
        assert!(!msg_brs.is_extended());
        assert_eq!(msg_brs.dlc(), 32);
    }

    #[test]
    fn new_fd_extended_sets_ext_flag() {
        let msg = CanMessage::new_fd(0x1FFF_FFFF, &[0xAA; 16], true, true).unwrap();
        assert!(msg.is_fd());
        assert!(msg.is_extended());
        assert!(msg.flags().contains(MessageFlags::EXT));
        assert!(!msg.flags().contains(MessageFlags::STD));
        assert_eq!(msg.id(), 0x1FFF_FFFF);
    }

    #[test]
    fn new_fd_rejects_extended_id_without_extended_flag() {
        assert!(CanMessage::new_fd(0x800, &[1, 2, 3], false, false).is_err());
    }

    #[test]
    fn new_fd_extended_rejects_id_over_ext_max() {
        assert!(CanMessage::new_fd(0x2000_0000, &[], false, true).is_err());
    }

    #[test]
    fn fd_dlc_rounds_up_to_valid_sizes() {
        // Exact valid sizes stay as-is
        for &exact in &[0, 1, 2, 3, 4, 5, 6, 7, 8, 12, 16, 20, 24, 32, 48, 64] {
            let msg = CanMessage::new_fd(0x100, &vec![0xAA; exact], false, false).unwrap();
            assert_eq!(msg.dlc() as usize, exact, "exact size {} should not change", exact);
            assert_eq!(msg.data().len(), exact);
        }

        // In-between sizes round up and zero-pad
        let cases: &[(usize, usize)] = &[
            (9, 12), (10, 12), (11, 12),
            (13, 16), (15, 16),
            (17, 20), (19, 20),
            (21, 24), (23, 24),
            (25, 32), (30, 32), (31, 32),
            (33, 48), (40, 48), (47, 48),
            (49, 64), (60, 64), (63, 64),
        ];
        for &(input_len, expected_dlc) in cases {
            let data: Vec<u8> = (0..input_len as u8).collect();
            let msg = CanMessage::new_fd(0x100, &data, false, false).unwrap();
            assert_eq!(
                msg.dlc() as usize, expected_dlc,
                "input len {} should round up to {}", input_len, expected_dlc
            );
            assert_eq!(msg.data().len(), expected_dlc);
            // Original data preserved
            assert_eq!(&msg.data()[..input_len], &data[..]);
            // Padding is zero
            for &b in &msg.data()[input_len..] {
                assert_eq!(b, 0, "padding should be zero for input len {}", input_len);
            }
        }
    }

    #[test]
    fn from_raw_with_rtr_creates_remote() {
        let msg = CanMessage::from_raw(0x1, vec![0xAA], 1, MessageFlags::STD | MessageFlags::RTR, 100);
        assert!(msg.is_rtr());
        assert!(!msg.is_error_frame());
        assert_eq!(msg.data(), &[]);
        assert_eq!(msg.dlc(), 1);
        assert_eq!(msg.timestamp(), Some(100));
    }

    #[test]
    fn from_raw_with_error_creates_error() {
        let msg = CanMessage::from_raw(0x1, vec![0xFF], 1, MessageFlags::ERROR_FRAME, 200);
        assert!(msg.is_error_frame());
        assert!(!msg.is_rtr());
        assert_eq!(msg.data(), &[0xFF]);
        assert_eq!(msg.timestamp(), Some(200));
    }

    #[test]
    fn from_raw_data_creates_data() {
        let msg = CanMessage::from_raw(0x100, vec![1, 2, 3], 3, MessageFlags::STD, 300);
        assert!(!msg.is_rtr());
        assert!(!msg.is_error_frame());
        assert_eq!(msg.id(), 0x100);
        assert_eq!(msg.data(), &[1, 2, 3]);
        assert_eq!(msg.timestamp(), Some(300));
    }

    #[test]
    fn from_raw_error_takes_priority_over_rtr() {
        // If both ERROR_FRAME and RTR are set, ERROR_FRAME wins
        let msg = CanMessage::from_raw(0x1, vec![], 0, MessageFlags::ERROR_FRAME | MessageFlags::RTR, 0);
        assert!(msg.is_error_frame());
        assert!(!msg.is_rtr());
    }

    #[test]
    fn new_rtr_validation() {
        let msg = CanMessage::new_rtr(0x100, 4).unwrap();
        assert!(msg.is_rtr());
        assert!(!msg.is_extended());
        assert_eq!(msg.id(), 0x100);
        assert_eq!(msg.dlc(), 4);
        assert_eq!(msg.data(), &[]);
        assert!(msg.flags().contains(MessageFlags::RTR));
        assert!(msg.flags().contains(MessageFlags::STD));

        // Rejects invalid ID
        assert!(CanMessage::new_rtr(0x800, 0).is_err());
        // Rejects invalid DLC
        assert!(CanMessage::new_rtr(0x100, 9).is_err());
    }

    #[test]
    fn new_rtr_extended_validation() {
        let msg = CanMessage::new_rtr_extended(0x1FFF_FFFF, 8).unwrap();
        assert!(msg.is_rtr());
        assert!(msg.is_extended());
        assert_eq!(msg.id(), 0x1FFF_FFFF);
        assert_eq!(msg.dlc(), 8);
        assert_eq!(msg.data(), &[]);
        assert!(msg.flags().contains(MessageFlags::RTR));
        assert!(msg.flags().contains(MessageFlags::EXT));

        // Rejects invalid ID
        assert!(CanMessage::new_rtr_extended(0x2000_0000, 0).is_err());
        // Rejects invalid DLC
        assert!(CanMessage::new_rtr_extended(0x100, 9).is_err());
    }

    #[test]
    fn pattern_match_variants() {
        let data_msg = CanMessage::new(0x100, &[1, 2]).unwrap();
        let rtr_msg = CanMessage::new_rtr(0x200, 4).unwrap();
        let error_msg = CanMessage::from_raw(0x0, vec![0xFF], 1, MessageFlags::ERROR_FRAME, 0);

        assert!(matches!(data_msg, CanMessage::Data(_)));
        assert!(matches!(rtr_msg, CanMessage::Remote(_)));
        assert!(matches!(error_msg, CanMessage::Error(_)));

        // Demonstrate destructuring
        if let CanMessage::Data(frame) = &data_msg {
            assert_eq!(frame.id, 0x100);
        } else {
            panic!("expected Data variant");
        }
    }

    #[test]
    fn message_flags_default_is_empty() {
        let flags = MessageFlags::default();
        assert!(flags.is_empty());
        assert!(!flags.contains(MessageFlags::STD));
        assert!(!flags.contains(MessageFlags::EXT));
    }

    #[test]
    fn empty_data_message() {
        let msg = CanMessage::new(0x0, &[]).unwrap();
        assert_eq!(msg.dlc(), 0);
        assert!(msg.data().is_empty());
    }

    #[test]
    fn new_rejects_id_over_std_max() {
        assert!(CanMessage::new(0x800, &[]).is_err());
        assert!(CanMessage::new(0xFFFF_FFFF, &[]).is_err());
    }

    #[test]
    fn new_rejects_data_over_8_bytes() {
        assert!(CanMessage::new(0x100, &[0; 9]).is_err());
    }

    #[test]
    fn new_extended_rejects_id_over_ext_max() {
        assert!(CanMessage::new_extended(0x2000_0000, &[]).is_err());
    }

    #[test]
    fn new_extended_rejects_data_over_8_bytes() {
        assert!(CanMessage::new_extended(0x100, &[0; 9]).is_err());
    }

    #[test]
    fn new_fd_rejects_data_over_64_bytes() {
        assert!(CanMessage::new_fd(0x100, &[0; 65], false, false).is_err());
    }

    #[test]
    fn from_raw_bypasses_validation() {
        let msg = CanMessage::from_raw(0xFFFF_FFFF, vec![0; 100], 100, MessageFlags::STD, 12345);
        assert_eq!(msg.id(), 0xFFFF_FFFF);
        assert_eq!(msg.data().len(), 100);
        assert_eq!(msg.timestamp(), Some(12345));
    }
}
