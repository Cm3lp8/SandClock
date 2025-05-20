![Build_Status](https://img.shields.io/badge/build-ok-green)
![dev_status](https://img.shields.io/badge/dev--status-beta-blue)

# SandClock ⏳

## Purpose
SandClock is a Rust module that can be used to track if an entity is still active.

## How it works
The tracked entity signals (updates) the  ```SandClock``` with its given key.
If signaling stops, a timeout triggers a callback that can inform the caller that the entity is no longer showing activity.
The timeout removes the entity from ```SandClock```.

### Example

```rust

 use sand_clock::prelude::*;

// Configure the clock, set the time-checking frequency :
let config = SandClockConfig::new().frequency(Duration::from_millis(200)); // ou SandClockConfig::default();
// Default is set to 2 seconds.

// Instantiate the SandClock, with the key type as generic argument. 
let user_connection_base = SandClock::<String>::new(config)
    .set_time_out_event(move |conn_update| match conn_update.event() {
        ClockEvent::TimeOut => {
            println!("No more known activity: [{:?}] has disconnected", conn_update.key());
        }
    })
    .set_time_out_duration(time_out_duration)
    .build()
    .unwrap();

// New activity !
user_connection_base.insert_or_update_timer("alf".to_string());

// Activity continue !
user_connection_base.insert_or_update_timer("alf".to_string());

```
