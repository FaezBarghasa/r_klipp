use anyhow::{anyhow, Result};
use dashmap::DashMap;
use serde_json::Value;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use tokio::sync::oneshot;

/// High performance JSON-RPC Request/Response Matcher.
#[derive(Clone, Default)]
pub struct JsonRpcDispatcher {
    next_id: Arc<AtomicU64>,
    pending: Arc<DashMap<u64, oneshot::Sender<Value>>>,
}

impl JsonRpcDispatcher {
    pub fn new() -> Self {
        Self {
            next_id: Arc::new(AtomicU64::new(1)),
            pending: Arc::new(DashMap::new()),
        }
    }

    /// Allocate the next monotonic request ID.
    pub fn next_request_id(&self) -> u64 {
        self.next_id.fetch_add(1, Ordering::SeqCst)
    }

    /// Register a pending request ID and return the receiver channel.
    pub fn register_pending(&self, id: u64) -> oneshot::Receiver<Value> {
        let (tx, rx) = oneshot::channel();
        self.pending.insert(id, tx);
        rx
    }

    /// Handle an incoming JSON-RPC response, matching the ID to a pending channel.
    pub fn handle_response(&self, id: u64, payload: Value) -> bool {
        if let Some((_, sender)) = self.pending.remove(&id) {
            let _ = sender.send(payload);
            true
        } else {
            false
        }
    }

    /// Cancel/timeout a pending request ID.
    pub fn cancel_request(&self, id: u64) {
        self.pending.remove(&id);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[tokio::test]
    async fn test_json_rpc_dispatcher_matching() {
        let dispatcher = JsonRpcDispatcher::new();
        let id = dispatcher.next_request_id();
        let rx = dispatcher.register_pending(id);

        let payload = json!({"id": id, "result": "ok"});
        let matched = dispatcher.handle_response(id, payload.clone());
        assert!(matched);

        let received = rx.await.expect("should receive payload");
        assert_eq!(received, payload);
    }
}
