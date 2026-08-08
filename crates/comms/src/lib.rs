#![no_std]

use heapless::spsc::{Queue, Consumer, Producer};
use portable_atomic::{AtomicBool, Ordering};

pub enum Message {
    MotionCommand,
    Telemetry,
    EmergencyStop,
    Heartbeat,
}

pub struct MessageBus<const N: usize> {
    queue: Queue<Message, N>,
}

impl<const N: usize> MessageBus<N> {
    pub fn new() -> Self {
        Self {
            queue: Queue::new(),
        }
    }

    pub fn split(&mut self) -> (MessageProducer<N>, MessageConsumer<N>) {
        let (producer, consumer) = self.queue.split();
        (
            MessageProducer { producer },
            MessageConsumer { consumer, subscribed: AtomicBool::new(true) },
        )
    }
}

pub struct MessageProducer<'a, const N: usize> {
    producer: Producer<'a, Message, N>,
}

impl<'a, const N: usize> MessageProducer<'a, N> {
    pub fn publish(&mut self, message: Message) -> Result<(), Message> {
        self.producer.enqueue(message)
    }
}

pub struct MessageConsumer<'a, const N: usize> {
    consumer: Consumer<'a, Message, N>,
    subscribed: AtomicBool,
}

impl<'a, const N: usize> MessageConsumer<'a, N> {
    pub fn subscribe(&self) {
        self.subscribed.store(true, Ordering::Relaxed);
    }

    pub fn unsubscribe(&self) {
        self.subscribed.store(false, Ordering::Relaxed);
    }

    pub fn poll(&mut self) -> Option<Message> {
        if self.subscribed.load(Ordering::Relaxed) {
            self.consumer.dequeue()
        } else {
            None
        }
    }
}
