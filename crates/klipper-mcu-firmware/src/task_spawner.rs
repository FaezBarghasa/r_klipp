use embassy_executor::{Spawner, SpawnToken};

#[derive(Clone, Copy)]
pub struct TaskSpawner {
    spawner: Spawner,
}

impl TaskSpawner {
    pub fn new(spawner: Spawner) -> Self {
        Self { spawner }
    }

    pub fn spawn<S>(&self, token: SpawnToken<S>) {
        self.spawner.spawn(token);
    }
}

