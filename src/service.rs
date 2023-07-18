use crate::helper::Identifier;
use crate::kbucket::KbucketTable;
use crate::message::{construct_msg, Message, MessageBody, MessageInner};
use crate::node::Peer;
use std::collections::HashMap;
use std::io::Result;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use tokio::net::UdpSocket;
use tokio::sync::mpsc;

pub struct Service {
    pub local_record: Peer,
    pub socket: Arc<UdpSocket>,
    node_rx: mpsc::Receiver<Message>,
    pub outbound_requests: HashMap<Identifier, Message>,
    pub table: Arc<Mutex<KbucketTable>>,
}

// TODO: Handle errors properly
impl Service {
    // Main service functionality
    // ---------------------------------------------------------------------------------------------------
    pub async fn spawn(
        local_record: Peer,
        table: Arc<Mutex<KbucketTable>>,
    ) -> Option<mpsc::Sender<Message>> {
        let (service_tx, node_rx) = mpsc::channel(32);

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
        };

        tokio::spawn(async move {
            service.start().await;
        });

        Some(service_tx)
    }

    // Node's main message processing loop
    pub async fn start(&mut self) {
        loop {
            let mut datagram = [0_u8; 1024];
            tokio::select! {
                // Service Requests:
                Some(service_msg) = self.node_rx.recv() => {
                    match service_msg.inner.body {
                        MessageBody::Ping(_, _) => {
                            let _ = self.send_message(service_msg).await;
                        }
                        _ => {
                            println!("TODO: Implement other RPCs");
                        }
                    }
                }
                // External Message Processing:
                Ok((_, socket_addr)) = self.socket.recv_from(&mut datagram) => {
                    let id: [u8; 32] = datagram[3..35].try_into().expect("Invalid slice length");
                    let target = Peer {id, socket_addr};
                    let inbound_req = construct_msg(datagram, target);

                    match &inbound_req.inner.body {
                        MessageBody::Ping(_, None) => {
                            self.table.lock().unwrap().add(target);
                            self.pong(inbound_req.inner.session, target).await;
                        }
                        MessageBody::Pong(_) => {
                            self.sessions_match(id, inbound_req);
                        }
                        // TODO:
                        MessageBody::FindNode(_) => {
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
                session,
                body: (MessageBody::Pong(self.local_record.id)),
            },
        };

        let _ = self.send_message(msg).await;
    }

    // TODO:
    // async fn found_node() {}

    // Helper Functions
    // ---------------------------------------------------------------------------------------------------
    async fn send_message(&mut self, msg: Message) -> Result<()> {
        let dest = SocketAddr::new(msg.target.socket_addr.ip(), msg.target.socket_addr.port());

        let message_bytes = msg.inner.to_bytes();
        let _ = self.socket.send_to(&message_bytes, dest).await.unwrap();

        // TODO: Implement multiple pending messages per target
        self.outbound_requests.insert(msg.target.id, msg);

        Ok(())
    }

    // Verifies the pong message received matches the ping originally sent and sends message to high level ping()
    fn sessions_match(&mut self, id: Identifier, inbound_req: Message) -> bool {
        let local_msg = self.outbound_requests.remove(&id).unwrap(); // Warning: This removes all outbound reqs to an individual node.
        if let MessageBody::Ping(_, tx) = local_msg.inner.body {
            if local_msg.inner.session == inbound_req.inner.session {
                println!("Successful ping. Removing k,v");
                let _ = tx.unwrap().send(true);
                true
            } else {
                println!("Local and remote sessions don't match");
                let _ = tx.unwrap().send(false);
                false
            }
        } else {
            println!("Client responded with incorrect message type");
            false
        }
    }
}
