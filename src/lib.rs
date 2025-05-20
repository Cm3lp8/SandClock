mod config;
mod errors;
#[cfg(test)]
mod test;
mod timer_loop;
mod user_table;

pub use config::SandClockConfig;
pub use errors::SandClockError;
pub use user_table::{ClockEvent, InsertSync, SandClock, SandClockInsertion};

pub mod prelude {
    use super::{
        ClockEvent, InsertSync, SandClock, SandClockConfig, SandClockError, SandClockInsertion,
    };
}
