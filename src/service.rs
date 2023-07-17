use crate::helper::Identifier;
use crate::kbucket::{KbucketTable, TableRecord};
use crate::message::{construct_msg, Message, MessageBody, MessageInner};
use crate::node::Peer;
use rand::Rng;
use std::collections::HashMap;
use std::io::Result;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use tokio::net::UdpSocket;
use tokio::sync::{mpsc, watch};

pub struct Service {
    pub local_record: Peer,
    pub socket: Arc<UdpSocket>,
    node_tx: watch::Sender<bool>, // For communicating valid/invalid pong response
    node_rx: mpsc::Receiver<Message>,
    pub outbound_requests: HashMap<Identifier, Message>,
    pub table: Arc<Mutex<KbucketTable>>,
    pub messages: Vec<Message>, // Note: Here for testing purposes
}

impl Service {
    // Main service functionality
    // ---------------------------------------------------------------------------------------------------
    pub async fn spawn(
        local_record: Peer,
        table: Arc<Mutex<KbucketTable>>,
    ) -> (mpsc::Sender<Message>, watch::Receiver<bool>) {
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
            table,
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
                            println!("Sending ping");
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
                    // Gather target peer info from message
                    let id: [u8; 32] = datagram[3..35].try_into().expect("Invalid slice length");

                    // TODO: Convert Peer to {id, socket_addr}
                    let target = Peer {id, record: TableRecord { ip_address: (sender_addr.ip()), udp_port: (sender_addr.port())}};

                    // TODO: Add target (pinging node) to routing table.

                    let inbound_req = construct_msg(datagram, target);
                    println!("Inbound req: {:?}", inbound_req);

                    match &inbound_req.inner.body {
                        MessageBody::Ping(requester_id) => {
                            let session = inbound_req.inner.session;
                            self.messages.push(inbound_req.clone());

                            self.pong(session, target).await;
                        }
                        // TODO:
                        MessageBody::Pong(requester_id) => {
                            println!("Pong request received");

                        }
                        // TODO:
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
