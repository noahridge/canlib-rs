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
    /// Convert this `CanError` back to the corresponding raw `canStatus` code.
    pub fn to_status_code(&self) -> sys::canStatus {
        match self {
            CanError::Param => sys::canERR_PARAM,
            CanError::NoMsg => sys::canERR_NOMSG,
            CanError::NotFound => sys::canERR_NOTFOUND,
            CanError::NoMem => sys::canERR_NOMEM,
            CanError::NoChannels => sys::canERR_NOCHANNELS,
            CanError::Interrupted => sys::canERR_INTERRUPTED,
            CanError::Timeout => sys::canERR_TIMEOUT,
            CanError::NotInitialized => sys::canERR_NOTINITIALIZED,
            CanError::NoHandles => sys::canERR_NOHANDLES,
            CanError::InvalidHandle => sys::canERR_INVHANDLE,
            CanError::IniFile => sys::canERR_INIFILE,
            CanError::Driver => sys::canERR_DRIVER,
            CanError::TxBufOverflow => sys::canERR_TXBUFOFL,
            CanError::Hardware => sys::canERR_HARDWARE,
            CanError::DynaLoad => sys::canERR_DYNALOAD,
            CanError::DynaLib => sys::canERR_DYNALIB,
            CanError::DynaInit => sys::canERR_DYNAINIT,
            CanError::NotSupported => sys::canERR_NOT_SUPPORTED,
            CanError::DriverLoad => sys::canERR_DRIVERLOAD,
            CanError::DriverFailed => sys::canERR_DRIVERFAILED,
            CanError::NoConfigMgr => sys::canERR_NOCONFIGMGR,
            CanError::NoCard => sys::canERR_NOCARD,
            CanError::Registry => sys::canERR_REGISTRY,
            CanError::License => sys::canERR_LICENSE,
            CanError::Internal => sys::canERR_INTERNAL,
            CanError::NoAccess => sys::canERR_NO_ACCESS,
            CanError::NotImplemented => sys::canERR_NOT_IMPLEMENTED,
            CanError::DeviceFile => sys::canERR_DEVICE_FILE,
            CanError::HostFile => sys::canERR_HOST_FILE,
            CanError::Disk => sys::canERR_DISK,
            CanError::Crc => sys::canERR_CRC,
            CanError::Config => sys::canERR_CONFIG,
            CanError::MemoFail => sys::canERR_MEMO_FAIL,
            CanError::ScriptFail => sys::canERR_SCRIPT_FAIL,
            CanError::ScriptWrongVersion => sys::canERR_SCRIPT_WRONG_VERSION,
            CanError::Unknown(code) => *code,
        }
    }

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_status_maps_known_error_codes() {
        assert_eq!(CanError::from_status(sys::canERR_PARAM), CanError::Param);
        assert_eq!(CanError::from_status(sys::canERR_NOMSG), CanError::NoMsg);
        assert_eq!(CanError::from_status(sys::canERR_NOTFOUND), CanError::NotFound);
        assert_eq!(CanError::from_status(sys::canERR_NOMEM), CanError::NoMem);
        assert_eq!(CanError::from_status(sys::canERR_NOCHANNELS), CanError::NoChannels);
        assert_eq!(CanError::from_status(sys::canERR_INTERRUPTED), CanError::Interrupted);
        assert_eq!(CanError::from_status(sys::canERR_TIMEOUT), CanError::Timeout);
        assert_eq!(CanError::from_status(sys::canERR_NOTINITIALIZED), CanError::NotInitialized);
        assert_eq!(CanError::from_status(sys::canERR_NOHANDLES), CanError::NoHandles);
        assert_eq!(CanError::from_status(sys::canERR_INVHANDLE), CanError::InvalidHandle);
        assert_eq!(CanError::from_status(sys::canERR_INIFILE), CanError::IniFile);
        assert_eq!(CanError::from_status(sys::canERR_DRIVER), CanError::Driver);
        assert_eq!(CanError::from_status(sys::canERR_TXBUFOFL), CanError::TxBufOverflow);
        assert_eq!(CanError::from_status(sys::canERR_HARDWARE), CanError::Hardware);
        assert_eq!(CanError::from_status(sys::canERR_DYNALOAD), CanError::DynaLoad);
        assert_eq!(CanError::from_status(sys::canERR_DYNALIB), CanError::DynaLib);
        assert_eq!(CanError::from_status(sys::canERR_DYNAINIT), CanError::DynaInit);
        assert_eq!(CanError::from_status(sys::canERR_NOT_SUPPORTED), CanError::NotSupported);
        assert_eq!(CanError::from_status(sys::canERR_DRIVERLOAD), CanError::DriverLoad);
        assert_eq!(CanError::from_status(sys::canERR_DRIVERFAILED), CanError::DriverFailed);
        assert_eq!(CanError::from_status(sys::canERR_NOCONFIGMGR), CanError::NoConfigMgr);
        assert_eq!(CanError::from_status(sys::canERR_NOCARD), CanError::NoCard);
        assert_eq!(CanError::from_status(sys::canERR_REGISTRY), CanError::Registry);
        assert_eq!(CanError::from_status(sys::canERR_LICENSE), CanError::License);
        assert_eq!(CanError::from_status(sys::canERR_INTERNAL), CanError::Internal);
        assert_eq!(CanError::from_status(sys::canERR_NO_ACCESS), CanError::NoAccess);
        assert_eq!(CanError::from_status(sys::canERR_NOT_IMPLEMENTED), CanError::NotImplemented);
        assert_eq!(CanError::from_status(sys::canERR_DEVICE_FILE), CanError::DeviceFile);
        assert_eq!(CanError::from_status(sys::canERR_HOST_FILE), CanError::HostFile);
        assert_eq!(CanError::from_status(sys::canERR_DISK), CanError::Disk);
        assert_eq!(CanError::from_status(sys::canERR_CRC), CanError::Crc);
        assert_eq!(CanError::from_status(sys::canERR_CONFIG), CanError::Config);
        assert_eq!(CanError::from_status(sys::canERR_MEMO_FAIL), CanError::MemoFail);
        assert_eq!(CanError::from_status(sys::canERR_SCRIPT_FAIL), CanError::ScriptFail);
        assert_eq!(CanError::from_status(sys::canERR_SCRIPT_WRONG_VERSION), CanError::ScriptWrongVersion);
    }

    #[test]
    fn from_status_maps_unknown_codes() {
        assert_eq!(CanError::from_status(-999), CanError::Unknown(-999));
        assert_eq!(CanError::from_status(-100), CanError::Unknown(-100));
        // Reserved codes that aren't mapped should become Unknown
        assert_eq!(CanError::from_status(sys::canERR_RESERVED_1), CanError::Unknown(sys::canERR_RESERVED_1));
    }

    #[test]
    fn check_status_ok_for_canok_and_positive() {
        assert!(check_status(sys::canOK).is_ok());
        assert!(check_status(1).is_ok());
        assert!(check_status(100).is_ok());
    }

    #[test]
    fn check_status_err_for_negative() {
        assert!(check_status(-1).is_err());
        assert!(check_status(-7).is_err());
        assert_eq!(check_status(sys::canERR_TIMEOUT).unwrap_err(), CanError::Timeout);
    }

    #[test]
    fn check_handle_ok_for_non_negative() {
        assert_eq!(check_handle(0).unwrap(), 0);
        assert_eq!(check_handle(42).unwrap(), 42);
    }

    #[test]
    fn check_handle_err_for_negative() {
        assert!(check_handle(-1).is_err());
        assert_eq!(check_handle(sys::canERR_NOHANDLES).unwrap_err(), CanError::NoHandles);
    }

    #[test]
    fn to_status_code_round_trips_all_known_variants() {
        let variants: Vec<CanError> = vec![
            CanError::Param,
            CanError::NoMsg,
            CanError::NotFound,
            CanError::NoMem,
            CanError::NoChannels,
            CanError::Interrupted,
            CanError::Timeout,
            CanError::NotInitialized,
            CanError::NoHandles,
            CanError::InvalidHandle,
            CanError::IniFile,
            CanError::Driver,
            CanError::TxBufOverflow,
            CanError::Hardware,
            CanError::DynaLoad,
            CanError::DynaLib,
            CanError::DynaInit,
            CanError::NotSupported,
            CanError::DriverLoad,
            CanError::DriverFailed,
            CanError::NoConfigMgr,
            CanError::NoCard,
            CanError::Registry,
            CanError::License,
            CanError::Internal,
            CanError::NoAccess,
            CanError::NotImplemented,
            CanError::DeviceFile,
            CanError::HostFile,
            CanError::Disk,
            CanError::Crc,
            CanError::Config,
            CanError::MemoFail,
            CanError::ScriptFail,
            CanError::ScriptWrongVersion,
            CanError::Unknown(-999),
        ];
        for variant in variants {
            let code = variant.to_status_code();
            let back = CanError::from_status(code);
            assert_eq!(back, variant, "round-trip failed for {:?}", variant);
        }
    }

    #[test]
    fn display_strings_are_non_empty() {
        let variants: Vec<CanError> = vec![
            CanError::Param,
            CanError::NoMsg,
            CanError::NotFound,
            CanError::NoMem,
            CanError::NoChannels,
            CanError::Interrupted,
            CanError::Timeout,
            CanError::NotInitialized,
            CanError::NoHandles,
            CanError::InvalidHandle,
            CanError::IniFile,
            CanError::Driver,
            CanError::TxBufOverflow,
            CanError::Hardware,
            CanError::DynaLoad,
            CanError::DynaLib,
            CanError::DynaInit,
            CanError::NotSupported,
            CanError::DriverLoad,
            CanError::DriverFailed,
            CanError::NoConfigMgr,
            CanError::NoCard,
            CanError::Registry,
            CanError::License,
            CanError::Internal,
            CanError::NoAccess,
            CanError::NotImplemented,
            CanError::DeviceFile,
            CanError::HostFile,
            CanError::Disk,
            CanError::Crc,
            CanError::Config,
            CanError::MemoFail,
            CanError::ScriptFail,
            CanError::ScriptWrongVersion,
            CanError::Unknown(-999),
        ];
        for variant in variants {
            let s = format!("{}", variant);
            assert!(!s.is_empty(), "Display for {:?} should be non-empty", variant);
        }
    }
}
