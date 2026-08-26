//! Klipper Unix Domain Socket (UDS) communication, ETX (\x03) codec, and JSON-RPC dispatcher.

pub mod codec;
pub mod dispatcher;
pub mod state;

pub use codec::EtxCodec;
pub use dispatcher::JsonRpcDispatcher;
pub use state::{KlippyState, KlippyStateManager};
