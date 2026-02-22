use std::time::Duration;

use canlib_sys as sys;

use crate::bus_params::{Bitrate, BusParams, BusParamsTq, DriverType};
use crate::error::{check_handle, check_status, CanError, Result};
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

/// A handle to an open CAN channel.
///
/// The channel is automatically taken off-bus and closed when dropped.
///
/// # Thread safety
///
/// CANLib handles are per-thread. `Channel` is `Send` but not `Sync`.
/// Each thread that needs CAN access must open its own channel.
pub struct Channel {
    handle: sys::canHandle,
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
        crate::ensure_initialized();
        let handle = unsafe { sys::canOpenChannel(channel_num, flags.bits()) };
        check_handle(handle)?;
        Ok(Self {
            handle,
            on_bus: false,
        })
    }

    /// Go on-bus. The channel must have bus parameters configured first.
    pub fn bus_on(&mut self) -> Result<()> {
        check_status(unsafe { sys::canBusOn(self.handle) })?;
        self.on_bus = true;
        Ok(())
    }

    /// Go off-bus.
    pub fn bus_off(&mut self) -> Result<()> {
        check_status(unsafe { sys::canBusOff(self.handle) })?;
        self.on_bus = false;
        Ok(())
    }

    /// Reset the CAN bus controller.
    pub fn reset_bus(&mut self) -> Result<()> {
        check_status(unsafe { sys::canResetBus(self.handle) })?;
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
        check_status(unsafe {
            sys::canSetBusParams(self.handle, bitrate.to_raw(), 0, 0, 0, 0, 0)
        })
    }

    /// Set custom bus parameters for classic CAN.
    pub fn set_bus_params(&self, params: &BusParams) -> Result<()> {
        check_status(unsafe {
            sys::canSetBusParams(
                self.handle,
                params.freq as std::os::raw::c_long,
                params.tseg1,
                params.tseg2,
                params.sjw,
                params.no_samp,
                params.sync_mode,
            )
        })
    }

    /// Set bus parameters using time quanta.
    pub fn set_bus_params_tq(&self, params: &BusParamsTq) -> Result<()> {
        check_status(unsafe { sys::canSetBusParamsTq(self.handle, params.to_raw()) })
    }

    /// Set CAN FD data-phase bus parameters.
    pub fn set_bus_params_fd(
        &self,
        freq_brs: i64,
        tseg1: u32,
        tseg2: u32,
        sjw: u32,
    ) -> Result<()> {
        check_status(unsafe {
            sys::canSetBusParamsFd(
                self.handle,
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
        check_status(unsafe {
            sys::canSetBusParamsFdTq(self.handle, arbitration.to_raw(), data.to_raw())
        })
    }

    /// Get the current bus parameters.
    pub fn get_bus_params(&self) -> Result<BusParams> {
        let mut freq: std::os::raw::c_long = 0;
        let mut tseg1: u32 = 0;
        let mut tseg2: u32 = 0;
        let mut sjw: u32 = 0;
        let mut no_samp: u32 = 0;
        let mut sync_mode: u32 = 0;
        check_status(unsafe {
            sys::canGetBusParams(
                self.handle,
                &mut freq,
                &mut tseg1,
                &mut tseg2,
                &mut sjw,
                &mut no_samp,
                &mut sync_mode,
            )
        })?;
        Ok(BusParams {
            freq: freq as i64,
            tseg1,
            tseg2,
            sjw,
            no_samp,
            sync_mode,
        })
    }

    /// Get bus parameters in time quanta format.
    pub fn get_bus_params_tq(&self) -> Result<BusParamsTq> {
        let mut raw = sys::kvBusParamsTq::default();
        check_status(unsafe { sys::canGetBusParamsTq(self.handle, &mut raw) })?;
        Ok(BusParamsTq::from_raw(raw))
    }

    // ---- Driver mode ----

    /// Set the CAN transceiver driver type.
    pub fn set_output_control(&self, driver: DriverType) -> Result<()> {
        check_status(unsafe { sys::canSetBusOutputControl(self.handle, driver.to_raw()) })
    }

    /// Get the current CAN transceiver driver type.
    pub fn get_output_control(&self) -> Result<DriverType> {
        let mut raw: u32 = 0;
        check_status(unsafe { sys::canGetBusOutputControl(self.handle, &mut raw) })?;
        DriverType::from_raw(raw).ok_or(CanError::Unknown(raw as i32))
    }

    // ---- Write ----

    /// Send a CAN message.
    pub fn write(&self, msg: &CanMessage) -> Result<()> {
        let mut data = [0u8; CANFD_MAX_DLC];
        let len = msg.data.len().min(CANFD_MAX_DLC);
        data[..len].copy_from_slice(&msg.data[..len]);

        check_status(unsafe {
            sys::canWrite(
                self.handle,
                msg.id as std::os::raw::c_long,
                data.as_mut_ptr() as *mut std::os::raw::c_void,
                msg.dlc as u32,
                msg.flags.bits(),
            )
        })
    }

    /// Send a CAN message and wait for it to be transmitted.
    pub fn write_wait(&self, msg: &CanMessage, timeout: Duration) -> Result<()> {
        let mut data = [0u8; CANFD_MAX_DLC];
        let len = msg.data.len().min(CANFD_MAX_DLC);
        data[..len].copy_from_slice(&msg.data[..len]);

        check_status(unsafe {
            sys::canWriteWait(
                self.handle,
                msg.id as std::os::raw::c_long,
                data.as_mut_ptr() as *mut std::os::raw::c_void,
                msg.dlc as u32,
                msg.flags.bits(),
                timeout.as_millis() as std::os::raw::c_ulong,
            )
        })
    }

    /// Wait for all messages in the transmit queue to be sent.
    pub fn write_sync(&self, timeout: Duration) -> Result<()> {
        check_status(unsafe {
            sys::canWriteSync(self.handle, timeout.as_millis() as std::os::raw::c_ulong)
        })
    }

    // ---- Read ----

    /// Read the next message from the receive queue.
    ///
    /// Returns `Err(CanError::NoMsg)` if no message is available.
    pub fn read(&self) -> Result<CanMessage> {
        let mut id: std::os::raw::c_long = 0;
        let mut data = [0u8; CANFD_MAX_DLC];
        let mut dlc: u32 = 0;
        let mut flags: u32 = 0;
        let mut timestamp: std::os::raw::c_ulong = 0;

        check_status(unsafe {
            sys::canRead(
                self.handle,
                &mut id,
                data.as_mut_ptr() as *mut std::os::raw::c_void,
                &mut dlc,
                &mut flags,
                &mut timestamp,
            )
        })?;

        let data_len = (dlc as usize).min(CANFD_MAX_DLC);
        Ok(CanMessage {
            id: id as u32,
            data: data[..data_len].to_vec(),
            dlc: dlc as u8,
            flags: MessageFlags::from_bits_truncate(flags),
            timestamp: Some(timestamp as u64),
        })
    }

    /// Read a message with a timeout.
    ///
    /// Blocks until a message arrives or the timeout expires.
    pub fn read_wait(&self, timeout: Duration) -> Result<CanMessage> {
        let mut id: std::os::raw::c_long = 0;
        let mut data = [0u8; CANFD_MAX_DLC];
        let mut dlc: u32 = 0;
        let mut flags: u32 = 0;
        let mut timestamp: std::os::raw::c_ulong = 0;

        check_status(unsafe {
            sys::canReadWait(
                self.handle,
                &mut id,
                data.as_mut_ptr() as *mut std::os::raw::c_void,
                &mut dlc,
                &mut flags,
                &mut timestamp,
                timeout.as_millis() as std::os::raw::c_ulong,
            )
        })?;

        let data_len = (dlc as usize).min(CANFD_MAX_DLC);
        Ok(CanMessage {
            id: id as u32,
            data: data[..data_len].to_vec(),
            dlc: dlc as u8,
            flags: MessageFlags::from_bits_truncate(flags),
            timestamp: Some(timestamp as u64),
        })
    }

    /// Wait until a message is available or the timeout expires.
    ///
    /// Does not consume the message — call [`read`](Self::read) afterwards.
    pub fn read_sync(&self, timeout: Duration) -> Result<()> {
        check_status(unsafe {
            sys::canReadSync(self.handle, timeout.as_millis() as std::os::raw::c_ulong)
        })
    }

    /// Read a message with a specific CAN ID.
    ///
    /// Returns the next message matching `id`, or `Err(CanError::NoMsg)`.
    pub fn read_specific(&self, id: u32) -> Result<CanMessage> {
        let mut data = [0u8; CANFD_MAX_DLC];
        let mut dlc: u32 = 0;
        let mut flags: u32 = 0;
        let mut timestamp: std::os::raw::c_ulong = 0;

        check_status(unsafe {
            sys::canReadSpecific(
                self.handle,
                id as std::os::raw::c_long,
                data.as_mut_ptr() as *mut std::os::raw::c_void,
                &mut dlc,
                &mut flags,
                &mut timestamp,
            )
        })?;

        let data_len = (dlc as usize).min(CANFD_MAX_DLC);
        Ok(CanMessage {
            id,
            data: data[..data_len].to_vec(),
            dlc: dlc as u8,
            flags: MessageFlags::from_bits_truncate(flags),
            timestamp: Some(timestamp as u64),
        })
    }

    /// Read a message with a specific ID, discarding non-matching messages.
    pub fn read_specific_skip(&self, id: u32) -> Result<CanMessage> {
        let mut data = [0u8; CANFD_MAX_DLC];
        let mut dlc: u32 = 0;
        let mut flags: u32 = 0;
        let mut timestamp: std::os::raw::c_ulong = 0;

        check_status(unsafe {
            sys::canReadSpecificSkip(
                self.handle,
                id as std::os::raw::c_long,
                data.as_mut_ptr() as *mut std::os::raw::c_void,
                &mut dlc,
                &mut flags,
                &mut timestamp,
            )
        })?;

        let data_len = (dlc as usize).min(CANFD_MAX_DLC);
        Ok(CanMessage {
            id,
            data: data[..data_len].to_vec(),
            dlc: dlc as u8,
            flags: MessageFlags::from_bits_truncate(flags),
            timestamp: Some(timestamp as u64),
        })
    }

    // ---- Acceptance filters ----

    /// Set an acceptance filter.
    ///
    /// `code` is the acceptance code, `mask` is the acceptance mask.
    /// If `extended` is true, the filter applies to 29-bit IDs; otherwise 11-bit.
    pub fn set_acceptance_filter(&self, code: u32, mask: u32, extended: bool) -> Result<()> {
        check_status(unsafe {
            sys::canSetAcceptanceFilter(self.handle, code, mask, extended as i32)
        })
    }

    // ---- Status & diagnostics ----

    /// Read the bus status flags.
    pub fn read_status(&self) -> Result<BusStatus> {
        let mut flags: std::os::raw::c_ulong = 0;
        check_status(unsafe { sys::canReadStatus(self.handle, &mut flags) })?;
        Ok(BusStatus::from_bits_truncate(flags as u64))
    }

    /// Read the error counters.
    pub fn read_error_counters(&self) -> Result<ErrorCounters> {
        let mut tx: u32 = 0;
        let mut rx: u32 = 0;
        let mut ov: u32 = 0;
        check_status(unsafe { sys::canReadErrorCounters(self.handle, &mut tx, &mut rx, &mut ov) })?;
        Ok(ErrorCounters {
            tx_errors: tx,
            rx_errors: rx,
            overrun_errors: ov,
        })
    }

    /// Request chip status from the hardware.
    pub fn request_chip_status(&self) -> Result<()> {
        check_status(unsafe { sys::canRequestChipStatus(self.handle) })
    }

    /// Request bus statistics from the hardware.
    pub fn request_bus_statistics(&self) -> Result<()> {
        check_status(unsafe { sys::canRequestBusStatistics(self.handle) })
    }

    /// Get the bus statistics (call [`request_bus_statistics`](Self::request_bus_statistics) first).
    pub fn get_bus_statistics(&self) -> Result<BusStatistics> {
        let mut raw = sys::canBusStatistics::default();
        check_status(unsafe {
            sys::canGetBusStatistics(
                self.handle,
                &mut raw,
                std::mem::size_of::<sys::canBusStatistics>(),
            )
        })?;
        Ok(BusStatistics::from_raw(&raw))
    }

    // ---- Queue management ----

    /// Flush the receive message queue.
    pub fn flush_rx(&self) -> Result<()> {
        check_status(unsafe { sys::canFlushReceiveQueue(self.handle) })
    }

    /// Flush the transmit message queue.
    pub fn flush_tx(&self) -> Result<()> {
        check_status(unsafe { sys::canFlushTransmitQueue(self.handle) })
    }

    /// Get the raw CANLib handle (for advanced use with `canlib-sys`).
    pub fn raw_handle(&self) -> sys::canHandle {
        self.handle
    }
}

impl Drop for Channel {
    fn drop(&mut self) {
        if self.on_bus {
            unsafe {
                sys::canBusOff(self.handle);
            }
        }
        unsafe {
            sys::canClose(self.handle);
        }
    }
}
