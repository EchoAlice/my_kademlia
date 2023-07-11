use tokio::sync::mpsc;

use crate::message::{Message, MessageBody};

type Channel<T> = mpsc::Receiver<T>;
pub struct Service {
    node_rx: Channel<Message>, // TODO: Channel<Message>
                               // pub socket: Arc<UdpSocket>,
                               // pub outbound_requests: HashMap<Identifier, (Message, mpsc::recieve<bool>)>,
}

impl Service {
    pub fn spawn() -> mpsc::Sender<Message> {
        let (tx, node_rx) = mpsc::channel(32);

        // TODO: Bind UDPSocket here.

        let mut service = Service { node_rx };

        println!("Spawning service");

        // Create loop that listens for a bool
        tokio::spawn(async move {
            service.start().await;
        });

        tx
    }

    pub async fn start(&mut self) {
        loop {
            let msg = self.node_rx.recv().await;
            if msg.is_none() {
                break;
            };
            match msg.unwrap().body {
                MessageBody::Ping(datagram) => {
                    println!("Ping was sent through channel to service");
                }
                _ => {
                    println!("TODO: Implement other message types for server");
                }
            }
        }
    }
}
