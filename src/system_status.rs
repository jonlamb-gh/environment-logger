use crate::alarm::AlarmStatus;

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Default)]
pub struct SystemStatus {
    pub uptime_sec: u32,
    pub alarm: AlarmStatus,
    pub alarm_warmed_up: bool,
    pub record_count: u32,
    pub storage_connected: bool,
    pub storage_full: bool,
    pub storage_error: bool,
}

impl SystemStatus {
    /// Call when storage is disconnected
    pub fn clear_storage_status(&mut self) {
        self.storage_full = false;
        self.storage_error = false;
        self.record_count = 0;
    }

    pub fn inc_records(&mut self) {
        self.record_count = self.record_count.saturating_add(1);
    }
}
