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
    freq: i64,
    tseg1: u32,
    tseg2: u32,
    sjw: u32,
    no_samp: u32,
    sync_mode: u32,
}

impl BusParams {
    /// Create new bus parameters.
    pub fn new(freq: i64, tseg1: u32, tseg2: u32, sjw: u32, no_samp: u32, sync_mode: u32) -> Self {
        Self { freq, tseg1, tseg2, sjw, no_samp, sync_mode }
    }

    /// Create bus parameters from raw FFI values.
    pub(crate) fn from_raw(freq: i64, tseg1: u32, tseg2: u32, sjw: u32, no_samp: u32, sync_mode: u32) -> Self {
        Self { freq, tseg1, tseg2, sjw, no_samp, sync_mode }
    }

    pub fn freq(&self) -> i64 { self.freq }
    pub fn tseg1(&self) -> u32 { self.tseg1 }
    pub fn tseg2(&self) -> u32 { self.tseg2 }
    pub fn sjw(&self) -> u32 { self.sjw }
    pub fn no_samp(&self) -> u32 { self.no_samp }
    pub fn sync_mode(&self) -> u32 { self.sync_mode }
}

/// Bus timing parameters using time quanta.
#[derive(Debug, Clone, Copy)]
pub struct BusParamsTq {
    tq: i32,
    phase1: i32,
    phase2: i32,
    sjw: i32,
    prop: i32,
    prescaler: i32,
}

impl BusParamsTq {
    /// Create new time-quanta bus parameters.
    pub fn new(tq: i32, phase1: i32, phase2: i32, sjw: i32, prop: i32, prescaler: i32) -> Self {
        Self { tq, phase1, phase2, sjw, prop, prescaler }
    }

    pub fn tq(&self) -> i32 { self.tq }
    pub fn phase1(&self) -> i32 { self.phase1 }
    pub fn phase2(&self) -> i32 { self.phase2 }
    pub fn sjw(&self) -> i32 { self.sjw }
    pub fn prop(&self) -> i32 { self.prop }
    pub fn prescaler(&self) -> i32 { self.prescaler }

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bitrate_to_raw_maps_correctly() {
        assert_eq!(Bitrate::Rate1M.to_raw(), sys::canBITRATE_1M);
        assert_eq!(Bitrate::Rate500K.to_raw(), sys::canBITRATE_500K);
        assert_eq!(Bitrate::Rate250K.to_raw(), sys::canBITRATE_250K);
        assert_eq!(Bitrate::Rate125K.to_raw(), sys::canBITRATE_125K);
        assert_eq!(Bitrate::Rate100K.to_raw(), sys::canBITRATE_100K);
        assert_eq!(Bitrate::Rate83K.to_raw(), sys::canBITRATE_83K);
        assert_eq!(Bitrate::Rate62K.to_raw(), sys::canBITRATE_62K);
        assert_eq!(Bitrate::Rate50K.to_raw(), sys::canBITRATE_50K);
        assert_eq!(Bitrate::Rate10K.to_raw(), sys::canBITRATE_10K);
    }

    #[test]
    fn fd_bitrate_to_raw_maps_correctly() {
        assert_eq!(FdBitrate::Rate500K80P.to_raw(), sys::canFD_BITRATE_500K_80P);
        assert_eq!(FdBitrate::Rate1M80P.to_raw(), sys::canFD_BITRATE_1M_80P);
        assert_eq!(FdBitrate::Rate2M80P.to_raw(), sys::canFD_BITRATE_2M_80P);
        assert_eq!(FdBitrate::Rate4M80P.to_raw(), sys::canFD_BITRATE_4M_80P);
        assert_eq!(FdBitrate::Rate8M60P.to_raw(), sys::canFD_BITRATE_8M_60P);
        assert_eq!(FdBitrate::Rate8M70P.to_raw(), sys::canFD_BITRATE_8M_70P);
        assert_eq!(FdBitrate::Rate8M80P.to_raw(), sys::canFD_BITRATE_8M_80P);
    }

    #[test]
    fn driver_type_round_trips() {
        let types = [
            DriverType::Normal,
            DriverType::Silent,
            DriverType::SelfReception,
            DriverType::Off,
        ];
        for dt in types {
            let raw = dt.to_raw();
            let back = DriverType::from_raw(raw).expect("round-trip should succeed");
            assert_eq!(dt, back);
        }
    }

    #[test]
    fn driver_type_from_raw_returns_none_for_unknown() {
        assert!(DriverType::from_raw(99).is_none());
        assert!(DriverType::from_raw(255).is_none());
    }

    #[test]
    fn bus_params_tq_round_trips() {
        let params = BusParamsTq::new(10, 5, 3, 1, 2, 4);
        let raw = params.to_raw();
        let back = BusParamsTq::from_raw(raw);
        assert_eq!(back.tq(), 10);
        assert_eq!(back.phase1(), 5);
        assert_eq!(back.phase2(), 3);
        assert_eq!(back.sjw(), 1);
        assert_eq!(back.prop(), 2);
        assert_eq!(back.prescaler(), 4);
    }

    #[test]
    fn bus_params_new_and_getters() {
        let params = BusParams::new(500_000, 4, 3, 1, 1, 0);
        assert_eq!(params.freq(), 500_000);
        assert_eq!(params.tseg1(), 4);
        assert_eq!(params.tseg2(), 3);
        assert_eq!(params.sjw(), 1);
        assert_eq!(params.no_samp(), 1);
        assert_eq!(params.sync_mode(), 0);
    }
}
