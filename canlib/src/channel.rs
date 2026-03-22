use std::time::Duration;

use canlib_sys as sys;

use crate::bus_params::{Bitrate, BusParams, BusParamsTq, DriverType, FdBitrate};
use crate::error::{check_handle, check_status, lib, CanError, Result};
use crate::message::{CanMessage, MessageFlags, CANFD_MAX_DLC};
use crate::status::{BusStatistics, BusStatus, ErrorCounters};

bitflags::bitflags! {
    /// Flags for opening a CAN channel.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct OpenFlags: i32 {
        /// Don't allow sharing of this channel.
        const EXCLUSIVE = sys::canOPEN_EXCLUSIVE;
        /// Require extended CAN (2.0B) support.
        const REQUIRE_EXTENDED = sys::canOPEN_REQUIRE_EXTENDED;
        /// Allow opening virtual channels.
        const ACCEPT_VIRTUAL = sys::canOPEN_ACCEPT_VIRTUAL;
        /// Override exclusive access from other handles.
        const OVERRIDE_EXCLUSIVE = sys::canOPEN_OVERRIDE_EXCLUSIVE;
        /// Require init access to set bus parameters.
        const REQUIRE_INIT_ACCESS = sys::canOPEN_REQUIRE_INIT_ACCESS;
        /// Open without init access.
        const NO_INIT_ACCESS = sys::canOPEN_NO_INIT_ACCESS;
        /// Accept DLC values greater than 8.
        const ACCEPT_LARGE_DLC = sys::canOPEN_ACCEPT_LARGE_DLC;
        /// Open for CAN FD (ISO 11898-1:2015).
        const CAN_FD = sys::canOPEN_CAN_FD;
        /// Open for CAN FD (non-ISO / Bosch).
        const CAN_FD_NONISO = sys::canOPEN_CAN_FD_NONISO;
    }
}

// ---------------------------------------------------------------------------
// Sub-traits
// ---------------------------------------------------------------------------

/// Bus control operations: on/off, bitrate, and bus parameters.
pub trait CanBusControl {
    fn bus_on(&mut self) -> Result<()>;
    fn bus_off(&mut self) -> Result<()>;
    fn reset_bus(&mut self) -> Result<()>;
    fn is_on_bus(&self) -> bool;
    fn set_bitrate(&self, bitrate: Bitrate) -> Result<()>;
    fn set_bus_params(&self, params: &BusParams) -> Result<()>;
}

/// Read operations on a CAN channel.
pub trait CanRead {
    fn read(&self) -> Result<CanMessage>;
    fn read_wait(&self, timeout: Duration) -> Result<CanMessage>;
    fn read_specific(&self, id: u32) -> Result<CanMessage>;
    fn read_specific_skip(&self, id: u32) -> Result<CanMessage>;
}

/// Write operations on a CAN channel.
pub trait CanWrite {
    fn write(&self, msg: &CanMessage) -> Result<()>;
    fn write_wait(&self, msg: &CanMessage, timeout: Duration) -> Result<()>;
    fn write_sync(&self, timeout: Duration) -> Result<()>;
}

/// Diagnostics: filters, status, error counters, and queue flushing.
pub trait CanDiagnostics {
    fn set_acceptance_filter(&self, code: u32, mask: u32, extended: bool) -> Result<()>;
    fn read_status(&self) -> Result<BusStatus>;
    fn read_error_counters(&self) -> Result<ErrorCounters>;
    fn flush_rx(&self) -> Result<()>;
    fn flush_tx(&self) -> Result<()>;
}

/// Combined trait encompassing all CAN channel operations.
pub trait CanChannel: CanBusControl + CanRead + CanWrite + CanDiagnostics {}

// Blanket impl: anything that implements all four sub-traits is a CanChannel.
impl<T: CanBusControl + CanRead + CanWrite + CanDiagnostics> CanChannel for T {}

// ---------------------------------------------------------------------------
// Delegation macro
// ---------------------------------------------------------------------------

/// Generate trait implementations for all four CAN sub-traits by delegating
/// to inherent methods of the same name on `$type`.
macro_rules! impl_can_channel {
    ($type:ty) => {
        impl CanBusControl for $type {
            fn bus_on(&mut self) -> Result<()> { <$type>::bus_on(self) }
            fn bus_off(&mut self) -> Result<()> { <$type>::bus_off(self) }
            fn reset_bus(&mut self) -> Result<()> { <$type>::reset_bus(self) }
            fn is_on_bus(&self) -> bool { <$type>::is_on_bus(self) }
            fn set_bitrate(&self, bitrate: Bitrate) -> Result<()> { <$type>::set_bitrate(self, bitrate) }
            fn set_bus_params(&self, params: &BusParams) -> Result<()> { <$type>::set_bus_params(self, params) }
        }

        impl CanRead for $type {
            fn read(&self) -> Result<CanMessage> { <$type>::read(self) }
            fn read_wait(&self, timeout: Duration) -> Result<CanMessage> { <$type>::read_wait(self, timeout) }
            fn read_specific(&self, id: u32) -> Result<CanMessage> { <$type>::read_specific(self, id) }
            fn read_specific_skip(&self, id: u32) -> Result<CanMessage> { <$type>::read_specific_skip(self, id) }
        }

        impl CanWrite for $type {
            fn write(&self, msg: &CanMessage) -> Result<()> { <$type>::write(self, msg) }
            fn write_wait(&self, msg: &CanMessage, timeout: Duration) -> Result<()> { <$type>::write_wait(self, msg, timeout) }
            fn write_sync(&self, timeout: Duration) -> Result<()> { <$type>::write_sync(self, timeout) }
        }

        impl CanDiagnostics for $type {
            fn set_acceptance_filter(&self, code: u32, mask: u32, extended: bool) -> Result<()> { <$type>::set_acceptance_filter(self, code, mask, extended) }
            fn read_status(&self) -> Result<BusStatus> { <$type>::read_status(self) }
            fn read_error_counters(&self) -> Result<ErrorCounters> { <$type>::read_error_counters(self) }
            fn flush_rx(&self) -> Result<()> { <$type>::flush_rx(self) }
            fn flush_tx(&self) -> Result<()> { <$type>::flush_tx(self) }
        }
    };
}

// ---------------------------------------------------------------------------
// ChannelHandle (RAII wrapper for the raw CANLib handle)
// ---------------------------------------------------------------------------

/// RAII wrapper around a raw `canHandle`.
///
/// Takes the channel off-bus and closes it when dropped, guaranteeing
/// cleanup even if the owning `Channel` is restructured in the future.
struct ChannelHandle {
    handle: sys::canHandle,
}

impl ChannelHandle {
    fn open(channel_num: i32, flags: OpenFlags) -> Result<Self> {
        crate::ensure_initialized()?;
        let l = lib()?;
        let handle = unsafe { (l.canOpenChannel)(channel_num, flags.bits()) };
        check_handle(handle)?;
        Ok(Self { handle })
    }

    fn raw(&self) -> sys::canHandle {
        self.handle
    }
}

impl Drop for ChannelHandle {
    fn drop(&mut self) {
        if let Ok(l) = lib() {
            unsafe {
                (l.canBusOff)(self.handle);
                (l.canClose)(self.handle);
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Channel (hardware-backed)
// ---------------------------------------------------------------------------

/// A handle to an open CAN channel.
///
/// The channel is automatically taken off-bus and closed when dropped
/// via the inner [`ChannelHandle`].
///
/// # Thread safety
///
/// CANLib handles are per-thread. `Channel` is `Send` but not `Sync`.
/// Each thread that needs CAN access must open its own channel.
pub struct Channel {
    inner: ChannelHandle,
    on_bus: bool,
}

// Channel can be moved to another thread, but cannot be shared across threads.
unsafe impl Send for Channel {}

impl Channel {
    /// Open a CAN channel.
    ///
    /// `channel_num` is the zero-based channel index.
    /// Use [`crate::get_number_of_channels`] to discover available channels.
    pub fn open(channel_num: i32, flags: OpenFlags) -> Result<Self> {
        Ok(Self {
            inner: ChannelHandle::open(channel_num, flags)?,
            on_bus: false,
        })
    }

    /// Go on-bus. The channel must have bus parameters configured first.
    pub fn bus_on(&mut self) -> Result<()> {
        let l = lib()?;
        check_status(unsafe { (l.canBusOn)(self.inner.raw()) })?;
        self.on_bus = true;
        Ok(())
    }

    /// Go off-bus.
    pub fn bus_off(&mut self) -> Result<()> {
        let l = lib()?;
        check_status(unsafe { (l.canBusOff)(self.inner.raw()) })?;
        self.on_bus = false;
        Ok(())
    }

    /// Reset the CAN bus controller.
    pub fn reset_bus(&mut self) -> Result<()> {
        let l = lib()?;
        check_status(unsafe { (l.canResetBus)(self.inner.raw()) })?;
        self.on_bus = false;
        Ok(())
    }

    /// Returns true if the channel is currently on-bus.
    pub fn is_on_bus(&self) -> bool {
        self.on_bus
    }

    // ---- Bus parameters ----

    /// Set the bitrate using a predefined constant.
    pub fn set_bitrate(&self, bitrate: Bitrate) -> Result<()> {
        let l = lib()?;
        check_status(unsafe {
            (l.canSetBusParams)(self.inner.raw(), bitrate.to_raw(), 0, 0, 0, 0, 0)
        })
    }

    /// Set custom bus parameters for classic CAN.
    pub fn set_bus_params(&self, params: &BusParams) -> Result<()> {
        let l = lib()?;
        check_status(unsafe {
            (l.canSetBusParams)(
                self.inner.raw(),
                params.freq() as std::os::raw::c_long,
                params.tseg1(),
                params.tseg2(),
                params.sjw(),
                params.no_samp(),
                params.sync_mode(),
            )
        })
    }

    /// Set bus parameters using time quanta.
    pub fn set_bus_params_tq(&self, params: &BusParamsTq) -> Result<()> {
        let l = lib()?;
        check_status(unsafe { (l.canSetBusParamsTq)(self.inner.raw(), params.to_raw()) })
    }

    /// Set the CAN FD data-phase bitrate using a predefined constant.
    pub fn set_fd_bitrate(&self, bitrate: FdBitrate) -> Result<()> {
        let l = lib()?;
        check_status(unsafe {
            (l.canSetBusParamsFd)(self.inner.raw(), bitrate.to_raw(), 0, 0, 0)
        })
    }

    /// Set CAN FD data-phase bus parameters.
    pub fn set_bus_params_fd(
        &self,
        freq_brs: i64,
        tseg1: u32,
        tseg2: u32,
        sjw: u32,
    ) -> Result<()> {
        let l = lib()?;
        check_status(unsafe {
            (l.canSetBusParamsFd)(
                self.inner.raw(),
                freq_brs as std::os::raw::c_long,
                tseg1,
                tseg2,
                sjw,
            )
        })
    }

    /// Set CAN FD bus parameters using time quanta (arbitration + data phases).
    pub fn set_bus_params_fd_tq(
        &self,
        arbitration: &BusParamsTq,
        data: &BusParamsTq,
    ) -> Result<()> {
        let l = lib()?;
        check_status(unsafe {
            (l.canSetBusParamsFdTq)(self.inner.raw(), arbitration.to_raw(), data.to_raw())
        })
    }

    /// Get the current bus parameters.
    pub fn get_bus_params(&self) -> Result<BusParams> {
        let l = lib()?;
        let mut freq: std::os::raw::c_long = 0;
        let mut tseg1: u32 = 0;
        let mut tseg2: u32 = 0;
        let mut sjw: u32 = 0;
        let mut no_samp: u32 = 0;
        let mut sync_mode: u32 = 0;
        check_status(unsafe {
            (l.canGetBusParams)(
                self.inner.raw(),
                &mut freq,
                &mut tseg1,
                &mut tseg2,
                &mut sjw,
                &mut no_samp,
                &mut sync_mode,
            )
        })?;
        Ok(BusParams::from_raw(freq as i64, tseg1, tseg2, sjw, no_samp, sync_mode))
    }

    /// Get bus parameters in time quanta format.
    pub fn get_bus_params_tq(&self) -> Result<BusParamsTq> {
        let l = lib()?;
        let mut raw = sys::kvBusParamsTq::default();
        check_status(unsafe { (l.canGetBusParamsTq)(self.inner.raw(), &mut raw) })?;
        Ok(BusParamsTq::from_raw(raw))
    }

    // ---- Driver mode ----

    /// Set the CAN transceiver driver type.
    pub fn set_output_control(&self, driver: DriverType) -> Result<()> {
        let l = lib()?;
        check_status(unsafe { (l.canSetBusOutputControl)(self.inner.raw(), driver.to_raw()) })
    }

    /// Get the current CAN transceiver driver type.
    pub fn get_output_control(&self) -> Result<DriverType> {
        let l = lib()?;
        let mut raw: u32 = 0;
        check_status(unsafe { (l.canGetBusOutputControl)(self.inner.raw(), &mut raw) })?;
        DriverType::from_raw(raw).ok_or(CanError::Unknown(raw as i32))
    }

    // ---- Write ----

    /// Send a CAN message.
    pub fn write(&self, msg: &CanMessage) -> Result<()> {
        let l = lib()?;
        let mut data = [0u8; CANFD_MAX_DLC];
        let src = msg.data();
        let len = src.len().min(CANFD_MAX_DLC);
        data[..len].copy_from_slice(&src[..len]);

        check_status(unsafe {
            (l.canWrite)(
                self.inner.raw(),
                msg.id() as std::os::raw::c_long,
                data.as_mut_ptr() as *mut std::os::raw::c_void,
                msg.dlc() as u32,
                msg.flags().bits(),
            )
        })
    }

    /// Send a CAN message and wait for it to be transmitted.
    pub fn write_wait(&self, msg: &CanMessage, timeout: Duration) -> Result<()> {
        let l = lib()?;
        let mut data = [0u8; CANFD_MAX_DLC];
        let src = msg.data();
        let len = src.len().min(CANFD_MAX_DLC);
        data[..len].copy_from_slice(&src[..len]);

        check_status(unsafe {
            (l.canWriteWait)(
                self.inner.raw(),
                msg.id() as std::os::raw::c_long,
                data.as_mut_ptr() as *mut std::os::raw::c_void,
                msg.dlc() as u32,
                msg.flags().bits(),
                timeout.as_millis() as std::os::raw::c_ulong,
            )
        })
    }

    /// Wait for all messages in the transmit queue to be sent.
    pub fn write_sync(&self, timeout: Duration) -> Result<()> {
        let l = lib()?;
        check_status(unsafe {
            (l.canWriteSync)(self.inner.raw(), timeout.as_millis() as std::os::raw::c_ulong)
        })
    }

    // ---- Read ----

    /// Read the next message from the receive queue.
    ///
    /// Returns `Err(CanError::NoMsg)` if no message is available.
    pub fn read(&self) -> Result<CanMessage> {
        let l = lib()?;
        let mut id: std::os::raw::c_long = 0;
        let mut data = [0u8; CANFD_MAX_DLC];
        let mut dlc: u32 = 0;
        let mut flags: u32 = 0;
        let mut timestamp: std::os::raw::c_ulong = 0;

        check_status(unsafe {
            (l.canRead)(
                self.inner.raw(),
                &mut id,
                data.as_mut_ptr() as *mut std::os::raw::c_void,
                &mut dlc,
                &mut flags,
                &mut timestamp,
            )
        })?;

        let data_len = (dlc as usize).min(CANFD_MAX_DLC);
        Ok(CanMessage::from_raw(
            id as u32,
            data[..data_len].to_vec(),
            dlc as u8,
            MessageFlags::from_bits_truncate(flags),
            timestamp as u64,
        ))
    }

    /// Read a message with a timeout.
    ///
    /// Blocks until a message arrives or the timeout expires.
    pub fn read_wait(&self, timeout: Duration) -> Result<CanMessage> {
        let l = lib()?;
        let mut id: std::os::raw::c_long = 0;
        let mut data = [0u8; CANFD_MAX_DLC];
        let mut dlc: u32 = 0;
        let mut flags: u32 = 0;
        let mut timestamp: std::os::raw::c_ulong = 0;

        check_status(unsafe {
            (l.canReadWait)(
                self.inner.raw(),
                &mut id,
                data.as_mut_ptr() as *mut std::os::raw::c_void,
                &mut dlc,
                &mut flags,
                &mut timestamp,
                timeout.as_millis() as std::os::raw::c_ulong,
            )
        })?;

        let data_len = (dlc as usize).min(CANFD_MAX_DLC);
        Ok(CanMessage::from_raw(
            id as u32,
            data[..data_len].to_vec(),
            dlc as u8,
            MessageFlags::from_bits_truncate(flags),
            timestamp as u64,
        ))
    }

    /// Wait until a message is available or the timeout expires.
    ///
    /// Does not consume the message — call [`read`](Self::read) afterwards.
    pub fn read_sync(&self, timeout: Duration) -> Result<()> {
        let l = lib()?;
        check_status(unsafe {
            (l.canReadSync)(self.inner.raw(), timeout.as_millis() as std::os::raw::c_ulong)
        })
    }

    /// Read a message with a specific CAN ID.
    ///
    /// Returns the next message matching `id`, or `Err(CanError::NoMsg)`.
    pub fn read_specific(&self, id: u32) -> Result<CanMessage> {
        let l = lib()?;
        let mut data = [0u8; CANFD_MAX_DLC];
        let mut dlc: u32 = 0;
        let mut flags: u32 = 0;
        let mut timestamp: std::os::raw::c_ulong = 0;

        check_status(unsafe {
            (l.canReadSpecific)(
                self.inner.raw(),
                id as std::os::raw::c_long,
                data.as_mut_ptr() as *mut std::os::raw::c_void,
                &mut dlc,
                &mut flags,
                &mut timestamp,
            )
        })?;

        let data_len = (dlc as usize).min(CANFD_MAX_DLC);
        Ok(CanMessage::from_raw(
            id,
            data[..data_len].to_vec(),
            dlc as u8,
            MessageFlags::from_bits_truncate(flags),
            timestamp as u64,
        ))
    }

    /// Read a message with a specific ID, discarding non-matching messages.
    pub fn read_specific_skip(&self, id: u32) -> Result<CanMessage> {
        let l = lib()?;
        let mut data = [0u8; CANFD_MAX_DLC];
        let mut dlc: u32 = 0;
        let mut flags: u32 = 0;
        let mut timestamp: std::os::raw::c_ulong = 0;

        check_status(unsafe {
            (l.canReadSpecificSkip)(
                self.inner.raw(),
                id as std::os::raw::c_long,
                data.as_mut_ptr() as *mut std::os::raw::c_void,
                &mut dlc,
                &mut flags,
                &mut timestamp,
            )
        })?;

        let data_len = (dlc as usize).min(CANFD_MAX_DLC);
        Ok(CanMessage::from_raw(
            id,
            data[..data_len].to_vec(),
            dlc as u8,
            MessageFlags::from_bits_truncate(flags),
            timestamp as u64,
        ))
    }

    // ---- Acceptance filters ----

    /// Set an acceptance filter.
    ///
    /// `code` is the acceptance code, `mask` is the acceptance mask.
    /// If `extended` is true, the filter applies to 29-bit IDs; otherwise 11-bit.
    pub fn set_acceptance_filter(&self, code: u32, mask: u32, extended: bool) -> Result<()> {
        let l = lib()?;
        check_status(unsafe {
            (l.canSetAcceptanceFilter)(self.inner.raw(), code, mask, extended as i32)
        })
    }

    // ---- Status & diagnostics ----

    /// Read the bus status flags.
    pub fn read_status(&self) -> Result<BusStatus> {
        let l = lib()?;
        let mut flags: std::os::raw::c_ulong = 0;
        check_status(unsafe { (l.canReadStatus)(self.inner.raw(), &mut flags) })?;
        Ok(BusStatus::from_bits_truncate(flags as u64))
    }

    /// Read the error counters.
    pub fn read_error_counters(&self) -> Result<ErrorCounters> {
        let l = lib()?;
        let mut tx: u32 = 0;
        let mut rx: u32 = 0;
        let mut ov: u32 = 0;
        check_status(unsafe { (l.canReadErrorCounters)(self.inner.raw(), &mut tx, &mut rx, &mut ov) })?;
        Ok(ErrorCounters {
            tx_errors: tx,
            rx_errors: rx,
            overrun_errors: ov,
        })
    }

    /// Request chip status from the hardware.
    pub fn request_chip_status(&self) -> Result<()> {
        let l = lib()?;
        check_status(unsafe { (l.canRequestChipStatus)(self.inner.raw()) })
    }

    /// Request bus statistics from the hardware.
    pub fn request_bus_statistics(&self) -> Result<()> {
        let l = lib()?;
        check_status(unsafe { (l.canRequestBusStatistics)(self.inner.raw()) })
    }

    /// Get the bus statistics (call [`request_bus_statistics`](Self::request_bus_statistics) first).
    pub fn get_bus_statistics(&self) -> Result<BusStatistics> {
        let l = lib()?;
        let mut raw = sys::canBusStatistics::default();
        check_status(unsafe {
            (l.canGetBusStatistics)(
                self.inner.raw(),
                &mut raw,
                std::mem::size_of::<sys::canBusStatistics>(),
            )
        })?;
        Ok(BusStatistics::from_raw(&raw))
    }

    // ---- Queue management ----

    /// Flush the receive message queue.
    pub fn flush_rx(&self) -> Result<()> {
        let l = lib()?;
        check_status(unsafe { (l.canFlushReceiveQueue)(self.inner.raw()) })
    }

    /// Flush the transmit message queue.
    pub fn flush_tx(&self) -> Result<()> {
        let l = lib()?;
        check_status(unsafe { (l.canFlushTransmitQueue)(self.inner.raw()) })
    }

    /// Get the raw CANLib handle (for advanced use with `canlib-sys`).
    pub fn raw_handle(&self) -> sys::canHandle {
        self.inner.raw()
    }
}

impl_can_channel!(Channel);

// ---------------------------------------------------------------------------
// MockChannel (test only)
// ---------------------------------------------------------------------------

#[cfg(test)]
pub(crate) mod mock {
    use super::*;
    use std::cell::RefCell;
    use std::collections::VecDeque;

    /// A mock CAN channel for unit testing.
    pub struct MockChannel {
        on_bus: bool,
        rx_queue: RefCell<VecDeque<CanMessage>>,
        tx_log: RefCell<Vec<CanMessage>>,
    }

    impl MockChannel {
        pub fn new() -> Self {
            Self {
                on_bus: false,
                rx_queue: RefCell::new(VecDeque::new()),
                tx_log: RefCell::new(Vec::new()),
            }
        }

        /// Enqueue a message that will be returned by the next `read`.
        pub fn push_rx(&self, msg: CanMessage) {
            self.rx_queue.borrow_mut().push_back(msg);
        }

        /// Take all transmitted messages out of the log.
        pub fn take_tx_log(&self) -> Vec<CanMessage> {
            self.tx_log.borrow_mut().drain(..).collect()
        }
    }

    impl CanBusControl for MockChannel {
        fn bus_on(&mut self) -> Result<()> {
            self.on_bus = true;
            Ok(())
        }
        fn bus_off(&mut self) -> Result<()> {
            self.on_bus = false;
            Ok(())
        }
        fn reset_bus(&mut self) -> Result<()> {
            self.on_bus = false;
            Ok(())
        }
        fn is_on_bus(&self) -> bool {
            self.on_bus
        }
        fn set_bitrate(&self, _bitrate: Bitrate) -> Result<()> {
            Ok(())
        }
        fn set_bus_params(&self, _params: &BusParams) -> Result<()> {
            Ok(())
        }
    }

    impl CanRead for MockChannel {
        fn read(&self) -> Result<CanMessage> {
            self.rx_queue
                .borrow_mut()
                .pop_front()
                .ok_or(CanError::NoMsg)
        }
        fn read_wait(&self, _timeout: Duration) -> Result<CanMessage> {
            self.read()
        }
        fn read_specific(&self, id: u32) -> Result<CanMessage> {
            let mut q = self.rx_queue.borrow_mut();
            let pos = q.iter().position(|m| m.id() == id);
            match pos {
                Some(i) => Ok(q.remove(i).unwrap()),
                None => Err(CanError::NoMsg),
            }
        }
        fn read_specific_skip(&self, id: u32) -> Result<CanMessage> {
            self.read_specific(id)
        }
    }

    impl CanWrite for MockChannel {
        fn write(&self, msg: &CanMessage) -> Result<()> {
            self.tx_log.borrow_mut().push(msg.clone());
            Ok(())
        }
        fn write_wait(&self, msg: &CanMessage, _timeout: Duration) -> Result<()> {
            self.write(msg)
        }
        fn write_sync(&self, _timeout: Duration) -> Result<()> {
            Ok(())
        }
    }

    impl CanDiagnostics for MockChannel {
        fn set_acceptance_filter(&self, _code: u32, _mask: u32, _extended: bool) -> Result<()> {
            Ok(())
        }
        fn read_status(&self) -> Result<BusStatus> {
            Ok(BusStatus::empty())
        }
        fn read_error_counters(&self) -> Result<ErrorCounters> {
            Ok(ErrorCounters::default())
        }
        fn flush_rx(&self) -> Result<()> {
            self.rx_queue.borrow_mut().clear();
            Ok(())
        }
        fn flush_tx(&self) -> Result<()> {
            self.tx_log.borrow_mut().clear();
            Ok(())
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use crate::message::MessageFlags;

        #[test]
        fn bus_on_off_state() {
            let mut mock = MockChannel::new();
            assert!(!mock.is_on_bus());
            mock.bus_on().unwrap();
            assert!(mock.is_on_bus());
            mock.bus_off().unwrap();
            assert!(!mock.is_on_bus());
        }

        #[test]
        fn write_captures_to_tx_log() {
            let mock = MockChannel::new();
            let msg = CanMessage::new(0x100, &[1, 2, 3]).unwrap();
            CanWrite::write(&mock, &msg).unwrap();
            let log = mock.take_tx_log();
            assert_eq!(log.len(), 1);
            assert_eq!(log[0].id(), 0x100);
            assert_eq!(log[0].data(), &[1, 2, 3]);
        }

        #[test]
        fn read_returns_enqueued_messages() {
            let mock = MockChannel::new();
            let msg = CanMessage::from_raw(0x200, vec![0xAA], 1, MessageFlags::STD, 1000);
            mock.push_rx(msg);
            let rx = CanRead::read(&mock).unwrap();
            assert_eq!(rx.id(), 0x200);
            assert_eq!(rx.data(), &[0xAA]);
        }

        #[test]
        fn empty_read_returns_no_msg() {
            let mock = MockChannel::new();
            let err = CanRead::read(&mock).unwrap_err();
            assert_eq!(err, CanError::NoMsg);
        }

        #[test]
        fn dyn_can_channel_acceptance() {
            let mut mock = MockChannel::new();
            let ch: &mut dyn CanChannel = &mut mock;
            ch.bus_on().unwrap();
            assert!(ch.is_on_bus());
        }
    }
}
