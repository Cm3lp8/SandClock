use std::time::{Duration, Instant};

use crate::user_table::*;

#[test]
fn test_users_connection_table() {
    let user_connection_base = UsersConnectedBase::new()
        .with_time_out_event(|conn_update| { /**/ })
        .set_time_out_duration(Duration::from_secs(5))
        .build()
        .unwrap();

    user_connection_base.insert_or_update_timer(0);
}

type UserId = usize;
#[test]
fn test_timer() {
    let channel = crossbeam_channel::bounded::<(UserId, bool)>(1);
    let sender = channel.0.clone();
    let time_out_duration = Duration::from_secs(6);
    let user_connection_base = UsersConnectedBase::new()
        .with_time_out_event(move |conn_update| match conn_update.connection_event() {
            ConnectionEvent::TimeOut => {
                //                log::info!("User [{}] deconnected", conn_update.id());
                sender.send((conn_update.id(), true)).unwrap();
            }
        })
        .set_time_out_duration(time_out_duration)
        .build()
        .unwrap();

    user_connection_base.insert_or_update_timer(0);
    user_connection_base.insert_or_update_timer(1);

    let mut has_deco = false;
    let elasped = Instant::now();
    let mut user_0_deconnection_confirmed = false;
    let mut user_1_deconnection_confirmed = false;
    'test: loop {
        if let Ok((user_id, has_deconnected)) = channel.1.try_recv() {
            has_deco = has_deconnected;

            if user_id == 0 {
                user_0_deconnection_confirmed = true;
            }
            if user_id == 1 {
                user_1_deconnection_confirmed = true;
            }
        }
        std::thread::sleep(Duration::from_millis(300));
        if elasped.elapsed() > time_out_duration + Duration::from_secs(3) {
            break 'test;
        }

        if elasped.elapsed() > Duration::from_secs(4) {
            user_connection_base.insert_or_update_timer(1);
        }
    }

    assert!(user_0_deconnection_confirmed && !user_1_deconnection_confirmed);
}
