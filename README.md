![Build_Status](https://img.shields.io/badge/build-ok-green)
![dev_status](https://img.shields.io/badge/dev--status-beta-blue)

# SandClock ⏳

 ## Purpose

 **SandClock** is a time-aware `HashMap` designed to track whether a given entity is still active.  
 It’s ideal for use cases such as presence detection, ephemeral sessions, or activity timeouts.

  ⚙️ Runtime-free design: SandClock uses a single background thread and requires no async runtime.

 ## How it works

 Each tracked entity can periodically **signal** the SandClock using its associated key.

 If signaling stops within a defined timeout, SandClock **automatically triggers a callback**, notifying the caller that the entity is no longer active.  
 The entity is then removed from the map.

 Internally, SandClock uses a lightweight polling loop to monitor timeouts.
 This approach aims to avoid the complexity of timers per entry, while maintaining predictable performance

 ### Example

 ```rust

  use sand_clock::prelude::*;

  //Configure the clock, set the time-checking frequency :
  let config = SandClockConfig::new().frequency(Duration::from_millis(200)); // ou SandClockConfig::default();
  //Default is set to 2 seconds.

 //Instantiate the SandClock, with the key type as generic argument.
 let user_connection_base = SandClock::<String>::new(config)
    .set_time_out_event(move |conn_update| match conn_update.event() {
        ClockEvent::TimeOut => {
            println!("No more known activity: [{:?}] has disconnected", conn_update.key());
        }
    })
    .set_time_out_duration(time_out_duration)
    .build()
    .unwrap();

 // *Signals* :
 // New activity !
 user_connection_base.insert_or_update_timer("alf".to_string());

 // Activity continue !
 user_connection_base.insert_or_update_timer("alf".to_string());

 ```

## Disclaimers

- SandClock uses a polling mechanism, so timeouts are not accurate to the millisecond.  
  **Do not use this crate if your application requires precise timeout accuracy.**
- This crate is currently in beta and has only been tested in my personal projects.  
  Use it at your own risk.

## License

This project is licensed under the MIT OR Apache-2.0 License.
