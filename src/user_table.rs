pub use connection_status::ConnectionStatus;
pub use connection_time_out::ConnectionTimout;
pub use connection_update::{ConnectionEvent, ConnectionUpdate};
pub use main_type::UsersConnectedBase;
mod main_type {
    use std::{sync::Arc, time::Duration};

    use dashmap::DashMap;

    use crate::{errors::UserConnectedBaseError, timer_loop::TimerLoop};

    use super::{connection_status::ConnectionStatus, connection_update::ConnectionUpdate};

    type UserId = usize;

    pub struct UsersConnectedBaseBuilder {
        time_out_event_call_back: Option<Arc<dyn Fn(ConnectionUpdate) + Send + Sync + 'static>>,
        time_out_duration: Option<Duration>,
    }
    impl UsersConnectedBaseBuilder {
        pub fn with_time_out_event(
            &mut self,
            t_o_event: impl Fn(ConnectionUpdate) + Send + Sync + 'static,
        ) -> &mut Self {
            self.time_out_event_call_back = Some(Arc::new(t_o_event));
            self
        }
        pub fn set_time_out_duration(&mut self, time_out_duration: Duration) -> &mut Self {
            self.time_out_duration = Some(time_out_duration);
            self
        }
        pub fn build(&mut self) -> Result<UsersConnectedBase, UserConnectedBaseError> {
            if let Some(time_out) = self.time_out_event_call_back.take() {
                let map = Arc::new(DashMap::new());

                let time_out_duration: Duration =
                    if let Some(duration) = self.time_out_duration.take() {
                        duration
                    } else {
                        return Err(UserConnectedBaseError::BuildErrorNoDurationSet);
                    };

                let time_out_sender = TimerLoop::run(&map, &time_out, time_out_duration);
                Ok(UsersConnectedBase {
                    map,
                    time_out_duration,
                })
            } else {
                Err(UserConnectedBaseError::BuildErrorNoTimeOutSet)
            }
        }
    }

    pub struct UsersConnectedBase {
        map: Arc<DashMap<UserId, ConnectionStatus>>,
        time_out_duration: Duration,
    }

    impl UsersConnectedBase {
        pub fn new() -> UsersConnectedBaseBuilder {
            UsersConnectedBaseBuilder {
                time_out_event_call_back: None,
                time_out_duration: None,
            }
        }
        /// User insertion with creation of the associated ConnectionStatus.
        /// If the K is already set, the function updates the timer of the ConnectionTimout with
        /// new Instant.
        pub fn insert_or_update_timer(&self, user_id: UserId) {
            self.map
                .entry(user_id)
                .and_modify(|conn_status| conn_status.time_out_handler().update_connection_status())
                .or_insert(ConnectionStatus::new());
        }
    }
}

mod connection_status {
    use super::connection_time_out::ConnectionTimout;

    /// Give informations about the connection status of the registered user.
    pub struct ConnectionStatus {
        connected: bool,
        time_out: ConnectionTimout,
    }

    impl ConnectionStatus {
        pub fn new() -> Self {
            Self {
                connected: true,
                time_out: ConnectionTimout::new(),
            }
        }
        pub fn disconnected(&mut self) {
            self.connected = false;
        }
        pub fn is_disconnected(&self) -> bool {
            !self.connected
        }
        pub fn time_out_handler(&mut self) -> &mut ConnectionTimout {
            &mut self.time_out
        }
        pub fn time_out_info(&self) -> &ConnectionTimout {
            &self.time_out
        }
    }
}

mod connection_time_out {
    use std::time::Instant;

    /// ConnectionTimout stores the last instant updated.
    pub struct ConnectionTimout {
        last_update: Instant,
    }
    impl ConnectionTimout {
        pub fn new() -> Self {
            Self {
                last_update: Instant::now(),
            }
        }
        pub fn get_last_instant_update(&self) -> Instant {
            self.last_update
        }
        pub fn update_connection_status(&mut self) {
            self.last_update = Instant::now();
        }
    }
}

mod connection_update {
    use std::fmt::Display;

    type UserId = usize;

    #[derive(Clone, Copy, Debug)]
    pub enum ConnectionEvent {
        TimeOut,
    }

    impl Display for ConnectionEvent {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match self {
                Self::TimeOut => {
                    write!(f, "Connnection timout ! ")
                }
            }
        }
    }

    /// ConnectionUpdate represents an information about an user connection that can be passed in
    /// time_out callback.
    pub struct ConnectionUpdate {
        user_id: UserId,
        connection_event: ConnectionEvent,
    }
    impl ConnectionUpdate {
        pub fn new(user_id: usize, connection_event: ConnectionEvent) -> Self {
            Self {
                user_id,
                connection_event,
            }
        }
        pub fn id(&self) -> UserId {
            self.user_id
        }
        pub fn connection_event(&self) -> ConnectionEvent {
            self.connection_event
        }
    }
}
