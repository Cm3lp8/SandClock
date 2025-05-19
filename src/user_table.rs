pub use main_type::SandClock;
pub use time_out::Timer;
pub use time_update::{ClockEvent, TimeOutUpdate};
pub use timer_status::TimerStatus;
mod main_type {
    use std::{fmt::Debug, hash::Hash, marker::PhantomData, sync::Arc, time::Duration};

    use dashmap::DashMap;

    use crate::{config::SandClockConfig, errors::SandClockError, timer_loop::TimerLoop};

    use super::{
        time_update::{ClockEvent, TimeOutUpdate},
        timer_status::TimerStatus,
    };

    type UserId = usize;

    pub struct SandClockBuilder<K: Hash + Eq + Send + Sync + Copy> {
        time_out_event_call_back: Option<Arc<dyn Fn(TimeOutUpdate<K>) + Send + Sync + 'static>>,
        time_out_duration: Option<Duration>,
        config: SandClockConfig,
        phantom_data: PhantomData<K>,
    }
    impl<K: Hash + Eq + Copy + Send + Sync + 'static + Debug> SandClockBuilder<K> {
        pub fn with_time_out_event(
            &mut self,
            t_o_event: impl Fn(TimeOutUpdate<K>) + Send + Sync + 'static,
        ) -> &mut Self {
            self.time_out_event_call_back = Some(Arc::new(t_o_event));
            self
        }
        pub fn set_time_out_duration(&mut self, time_out_duration: Duration) -> &mut Self {
            self.time_out_duration = Some(time_out_duration);
            self
        }
        pub fn build(&mut self) -> Result<SandClock<K>, SandClockError> {
            if let Some(time_out) = self.time_out_event_call_back.take() {
                let map = Arc::new(DashMap::new());

                let time_out_duration: Duration =
                    if let Some(duration) = self.time_out_duration.take() {
                        duration
                    } else {
                        return Err(SandClockError::BuildErrorNoDurationSet);
                    };

                let time_out_sender =
                    TimerLoop::run(&self.config, &map, &time_out, time_out_duration);
                Ok(SandClock {
                    map,
                    config: std::mem::replace(&mut self.config, SandClockConfig::default()),
                    time_out_duration,
                })
            } else {
                Err(SandClockError::BuildErrorNoTimeOutSet)
            }
        }
    }

    pub struct SandClock<K: Hash + Eq + Copy + Debug> {
        map: Arc<DashMap<K, TimerStatus>>,
        config: SandClockConfig,
        time_out_duration: Duration,
    }

    impl<K: Hash + Eq + Copy + Send + Sync + 'static + Debug> SandClock<K> {
        pub fn new(config: SandClockConfig) -> SandClockBuilder<K> {
            SandClockBuilder {
                time_out_event_call_back: None,
                time_out_duration: None,
                config,
                phantom_data: PhantomData::<K>,
            }
        }
        /// Key insertion with creation of the associated TimerStatus.
        /// If the K is already set, the function updates the timer of the TimerStatus with
        /// new Instant.
        pub fn insert_or_update_timer(&self, key: K) {
            self.map
                .entry(key)
                .and_modify(|conn_status| conn_status.time_out_handler().update_timer())
                .or_insert(TimerStatus::new());
        }
    }
}

mod timer_status {
    use super::time_out::Timer;

    /// Give informations about the connection status of the registered user.
    #[derive(Clone)]
    pub struct TimerStatus {
        expired: bool,
        time_out: Timer,
    }

    impl TimerStatus {
        pub fn new() -> Self {
            Self {
                expired: false,
                time_out: Timer::new(),
            }
        }
        pub fn expired(&mut self) {
            self.expired = true;
        }
        pub fn is_expired(&self) -> bool {
            self.expired
        }
        pub fn time_out_handler(&mut self) -> &mut Timer {
            &mut self.time_out
        }
        pub fn time_out_info(&self) -> &Timer {
            &self.time_out
        }
    }
}

mod time_out {
    use std::time::Instant;

    /// TimerStatus stores the last instant updated.
    #[derive(Clone)]
    pub struct Timer {
        last_update: Instant,
    }
    impl Timer {
        pub fn new() -> Self {
            Self {
                last_update: Instant::now(),
            }
        }
        pub fn get_last_instant_update(&self) -> Instant {
            self.last_update
        }
        pub fn update_timer(&mut self) {
            self.last_update = Instant::now();
        }
    }
}

mod time_update {
    use std::{fmt::Display, hash::Hash};

    type UserId = usize;

    #[derive(Clone, Copy, Debug)]
    pub enum ClockEvent {
        TimeOut,
    }

    impl Display for ClockEvent {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match self {
                Self::TimeOut => {
                    write!(f, "Connnection timout ! ")
                }
            }
        }
    }

    /// TimeOutUpdate represents an information about an user connection that can be passed in
    /// time_out callback.
    pub struct TimeOutUpdate<K: Hash + Eq + Copy> {
        key: K,
        event: ClockEvent,
    }
    impl<K: Hash + Eq + Copy> TimeOutUpdate<K> {
        pub fn new(key: K, event: ClockEvent) -> Self {
            Self { key, event }
        }
        pub fn key(&self) -> K {
            self.key
        }
        pub fn event(&self) -> ClockEvent {
            self.event
        }
    }
}
