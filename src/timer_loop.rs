use dashmap::DashMap;

use crate::user_table::{ConnectionEvent, ConnectionStatus, ConnectionUpdate};
use std::{
    fmt::write,
    sync::Arc,
    time::{Duration, Instant},
};

type UserId = usize;
pub struct TimerLoop {
    t_o_cb: Arc<dyn Fn(ConnectionUpdate) + Send + Sync + 'static>,
    map: Arc<DashMap<UserId, ConnectionStatus>>,
}

impl TimerLoop {
    /// TimeLoop runs on a separate thread and loop over registered entries. Each loop round compares
    /// Instant::now() with the last updated instant value of the current entry iterated.
    /// If the duration between the two is greater than the tim_out threshold set by user, time_out
    /// event callback can be called.
    ///
    ///
    pub fn run(
        map: &Arc<DashMap<UserId, ConnectionStatus>>,
        t_o_cb: &Arc<dyn Fn(ConnectionUpdate) + Send + Sync + 'static>,
        time_out: Duration,
    ) {
        let timer_loop: TimerLoop = TimerLoop {
            t_o_cb: t_o_cb.clone(),
            map: map.clone(),
        };
        let map = map.clone();
        let t_o_cb = t_o_cb.clone();

        std::thread::spawn(move || {
            loop {
                let mut conn_it = map.iter_mut();
                'inner_it: loop {
                    if let Some(mut connection_status_ref) = conn_it.next() {
                        let connection_status = connection_status_ref.value();

                        if connection_status.is_disconnected() {
                            std::thread::sleep(Duration::from_millis(100));
                            continue 'inner_it;
                        }
                        let last_updated_instant =
                            connection_status.time_out_info().get_last_instant_update();

                        let now = Instant::now();
                        let duration_since = now.duration_since(last_updated_instant);

                        if now.duration_since(last_updated_instant) >= time_out {
                            (*t_o_cb)(ConnectionUpdate::new(
                                *connection_status_ref.key(),
                                ConnectionEvent::TimeOut,
                            ));
                            connection_status_ref.value_mut().disconnected();
                        };
                    } else {
                        //todo clean the map (remove disconnected)
                        break 'inner_it;
                    }
                    std::thread::sleep(Duration::from_millis(100));
                }
            }
        });
    }
}
