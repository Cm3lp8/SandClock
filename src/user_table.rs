pub use main_type::SandClock;
pub use sync_insertion::*;
pub use time_update::{ClockEvent, TimeOutUpdate};
pub use timer_status::TimerStatus;
mod main_type {
    use std::{fmt::Debug, hash::Hash, marker::PhantomData, sync::Arc, time::Duration};

    use dashmap::DashMap;

    use crate::{
        InsertSync, SandClockInsertion, config::SandClockConfig, errors::SandClockError,
        timer_loop::TimerLoop,
    };

    use super::{
        time_update::{ClockEvent, TimeOutUpdate},
        timer_status::TimerStatus,
    };

    type UserId = usize;

    pub struct SandClockBuilder<K: SandClockInsertion + Debug> {
        time_out_event_call_back: Option<Arc<dyn Fn(TimeOutUpdate<K>) + Send + Sync + 'static>>,
        time_out_duration: Option<Duration>,
        config: SandClockConfig,
        phantom_data: PhantomData<K>,
    }
    impl<K: SandClockInsertion + Debug> SandClockBuilder<K> {
        pub fn set_time_out_event(
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

    pub struct SandClock<K: SandClockInsertion> {
        map: Arc<DashMap<InsertSync<K>, TimerStatus>>,
        config: SandClockConfig,
        time_out_duration: Duration,
    }
    impl<K: SandClockInsertion> Clone for SandClock<K> {
        fn clone(&self) -> Self {
            Self {
                map: self.map.clone(),
                config: self.config.clone(),
                time_out_duration: self.time_out_duration.clone(),
            }
        }
    }

    impl<K: SandClockInsertion + Debug> SandClock<K> {
        ///
        ///
        /// Creates a new [`SandClockBuilder<K>`] to configure and build a [`SandClock<K>`].
        ///
        /// The generic type `K` must implement the [`SandClockInsertion`] trait,
        /// which ensures the key is safely usable in a multi-threaded context.
        ///
        /// Internally, the key is wrapped in a type that provides compatibility with [`DashMap`] and atomic operations.
        ///
        /// ### Example
        /// ```rust
        /// let sand_clock = SandClock::<usize>::new(SandClockConfig::default())
        ///     .set_time_out_event(|clock_event| {
        ///         println!("Timeout for key: {:?}", clock_event.key());
        ///     })
        ///     .set_time_out_duration(Duration::from_secs(10));
        /// ```

        pub fn new(config: SandClockConfig) -> SandClockBuilder<K> {
            SandClockBuilder {
                time_out_event_call_back: None,
                time_out_duration: None,
                config,
                phantom_data: PhantomData::<K>,
            }
        }
        ////// Inserts a new key into the `SandClock`, or updates its timer if it already exists.
        ///
        /// When inserting a key for the first time, a new [`TimerStatus`] is created and tracked.
        /// If the key already exists, its associated timer is refreshed with a new [`Instant::now()`],
        /// effectively extending its lifetime within the clock.
        ///
        /// This function is typically called periodically to signal that the entity associated
        /// with the key is still active.
        ///
        /// ### Example
        /// ```rust
        /// sand_clock.insert_or_update_timer(user_id);
        /// ```

        pub fn insert_or_update_timer(&self, key: K) {
            self.map
                .entry(key.to_insert_sync())
                .and_modify(|conn_status| conn_status.time_out_handler().update_timer())
                .or_insert(TimerStatus::new());
        }
    }
}

mod timer_status {
    use super::time_out::Timer;

    /// Stores timeout-related state for a key registered in the [`SandClock`].
    ///
    /// This struct keeps track of whether the associated entity has expired,
    /// and holds a [`Timer`] used to measure elapsed inactivity time.
    ///
    /// It is managed internally by the `SandClock` and usually does not need
    /// to be manipulated directly by users.
    ///
    /// ### Fields
    /// - `expired`: A flag indicating whether the timeout has already occurred.
    /// - `time_out`: A [`Timer`] that tracks the time since last activity.

    #[derive(Clone)]
    pub struct TimerStatus {
        expired: bool,
        time_out: Timer,
    }

    impl TimerStatus {
        /// Creates a new, non-expired [`TimerStatus`] with a fresh internal timer.
        pub fn new() -> Self {
            Self {
                expired: false,
                time_out: Timer::new(),
            }
        }
        /// Marks this status as expired.
        ///
        /// This is called internally when a timeout is detected.
        pub fn expired(&mut self) {
            self.expired = true;
        }
        /// Returns `true` if this status has been marked as expired.
        ///
        /// This can be used to skip already-handled entries in the timeout loop.
        pub fn is_expired(&self) -> bool {
            self.expired
        }
        /// Returns a mutable reference to the internal [`Timer`],
        /// allowing updates (e.g., to refresh the last activity time).
        ///
        /// This is used internally when an entity signals that it is still active.
        pub fn time_out_handler(&mut self) -> &mut Timer {
            &mut self.time_out
        }
        /// Returns an immutable reference to the internal [`Timer`],
        /// allowing inspection of the last activity timestamp.
        pub fn time_out_info(&self) -> &Timer {
            &self.time_out
        }
    }
}

mod time_out {
    use std::time::Instant;

    /// Lightweight timer used internally by [`TimerStatus`] to track activity timestamps.
    ///
    /// `Timer` holds the last known [`Instant`] at which the associated entity signaled activity.
    /// It is updated whenever the entity is considered active, and can be queried to determine
    /// how much time has passed since the last signal.
    ///
    /// This struct is intended for internal use within [`SandClock`].
    #[derive(Clone)]
    pub struct Timer {
        last_update: Instant,
    }
    impl Timer {
        /// Creates a new `Timer` initialized with the current time (`Instant::now()`).
        pub fn new() -> Self {
            Self {
                last_update: Instant::now(),
            }
        }
        /// Returns the [`Instant`] of the last recorded update.
        ///
        /// This can be used to compute the elapsed time since the last activity signal.
        pub fn get_last_instant_update(&self) -> Instant {
            self.last_update
        }
        /// Updates the internal timestamp to the current time.
        ///
        /// This should be called whenever the entity associated with this timer signals activity.
        pub fn update_timer(&mut self) {
            self.last_update = Instant::now();
        }
    }
}

mod time_update {
    use std::{
        fmt::{Debug, Display},
        hash::Hash,
    };

    use crate::{InsertSync, SandClockInsertion};

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

    /// Information passed to the timeout event callback.
    ///
    /// Contains the key and the [`ClockEvent`] that triggered the timeout.
    pub struct TimeOutUpdate<K: SandClockInsertion + Debug> {
        key: InsertSync<K>,
        event: ClockEvent,
    }

    impl<K: SandClockInsertion + Debug> TimeOutUpdate<K> {
        pub fn new(key: InsertSync<K>, event: ClockEvent) -> Self {
            Self { key, event }
        }
        pub fn key(&self) -> InsertSync<K> {
            self.key.clone()
        }
        pub fn event(&self) -> ClockEvent {
            self.event
        }
    }
}

mod sync_insertion {
    use std::{any::TypeId, hash::Hash, ops::Deref, sync::Arc};

    /// ```sync_insertion``` defines utils to safely use DashMap with Send + Sync key.

    /// Wrapper enum used internally by `SandClock` to safely store keys in a concurrent map.
    ///
    /// `InsertSync<T>` allows your key type `T` to be used inside a multithreaded [`DashMap`],
    /// even if `T` is not `Sync` by itself.
    ///
    /// This is done by supporting two variants:
    /// - [`Plain(T)`] for keys that are inherently `Send + Sync`,
    /// - [`Shared(Arc<T>)`] for types wrapped explicitly in an `Arc`.
    ///
    /// This abstraction allows `SandClock` to handle both simple key types (`usize`, `String`)
    /// and more complex ones that require reference counting.
    ///
    /// You typically don’t need to construct `InsertSync` manually. It is generated via the
    /// [`SandClockInsertion::to_insert_sync()`] trait implementation.
    #[derive(Hash, PartialEq, Eq, Debug)]
    pub enum InsertSync<T> {
        /// A plain, owned value of type `T`.
        Plain(T),
        /// A shared reference-counted value (`Arc<T>`), used for keys that need `Sync` guarantee.
        Shared(Arc<T>),
    }

    impl<T> Deref for InsertSync<T> {
        type Target = T;
        fn deref(&self) -> &Self::Target {
            match self {
                Self::Plain(v) => &v,
                Self::Shared(v) => &v,
            }
        }
    }
    impl<T: Clone> Clone for InsertSync<T> {
        fn clone(&self) -> Self {
            match self {
                Self::Plain(v) => Self::Plain(v.clone()),
                Self::Shared(v) => Self::Shared(v.clone()),
            }
        }
    }

    impl<T: Clone> InsertSync<T> {
        pub fn into_inner(self) -> T {
            match self {
                InsertSync::Plain(v) => v,
                InsertSync::Shared(v) => (*v).clone(),
            }
        }
    }
    /// Trait implemented by types that can be used as keys in a [`SandClock`] instance.
    ///
    /// This trait ensures that keys are:
    /// - clonable,
    /// - hashable,
    /// - comparable (via `Eq`),
    /// - thread-safe (`Send + Sync`),
    /// - and `'static`, to allow use across threads.
    ///
    /// The `to_insert_sync` method wraps the key into an [`InsertSync`] type, which abstracts
    /// over plain and shared references (`Arc<T>`) for internal storage in the concurrent map.
    ///
    /// ### Note
    /// This trait is **not object-safe**, and cannot be used as a trait object (`dyn SandClockInsertion`).
    /// It is designed to be automatically implemented for most common key types.
    ///
    /// ### Blanket implementation
    /// ```rust
    /// impl<T: Send + Sync + Clone + Eq + Hash + 'static> SandClockInsertion for T
    /// ```
    /// This means you can use `usize`, `String`, `Arc<T>`, or any other `T` satisfying the bounds.

    pub trait SandClockInsertion: Sized + Send + Sync + Clone + Hash + Eq + 'static {
        ///
        ////// Converts the key into an [`InsertSync`] wrapper used internally by `SandClock`.
        ///
        /// This is called automatically when inserting or updating a key.
        ///
        /// You usually don’t need to call this directly unless you're extending the internal logic.
        fn to_insert_sync(self) -> InsertSync<Self>;
    }

    impl<T: Send + Sync + Clone + Eq + Hash + 'static> SandClockInsertion for T {
        fn to_insert_sync(self) -> InsertSync<Self> {
            if std::mem::size_of::<T>() <= 8 {
                InsertSync::Plain(self)
            } else {
                InsertSync::Shared(Arc::new(self))
            }
        }
    }
}
