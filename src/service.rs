use crate::kbucket::TableRecord;
use crate::message::{Message, MessageBody};
use crate::node::Peer;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::UdpSocket;
use tokio::sync::mpsc;

type Channel<T> = mpsc::Receiver<T>;
pub struct Service {
    pub local_record: Peer,
    pub socket: Arc<UdpSocket>,
    node_rx: Channel<Message>, // TODO: Channel<Message>
                               // pub outbound_requests: HashMap<Identifier, (Message, mpsc::recieve<bool>)>,
}

impl Service {
    pub async fn spawn(local_record: Peer) -> mpsc::Sender<Message> {
        let (tx, node_rx) = mpsc::channel(32);

        let mut service = Service {
            local_record,
            socket: Arc::new(
                UdpSocket::bind(SocketAddr::new(
                    local_record.record.ip_address,
                    local_record.record.udp_port,
                ))
                .await
                .unwrap(),
            ),
            node_rx,
        };

        println!("Spawning service");

        // Create loop that listens for a bool
        tokio::spawn(async move {
            service.start().await;
        });

        tx
    }

    /// Main loop that continuously processes messages.
    ///
    ///       Node -> service   --->   target
    pub async fn start(&mut self) {
        loop {
            let msg = self.node_rx.recv().await.unwrap();

            // TODO: Get target address from
            match msg.body {
                MessageBody::Ping(datagram) => {
                    println!("Ping was sent through channel to service.");
                    // TODO: Send ping message to target peer
                }

                _ => {
                    println!("TODO: Implement other message types for server");
                }
            }
        }
    }
}
