use crate::kbucket::TableRecord;
use crate::message::{Message, MessageBody};
use crate::node::Peer;
use std::io::Result;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::UdpSocket;
use tokio::sync::mpsc;

type Channel<T> = mpsc::Receiver<T>;
pub struct Service {
    pub local_record: Peer,
    pub socket: Arc<UdpSocket>,
    node_rx: Channel<Message>,
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

        // Starts our main message-processing loop
        tokio::spawn(async move {
            service.start().await;
        });

        tx
    }

    pub async fn start(&mut self) {
        loop {
            // Client side:  Node -> Service -> Target
            // ------------------------------
            let internal_msg = self.node_rx.recv().await.unwrap();
            match internal_msg.inner.body {
                MessageBody::Ping(datagram) => {
                    println!("Ping was sent through channel to service.");
                    println!("{:?}", internal_msg.target);

                    self.send_message(internal_msg).await;
                }

                _ => {
                    println!("TODO: Implement other message types for server");
                }
            }

            // Server side: Listens for inbound requests
            // ------------------------------------------
            let mut external_msg = [0_u8; 1024];
            let Ok((size, sender_addr)) = self.socket.recv_from(&mut external_msg).await else { todo!() };
            match &external_msg[0..2] {
                _ => {
                    unimplemented!()
                }
            }
        }
    }

    async fn send_message(&self, msg: Message) -> Result<()> {
        let dest = SocketAddr::new(msg.target.record.ip_address, msg.target.record.udp_port);

        // TODO: Implement outbound requests.

        let message_bytes = msg.inner.to_bytes();
        let len = self.socket.send_to(&message_bytes, dest).await.unwrap();
        println!("message length sent: {:?}", len);

        Ok(())
    }
}

// TODO:  Create tests in here!!!
