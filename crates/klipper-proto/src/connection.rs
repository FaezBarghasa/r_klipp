//! Connection Lifecycle and Graceful Disconnect/Reconnect Manager.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnectionState {
    Disconnected,
    Connecting,
    Handshake,
    Connected,
    Reconnecting { attempt: u32 },
    Faulted,
}

pub struct ConnectionManager {
    pub state: ConnectionState,
    pub max_reconnect_attempts: u32,
    pub heartbeat_interval_ms: u32,
    pub heartbeat_timeout_ms: u32,
    pub last_heartbeat_rx_tick: u64,
}

impl ConnectionManager {
    pub fn new(heartbeat_interval_ms: u32, heartbeat_timeout_ms: u32) -> Self {
        Self {
            state: ConnectionState::Disconnected,
            max_reconnect_attempts: 5,
            heartbeat_interval_ms,
            heartbeat_timeout_ms,
            last_heartbeat_rx_tick: 0,
        }
    }

    pub fn on_connected(&mut self, current_tick_ms: u64) {
        self.state = ConnectionState::Connected;
        self.last_heartbeat_rx_tick = current_tick_ms;
    }

    pub fn on_heartbeat_received(&mut self, current_tick_ms: u64) {
        self.last_heartbeat_rx_tick = current_tick_ms;
    }

    pub fn check_health(&mut self, current_tick_ms: u64) -> Result<(), &'static str> {
        if self.state == ConnectionState::Connected {
            let elapsed = current_tick_ms.saturating_sub(self.last_heartbeat_rx_tick);
            if elapsed > self.heartbeat_timeout_ms as u64 {
                self.trigger_reconnect();
                return Err("Heartbeat timeout - triggering reconnection");
            }
        }
        Ok(())
    }

    pub fn trigger_reconnect(&mut self) {
        match self.state {
            ConnectionState::Reconnecting { attempt } if attempt < self.max_reconnect_attempts => {
                self.state = ConnectionState::Reconnecting { attempt: attempt + 1 };
            }
            ConnectionState::Reconnecting { .. } => {
                self.state = ConnectionState::Faulted;
            }
            _ => {
                self.state = ConnectionState::Reconnecting { attempt: 1 };
            }
        }
    }

    pub fn on_disconnect(&mut self) {
        self.state = ConnectionState::Disconnected;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_connection_lifecycle_and_timeout() {
        let mut mgr = ConnectionManager::new(500, 1500);
        assert_eq!(mgr.state, ConnectionState::Disconnected);

        mgr.on_connected(1000);
        assert_eq!(mgr.state, ConnectionState::Connected);

        assert!(mgr.check_health(2000).is_ok()); // 1000ms elapsed < 1500ms

        // 1600ms elapsed > 1500ms -> timeout
        assert!(mgr.check_health(2600).is_err());
        assert_eq!(mgr.state, ConnectionState::Reconnecting { attempt: 1 });
    }
}
