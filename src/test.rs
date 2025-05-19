use std::{
    net::Ipv4Addr,
    time::{Duration, Instant},
};

use crate::{config::SandClockConfig, user_table::*};

#[test]
fn test_users_connection_table() {
    let user_connection_base = SandClock::new(SandClockConfig::default())
        .with_time_out_event(|conn_update| { /**/ })
        .set_time_out_duration(Duration::from_secs(5))
        .build()
        .unwrap();

    user_connection_base.insert_or_update_timer(0);

    let user_connection_base = SandClock::new(SandClockConfig::default())
        .with_time_out_event(|conn_update| { /**/ })
        .set_time_out_duration(Duration::from_secs(5))
        .build()
        .unwrap();

    user_connection_base.insert_or_update_timer(Ipv4Addr::new(192, 168, 1, 22));
}

type UserId = usize;
#[test]
fn test_timer() {
    let channel = crossbeam_channel::bounded::<(UserId, bool)>(13);
    let sender = channel.0.clone();
    let time_out_duration = Duration::from_secs(6);
    let config = SandClockConfig::new().frequence(Duration::from_millis(200));
    let user_connection_base = SandClock::new(config)
        .with_time_out_event(move |conn_update| match conn_update.event() {
            ClockEvent::TimeOut => {
                println!("has_deconnected [{}]", conn_update.key());
                //                log::info!("User [{}] deconnected", conn_update.id());
                sender.send((conn_update.key(), true)).unwrap();
            }
        })
        .set_time_out_duration(time_out_duration)
        .build()
        .unwrap();

    user_connection_base.insert_or_update_timer(0);
    user_connection_base.insert_or_update_timer(1);
    user_connection_base.insert_or_update_timer(2);
    user_connection_base.insert_or_update_timer(3);
    user_connection_base.insert_or_update_timer(4);
    user_connection_base.insert_or_update_timer(5);
    user_connection_base.insert_or_update_timer(6);
    user_connection_base.insert_or_update_timer(7);
    user_connection_base.insert_or_update_timer(8);
    user_connection_base.insert_or_update_timer(9);
    user_connection_base.insert_or_update_timer(10);
    user_connection_base.insert_or_update_timer(11);
    user_connection_base.insert_or_update_timer(12);
    user_connection_base.insert_or_update_timer(13);

    let elasped = Instant::now();
    let mut user_0_deconnection_confirmed = false;
    let mut user_1_deconnection_confirmed = false;
    let mut updated = false;
    'test: loop {
        if let Ok((user_id, has_deconnected)) = channel.1.try_recv() {
            if user_id == 0 {
                user_0_deconnection_confirmed = true;
            }
            if user_id == 1 {
                user_1_deconnection_confirmed = true;
            }
        }
        std::thread::sleep(Duration::from_millis(50));
        if elasped.elapsed() > time_out_duration + Duration::from_secs(2) {
            break 'test;
        }

        if elasped.elapsed() > Duration::from_secs(4) && !updated {
            user_connection_base.insert_or_update_timer(1);
            user_connection_base.insert_or_update_timer(1);
            updated = true;
        }
    }

    println!(
        "user [{}]  user2 [{}]",
        user_0_deconnection_confirmed, user_1_deconnection_confirmed
    );
    assert!(user_0_deconnection_confirmed && !user_1_deconnection_confirmed);
}
