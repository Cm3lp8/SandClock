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

    // Configuration of the SandClock : set the time-checking frequency.
    let config = SandClockConfig::new().frequence(Duration::from_millis(200)); // or SandClockConfig::default();
                                                                               // which is set to 2 seconds.

    // Instantiate the SandClock with the key type passed in the generic parameter.
    let user_connection_base = SandClock::<String>::new(config)
               .with_time_out_event(move |conn_update| match conn_update.event() {
                             ClockEvent::TimeOut => {
                                println!(" No more known activity :  [{:?}] has deconnected", conn_update.key());
                                                    }
                                  })
               .set_time_out_duration(time_out_duration) 
               .build()
               .unwrap();

// New activity ! You can insert the key for the entity like this :
 user_connection_base.insert_or_update_timer("alf".to_string());

// Activity continue ! you signal the clock like the same way :
 user_connection_base.insert_or_update_timer("alf".to_string());


```
