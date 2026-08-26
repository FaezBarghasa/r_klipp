use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum KlippyState {
    Disconnected,
    Startup,
    Ready,
    Error,
    Shutdown,
}

impl std::fmt::Display for KlippyState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            KlippyState::Disconnected => write!(f, "disconnected"),
            KlippyState::Startup => write!(f, "startup"),
            KlippyState::Ready => write!(f, "ready"),
            KlippyState::Error => write!(f, "error"),
            KlippyState::Shutdown => write!(f, "shutdown"),
        }
    }
}

/// Tracks Klippy connection state and broadcasts state changes.
#[derive(Clone)]
pub struct KlippyStateManager {
    state: Arc<RwLock<KlippyState>>,
    state_message: Arc<RwLock<String>>,
    state_tx: broadcast::Sender<(KlippyState, String)>,
}

impl KlippyStateManager {
    pub fn new() -> Self {
        let (state_tx, _) = broadcast::channel(32);
        Self {
            state: Arc::new(RwLock::new(KlippyState::Startup)),
            state_message: Arc::new(RwLock::new("Printer is starting up".to_string())),
            state_tx,
        }
    }

    pub async fn get_state(&self) -> KlippyState {
        *self.state.read().await
    }

    pub async fn get_state_message(&self) -> String {
        self.state_message.read().await.clone()
    }

    pub async fn set_state(&self, new_state: KlippyState, message: &str) {
        let mut s = self.state.write().await;
        let mut msg = self.state_message.write().await;
        *s = new_state;
        *msg = message.to_string();
        let _ = self.state_tx.send((new_state, message.to_string()));
    }

    pub fn subscribe(&self) -> broadcast::Receiver<(KlippyState, String)> {
        self.state_tx.subscribe()
    }
}
