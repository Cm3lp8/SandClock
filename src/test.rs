use std::{
    net::Ipv4Addr,
    time::{Duration, Instant},
};

use crate::prelude::*;

#[test]
fn test_users_connection_table() {
    let user_connection_base = SandClock::new(SandClockConfig::default())
        .set_time_out_event(|_conn_update| { /**/ })
        .set_time_out_duration(Duration::from_secs(5))
        .build()
        .unwrap();

    user_connection_base.insert_or_update_timer(0);

    let user_connection_base = SandClock::new(SandClockConfig::default())
        .set_time_out_event(|_conn_update| { /**/ })
        .set_time_out_duration(Duration::from_secs(5))
        .build()
        .unwrap();

    user_connection_base.insert_or_update_timer(Ipv4Addr::new(192, 168, 1, 22));
}

#[test]
fn remove_key() {
    let user_connection_base = SandClock::new(SandClockConfig::default())
        .set_time_out_event(|_conn_update| { /**/ })
        .set_time_out_duration(Duration::from_secs(3))
        .build()
        .unwrap();

    user_connection_base.insert_or_update_timer(0);

    std::thread::sleep(Duration::from_secs(1));

    user_connection_base.remove_key(0);
    user_connection_base.remove_key(0);
    user_connection_base.remove_key(1);
    assert!(user_connection_base.get_entries_count() == 0);
}

#[test]
fn test_timer() {
    let channel = crossbeam_channel::bounded::<(String, bool)>(13);
    let sender = channel.0.clone();
    let time_out_duration = Duration::from_secs(6);
    let config = SandClockConfig::new().frequency(Duration::from_millis(200));
    let mut user_0_deconnection_confirmed = false;
    let mut user_1_deconnection_confirmed = false;
    {
        let user_connection_base = SandClock::<String>::new(config)
            .set_time_out_event(move |clock_event| match clock_event {
                ClockEvent::TimeOut(key) => {
                    println!("has_deconnected [{:?}]", key);
                    if let Err(e) = sender.send((key, true)) {
                        println!("Failed to send key deconnection info [{e:?}]")
                    }
                }
                ClockEvent::SandClockDrop => {
                    println!("Clock has dropped");
                }
            })
            .set_time_out_duration(time_out_duration)
            .build()
            .unwrap();

        user_connection_base.insert_or_update_timer("camille".to_string());
        user_connection_base.insert_or_update_timer("alf".to_string());
        /*
            user_connection_base.insert_or_update_timer("Jeannine");
            user_connection_base.insert_or_update_timer("Marje");
            user_connection_base.insert_or_update_timer("Franck");
            user_connection_base.insert_or_update_timer("Geo");
            user_connection_base.insert_or_update_timer("Albert");
            user_connection_base.insert_or_update_timer("Andrew");
            user_connection_base.insert_or_update_timer("Geraldine");
            user_connection_base.insert_or_update_timer("Dino");
            user_connection_base.insert_or_update_timer("Mel");
            user_connection_base.insert_or_update_timer("Elod");
        */
        let elasped = Instant::now();
        let mut updated = false;
        'test: loop {
            if let Ok((user_id, _has_deconnected)) = channel.1.try_recv() {
                if user_id == "alf" {
                    user_0_deconnection_confirmed = true;
                }
                if user_id == "camille" {
                    user_1_deconnection_confirmed = true;
                }
            }
            std::thread::sleep(Duration::from_millis(50));
            if elasped.elapsed() > time_out_duration + Duration::from_secs(2) {
                break 'test;
            }

            if elasped.elapsed() > Duration::from_secs(4) && !updated {
                user_connection_base.insert_or_update_timer("camille".to_string());
                user_connection_base.insert_or_update_timer("camille".to_string());
                updated = true;
            }
        }
    }
    println!(
        "user [{}]  user2 [{}]",
        user_0_deconnection_confirmed, user_1_deconnection_confirmed
    );
    assert!(user_0_deconnection_confirmed && !user_1_deconnection_confirmed);
}
