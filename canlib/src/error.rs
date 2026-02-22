use canlib_sys as sys;

/// Result type alias used throughout the canlib crate.
pub type Result<T> = std::result::Result<T, CanError>;

/// Error type representing all CANLib status codes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum CanError {
    #[error("invalid parameter")]
    Param,
    #[error("no message available")]
    NoMsg,
    #[error("channel not found")]
    NotFound,
    #[error("out of memory")]
    NoMem,
    #[error("no channels available")]
    NoChannels,
    #[error("operation interrupted")]
    Interrupted,
    #[error("operation timed out")]
    Timeout,
    #[error("library not initialized")]
    NotInitialized,
    #[error("no free handles")]
    NoHandles,
    #[error("invalid handle")]
    InvalidHandle,
    #[error("ini-file error")]
    IniFile,
    #[error("driver error")]
    Driver,
    #[error("transmit buffer overflow")]
    TxBufOverflow,
    #[error("hardware error")]
    Hardware,
    #[error("dynamic library load error")]
    DynaLoad,
    #[error("dynamic library not found")]
    DynaLib,
    #[error("dynamic library init error")]
    DynaInit,
    #[error("operation not supported")]
    NotSupported,
    #[error("driver load error")]
    DriverLoad,
    #[error("driver failed")]
    DriverFailed,
    #[error("no configuration manager")]
    NoConfigMgr,
    #[error("no card found")]
    NoCard,
    #[error("registry error")]
    Registry,
    #[error("license error")]
    License,
    #[error("internal error")]
    Internal,
    #[error("no access")]
    NoAccess,
    #[error("not implemented")]
    NotImplemented,
    #[error("device file error")]
    DeviceFile,
    #[error("host file error")]
    HostFile,
    #[error("disk error")]
    Disk,
    #[error("CRC error")]
    Crc,
    #[error("configuration error")]
    Config,
    #[error("memo failure")]
    MemoFail,
    #[error("script failure")]
    ScriptFail,
    #[error("script wrong version")]
    ScriptWrongVersion,
    #[error("unknown error (code {0})")]
    Unknown(i32),
}

impl CanError {
    /// Convert a raw `canStatus` code into a `CanError`.
    pub fn from_status(status: sys::canStatus) -> Self {
        match status {
            sys::canERR_PARAM => CanError::Param,
            sys::canERR_NOMSG => CanError::NoMsg,
            sys::canERR_NOTFOUND => CanError::NotFound,
            sys::canERR_NOMEM => CanError::NoMem,
            sys::canERR_NOCHANNELS => CanError::NoChannels,
            sys::canERR_INTERRUPTED => CanError::Interrupted,
            sys::canERR_TIMEOUT => CanError::Timeout,
            sys::canERR_NOTINITIALIZED => CanError::NotInitialized,
            sys::canERR_NOHANDLES => CanError::NoHandles,
            sys::canERR_INVHANDLE => CanError::InvalidHandle,
            sys::canERR_INIFILE => CanError::IniFile,
            sys::canERR_DRIVER => CanError::Driver,
            sys::canERR_TXBUFOFL => CanError::TxBufOverflow,
            sys::canERR_HARDWARE => CanError::Hardware,
            sys::canERR_DYNALOAD => CanError::DynaLoad,
            sys::canERR_DYNALIB => CanError::DynaLib,
            sys::canERR_DYNAINIT => CanError::DynaInit,
            sys::canERR_NOT_SUPPORTED => CanError::NotSupported,
            sys::canERR_DRIVERLOAD => CanError::DriverLoad,
            sys::canERR_DRIVERFAILED => CanError::DriverFailed,
            sys::canERR_NOCONFIGMGR => CanError::NoConfigMgr,
            sys::canERR_NOCARD => CanError::NoCard,
            sys::canERR_REGISTRY => CanError::Registry,
            sys::canERR_LICENSE => CanError::License,
            sys::canERR_INTERNAL => CanError::Internal,
            sys::canERR_NO_ACCESS => CanError::NoAccess,
            sys::canERR_NOT_IMPLEMENTED => CanError::NotImplemented,
            sys::canERR_DEVICE_FILE => CanError::DeviceFile,
            sys::canERR_HOST_FILE => CanError::HostFile,
            sys::canERR_DISK => CanError::Disk,
            sys::canERR_CRC => CanError::Crc,
            sys::canERR_CONFIG => CanError::Config,
            sys::canERR_MEMO_FAIL => CanError::MemoFail,
            sys::canERR_SCRIPT_FAIL => CanError::ScriptFail,
            sys::canERR_SCRIPT_WRONG_VERSION => CanError::ScriptWrongVersion,
            other => CanError::Unknown(other),
        }
    }
}

/// Check a `canStatus` return value and convert to `Result<()>`.
pub(crate) fn check_status(status: sys::canStatus) -> Result<()> {
    if status >= sys::canOK {
        Ok(())
    } else {
        Err(CanError::from_status(status))
    }
}

/// Check a `canStatus` return value that may be a handle (non-negative = success).
pub(crate) fn check_handle(status: sys::canStatus) -> Result<sys::canHandle> {
    if status >= 0 {
        Ok(status)
    } else {
        Err(CanError::from_status(status))
    }
}
