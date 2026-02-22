use canlib_sys as sys;

/// Predefined CAN bitrates for classic CAN.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Bitrate {
    /// 1 Mbit/s
    Rate1M,
    /// 500 kbit/s
    Rate500K,
    /// 250 kbit/s
    Rate250K,
    /// 125 kbit/s
    Rate125K,
    /// 100 kbit/s
    Rate100K,
    /// 83.333 kbit/s
    Rate83K,
    /// 62.5 kbit/s
    Rate62K,
    /// 50 kbit/s
    Rate50K,
    /// 10 kbit/s
    Rate10K,
}

impl Bitrate {
    /// Convert to the raw CANLib constant.
    pub(crate) fn to_raw(self) -> std::os::raw::c_long {
        match self {
            Bitrate::Rate1M => sys::canBITRATE_1M,
            Bitrate::Rate500K => sys::canBITRATE_500K,
            Bitrate::Rate250K => sys::canBITRATE_250K,
            Bitrate::Rate125K => sys::canBITRATE_125K,
            Bitrate::Rate100K => sys::canBITRATE_100K,
            Bitrate::Rate83K => sys::canBITRATE_83K,
            Bitrate::Rate62K => sys::canBITRATE_62K,
            Bitrate::Rate50K => sys::canBITRATE_50K,
            Bitrate::Rate10K => sys::canBITRATE_10K,
        }
    }
}

/// Predefined CAN FD data-phase bitrates.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FdBitrate {
    /// 500 kbit/s, 80% sample point
    Rate500K80P,
    /// 1 Mbit/s, 80% sample point
    Rate1M80P,
    /// 2 Mbit/s, 80% sample point
    Rate2M80P,
    /// 4 Mbit/s, 80% sample point
    Rate4M80P,
    /// 8 Mbit/s, 60% sample point
    Rate8M60P,
    /// 8 Mbit/s, 70% sample point
    Rate8M70P,
    /// 8 Mbit/s, 80% sample point
    Rate8M80P,
}

impl FdBitrate {
    /// Convert to the raw CANLib constant.
    pub fn to_raw(self) -> std::os::raw::c_long {
        match self {
            FdBitrate::Rate500K80P => sys::canFD_BITRATE_500K_80P,
            FdBitrate::Rate1M80P => sys::canFD_BITRATE_1M_80P,
            FdBitrate::Rate2M80P => sys::canFD_BITRATE_2M_80P,
            FdBitrate::Rate4M80P => sys::canFD_BITRATE_4M_80P,
            FdBitrate::Rate8M60P => sys::canFD_BITRATE_8M_60P,
            FdBitrate::Rate8M70P => sys::canFD_BITRATE_8M_70P,
            FdBitrate::Rate8M80P => sys::canFD_BITRATE_8M_80P,
        }
    }
}

/// Custom bus timing parameters for classic CAN.
#[derive(Debug, Clone, Copy)]
pub struct BusParams {
    pub freq: i64,
    pub tseg1: u32,
    pub tseg2: u32,
    pub sjw: u32,
    pub no_samp: u32,
    pub sync_mode: u32,
}

/// Bus timing parameters using time quanta.
#[derive(Debug, Clone, Copy)]
pub struct BusParamsTq {
    pub tq: i32,
    pub phase1: i32,
    pub phase2: i32,
    pub sjw: i32,
    pub prop: i32,
    pub prescaler: i32,
}

impl BusParamsTq {
    pub(crate) fn to_raw(self) -> sys::kvBusParamsTq {
        sys::kvBusParamsTq {
            tq: self.tq,
            phase1: self.phase1,
            phase2: self.phase2,
            sjw: self.sjw,
            prop: self.prop,
            prescaler: self.prescaler,
        }
    }

    pub(crate) fn from_raw(raw: sys::kvBusParamsTq) -> Self {
        Self {
            tq: raw.tq,
            phase1: raw.phase1,
            phase2: raw.phase2,
            sjw: raw.sjw,
            prop: raw.prop,
            prescaler: raw.prescaler,
        }
    }
}

/// Driver output mode for the CAN transceiver.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DriverType {
    /// Normal mode.
    Normal,
    /// Silent / listen-only mode.
    Silent,
    /// Self-reception mode.
    SelfReception,
    /// Off.
    Off,
}

impl DriverType {
    pub(crate) fn to_raw(self) -> u32 {
        match self {
            DriverType::Normal => sys::canDRIVER_NORMAL,
            DriverType::Silent => sys::canDRIVER_SILENT,
            DriverType::SelfReception => sys::canDRIVER_SELFRECEPTION,
            DriverType::Off => sys::canDRIVER_OFF,
        }
    }

    pub(crate) fn from_raw(raw: u32) -> Option<Self> {
        match raw {
            sys::canDRIVER_NORMAL => Some(DriverType::Normal),
            sys::canDRIVER_SILENT => Some(DriverType::Silent),
            sys::canDRIVER_SELFRECEPTION => Some(DriverType::SelfReception),
            sys::canDRIVER_OFF => Some(DriverType::Off),
            _ => None,
        }
    }
}
