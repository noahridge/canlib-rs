use canlib_sys as sys;

bitflags::bitflags! {
    /// CAN bus status flags.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct BusStatus: u64 {
        /// The bus is in error passive state.
        const ERROR_PASSIVE = sys::canSTAT_ERROR_PASSIVE as u64;
        /// The bus is off (too many errors).
        const BUS_OFF = sys::canSTAT_BUS_OFF as u64;
        /// Error warning level reached.
        const ERROR_WARNING = sys::canSTAT_ERROR_WARNING as u64;
        /// The bus is error active (normal operation).
        const ERROR_ACTIVE = sys::canSTAT_ERROR_ACTIVE as u64;
        /// There are messages waiting to be transmitted.
        const TX_PENDING = sys::canSTAT_TX_PENDING as u64;
        /// There are messages in the receive queue.
        const RX_PENDING = sys::canSTAT_RX_PENDING as u64;
        /// A receive buffer overrun has occurred.
        const OVERRUN = sys::canSTAT_OVERRUN as u64;
    }
}

/// CAN error counters.
#[derive(Debug, Clone, Copy, Default)]
pub struct ErrorCounters {
    /// Transmit error counter.
    pub tx_errors: u32,
    /// Receive error counter.
    pub rx_errors: u32,
    /// Overrun error counter.
    pub overrun_errors: u32,
}

/// CAN bus statistics.
#[derive(Debug, Clone, Copy, Default)]
pub struct BusStatistics {
    /// Number of standard data frames.
    pub std_data: u64,
    /// Number of standard remote frames.
    pub std_remote: u64,
    /// Number of extended data frames.
    pub ext_data: u64,
    /// Number of extended remote frames.
    pub ext_remote: u64,
    /// Number of error frames.
    pub err_frames: u64,
    /// Bus load (0-10000, representing 0.00%-100.00%).
    pub bus_load: u64,
    /// Number of overruns.
    pub overruns: u64,
}

impl BusStatistics {
    pub(crate) fn from_raw(raw: &sys::canBusStatistics) -> Self {
        Self {
            std_data: raw.stdData as u64,
            std_remote: raw.stdRemote as u64,
            ext_data: raw.extData as u64,
            ext_remote: raw.extRemote as u64,
            err_frames: raw.errFrame as u64,
            bus_load: raw.busLoad as u64,
            overruns: raw.overruns as u64,
        }
    }

    /// Bus load as a percentage (0.0 - 100.0).
    pub fn bus_load_percent(&self) -> f64 {
        self.bus_load as f64 / 100.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bus_load_percent_computes_correctly() {
        let stats = BusStatistics {
            bus_load: 5000,
            ..Default::default()
        };
        assert!((stats.bus_load_percent() - 50.0).abs() < f64::EPSILON);

        let stats_zero = BusStatistics::default();
        assert!((stats_zero.bus_load_percent() - 0.0).abs() < f64::EPSILON);

        let stats_full = BusStatistics {
            bus_load: 10000,
            ..Default::default()
        };
        assert!((stats_full.bus_load_percent() - 100.0).abs() < f64::EPSILON);
    }

    #[test]
    fn bus_statistics_from_raw() {
        let raw = sys::canBusStatistics {
            stdData: 10,
            stdRemote: 20,
            extData: 30,
            extRemote: 40,
            errFrame: 5,
            busLoad: 1234,
            overruns: 2,
        };
        let stats = BusStatistics::from_raw(&raw);
        assert_eq!(stats.std_data, 10);
        assert_eq!(stats.std_remote, 20);
        assert_eq!(stats.ext_data, 30);
        assert_eq!(stats.ext_remote, 40);
        assert_eq!(stats.err_frames, 5);
        assert_eq!(stats.bus_load, 1234);
        assert_eq!(stats.overruns, 2);
    }

    #[test]
    fn bus_status_flag_composition() {
        let status = BusStatus::ERROR_ACTIVE | BusStatus::TX_PENDING;
        assert!(status.contains(BusStatus::ERROR_ACTIVE));
        assert!(status.contains(BusStatus::TX_PENDING));
        assert!(!status.contains(BusStatus::BUS_OFF));
        assert!(!status.contains(BusStatus::OVERRUN));
    }

    #[test]
    fn error_counters_default() {
        let ec = ErrorCounters::default();
        assert_eq!(ec.tx_errors, 0);
        assert_eq!(ec.rx_errors, 0);
        assert_eq!(ec.overrun_errors, 0);
    }
}
