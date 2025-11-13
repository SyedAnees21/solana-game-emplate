use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};

pub struct Transceiver<A, B = A> {
    receiver: UnboundedReceiver<A>,
    transmitter: UnboundedSender<B>,
}

impl<A, B> Transceiver<A, B> {
    pub fn new(rx: UnboundedReceiver<A>, tx: UnboundedSender<B>) -> Self {
        Self {
            receiver: rx,
            transmitter: tx
        }
    }
}