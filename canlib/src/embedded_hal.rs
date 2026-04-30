//! [`embedded-can`] integration (gated by the `embedded-hal` feature).
//!
//! This module provides:
//! - [`EmbeddedCanError`], a wrapper that distinguishes software/driver errors
//!   ([`CanError`]) from on-the-wire CAN protocol errors decoded from received
//!   error frames.
//! - An [`embedded_can::Frame`] impl for [`CanMessage`].
//! - Impls of [`embedded_can::nb::Can`] and [`embedded_can::blocking::Can`] for
//!   the hardware [`crate::Channel`].
//!
//! # Caveats
//!
//! - `embedded_can::Frame` describes CAN 2.0 frames (≤ 8 byte payload). Constructing
//!   a frame via the trait will reject larger payloads. Frames received from the
//!   wire that happen to be CAN FD will still flow through the trait accessors
//!   correctly, but bear larger DLCs.
//! - `nb::Can::transmit` cannot perform priority replacement on Kvaser hardware,
//!   so it always returns `Ok(None)` on success.
//! - `blocking::Can::receive` blocks for an effectively infinite duration
//!   (~49 days, the maximum `c_ulong` ms accepted by `canReadWait`).

use core::time::Duration;

use embedded_can::ErrorKind;

use crate::channel::{CanRead, CanWrite, Channel};
use crate::error::CanError;
use crate::message::{CanMessage, MessageFlags, CAN_MAX_DLC};

/// Error type bridging CANLib errors and CAN bus protocol errors for `embedded-can`.
///
/// `Lib` carries software/driver-layer errors from the CANLib SDK
/// (timeouts, hardware faults, parameter validation, etc.).
///
/// `Bus` carries on-the-wire CAN protocol errors decoded from a received
/// error frame (`Stuff`, `Form`, `Crc`, `Bit`, `Overrun`).
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum EmbeddedCanError {
    /// Software/driver-layer error from CANLib.
    #[error("library error: {0}")]
    Lib(#[from] CanError),
    /// CAN bus protocol error decoded from a received error frame.
    #[error("bus protocol error: {0:?}")]
    Bus(ErrorKind),
}

impl embedded_can::Error for EmbeddedCanError {
    fn kind(&self) -> ErrorKind {
        match self {
            EmbeddedCanError::Lib(_) => ErrorKind::Other,
            EmbeddedCanError::Bus(k) => *k,
        }
    }
}

/// Decode the flags of an error frame into an `embedded_can::ErrorKind`.
///
/// Multiple bits may be set on a single error frame; the most specific
/// protocol error is preferred over `Other`. Overrun (HW or SW) takes
/// precedence to surface buffer-loss conditions explicitly.
pub fn decode_error_frame(flags: MessageFlags) -> ErrorKind {
    if flags.intersects(MessageFlags::ERR_HW_OVERRUN | MessageFlags::ERR_SW_OVERRUN) {
        ErrorKind::Overrun
    } else if flags.contains(MessageFlags::ERR_STUFF) {
        ErrorKind::Stuff
    } else if flags.contains(MessageFlags::ERR_FORM) {
        ErrorKind::Form
    } else if flags.contains(MessageFlags::ERR_CRC) {
        ErrorKind::Crc
    } else if flags.intersects(MessageFlags::ERR_BIT0 | MessageFlags::ERR_BIT1) {
        ErrorKind::Bit
    } else {
        ErrorKind::Other
    }
}

impl embedded_can::Frame for CanMessage {
    fn new(id: impl Into<embedded_can::Id>, data: &[u8]) -> Option<Self> {
        match id.into() {
            embedded_can::Id::Standard(s) => {
                CanMessage::new(u32::from(s.as_raw()), data).ok()
            }
            embedded_can::Id::Extended(e) => {
                CanMessage::new_extended(e.as_raw(), data).ok()
            }
        }
    }

    fn new_remote(id: impl Into<embedded_can::Id>, dlc: usize) -> Option<Self> {
        if dlc > CAN_MAX_DLC {
            return None;
        }
        match id.into() {
            embedded_can::Id::Standard(s) => {
                CanMessage::new_rtr(u32::from(s.as_raw()), dlc as u8).ok()
            }
            embedded_can::Id::Extended(e) => {
                CanMessage::new_rtr_extended(e.as_raw(), dlc as u8).ok()
            }
        }
    }

    fn is_extended(&self) -> bool {
        CanMessage::is_extended(self)
    }

    fn is_remote_frame(&self) -> bool {
        CanMessage::is_rtr(self)
    }

    fn id(&self) -> embedded_can::Id {
        let raw = CanMessage::id(self);
        if CanMessage::is_extended(self) {
            embedded_can::ExtendedId::new(raw)
                .map(embedded_can::Id::Extended)
                .unwrap_or(embedded_can::Id::Extended(embedded_can::ExtendedId::ZERO))
        } else {
            embedded_can::StandardId::new(raw as u16)
                .map(embedded_can::Id::Standard)
                .unwrap_or(embedded_can::Id::Standard(embedded_can::StandardId::ZERO))
        }
    }

    fn dlc(&self) -> usize {
        CanMessage::dlc(self) as usize
    }

    fn data(&self) -> &[u8] {
        CanMessage::data(self)
    }
}

/// Effectively-infinite timeout for blocking calls.
///
/// CANLib timeouts are `c_ulong` milliseconds; `u32::MAX` ms ≈ 49.7 days.
const FOREVER_MS: u64 = u32::MAX as u64;

/// Generic non-blocking implementation: read once and lift the
/// "no message" / "tx buffer full" cases into `nb::Error::WouldBlock`.
fn nb_transmit<C: CanWrite>(
    chan: &C,
    frame: &CanMessage,
) -> nb::Result<Option<CanMessage>, EmbeddedCanError> {
    match chan.write(frame) {
        Ok(()) => Ok(None),
        Err(CanError::TxBufOverflow) => Err(nb::Error::WouldBlock),
        Err(e) => Err(nb::Error::Other(EmbeddedCanError::Lib(e))),
    }
}

fn nb_receive<C: CanRead>(chan: &C) -> nb::Result<CanMessage, EmbeddedCanError> {
    match chan.read() {
        Ok(msg) if msg.is_error_frame() => Err(nb::Error::Other(
            EmbeddedCanError::Bus(decode_error_frame(msg.flags())),
        )),
        Ok(msg) => Ok(msg),
        Err(CanError::NoMsg) => Err(nb::Error::WouldBlock),
        Err(e) => Err(nb::Error::Other(EmbeddedCanError::Lib(e))),
    }
}

/// Generic blocking implementation: wait essentially forever on the
/// underlying channel; surface error frames as bus errors.
fn blocking_transmit<C: CanWrite>(
    chan: &C,
    frame: &CanMessage,
) -> Result<(), EmbeddedCanError> {
    loop {
        match chan.write_wait(frame, Duration::from_millis(FOREVER_MS)) {
            Ok(()) => return Ok(()),
            // Spurious timeout at the practical-infinity bound — keep waiting.
            Err(CanError::Timeout) => continue,
            Err(e) => return Err(EmbeddedCanError::Lib(e)),
        }
    }
}

fn blocking_receive<C: CanRead>(chan: &C) -> Result<CanMessage, EmbeddedCanError> {
    loop {
        match chan.read_wait(Duration::from_millis(FOREVER_MS)) {
            Ok(msg) if msg.is_error_frame() => {
                return Err(EmbeddedCanError::Bus(decode_error_frame(msg.flags())));
            }
            Ok(msg) => return Ok(msg),
            Err(CanError::Timeout) => continue,
            Err(e) => return Err(EmbeddedCanError::Lib(e)),
        }
    }
}

impl embedded_can::nb::Can for Channel {
    type Frame = CanMessage;
    type Error = EmbeddedCanError;

    fn transmit(
        &mut self,
        frame: &Self::Frame,
    ) -> nb::Result<Option<Self::Frame>, Self::Error> {
        nb_transmit(self, frame)
    }

    fn receive(&mut self) -> nb::Result<Self::Frame, Self::Error> {
        nb_receive(self)
    }
}

impl embedded_can::blocking::Can for Channel {
    type Frame = CanMessage;
    type Error = EmbeddedCanError;

    fn transmit(&mut self, frame: &Self::Frame) -> Result<(), Self::Error> {
        blocking_transmit(self, frame)
    }

    fn receive(&mut self) -> Result<Self::Frame, Self::Error> {
        blocking_receive(self)
    }
}

#[cfg(test)]
impl embedded_can::nb::Can for crate::channel::mock::MockChannel {
    type Frame = CanMessage;
    type Error = EmbeddedCanError;

    fn transmit(
        &mut self,
        frame: &Self::Frame,
    ) -> nb::Result<Option<Self::Frame>, Self::Error> {
        nb_transmit(self, frame)
    }

    fn receive(&mut self) -> nb::Result<Self::Frame, Self::Error> {
        nb_receive(self)
    }
}

#[cfg(test)]
impl embedded_can::blocking::Can for crate::channel::mock::MockChannel {
    type Frame = CanMessage;
    type Error = EmbeddedCanError;

    fn transmit(&mut self, frame: &Self::Frame) -> Result<(), Self::Error> {
        blocking_transmit(self, frame)
    }

    fn receive(&mut self) -> Result<Self::Frame, Self::Error> {
        blocking_receive(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::channel::mock::MockChannel;
    use embedded_can::{blocking::Can as BlockingCan, nb::Can as NbCan, Error as _, Frame as _, Id};

    // ---- EmbeddedCanError / kind() ----

    #[test]
    fn lib_errors_map_to_other_kind() {
        let e = EmbeddedCanError::Lib(CanError::Hardware);
        assert_eq!(e.kind(), ErrorKind::Other);

        let e = EmbeddedCanError::Lib(CanError::Timeout);
        assert_eq!(e.kind(), ErrorKind::Other);
    }

    #[test]
    fn bus_errors_pass_through_kind() {
        for k in [
            ErrorKind::Overrun,
            ErrorKind::Bit,
            ErrorKind::Stuff,
            ErrorKind::Crc,
            ErrorKind::Form,
            ErrorKind::Acknowledge,
            ErrorKind::Other,
        ] {
            assert_eq!(EmbeddedCanError::Bus(k).kind(), k);
        }
    }

    #[test]
    fn from_can_error_lifts_into_lib_variant() {
        let e: EmbeddedCanError = CanError::Driver.into();
        assert_eq!(e, EmbeddedCanError::Lib(CanError::Driver));
    }

    // ---- decode_error_frame ----

    #[test]
    fn decode_recognises_each_bus_error() {
        assert_eq!(decode_error_frame(MessageFlags::ERR_HW_OVERRUN), ErrorKind::Overrun);
        assert_eq!(decode_error_frame(MessageFlags::ERR_SW_OVERRUN), ErrorKind::Overrun);
        assert_eq!(decode_error_frame(MessageFlags::ERR_STUFF), ErrorKind::Stuff);
        assert_eq!(decode_error_frame(MessageFlags::ERR_FORM), ErrorKind::Form);
        assert_eq!(decode_error_frame(MessageFlags::ERR_CRC), ErrorKind::Crc);
        assert_eq!(decode_error_frame(MessageFlags::ERR_BIT0), ErrorKind::Bit);
        assert_eq!(decode_error_frame(MessageFlags::ERR_BIT1), ErrorKind::Bit);
    }

    #[test]
    fn decode_falls_back_to_other_when_no_specific_bit() {
        // ERROR_FRAME bit alone, no specific protocol bits
        assert_eq!(decode_error_frame(MessageFlags::ERROR_FRAME), ErrorKind::Other);
        assert_eq!(decode_error_frame(MessageFlags::empty()), ErrorKind::Other);
    }

    #[test]
    fn decode_overrun_takes_precedence() {
        // If overrun is set alongside a protocol-error bit, overrun wins
        // because buffer loss is a more actionable signal.
        let flags = MessageFlags::ERR_SW_OVERRUN | MessageFlags::ERR_STUFF;
        assert_eq!(decode_error_frame(flags), ErrorKind::Overrun);
    }

    // ---- Frame impl ----

    #[test]
    fn frame_new_with_standard_id() {
        let id = embedded_can::StandardId::new(0x123).unwrap();
        let f = <CanMessage as embedded_can::Frame>::new(id, &[1, 2, 3]).unwrap();
        assert!(!f.is_extended());
        assert!(!f.is_remote_frame());
        assert!(<CanMessage as embedded_can::Frame>::is_data_frame(&f));
        assert_eq!(<CanMessage as embedded_can::Frame>::dlc(&f), 3);
        assert_eq!(<CanMessage as embedded_can::Frame>::data(&f), &[1, 2, 3]);
        match <CanMessage as embedded_can::Frame>::id(&f) {
            Id::Standard(s) => assert_eq!(s.as_raw(), 0x123),
            Id::Extended(_) => panic!("expected standard id"),
        }
    }

    #[test]
    fn frame_new_with_extended_id() {
        let id = embedded_can::ExtendedId::new(0x1FFF_FFFF).unwrap();
        let f = <CanMessage as embedded_can::Frame>::new(id, &[0xAA]).unwrap();
        assert!(<CanMessage as embedded_can::Frame>::is_extended(&f));
        match <CanMessage as embedded_can::Frame>::id(&f) {
            Id::Extended(e) => assert_eq!(e.as_raw(), 0x1FFF_FFFF),
            Id::Standard(_) => panic!("expected extended id"),
        }
    }

    #[test]
    fn frame_new_rejects_oversized_data() {
        let id = embedded_can::StandardId::new(0x1).unwrap();
        // CAN 2.0: > 8 bytes is invalid
        let r = <CanMessage as embedded_can::Frame>::new(id, &[0u8; 9]);
        assert!(r.is_none());
    }

    #[test]
    fn frame_new_remote_standard_and_extended() {
        let std_id = embedded_can::StandardId::new(0x100).unwrap();
        let f = <CanMessage as embedded_can::Frame>::new_remote(std_id, 4).unwrap();
        assert!(f.is_remote_frame());
        assert!(!f.is_extended());
        assert_eq!(<CanMessage as embedded_can::Frame>::dlc(&f), 4);
        assert_eq!(<CanMessage as embedded_can::Frame>::data(&f), &[]);

        let ext_id = embedded_can::ExtendedId::new(0x1234).unwrap();
        let f = <CanMessage as embedded_can::Frame>::new_remote(ext_id, 8).unwrap();
        assert!(f.is_remote_frame());
        assert!(<CanMessage as embedded_can::Frame>::is_extended(&f));
        assert_eq!(<CanMessage as embedded_can::Frame>::dlc(&f), 8);
    }

    #[test]
    fn frame_new_remote_rejects_oversized_dlc() {
        let id = embedded_can::StandardId::new(0x1).unwrap();
        assert!(<CanMessage as embedded_can::Frame>::new_remote(id, 9).is_none());
    }

    // ---- nb::Can ----

    #[test]
    fn nb_receive_returns_would_block_when_empty() {
        let mut mock = MockChannel::new();
        let r: nb::Result<CanMessage, EmbeddedCanError> = NbCan::receive(&mut mock);
        assert!(matches!(r, Err(nb::Error::WouldBlock)));
    }

    #[test]
    fn nb_receive_returns_data_frame() {
        let mut mock = MockChannel::new();
        let id = embedded_can::StandardId::new(0x42).unwrap();
        let f = <CanMessage as embedded_can::Frame>::new(id, &[9, 8, 7]).unwrap();
        mock.push_rx(f);
        let got = NbCan::receive(&mut mock).unwrap();
        assert_eq!(<CanMessage as embedded_can::Frame>::data(&got), &[9, 8, 7]);
    }

    #[test]
    fn nb_receive_filters_error_frames_into_bus_errors() {
        let mut mock = MockChannel::new();
        // Construct a synthetic error frame with a CRC bit set.
        let err = CanMessage::from_raw(
            0,
            vec![],
            0,
            MessageFlags::ERROR_FRAME | MessageFlags::ERR_CRC,
            0,
        );
        mock.push_rx(err);
        let r = NbCan::receive(&mut mock);
        match r {
            Err(nb::Error::Other(EmbeddedCanError::Bus(ErrorKind::Crc))) => {}
            other => panic!("expected Bus(Crc), got {:?}", other),
        }
    }

    #[test]
    fn nb_transmit_records_frame_and_returns_none() {
        let mut mock = MockChannel::new();
        let id = embedded_can::StandardId::new(0x55).unwrap();
        let f = <CanMessage as embedded_can::Frame>::new(id, &[1, 2]).unwrap();
        let r = NbCan::transmit(&mut mock, &f).unwrap();
        assert!(r.is_none());
        let log = mock.take_tx_log();
        assert_eq!(log.len(), 1);
        assert_eq!(<CanMessage as embedded_can::Frame>::data(&log[0]), &[1, 2]);
    }

    // ---- blocking::Can ----

    #[test]
    fn blocking_receive_returns_data_frame() {
        let mut mock = MockChannel::new();
        let id = embedded_can::StandardId::new(0x77).unwrap();
        let f = <CanMessage as embedded_can::Frame>::new(id, &[0xDE, 0xAD]).unwrap();
        mock.push_rx(f);
        let got = BlockingCan::receive(&mut mock).unwrap();
        assert_eq!(<CanMessage as embedded_can::Frame>::data(&got), &[0xDE, 0xAD]);
    }

    #[test]
    fn blocking_receive_surfaces_bus_error() {
        let mut mock = MockChannel::new();
        let err = CanMessage::from_raw(
            0,
            vec![],
            0,
            MessageFlags::ERROR_FRAME | MessageFlags::ERR_BIT0,
            0,
        );
        mock.push_rx(err);
        match BlockingCan::receive(&mut mock) {
            Err(EmbeddedCanError::Bus(ErrorKind::Bit)) => {}
            other => panic!("expected Bus(Bit), got {:?}", other.err()),
        }
    }

    #[test]
    fn blocking_transmit_records_frame() {
        let mut mock = MockChannel::new();
        let id = embedded_can::StandardId::new(0x99).unwrap();
        let f = <CanMessage as embedded_can::Frame>::new(id, &[0xC0, 0xDE]).unwrap();
        BlockingCan::transmit(&mut mock, &f).unwrap();
        let log = mock.take_tx_log();
        assert_eq!(log.len(), 1);
        assert_eq!(<CanMessage as embedded_can::Frame>::data(&log[0]), &[0xC0, 0xDE]);
    }
}
