#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Default)]
pub struct SystemStatus {
    pub alarm_on: bool,
    pub storage_connected: bool,
    pub storage_full: bool,
    pub storage_error: bool,
    pub uptime_sec: u32,
    pub records_since_boot: u32,
}
