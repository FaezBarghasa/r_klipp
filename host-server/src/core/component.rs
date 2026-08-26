use anyhow::Result;
use async_trait::async_trait;
use std::any::Any;

use super::context::Context;

/// Core trait implemented by all modular Moonraker components.
#[async_trait]
pub trait Component: Send + Sync + 'static {
    /// Unique name of the component (e.g. "file_manager", "power", "klippy").
    fn name(&self) -> &str;

    /// Initialize the component with application context and configuration.
    async fn init(&mut self, _ctx: &Context) -> Result<()> {
        Ok(())
    }

    /// Start background workers, timers, or network loops.
    async fn start(&mut self, _ctx: &Context) -> Result<()> {
        Ok(())
    }

    /// Graceful shutdown hook.
    async fn stop(&mut self, _ctx: &Context) -> Result<()> {
        Ok(())
    }

    /// For downcasting from Arc<dyn Component> to concrete types.
    fn as_any(&self) -> &dyn Any;
}
