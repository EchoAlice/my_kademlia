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
use tokio::sync::{mpsc, watch};

pub struct Service {
    pub local_record: Peer,
    pub socket: Arc<UdpSocket>,
    node_tx: watch::Sender<bool>, // For communicating valid/invalid pong response
    node_rx: mpsc::Receiver<Message>,
    pub outbound_requests: HashMap<Identifier, Message>,
    pub messages: Vec<Message>, // Note: Here for testing purposes
}

impl Service {
    // Main service functionality
    // ---------------------------------------------------------------------------------------------------
    pub async fn spawn(local_record: Peer) -> (mpsc::Sender<Message>, watch::Receiver<bool>) {
        let (service_tx, node_rx) = mpsc::channel(32);
        let (node_tx, service_rx) = watch::channel(false);

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
            node_tx,
            node_rx,
            outbound_requests: Default::default(),
            messages: Default::default(),
        };

        tokio::spawn(async move {
            service.start().await;
        });

        (service_tx, service_rx)
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
                            // self.messages.push(inbound_req.clone());

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
    async fn pong(&mut self, session: u8, target: Peer) {
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
    // ---------------------------------------------------------------------------------------------------

    // TODO: Figure out whether I need a channel to communicate with node struct or not.
    // async fn send_message(&self, msg: Message) ->  mpsc::Receiver<bool>{
    async fn send_message(&mut self, msg: Message) -> Result<()> {
        let dest = SocketAddr::new(msg.target.record.ip_address, msg.target.record.udp_port);

        // TODO: Implement multiple pending messages per target
        self.outbound_requests.insert(msg.target.id, msg.clone());
        self.messages.push(msg.clone());

        let message_bytes = msg.inner.to_bytes();
        let len = self.socket.send_to(&message_bytes, dest).await.unwrap();

        Ok(())
    }
}
