use std::time::Duration;

pub struct SandClockConfig {
    refresh_duration: Duration,
}

impl Default for SandClockConfig {
    fn default() -> Self {
        Self {
            refresh_duration: Duration::from_millis(1000),
        }
    }
}
impl SandClockConfig {
    pub fn new() -> Self {
        let config = Self::default();
        config
    }
    pub fn get_timer_loop_refreshing_duration(&self) -> Duration {
        self.refresh_duration
    }
    pub fn frequence(mut self, frequence_duration: Duration) -> Self {
        self.refresh_duration = frequence_duration;
        self
    }
}
