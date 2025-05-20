use dashmap::DashMap;

use crate::{
    InsertSync, SandClockInsertion,
    config::SandClockConfig,
    user_table::{ClockEvent, TimeOutUpdate, TimerStatus},
};
use std::{
    fmt::Debug,
    sync::Arc,
    time::{Duration, Instant},
};

pub struct TimerLoop<K: SandClockInsertion + Debug> {
    t_o_cb: Arc<dyn Fn(TimeOutUpdate<K>) + Send + Sync + 'static>,
    map: Arc<DashMap<InsertSync<K>, TimerStatus>>,
}

impl<K: SandClockInsertion + Debug> TimerLoop<K> {
    /// TimeLoop runs on a separate thread and loop over registered entries. Each loop round compares
    /// Instant::now() with the last updated instant value of the current entry iterated.
    /// If the duration between the two is greater than the tim_out threshold set by user, time_out
    /// event callback can be called.
    ///
    ///
    pub fn run(
        config: &SandClockConfig,
        map: &Arc<DashMap<InsertSync<K>, TimerStatus>>,
        t_o_cb: &Arc<dyn Fn(TimeOutUpdate<K>) + Send + Sync + 'static>,
        time_out: Duration,
    ) {
        let timer_loop: TimerLoop<K> = TimerLoop {
            t_o_cb: t_o_cb.clone(),
            map: map.clone(),
        };
        let map = map.clone();
        let t_o_cb = t_o_cb.clone();

        let refresh_duration = config.get_timer_loop_refreshing_duration();

        std::thread::spawn(move || {
            let mut expired_queue: Vec<InsertSync<K>> = vec![];
            loop {
                let mut conn_it = map.iter_mut();
                'inner_it: loop {
                    if let Some(mut connection_status_ref) = conn_it.next() {
                        let connection_status = connection_status_ref.value();

                        if connection_status.is_expired() {
                            continue 'inner_it;
                        }
                        let last_updated_instant =
                            connection_status.time_out_info().get_last_instant_update();

                        let now = Instant::now();
                        let duration_since = now.duration_since(last_updated_instant);

                        if now.duration_since(last_updated_instant) >= time_out {
                            let key = connection_status_ref.key().clone();
                            (*t_o_cb)(TimeOutUpdate::new(key.clone(), ClockEvent::TimeOut));
                            connection_status_ref.value_mut().expired();

                            // store expired keys in queue and clean the map later.
                            if !expired_queue.contains(&key) {
                                expired_queue.push(key);
                            }
                        };
                    } else {
                        //todo clean the map (remove disconnected)
                        break 'inner_it;
                    }
                }
                std::thread::sleep(refresh_duration);

                for k in &expired_queue {
                    map.remove(k);
                }
            }
        });
    }
}
