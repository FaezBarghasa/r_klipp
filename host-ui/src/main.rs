use anyhow::Result;
use tokio::sync::mpsc;

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();
    let (tx, _rx) = mpsc::channel(100);
    host_ui::run_ui(tx).await
}
