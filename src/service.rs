use crate::helper::Identifier;
use crate::kbucket::KbucketTable;
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
                    local_record.socket_addr.ip(),
                    local_record.socket_addr.port(),
                ))
                .await
                .unwrap(),
            ),
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
                // Node to it's own server:
                Some(service_msg) = self.node_rx.recv() => {
                    match service_msg.inner.body {
                        MessageBody::Ping(_, _) => {
                            println!("Sending ping");
                            println!("\n");
                            self.send_message(service_msg).await;
                        }
                        _ => {
                            println!("TODO: Implement other RPCs");
                        }
                    }
                }
                // Server processing other peers' messages:
                Ok((size, socket_addr)) = self.socket.recv_from(&mut datagram) => {
                    let id: [u8; 32] = datagram[3..35].try_into().expect("Invalid slice length");
                    let target = Peer {id, socket_addr};
                    let inbound_req = construct_msg(datagram, target);
                    let session = inbound_req.inner.session;

                    match &inbound_req.inner.body {
                        MessageBody::Ping(requester_id, None) => {
                            self.table.lock().unwrap().add(target);
                            // self.messages.push(inbound_req.clone());
                            self.pong(session, target).await;
                        }
                        MessageBody::Pong(requester_id) => {
                            // self.messages.push(inbound_req.clone());
                            let local_msg = self.outbound_requests.get(&id);
                            if local_msg.is_none() {
                                println!("No outbound requests for node");
                                //drop message
                            }
                            let local_msg = local_msg.unwrap();
                            let local_msg = self.outbound_requests.remove(&id).unwrap(); // Warning: This removes all outbound reqs to an individual node.
                            if let MessageBody::Ping(_, tx) = local_msg.inner.body {
                                // Verifies the pong message recieved matches the ping originally sent.  Sends message to high level ping()
                                println!("{:?}", local_msg.inner.session);
                                println!("{:?}", inbound_req.inner.session);
                                if local_msg.inner.session == inbound_req.inner.session {
                                    println!("Successful ping. Removing k,v");
                                    tx.unwrap().send(true);
                                    // TODO: Handle errors properly
                                } else {
                                    println!("Local and remote sessions don't match");
                                    tx.unwrap().send(false);
                                }
                            } else {
                                println!("Client responded with incorrect message type")
                            }
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

    // Helper
    // ---------------------------------------------------------------------------------------------------
    async fn send_message(&mut self, msg: Message) -> Result<()> {
        let dest = SocketAddr::new(msg.target.socket_addr.ip(), msg.target.socket_addr.port());

        let message_bytes = msg.inner.to_bytes();
        let len = self.socket.send_to(&message_bytes, dest).await.unwrap();

        // TODO: Implement multiple pending messages per target
        self.outbound_requests.insert(msg.target.id, msg);
        // self.messages.push(msg.clone());

        Ok(())
    }
}
