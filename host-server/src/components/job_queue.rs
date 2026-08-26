use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PrintJobState {
    Queued,
    Loading,
    InProgress,
    Paused,
    Completed,
    Cancelled,
    Error,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrintJob {
    pub job_id: String,
    pub filename: String,
    pub state: PrintJobState,
    pub time_added: u64,
    pub time_started: Option<u64>,
}

#[derive(Clone, Default)]
pub struct JobQueue {
    queue: Arc<RwLock<VecDeque<PrintJob>>>,
    current_job: Arc<RwLock<Option<PrintJob>>>,
}

impl JobQueue {
    pub fn new() -> Self {
        Self {
            queue: Arc::new(RwLock::new(VecDeque::new())),
            current_job: Arc::new(RwLock::new(None)),
        }
    }

    pub async fn enqueue(&self, filename: &str) -> PrintJob {
        let job = PrintJob {
            job_id: uuid::Uuid::new_v4().to_string(),
            filename: filename.to_string(),
            state: PrintJobState::Queued,
            time_added: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            time_started: None,
        };

        let mut q = self.queue.write().await;
        q.push_back(job.clone());
        job
    }

    pub async fn get_queue(&self) -> Vec<PrintJob> {
        self.queue.read().await.iter().cloned().collect()
    }

    pub async fn get_current_job(&self) -> Option<PrintJob> {
        self.current_job.read().await.clone()
    }

    pub async fn start_next_job(&self) -> Option<PrintJob> {
        let mut q = self.queue.write().await;
        if let Some(mut job) = q.pop_front() {
            job.state = PrintJobState::InProgress;
            job.time_started = Some(
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs(),
            );
            let mut cur = self.current_job.write().await;
            *cur = Some(job.clone());
            Some(job)
        } else {
            None
        }
    }

    pub async fn finish_current_job(&self, state: PrintJobState) -> Result<()> {
        let mut cur = self.current_job.write().await;
        if let Some(ref mut job) = *cur {
            job.state = state;
            *cur = None;
            Ok(())
        } else {
            Err(anyhow!("No active job running"))
        }
    }
}
