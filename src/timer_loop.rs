use dashmap::DashMap;
use log::info;
use rayon::ThreadPoolBuilder;

use crate::{
    InsertSync, SandClockInsertion,
    config::SandClockConfig,
    user_table::{ClockEvent, ClockEventIntern, TimerStatus},
};
use std::{
    fmt::Debug,
    sync::{
        Arc,
        atomic::{AtomicBool, AtomicUsize},
    },
    time::{Duration, Instant},
};

#[allow(dead_code)]
pub struct TimerLoop<K: SandClockInsertion + Debug> {
    t_o_cb: Arc<dyn Fn(ClockEvent<K>) + Send + Sync + 'static>,
    map: Arc<DashMap<InsertSync<K>, TimerStatus>>,
}

impl<K: SandClockInsertion + Debug> TimerLoop<K> {
    ///
    ////// Starts the internal timer loop in a dedicated background thread.
    ///
    /// This loop periodically scans all registered entries in the `SandClock`.
    /// For each entry, it compares the current time (`Instant::now()`) with the last recorded update time.
    /// If the elapsed duration exceeds the user-defined timeout threshold,
    /// the corresponding timeout event callback is triggered.
    ///
    /// The refresh interval of the loop is configured via the provided [`SandClockConfig`].
    ///
    /// This method is called internally by [`SandClock::build()`] and should not need
    /// to be invoked manually under normal usage.
    ///
    /// # Arguments
    /// - `config`: The configuration object that sets the refresh interval of the loop.
    /// - `map`: Shared concurrent map storing entries and their timeout info.
    /// - `t_o_cb`: User-defined callback triggered on timeout.
    /// - `time_out`: The duration threshold beyond which a key is considered inactive.
    ///
    /// # Note
    /// Expired entries are removed after each polling cycle to free resources.
    pub fn run(
        config: &SandClockConfig,
        counter: &Arc<AtomicUsize>,
        map: &Arc<DashMap<InsertSync<K>, TimerStatus>>,
        t_o_cb: &Arc<dyn Fn(ClockEvent<K>) + Send + Sync + 'static>,
        time_out: Duration,
        closing_trigger: &Arc<AtomicBool>,
    ) {
        let _timer_loop: TimerLoop<K> = TimerLoop {
            t_o_cb: t_o_cb.clone(),
            map: map.clone(),
        };
        let map = map.clone();
        let t_o_cb = t_o_cb.clone();

        let (job_sender, job_receiver) =
            crossbeam_channel::unbounded::<ClockEventIntern<InsertSync<K>>>();

        let counter = counter.clone();
        let closing_trigger_0 = closing_trigger.clone();
        std::thread::spawn(move || {
            if let Ok(thread_pool) = ThreadPoolBuilder::new().num_threads(4).build() {
                while let Ok(key) = job_receiver.recv() {
                    let close = thread_pool.install(|| match key {
                        ClockEventIntern::TimeOutIntern(key) => {
                            (*t_o_cb)(ClockEvent::TimeOut(key.into_inner()));
                            false
                        }
                        ClockEventIntern::SandClockDrop => {
                            (*t_o_cb)(ClockEvent::SandClockDrop);
                            true
                        }
                    });
                    if close {
                        break;
                    }
                }
            }
        });

        let refresh_duration = config.get_timer_loop_refreshing_duration();

        std::thread::spawn(move || {
            let mut expired_queue: Vec<InsertSync<K>> = vec![];

            'outter: loop {
                if closing_trigger_0.load(std::sync::atomic::Ordering::Relaxed) {
                    // Send a Close event to the time_out callback.
                    if let Err(e) = job_sender.send(ClockEventIntern::SandClockDrop) {
                        info!("Failed to Send Drop Signal to user [{e:?}]");
                    }
                    // stops the loops, expires the thread.
                    break 'outter;
                }
                let mut conn_it = map.iter_mut();

                let now = Instant::now();

                'inner_it: loop {
                    if let Some(mut connection_status_ref) = conn_it.next() {
                        let connection_status = connection_status_ref.value();

                        if connection_status.is_expired() {
                            continue 'inner_it;
                        }
                        let last_updated_instant =
                            connection_status.time_out_info().get_last_instant_update();

                        if now.duration_since(last_updated_instant) >= time_out {
                            let key = connection_status_ref.key().clone();

                            if let Err(e) =
                                job_sender.send(ClockEventIntern::TimeOutIntern(key.clone()))
                            {
                                info!("failed to externalize the expired key [{e:?}]");
                            }
                            connection_status_ref.value_mut().expired();

                            // store expired keys in queue and clean the map later.
                            if !expired_queue.contains(&key) {
                                expired_queue.push(key);
                            }
                        }
                    } else {
                        break 'inner_it;
                    }
                }
                std::thread::sleep(refresh_duration);

                let removables = expired_queue.len();
                for k in &expired_queue {
                    map.remove(k);
                }
                counter.fetch_sub(removables, std::sync::atomic::Ordering::Relaxed);
            }
        });
    }
}
