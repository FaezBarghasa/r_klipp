//! Core component traits, registry, dependency injection, and in-memory data store.

pub mod component;
pub mod context;
pub mod datastore;

pub use component::Component;
pub use context::Context;
pub use datastore::DataStore;
