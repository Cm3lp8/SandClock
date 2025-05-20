//!# SandClock ⏳
//!
//! ## Purpose
//!
//! **SandClock** is a time-aware `HashMap` designed to track whether a given entity is still active.  
//! It’s ideal for use cases such as presence detection, ephemeral sessions, or activity timeouts.
//!
//!  ⚙️ Runtime-free design: SandClock uses a single background thread and requires no async runtime.
//!
//! ## Quick links
//!
//! - [`SandClock`] — main entry point, used to insert or update tracked entities
//! - [`SandClockConfig`] — configures the loop frequency
//! - [`ClockEvent`] — type of events triggered on timeout
//! - [`TimeOutUpdate`] — passed to your callback when a timeout occurs
//! - [`InsertSync`] — internal wrapper around your key type
//!
//! ## How it works
//!
//! Each tracked entity can periodically **signal** the SandClock using its associated key.
//!
//! If signaling stops within a defined timeout, SandClock **automatically triggers a callback**, notifying the caller that the entity is no longer active.  
//! The entity is then removed from the map.
//!
//! Internally, SandClock uses a lightweight polling loop to monitor timeouts.
//! This approach aims to avoid the complexity of timers per entry, while maintaining predictable performance
//!
//! ### Example
//!
//! ```rust
//!
//!  use sand_clock::prelude::*;
//!  use std::time::Duration;
//!
//!  //Configure the clock, set the time-checking frequency :
//!  let config = SandClockConfig::new().frequency(Duration::from_millis(200)); // ou SandClockConfig::default();
//!  //Default is set to 2 seconds.
//!
//! //Instantiate the SandClock, with the key type as generic argument.
//! let user_connection_base = SandClock::<String>::new(config)
//!    .set_time_out_event(move |conn_update| match conn_update.event() {
//!        ClockEvent::TimeOut => {
//!            println!("No more known activity: [{:?}] has disconnected", conn_update.key());
//!        }
//!    })
//!    .set_time_out_duration(Duration::from_millis(15_000))
//!    .build()
//!    .unwrap();
//!
//! // *Signals* :
//! // New activity !
//! user_connection_base.insert_or_update_timer("alf".to_string());
//!
//! // Activity continue !
//! user_connection_base.insert_or_update_timer("alf".to_string());
//!
//! ```

pub mod config;

pub mod errors;
#[cfg(test)]
mod test;
pub mod timer_loop;
pub mod user_table;

//pub use config::SandClockConfig;
//pub use errors::SandClockError;
//pub use user_table::{ClockEvent, InsertSync, SandClock, SandClockInsertion};

pub mod prelude {
    pub use super::{
        config::SandClockConfig, errors::SandClockError, user_table::ClockEvent,
        user_table::InsertSync, user_table::SandClock, user_table::SandClockInsertion,
        user_table::TimeOutUpdate,
    };
}

pub use {
    config::SandClockConfig, errors::SandClockError, user_table::ClockEvent,
    user_table::InsertSync, user_table::SandClock, user_table::SandClockInsertion,
    user_table::TimeOutUpdate,
};
