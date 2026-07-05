// Placeholder for SerialBridge
pub struct SerialBridge;

impl SerialBridge {
    pub async fn new() -> anyhow::Result<Self> {
        Ok(SerialBridge)
    }

    pub async fn run(&self) -> anyhow::Result<()> {
        // Simulate sending telemetry
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
        Ok(())
    }
}