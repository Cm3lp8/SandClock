//! SandClock Configuration
use std::time::Duration;

/// Configuration object for a [`SandClock`] instance.
///
/// `SandClockConfig` allows you to control the internal polling loop
/// frequency that determines how often timeouts are checked.
///
/// This configuration is optional: using [`SandClockConfig::default()`]
/// gives a 2-second polling interval, which is suitable for most use cases.
///
/// To customize the behavior, use [`SandClockConfig::new()`] and adjust the
/// frequency using [`SandClockConfig::frequency()`].
///
/// # Example
///
/// ```rust
/// use sand_clock::SandClockConfig;
/// use std::time::Duration;
///
/// let config = SandClockConfig::new()
///     .frequency(Duration::from_millis(500)); // Check timeouts every 500ms
/// ```

#[derive(Clone)]
pub struct SandClockConfig {
    refresh_duration: Duration,
}

impl Default for SandClockConfig {
    /// Returns a `SandClockConfig` with a default polling interval of 2 seconds.
    ///
    /// Equivalent to:
    /// ```rust
    /// use std::time::Duration;
    /// use sand_clock::SandClockConfig;
    /// SandClockConfig::new().frequency(Duration::from_secs(2));
    /// ```
    fn default() -> Self {
        Self {
            refresh_duration: Duration::from_millis(1000),
        }
    }
}
impl SandClockConfig {
    /// Creates a new [`SandClockConfig`] instance with default values.
    ///
    /// To customize the frequency, use [`Self::frequency()`] on the returned config.
    ///
    /// This method is equivalent to [`Default::default()`].
    pub fn new() -> Self {
        Self::default()
    }
    /// Returns the duration currently set as the polling interval.
    ///
    /// This is the frequency at which the internal timer loop runs to check for expired entries.
    pub fn get_timer_loop_refreshing_duration(&self) -> Duration {
        self.refresh_duration
    }
    pub fn frequency(mut self, frequence_duration: Duration) -> Self {
        self.refresh_duration = frequence_duration;
        self
    }
}
