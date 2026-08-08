use embassy_executor::Spawner;

#[derive(Clone, Copy)]
pub struct TaskSpawner {
    spawner: Spawner,
}

impl TaskSpawner {
    pub fn new(spawner: Spawner) -> Self {
        Self { spawner }
    }

    pub fn spawn<F>(&self, future: F)
    where
        F: core::future::Future + Send + 'static,
        F::Output: Send + 'static,
    {
        self.spawner.spawn(future).unwrap();
    }
}
