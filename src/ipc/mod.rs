use heapless::spsc::{Queue, Producer, Consumer};
use portable_atomic::{AtomicBool, Ordering};

// A command to the step generation ISR.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct StepCommand {
    pub ticks: u32,
    pub direction: bool,
    pub axis: u8,
}

// Telemetry data sent from the ISR to the async world.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TelemetryPacket {
    pub timestamp: u64,
    pub encoder_position: i32,
    pub current_sense: u16,
}

const QUEUE_SIZE: usize = 1024;

/// A lock-free, single-producer, single-consumer queue for sending commands
/// from the async planner to the hard real-time step generation ISR.
pub struct StepCommandQueue {
    queue: Queue<StepCommand, QUEUE_SIZE>,
}

impl StepCommandQueue {
    pub const fn new() -> Self {
        Self {
            queue: Queue::new(),
        }
    }

    pub fn split<'a>(&'a mut self) -> (StepCommandProducer<'a>, StepCommandConsumer<'a>) {
        let (producer, consumer) = self.queue.split();
        (
            StepCommandProducer { producer },
            StepCommandConsumer { consumer },
        )
    }
}

pub struct StepCommandProducer<'a> {
    producer: Producer<'a, StepCommand, QUEUE_SIZE>,
}

impl<'a> StepCommandProducer<'a> {
    pub fn enqueue(&mut self, command: StepCommand) -> Result<(), StepCommand> {
        self.producer.enqueue(command)
    }
}

pub struct StepCommandConsumer<'a> {
    consumer: Consumer<'a, StepCommand, QUEUE_SIZE>,
}

impl<'a> StepCommandConsumer<'a> {
    pub fn dequeue(&mut self) -> Option<StepCommand> {
        self.consumer.dequeue()
    }
}


/// A lock-free, single-producer, single-consumer queue for sending telemetry
/// from the hard real-time ISR to the async telemetry task.
pub struct TelemetryQueue {
    queue: Queue<TelemetryPacket, QUEUE_SIZE>,
}

impl TelemetryQueue {
    pub const fn new() -> Self {
        Self {
            queue: Queue::new(),
        }
    }

    pub fn split<'a>(&'a mut self) -> (TelemetryProducer<'a>, TelemetryConsumer<'a>) {
        let (producer, consumer) = self.queue.split();
        (
            TelemetryProducer { producer },
            TelemetryConsumer { consumer },
        )
    }
}

pub struct TelemetryProducer<'a> {
    producer: Producer<'a, TelemetryPacket, QUEUE_SIZE>,
}

impl<'a> TelemetryProducer<'a> {
    pub fn enqueue(&mut self, packet: TelemetryPacket) -> Result<(), TelemetryPacket> {
        self.producer.enqueue(packet)
    }
}

pub struct TelemetryConsumer<'a> {
    consumer: Consumer<'a, TelemetryPacket, QUEUE_SIZE>,
}

impl<'a> TelemetryConsumer<'a> {
    pub fn dequeue(&mut self) -> Option<TelemetryPacket> {
        self.consumer.dequeue()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;

    #[test]
    fn test_step_command_queue_concurrent() {
        let mut queue = StepCommandQueue::new();
        let (mut producer, mut consumer) = queue.split();

        let producer_thread = thread::spawn(move || {
            for i in 0..1000 {
                let cmd = StepCommand { ticks: i, direction: i % 2 == 0, axis: 0 };
                while producer.enqueue(cmd).is_err() {}
            }
        });

        let consumer_thread = thread::spawn(move || {
            for i in 0..1000 {
                loop {
                    if let Some(cmd) = consumer.dequeue() {
                        assert_eq!(cmd.ticks, i);
                        break;
                    }
                }
            }
        });

        producer_thread.join().unwrap();
        consumer_thread.join().unwrap();
    }

    #[bench]
    fn bench_ipc_queue_throughput(b: &mut test::Bencher) {
        let mut queue = StepCommandQueue::new();
        let (mut producer, mut consumer) = queue.split();
        let cmd = StepCommand { ticks: 1, direction: true, axis: 0 };

        b.iter(|| {
            for _ in 0..1000 {
                producer.enqueue(cmd).ok();
                consumer.dequeue();
            }
        });
    }
}
