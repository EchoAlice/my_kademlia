use crate::helper::{Identifier, PING_MESSAGE_SIZE};
use crate::kbucket::{Bucket, KbucketTable, TableRecord};
use crate::message::{Message, MessageBody, MessageInner};
use crate::node;
use crate::service::Service;

use core::panic;
use rand::Rng;
use std::collections::HashMap;
use std::net::{IpAddr, SocketAddr, SocketAddrV4};
use std::ops::Deref;
use std::sync::{Arc, Mutex};
use tokio::io;
use tokio::net::UdpSocket;
use tokio::sync::mpsc;
use tokio::sync::mpsc::error::SendError;
use tokio::sync::{oneshot, oneshot::error::RecvError};
use tokio::time::Duration;

type ServiceChannel<T> = Option<mpsc::Sender<T>>;
type ReqChannel<T> = Arc<mpsc::Sender<T>>;

const NODES_TO_QUERY: usize = 1; // "a"

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Peer {
    pub id: Identifier,
    pub record: TableRecord,
}

#[derive(Clone, Debug)]
pub struct State {
    pub table: KbucketTable,
    // pub outbound_requests: HashMap<Identifier, (Message, ReqChannel<bool>)>,
    pub outbound_requests: HashMap<Identifier, (MessageInner, ReqChannel<bool>)>,
}

// The main Kademlia client struct.
// Provides user-level API for performing querie and interacting with the underlying service.
#[derive(Clone, Debug)]
pub struct Node {
    pub id: Identifier,
    pub local_record: Peer,
    pub service_channel: ServiceChannel<Message>,
    // pub socket: Arc<UdpSocket>,
    pub messages: Arc<Mutex<Vec<Message>>>, // Note: Here for testing purposes
    pub state: Arc<Mutex<State>>,
}

impl Node {
    pub async fn new(local_record: Peer) -> Self {
        Self {
            id: local_record.id,
            local_record,
            service_channel: None,
            // socket: Arc::new(
            //     UdpSocket::bind(SocketAddr::new(
            //         local_record.record.ip_address,
            //         local_record.record.udp_port,
            //     ))
            //     .await
            //     .unwrap(),
            // ),
            messages: Default::default(),
            state: Arc::new(Mutex::new(State {
                table: (KbucketTable::new(local_record)),
                outbound_requests: (Default::default()),
            })),
        }
    }

    // TODO: node_lookup(self, id) -> peer {}

    // Protocol's Exposed functions:
    // ---------------------------------------------------------------------------------------------------
    pub async fn ping(&mut self, id: Identifier) /*-> bool*/
    {
        let peer = {
            let table = &self.state.lock().unwrap().table;
            let target = table.get(&id);
            if target.is_none() {
                return;
            }
            let record = *target.unwrap();
            Peer { id, record }
        };

        let msg = Message {
            target: peer,
            inner: MessageInner {
                session: (rand::thread_rng().gen_range(0..=255)),
                body: (MessageBody::Ping(self.id)),
            },
        };

        // TODO: Implement send_message functionality before worrying about pong verification logic!
        let result = &self.service_channel.as_ref().unwrap().send(msg).await;

        // TODO: Implement verification channel
        // let rx = &mut self.send_message(msg, &peer).await;
        // rx.recv().await.unwrap()
    }

    /// "The most important procedure a Kademlia participant must perform is to locate the k closest nodes
    /// to some given node ID.  We call this procedure a **node lookup**".
    ///
    /// TODO:  1. Set up networking communication for find_node() **with non-empty bucket**.
    ///        2. Create complete routing table logic (return K closest nodes instead of indexed bucket)
    pub async fn find_node(&mut self, id: &Identifier, target: &Peer) -> u8 {
        let msg = MessageInner {
            session: rand::thread_rng().gen_range(0..=255),
            body: MessageBody::FindNode([self.id, *id]),
        };
        // self.send_message(msg, target).await;

        unimplemented!()
    }

    // TODO:
    // pub fn find_value() {}

    // TODO:
    // pub fn store(&mut self, key: Identifier, value: Vec<u8>) {}

    // ---------------------------------------------------------------------------------------------------

    pub async fn start(&mut self) -> Result<(), &'static str> {
        let service_channel = Service::spawn(self.local_record.clone()).await;
        self.service_channel = Some(service_channel);

        if self.service_channel.is_none() {
            return Err("Service channel wasn't created");
        }

        Ok(())
    }

    async fn pong(&self, session: u8, target: &Peer) {
        let msg = MessageInner {
            session,
            body: MessageBody::Pong(self.id),
        };
        // self.send_message(msg, target).await;
    }

    /*
        // TODO: Delete this once this functionality is in place within service.
        async fn send_message(&self, msg: Message) -> mpsc::Receiver<bool> {
            let dest = SocketAddr::new(msg.target.record.ip_address, msg.target.record.udp_port);

            let (tx, rx) = mpsc::channel(32);
            let mutex_tx = Arc::new(tx);

            // TODO: Implement multiple pending messages per target
            self.state
                .lock()
                .unwrap()
                .outbound_requests
                .insert(msg.target.id, (msg.inner.clone(), mutex_tx));

            // self.messages.lock().unwrap().push(msg.clone());
            let message_bytes = msg.inner.to_bytes();

            self.socket.send_to(&message_bytes, dest).await.unwrap();
            rx
        }
    */
    /*
       pub async fn start_server(&mut self, mut buffer: [u8; 1024]) {
           loop {
               let Ok((size, sender_addr)) = self.socket.recv_from(&mut buffer).await else { todo!() };
               let requester_id: [u8; 32] = buffer[3..35].try_into().expect("Invalid slice length");

               match &buffer[0..2] {
                   b"01" => {
                       let message = MessageInner {
                           session: buffer[2],
                           body: MessageBody::Ping(requester_id),
                       };
                       self.process(message, &sender_addr).await;
                   }
                   b"02" => {
                       let message = MessageInner {
                           session: buffer[2],
                           body: MessageBody::Pong(requester_id),
                       };
                       self.process(message, &sender_addr).await;
                   }
                   b"03" => {
                       let message = MessageInner {
                           session: buffer[2],
                           body: MessageBody::FindNode([
                               requester_id,
                               buffer[35..67].try_into().expect("Invalid slice length"),
                           ]),
                       };
                       self.process(message, &sender_addr).await;
                   }
                   _ => {
                       panic!("Message wasn't legitimate");
                   }
               }
           }
       }
    */
    async fn process(&mut self, message: MessageInner, sender_addr: &SocketAddr) {
        match message.body {
            MessageBody::Ping(datagram) => {
                let session = message.session;
                // self.messages.lock().unwrap().push(message);
                let requester = Peer {
                    id: datagram[0..32].try_into().expect("Invalid slice length"),
                    record: TableRecord {
                        ip_address: (sender_addr.ip()),
                        udp_port: (sender_addr.port()),
                    },
                };
                println!("send pong");
                self.pong(session, &requester).await;
            }
            MessageBody::Pong(datagram) => {
                let node_id = &datagram[0..32];

                let (local_msg, tx) = {
                    let state = self.state.lock().unwrap();
                    let target = state.outbound_requests.get(node_id);

                    if target.is_none() {
                        println!("No outbound requests for node");
                    }

                    let (msg, tx) = target.unwrap();
                    (msg.clone(), tx.clone())
                };

                // Verifyies the pong message recieved matches the ping originally sent.  Sends message to high level ping()
                if local_msg.session == message.session {
                    println!("Successful ping. Removing k,v");
                    tx.send(true).await;

                    let state = &mut self.state.lock().unwrap();
                    // self.messages.lock().unwrap().push(message);

                    state.outbound_requests.remove(node_id); // Warning: This removes all outbound reqs to an individual node.
                } else {
                    tx.send(false).await;
                    println!("Local and remote sessions don't match");
                }
            }
            MessageBody::FindNode(datagram) => {
                println!("FindNode datagram: {:?}", datagram)
            }
            _ => println!("Message was not ping, pong, or FindNode"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{helper::PING_MESSAGE_SIZE, node};
    use std::sync::LockResult;

    async fn make_nodes(n: u8) -> (Node, Vec<Node>) {
        let ip_address = String::from("127.0.0.1").parse::<IpAddr>().unwrap();
        let port_start = 9000_u16;

        let local_node = make_node(0).await;
        let mut remote_nodes = Vec::new();

        for i in 1..n {
            remote_nodes.push(make_node(i).await);
        }

        (local_node, remote_nodes)
    }

    async fn make_node(index: u8) -> Node {
        let ip_address = String::from("127.0.0.1").parse::<IpAddr>().unwrap();
        let port_start = 9000_u16;

        let mut id = [0_u8; 32];
        id[31] += index;
        let udp_port = port_start + index as u16;

        let record = TableRecord {
            ip_address,
            udp_port,
        };
        let peer = Peer { id, record };

        Node::new(peer).await
    }

    // Run tests independently.  Tests fail when they're run together bc of address reuse.
    // TIP: If you don't give the thing a port, a free port is given automatically
    #[tokio::test]
    async fn add_redundant_node() {
        let (local_node, remote_nodes) = make_nodes(2).await;
        let mut local_table = &mut local_node.state.lock().unwrap().table;
        let remote_table = &remote_nodes[0].state.lock().unwrap().table;

        let result = local_table.add(remote_table.peer);
        let result2 = local_table.add(remote_table.peer);
        assert!(result);
        assert!(!result2);
    }

    #[tokio::test]
    async fn start_server() {
        let mut local = make_node(0).await;
        let result = local.start().await;
        tokio::time::sleep(Duration::from_secs(1)).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn ping() {
        let mut local = make_node(0).await;
        let mut remote = make_node(1).await;
        local.state.lock().unwrap().table.add(remote.local_record);

        local.start().await;
        tokio::time::sleep(Duration::from_secs(1)).await;
        local.ping(remote.id).await;
        tokio::time::sleep(Duration::from_secs(1)).await;

        let mut dummy = make_node(2).await;
        let result = local.ping(dummy.id).await;
        // assert!(!result)
    }

    #[tokio::test]
    async fn find_node() {
        // TODO:  try to make_node() not await
        let mut local = make_node(0).await;
        let mut local_copy = local.clone();
        let mut remote = make_node(1).await;
        let mut remote_copy = remote.clone();

        // add remote peer to local node
        local.state.lock().unwrap().table.add(remote.local_record);

        let node_to_store = make_node(2).await;
        remote
            .state
            .lock()
            .unwrap()
            .table
            .add(node_to_store.local_record);

        // start local node
        // start remote

        // assert!(local.find_node(id).await, &vec![id]);
    }
}
