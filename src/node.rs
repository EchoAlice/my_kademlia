use crate::helper::Identifier;
use crate::kbucket::{Bucket, KbucketTable, TableRecord};
use crate::message::{Message, MessageBody, MessageInner};
use crate::service::Service;

use core::panic;
use rand::Rng;
use std::collections::HashMap;
use std::net::{IpAddr, SocketAddr};
use std::sync::{Arc, Mutex};
use tokio::net::UdpSocket;
use tokio::sync::{mpsc, watch};
use tokio::time::Duration;

type ServiceTx<T> = Option<mpsc::Sender<T>>;
type ServiceRx<T> = Option<watch::Receiver<T>>;

const NODES_TO_QUERY: usize = 1; // "a"

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Peer {
    pub id: Identifier,
    pub record: TableRecord,
}

// The main Kademlia client struct.
// Provides user-level API for performing querie and interacting with the underlying service.
#[derive(Clone, Debug)]
pub struct Node {
    pub id: Identifier,
    pub local_record: Peer,
    pub service_tx: ServiceTx<Message>,
    pub service_rx: ServiceRx<bool>,
    pub table: Arc<Mutex<KbucketTable>>,
    pub outbound_requests: HashMap<Identifier, MessageInner>,
}

impl Node {
    pub async fn new(local_record: Peer) -> Self {
        Self {
            id: local_record.id,
            local_record,
            service_tx: None,
            service_rx: None,
            table: Arc::new(Mutex::new(KbucketTable::new(local_record))),
            outbound_requests: (Default::default()),
        }
    }

    // TODO: node_lookup(self, id) -> peer {}

    // Protocol's Exposed functions:
    // ---------------------------------------------------------------------------------------------------
    pub async fn ping(&mut self, id: Identifier) /*-> bool*/
    {
        let peer = {
            let table = &self.table.lock().unwrap();
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

        // TODO: Implement pong verification logic
        let result = &self.service_tx.as_ref().unwrap().send(msg).await;

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
        let (service_tx, service_rx) = Service::spawn(self.local_record.clone()).await;
        self.service_tx = Some(service_tx);

        if self.service_tx.is_none() {
            return Err("Service channel wasn't created");
        }

        Ok(())
    }

    /*
        // TODO: Delete this once this functionality is in place within service.
        async fn send_message(&self, msg: Message) -> mpsc::Receiver<bool> {
            let dest = SocketAddr::new(msg.target.record.ip_address, msg.target.record.udp_port);

            let (tx, rx) = mpsc::channel(32);
            let mutex_tx = Arc::new(tx);


            self.socket.send_to(&message_bytes, dest).await.unwrap();
            rx
        }
    */

    // TODO: Transfer remaining functionality to service
    async fn process(&mut self, message: MessageInner, sender_addr: &SocketAddr) {
        match message.body {
            MessageBody::Pong(datagram) => {
                let node_id = &datagram[0..32];

                let local_msg = {
                    // let state = self.state.lock().unwrap();
                    let target = self.outbound_requests.get(node_id);

                    if target.is_none() {
                        println!("No outbound requests for node");
                    }

                    let msg = target.unwrap();
                    msg.clone()
                };

                // Verifyies the pong message recieved matches the ping originally sent.  Sends message to high level ping()
                if local_msg.session == message.session {
                    println!("Successful ping. Removing k,v");
                    // tx.send(true).await;

                    // self.messages.lock().unwrap().push(message);

                    self.outbound_requests.remove(node_id); // Warning: This removes all outbound reqs to an individual node.
                } else {
                    // tx.send(false).await;
                    println!("Local and remote sessions don't match");
                }
            }
            MessageBody::FindNode(datagram) => {
                println!("FindNode datagram: {:?}", datagram)
            }
            _ => println!("Message was not pong or FindNode"),
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
        let mut local_table = &mut local_node.table.lock().unwrap();
        let remote_table = &remote_nodes[0].table.lock().unwrap();

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
        local.table.lock().unwrap().add(remote.local_record);

        local.start().await;
        remote.start().await;
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
        local.table.lock().unwrap().add(remote.local_record);

        let node_to_store = make_node(2).await;
        remote.table.lock().unwrap().add(node_to_store.local_record);

        // start local node
        // start remote

        // assert!(local.find_node(id).await, &vec![id]);
    }
}
