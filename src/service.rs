use crate::helper::Identifier;
use crate::kbucket::TableRecord;
use crate::message::{construct_inner_msg, Message, MessageBody, MessageInner};
use crate::node::Peer;
use rand::Rng;
use std::collections::HashMap;
use std::io::Result;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::UdpSocket;
use tokio::sync::mpsc;

type RxChannel<T> = mpsc::Receiver<T>;
pub struct Service {
    pub local_record: Peer,
    pub socket: Arc<UdpSocket>,
    node_rx: RxChannel<Message>,
    // TODO: Create channel to send mpsc::Sender<bool> back to our Node struct!
    // pub outbound_requests: HashMap<Identifier, (Message, mpsc::Receiver<bool>)>,
}

impl Service {
    // Main service functionality
    // ---------------------------------------------------------------------------------------------------
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

        tokio::spawn(async move {
            service.start().await;
        });

        tx
    }

    // Node's main message processing loop
    pub async fn start(&mut self) {
        loop {
            let mut datagram = [0_u8; 1024];
            tokio::select! {
                // Client side:  Node -> Service -> Target
                // ----------------------------------------
                Some(service_msg) = self.node_rx.recv() => {
                    match service_msg.inner.body {
                        MessageBody::Ping(datagram) => {
                            println!("sending ping");
                            println!("\n");
                            self.send_message(service_msg).await;
                        }
                        _ => {
                            println!("TODO: Implement other RPCs");
                        }
                    }
                }
                // Server side:
                Ok((size, sender_addr)) = self.socket.recv_from(&mut datagram) => {
                    let inbound_req = construct_inner_msg(datagram);
                    println!("Inbound req: {:?}", inbound_req);

                    // TODO: Process Pong and FindNode msgs
                    match &inbound_req.body {
                        MessageBody::Ping(requester_id) => {
                            println!("Ping request received");
                            let session = inbound_req.session;
                            // self.messages.lock().unwrap().push(message);
                            let requester = Peer {
                                id: datagram[0..32].try_into().expect("Invalid slice length"),
                                record: TableRecord {
                                    ip_address: (sender_addr.ip()),
                                    udp_port: (sender_addr.port()),
                                },
                            };
                            self.pong(session, requester).await;
                        }
                        MessageBody::Pong(requester_id) => {
                            println!("Pong request received")
                        }
                        MessageBody::FindNode(requester_id) => {
                            println!("FindNode request received")
                        }
                        _ => {
                            unimplemented!()
                        }
                    }
                }
            }
        }
    }

    // Response Messages
    // ---------------------------------------------------------------------------------------------------
    async fn pong(&self, session: u8, target: Peer) {
        let msg = Message {
            target,
            inner: MessageInner {
                session: (rand::thread_rng().gen_range(0..=255)),
                body: (MessageBody::Pong(self.local_record.id)),
            },
        };

        self.send_message(msg).await;
    }

    async fn found_node() {}

    // TODO: Figure out whether I need a channel to communicate with node struct or not.
    // async fn send_message(&self, msg: Message) ->  mpsc::Receiver<bool>{
    async fn send_message(&self, msg: Message) -> Result<()> {
        let dest = SocketAddr::new(msg.target.record.ip_address, msg.target.record.udp_port);

        // TODO: Implement outbound requests.

        let message_bytes = msg.inner.to_bytes();
        let len = self.socket.send_to(&message_bytes, dest).await.unwrap();

        Ok(())
    }
}
