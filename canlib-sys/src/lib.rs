//! Raw FFI bindings to the Kvaser CANLib SDK.
//!
//! This crate provides unsafe, low-level bindings loaded at runtime via `libloading`.
//! For a safe, idiomatic Rust API, use the `canlib` crate instead.

#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(clippy::all)]

// Re-export commonly used types for convenience
pub type canHandle = ::std::os::raw::c_int;
pub type canStatus = ::std::os::raw::c_int;

// ---- Status codes ----

pub const canOK: canStatus = 0;
pub const canERR_PARAM: canStatus = -1;
pub const canERR_NOMSG: canStatus = -2;
pub const canERR_NOTFOUND: canStatus = -3;
pub const canERR_NOMEM: canStatus = -4;
pub const canERR_NOCHANNELS: canStatus = -5;
pub const canERR_INTERRUPTED: canStatus = -6;
pub const canERR_TIMEOUT: canStatus = -7;
pub const canERR_NOTINITIALIZED: canStatus = -8;
pub const canERR_NOHANDLES: canStatus = -9;
pub const canERR_INVHANDLE: canStatus = -10;
pub const canERR_INIFILE: canStatus = -11;
pub const canERR_DRIVER: canStatus = -12;
pub const canERR_TXBUFOFL: canStatus = -13;
pub const canERR_RESERVED_1: canStatus = -14;
pub const canERR_HARDWARE: canStatus = -15;
pub const canERR_DYNALOAD: canStatus = -16;
pub const canERR_DYNALIB: canStatus = -17;
pub const canERR_DYNAINIT: canStatus = -18;
pub const canERR_NOT_SUPPORTED: canStatus = -19;
pub const canERR_RESERVED_5: canStatus = -20;
pub const canERR_RESERVED_6: canStatus = -21;
pub const canERR_RESERVED_2: canStatus = -22;
pub const canERR_DRIVERLOAD: canStatus = -23;
pub const canERR_DRIVERFAILED: canStatus = -24;
pub const canERR_NOCONFIGMGR: canStatus = -25;
pub const canERR_NOCARD: canStatus = -26;
pub const canERR_RESERVED_7: canStatus = -27;
pub const canERR_REGISTRY: canStatus = -28;
pub const canERR_LICENSE: canStatus = -29;
pub const canERR_INTERNAL: canStatus = -30;
pub const canERR_NO_ACCESS: canStatus = -31;
pub const canERR_NOT_IMPLEMENTED: canStatus = -32;
pub const canERR_DEVICE_FILE: canStatus = -33;
pub const canERR_HOST_FILE: canStatus = -34;
pub const canERR_DISK: canStatus = -35;
pub const canERR_CRC: canStatus = -36;
pub const canERR_CONFIG: canStatus = -37;
pub const canERR_MEMO_FAIL: canStatus = -38;
pub const canERR_SCRIPT_FAIL: canStatus = -39;
pub const canERR_SCRIPT_WRONG_VERSION: canStatus = -40;

// ---- Open channel flags ----

pub const canOPEN_EXCLUSIVE: ::std::os::raw::c_int = 0x0008;
pub const canOPEN_REQUIRE_EXTENDED: ::std::os::raw::c_int = 0x0010;
pub const canOPEN_ACCEPT_VIRTUAL: ::std::os::raw::c_int = 0x0020;
pub const canOPEN_OVERRIDE_EXCLUSIVE: ::std::os::raw::c_int = 0x0040;
pub const canOPEN_REQUIRE_INIT_ACCESS: ::std::os::raw::c_int = 0x0080;
pub const canOPEN_NO_INIT_ACCESS: ::std::os::raw::c_int = 0x0100;
pub const canOPEN_ACCEPT_LARGE_DLC: ::std::os::raw::c_int = 0x0200;
pub const canOPEN_CAN_FD: ::std::os::raw::c_int = 0x0400;
pub const canOPEN_CAN_FD_NONISO: ::std::os::raw::c_int = 0x0800;

// ---- Predefined bitrates ----

pub const canBITRATE_1M: ::std::os::raw::c_long = -1;
pub const canBITRATE_500K: ::std::os::raw::c_long = -2;
pub const canBITRATE_250K: ::std::os::raw::c_long = -3;
pub const canBITRATE_125K: ::std::os::raw::c_long = -4;
pub const canBITRATE_100K: ::std::os::raw::c_long = -5;
pub const canBITRATE_62K: ::std::os::raw::c_long = -6;
pub const canBITRATE_50K: ::std::os::raw::c_long = -7;
pub const canBITRATE_83K: ::std::os::raw::c_long = -8;
pub const canBITRATE_10K: ::std::os::raw::c_long = -9;

pub const canFD_BITRATE_500K_80P: ::std::os::raw::c_long = -1000;
pub const canFD_BITRATE_1M_80P: ::std::os::raw::c_long = -1001;
pub const canFD_BITRATE_2M_80P: ::std::os::raw::c_long = -1002;
pub const canFD_BITRATE_4M_80P: ::std::os::raw::c_long = -1003;
pub const canFD_BITRATE_8M_60P: ::std::os::raw::c_long = -1004;
pub const canFD_BITRATE_8M_80P: ::std::os::raw::c_long = -1005;
pub const canFD_BITRATE_8M_70P: ::std::os::raw::c_long = -1006;

// ---- Message flags ----

pub const canMSG_RTR: ::std::os::raw::c_uint = 0x0001;
pub const canMSG_STD: ::std::os::raw::c_uint = 0x0002;
pub const canMSG_EXT: ::std::os::raw::c_uint = 0x0004;
pub const canMSG_WAKEUP: ::std::os::raw::c_uint = 0x0008;
pub const canMSG_NERR: ::std::os::raw::c_uint = 0x0010;
pub const canMSG_ERROR_FRAME: ::std::os::raw::c_uint = 0x0020;
pub const canMSG_TXACK: ::std::os::raw::c_uint = 0x0040;
pub const canMSG_TXRQ: ::std::os::raw::c_uint = 0x0080;
pub const canMSG_DELAY_MSG: ::std::os::raw::c_uint = 0x0100;

pub const canFDMSG_FDF: ::std::os::raw::c_uint = 0x010000;
pub const canFDMSG_BRS: ::std::os::raw::c_uint = 0x020000;
pub const canFDMSG_ESI: ::std::os::raw::c_uint = 0x040000;

// ---- Driver types ----

pub const canDRIVER_NORMAL: ::std::os::raw::c_uint = 4;
pub const canDRIVER_SILENT: ::std::os::raw::c_uint = 1;
pub const canDRIVER_SELFRECEPTION: ::std::os::raw::c_uint = 8;
pub const canDRIVER_OFF: ::std::os::raw::c_uint = 0;

// ---- Filter flags ----

pub const canFILTER_ACCEPT: ::std::os::raw::c_uint = 1;
pub const canFILTER_REJECT: ::std::os::raw::c_uint = 2;
pub const canFILTER_SET_CODE_STD: ::std::os::raw::c_uint = 3;
pub const canFILTER_SET_MASK_STD: ::std::os::raw::c_uint = 4;
pub const canFILTER_SET_CODE_EXT: ::std::os::raw::c_uint = 5;
pub const canFILTER_SET_MASK_EXT: ::std::os::raw::c_uint = 6;

// ---- Channel data items ----

pub const canCHANNELDATA_CARD_SERIAL_NO: ::std::os::raw::c_int = 7;
pub const canCHANNELDATA_CARD_UPC_NO: ::std::os::raw::c_int = 8;
pub const canCHANNELDATA_CARD_FIRMWARE_REV: ::std::os::raw::c_int = 9;
pub const canCHANNELDATA_CARD_HARDWARE_REV: ::std::os::raw::c_int = 10;
pub const canCHANNELDATA_CHANNEL_CAP: ::std::os::raw::c_int = 1;
pub const canCHANNELDATA_DEVDESCR_ASCII: ::std::os::raw::c_int = 26;
pub const canCHANNELDATA_CHANNEL_NAME: ::std::os::raw::c_int = 13;

// ---- Bus status flags ----

pub const canSTAT_ERROR_PASSIVE: ::std::os::raw::c_ulong = 0x00000001;
pub const canSTAT_BUS_OFF: ::std::os::raw::c_ulong = 0x00000002;
pub const canSTAT_ERROR_WARNING: ::std::os::raw::c_ulong = 0x00000004;
pub const canSTAT_ERROR_ACTIVE: ::std::os::raw::c_ulong = 0x00000008;
pub const canSTAT_TX_PENDING: ::std::os::raw::c_ulong = 0x00000010;
pub const canSTAT_RX_PENDING: ::std::os::raw::c_ulong = 0x00000020;
pub const canSTAT_OVERRUN: ::std::os::raw::c_ulong = 0x00000080;

// ---- Structs ----

#[repr(C)]
#[derive(Debug, Default, Clone, Copy)]
pub struct kvBusParamsTq {
    pub tq: ::std::os::raw::c_int,
    pub phase1: ::std::os::raw::c_int,
    pub phase2: ::std::os::raw::c_int,
    pub sjw: ::std::os::raw::c_int,
    pub prop: ::std::os::raw::c_int,
    pub prescaler: ::std::os::raw::c_int,
}

#[repr(C)]
#[derive(Debug, Default, Clone, Copy)]
pub struct canBusStatistics {
    pub stdData: ::std::os::raw::c_ulong,
    pub stdRemote: ::std::os::raw::c_ulong,
    pub extData: ::std::os::raw::c_ulong,
    pub extRemote: ::std::os::raw::c_ulong,
    pub errFrame: ::std::os::raw::c_ulong,
    pub busLoad: ::std::os::raw::c_ulong,
    pub overruns: ::std::os::raw::c_ulong,
}

// ---- Dynamic library loading ----

use std::sync::OnceLock;

/// The loaded CANLib shared library and its function pointers.
pub struct CanLib {
    // Hold the library handle to keep it loaded.
    _lib: libloading::Library,

    // Function pointers
    pub canInitializeLibrary: unsafe extern "C" fn(),
    pub canGetVersion: unsafe extern "C" fn() -> ::std::os::raw::c_ushort,

    pub canGetErrorText: unsafe extern "C" fn(
        err: canStatus,
        buf: *mut ::std::os::raw::c_char,
        bufsiz: ::std::os::raw::c_uint,
    ) -> canStatus,

    pub canGetNumberOfChannels:
        unsafe extern "C" fn(channelCount: *mut ::std::os::raw::c_int) -> canStatus,

    pub canGetChannelData: unsafe extern "C" fn(
        channel: ::std::os::raw::c_int,
        item: ::std::os::raw::c_int,
        buffer: *mut ::std::os::raw::c_void,
        bufsize: usize,
    ) -> canStatus,

    pub canOpenChannel: unsafe extern "C" fn(
        channel: ::std::os::raw::c_int,
        flags: ::std::os::raw::c_int,
    ) -> canHandle,

    pub canClose: unsafe extern "C" fn(hnd: canHandle) -> canStatus,
    pub canBusOn: unsafe extern "C" fn(hnd: canHandle) -> canStatus,
    pub canBusOff: unsafe extern "C" fn(hnd: canHandle) -> canStatus,
    pub canResetBus: unsafe extern "C" fn(hnd: canHandle) -> canStatus,

    pub canSetBusParams: unsafe extern "C" fn(
        hnd: canHandle,
        freq: ::std::os::raw::c_long,
        tseg1: ::std::os::raw::c_uint,
        tseg2: ::std::os::raw::c_uint,
        sjw: ::std::os::raw::c_uint,
        noSamp: ::std::os::raw::c_uint,
        syncmode: ::std::os::raw::c_uint,
    ) -> canStatus,

    pub canGetBusParams: unsafe extern "C" fn(
        hnd: canHandle,
        freq: *mut ::std::os::raw::c_long,
        tseg1: *mut ::std::os::raw::c_uint,
        tseg2: *mut ::std::os::raw::c_uint,
        sjw: *mut ::std::os::raw::c_uint,
        noSamp: *mut ::std::os::raw::c_uint,
        syncmode: *mut ::std::os::raw::c_uint,
    ) -> canStatus,

    pub canSetBitrate: unsafe extern "C" fn(
        hnd: canHandle,
        bitrate: ::std::os::raw::c_int,
    ) -> canStatus,

    pub canSetBusParamsFd: unsafe extern "C" fn(
        hnd: canHandle,
        freq_brs: ::std::os::raw::c_long,
        tseg1_brs: ::std::os::raw::c_uint,
        tseg2_brs: ::std::os::raw::c_uint,
        sjw_brs: ::std::os::raw::c_uint,
    ) -> canStatus,

    pub canGetBusParamsFd: unsafe extern "C" fn(
        hnd: canHandle,
        freq_brs: *mut ::std::os::raw::c_long,
        tseg1_brs: *mut ::std::os::raw::c_uint,
        tseg2_brs: *mut ::std::os::raw::c_uint,
        sjw_brs: *mut ::std::os::raw::c_uint,
    ) -> canStatus,

    pub canSetBusParamsTq:
        unsafe extern "C" fn(hnd: canHandle, nominal: kvBusParamsTq) -> canStatus,

    pub canGetBusParamsTq:
        unsafe extern "C" fn(hnd: canHandle, nominal: *mut kvBusParamsTq) -> canStatus,

    pub canSetBusParamsFdTq: unsafe extern "C" fn(
        hnd: canHandle,
        arbitration: kvBusParamsTq,
        data: kvBusParamsTq,
    ) -> canStatus,

    pub canGetBusParamsFdTq: unsafe extern "C" fn(
        hnd: canHandle,
        nominal: *mut kvBusParamsTq,
        data: *mut kvBusParamsTq,
    ) -> canStatus,

    pub canSetBusOutputControl: unsafe extern "C" fn(
        hnd: canHandle,
        drivertype: ::std::os::raw::c_uint,
    ) -> canStatus,

    pub canGetBusOutputControl: unsafe extern "C" fn(
        hnd: canHandle,
        drivertype: *mut ::std::os::raw::c_uint,
    ) -> canStatus,

    pub canWrite: unsafe extern "C" fn(
        hnd: canHandle,
        id: ::std::os::raw::c_long,
        msg: *mut ::std::os::raw::c_void,
        dlc: ::std::os::raw::c_uint,
        flag: ::std::os::raw::c_uint,
    ) -> canStatus,

    pub canWriteWait: unsafe extern "C" fn(
        hnd: canHandle,
        id: ::std::os::raw::c_long,
        msg: *mut ::std::os::raw::c_void,
        dlc: ::std::os::raw::c_uint,
        flag: ::std::os::raw::c_uint,
        timeout: ::std::os::raw::c_ulong,
    ) -> canStatus,

    pub canWriteSync:
        unsafe extern "C" fn(hnd: canHandle, timeout: ::std::os::raw::c_ulong) -> canStatus,

    pub canRead: unsafe extern "C" fn(
        hnd: canHandle,
        id: *mut ::std::os::raw::c_long,
        msg: *mut ::std::os::raw::c_void,
        dlc: *mut ::std::os::raw::c_uint,
        flag: *mut ::std::os::raw::c_uint,
        time: *mut ::std::os::raw::c_ulong,
    ) -> canStatus,

    pub canReadWait: unsafe extern "C" fn(
        hnd: canHandle,
        id: *mut ::std::os::raw::c_long,
        msg: *mut ::std::os::raw::c_void,
        dlc: *mut ::std::os::raw::c_uint,
        flag: *mut ::std::os::raw::c_uint,
        time: *mut ::std::os::raw::c_ulong,
        timeout: ::std::os::raw::c_ulong,
    ) -> canStatus,

    pub canReadSync:
        unsafe extern "C" fn(hnd: canHandle, timeout: ::std::os::raw::c_ulong) -> canStatus,

    pub canReadSpecific: unsafe extern "C" fn(
        hnd: canHandle,
        id: ::std::os::raw::c_long,
        msg: *mut ::std::os::raw::c_void,
        dlc: *mut ::std::os::raw::c_uint,
        flag: *mut ::std::os::raw::c_uint,
        time: *mut ::std::os::raw::c_ulong,
    ) -> canStatus,

    pub canReadSpecificSkip: unsafe extern "C" fn(
        hnd: canHandle,
        id: ::std::os::raw::c_long,
        msg: *mut ::std::os::raw::c_void,
        dlc: *mut ::std::os::raw::c_uint,
        flag: *mut ::std::os::raw::c_uint,
        time: *mut ::std::os::raw::c_ulong,
    ) -> canStatus,

    pub canReadSyncSpecific: unsafe extern "C" fn(
        hnd: canHandle,
        id: ::std::os::raw::c_long,
        timeout: ::std::os::raw::c_ulong,
    ) -> canStatus,

    pub canAccept: unsafe extern "C" fn(
        hnd: canHandle,
        envelope: ::std::os::raw::c_long,
        flag: ::std::os::raw::c_uint,
    ) -> canStatus,

    pub canSetAcceptanceFilter: unsafe extern "C" fn(
        hnd: canHandle,
        code: ::std::os::raw::c_uint,
        mask: ::std::os::raw::c_uint,
        is_extended: ::std::os::raw::c_int,
    ) -> canStatus,

    pub canReadStatus:
        unsafe extern "C" fn(hnd: canHandle, flags: *mut ::std::os::raw::c_ulong) -> canStatus,

    pub canReadErrorCounters: unsafe extern "C" fn(
        hnd: canHandle,
        txErr: *mut ::std::os::raw::c_uint,
        rxErr: *mut ::std::os::raw::c_uint,
        ovErr: *mut ::std::os::raw::c_uint,
    ) -> canStatus,

    pub canRequestChipStatus: unsafe extern "C" fn(hnd: canHandle) -> canStatus,
    pub canRequestBusStatistics: unsafe extern "C" fn(hnd: canHandle) -> canStatus,

    pub canGetBusStatistics: unsafe extern "C" fn(
        hnd: canHandle,
        stat: *mut canBusStatistics,
        bufsiz: usize,
    ) -> canStatus,

    pub canFlushReceiveQueue: unsafe extern "C" fn(hnd: canHandle) -> canStatus,
    pub canFlushTransmitQueue: unsafe extern "C" fn(hnd: canHandle) -> canStatus,
}

// SAFETY: The CANLib shared library functions are thread-safe per Kvaser documentation.
// The Library handle itself is Send+Sync, and function pointers are just addresses.
unsafe impl Send for CanLib {}
unsafe impl Sync for CanLib {}

/// The library name to load at runtime.
#[cfg(target_os = "windows")]
const LIB_NAME: &str = "canlib32.dll";

#[cfg(target_os = "linux")]
const LIB_NAME: &str = "libcanlib.so";

#[cfg(not(any(target_os = "windows", target_os = "linux")))]
compile_error!("canlib-sys only supports Windows and Linux");

/// Error returned when the CANLib shared library cannot be loaded.
#[derive(Debug)]
pub enum LoadError {
    /// The shared library file could not be found or loaded.
    LibraryNotFound(libloading::Error),
    /// A required function symbol was not found in the library.
    SymbolNotFound(&'static str, libloading::Error),
}

impl std::fmt::Display for LoadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LoadError::LibraryNotFound(e) => {
                write!(
                    f,
                    "Failed to load {}: {}. \
                     Install the Kvaser CANLib SDK or ensure the library is on the system PATH.",
                    LIB_NAME, e
                )
            }
            LoadError::SymbolNotFound(name, e) => {
                write!(
                    f,
                    "Symbol '{}' not found in {}: {}",
                    name, LIB_NAME, e
                )
            }
        }
    }
}

impl std::error::Error for LoadError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            LoadError::LibraryNotFound(e) | LoadError::SymbolNotFound(_, e) => Some(e),
        }
    }
}

/// Helper macro to load a symbol from the library.
macro_rules! load_sym {
    ($lib:expr, $name:literal) => {{
        let sym = $lib
            .get::<unsafe extern "C" fn()>($name.as_bytes())
            .map_err(|e| LoadError::SymbolNotFound($name, e))?;
        // SAFETY: We trust that the symbol has the correct signature as declared by the
        // Kvaser CANLib SDK headers. The caller must ensure argument types match.
        std::mem::transmute(sym.into_raw().into_raw())
    }};
}

impl CanLib {
    /// Load the CANLib shared library and resolve all function symbols.
    ///
    /// # Safety
    ///
    /// The loaded library must be a genuine Kvaser CANLib implementation with
    /// the expected ABI. This is satisfied by the official SDK installation.
    pub unsafe fn load() -> Result<Self, LoadError> {
        let lib =
            libloading::Library::new(LIB_NAME).map_err(LoadError::LibraryNotFound)?;

        Ok(Self {
            canInitializeLibrary: load_sym!(lib, "canInitializeLibrary"),
            canGetVersion: load_sym!(lib, "canGetVersion"),
            canGetErrorText: load_sym!(lib, "canGetErrorText"),
            canGetNumberOfChannels: load_sym!(lib, "canGetNumberOfChannels"),
            canGetChannelData: load_sym!(lib, "canGetChannelData"),
            canOpenChannel: load_sym!(lib, "canOpenChannel"),
            canClose: load_sym!(lib, "canClose"),
            canBusOn: load_sym!(lib, "canBusOn"),
            canBusOff: load_sym!(lib, "canBusOff"),
            canResetBus: load_sym!(lib, "canResetBus"),
            canSetBusParams: load_sym!(lib, "canSetBusParams"),
            canGetBusParams: load_sym!(lib, "canGetBusParams"),
            canSetBitrate: load_sym!(lib, "canSetBitrate"),
            canSetBusParamsFd: load_sym!(lib, "canSetBusParamsFd"),
            canGetBusParamsFd: load_sym!(lib, "canGetBusParamsFd"),
            canSetBusParamsTq: load_sym!(lib, "canSetBusParamsTq"),
            canGetBusParamsTq: load_sym!(lib, "canGetBusParamsTq"),
            canSetBusParamsFdTq: load_sym!(lib, "canSetBusParamsFdTq"),
            canGetBusParamsFdTq: load_sym!(lib, "canGetBusParamsFdTq"),
            canSetBusOutputControl: load_sym!(lib, "canSetBusOutputControl"),
            canGetBusOutputControl: load_sym!(lib, "canGetBusOutputControl"),
            canWrite: load_sym!(lib, "canWrite"),
            canWriteWait: load_sym!(lib, "canWriteWait"),
            canWriteSync: load_sym!(lib, "canWriteSync"),
            canRead: load_sym!(lib, "canRead"),
            canReadWait: load_sym!(lib, "canReadWait"),
            canReadSync: load_sym!(lib, "canReadSync"),
            canReadSpecific: load_sym!(lib, "canReadSpecific"),
            canReadSpecificSkip: load_sym!(lib, "canReadSpecificSkip"),
            canReadSyncSpecific: load_sym!(lib, "canReadSyncSpecific"),
            canAccept: load_sym!(lib, "canAccept"),
            canSetAcceptanceFilter: load_sym!(lib, "canSetAcceptanceFilter"),
            canReadStatus: load_sym!(lib, "canReadStatus"),
            canReadErrorCounters: load_sym!(lib, "canReadErrorCounters"),
            canRequestChipStatus: load_sym!(lib, "canRequestChipStatus"),
            canRequestBusStatistics: load_sym!(lib, "canRequestBusStatistics"),
            canGetBusStatistics: load_sym!(lib, "canGetBusStatistics"),
            canFlushReceiveQueue: load_sym!(lib, "canFlushReceiveQueue"),
            canFlushTransmitQueue: load_sym!(lib, "canFlushTransmitQueue"),
            _lib: lib,
        })
    }
}

static CANLIB: OnceLock<CanLib> = OnceLock::new();

/// Get a reference to the globally loaded CANLib instance.
///
/// The library is loaded on the first call. Returns an error if the shared
/// library cannot be found or a required symbol is missing.
pub fn get() -> Result<&'static CanLib, &'static LoadError> {
    static LOAD_ERROR: OnceLock<LoadError> = OnceLock::new();

    if let Some(lib) = CANLIB.get() {
        return Ok(lib);
    }

    // Attempt to load
    match unsafe { CanLib::load() } {
        Ok(lib) => {
            // Another thread may have raced us; that's fine, OnceLock handles it.
            let _ = CANLIB.set(lib);
            Ok(CANLIB.get().unwrap())
        }
        Err(e) => {
            let _ = LOAD_ERROR.set(e);
            Err(LOAD_ERROR.get().unwrap())
        }
    }
}
