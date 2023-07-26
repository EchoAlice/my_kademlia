use crate::helper::{Identifier, SocketAddr};
use crate::kbucket::KbucketTable;
use crate::message::{Message, MessageBody, MessageInner};
use crate::service::Service;
use rand::Rng;
use std::{
    collections::HashMap,
    future::Future,
    sync::{Arc, Mutex},
};
use tokio::sync::{mpsc, oneshot};

#[derive(Clone, Copy, Debug, PartialEq, fastrlp_derive::Encodable, fastrlp_derive::Decodable)]
pub struct Peer {
    pub id: Identifier,
    pub socket_addr: SocketAddr,
}

// TODO: Handle errors properly

// The main Kademlia client struct.
// Provides user-level API for performing querie and interacting with the underlying service.
#[derive(Debug)]
pub struct Node {
    pub id: Identifier,
    pub local_record: Peer,
    pub service_tx: Option<mpsc::Sender<Message>>,
    pub table: Arc<Mutex<KbucketTable>>,
    pub outbound_requests: HashMap<Identifier, MessageInner>,
}

impl Node {
    pub async fn new(local_record: Peer) -> Self {
        Self {
            id: local_record.id,
            local_record,
            service_tx: None,
            table: Arc::new(Mutex::new(KbucketTable::new(local_record))),
            outbound_requests: (Default::default()),
        }
    }

    /// "The most important procedure a Kademlia participant must perform is to locate the k closest nodes
    /// to some given node ID.  We call this procedure a **node lookup**".
    ///
    // TODO: node_lookup(self, id) -> peer {}

    // Protocol's Exposed functions:
    // ---------------------------------------------------------------------------------------------------

    pub fn ping(&mut self, id: Identifier) -> impl Future<Output = bool> + '_ {
        async move {
            let peer = {
                let table = &self.table.lock().unwrap();
                let target = table.get(&id);
                if target.is_none() {
                    return false;
                }
                let socket_addr = *target.unwrap();
                Peer { id, socket_addr }
            };

            let (tx, rx) = oneshot::channel();

            let msg = Message {
                target: peer,
                inner: MessageInner {
                    session: (rand::thread_rng().gen_range(0..=255)),
                    body: (MessageBody::Ping(self.id, Some(tx))),
                },
            };

            let _ = self.service_tx.as_ref().unwrap().send(msg).await;
            rx.await.unwrap()
        }
    }

    #[allow(warnings)]
    pub fn find_node(&mut self, id: Identifier) -> impl Future<Output = Option<Vec<Peer>>> + '_ {
        async move {
            let table = &self.table.lock().unwrap();
            let target = table.get(&id);
            if target.is_none() {
                if let Some(target) = table.get_closest_node(&id) {
                    let (tx, rx) = oneshot::channel();

                    let msg = Message {
                        target,
                        inner: MessageInner {
                            session: (rand::thread_rng().gen_range(0..=255)),
                            body: (MessageBody::FindNode(self.id, id, Some(tx))),
                        },
                    };

                    let _ = self.service_tx.as_ref().unwrap().send(msg).await;
                    rx.await.unwrap()
                } else {
                    println!("No peer was returned");
                    return None;
                }
            } else {
                println!("Node was already in table");
                return None;
            }
        }
    }

    // TODO: Later
    // pub fn find_value() {}

    // TODO: Later
    // pub fn store(&mut self, value: Vec<u8>) {}

    // ---------------------------------------------------------------------------------------------------

    pub async fn start(&mut self) -> Result<(), &'static str> {
        if let Some(service_tx) = Service::spawn(self.local_record, self.table.clone()).await {
            self.service_tx = Some(service_tx);
            Ok(())
        } else {
            Err("Service wasn't created")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::node;
    use std::net;
    use std::net::IpAddr;
    use tokio::time::Duration;

    // TODO: Move make_node and make_nodes to helper
    async fn make_nodes(n: u8) -> (Node, Vec<Node>) {
        let local_node = make_node(0).await;
        let mut remote_nodes = Vec::new();

        for i in 1..n {
            remote_nodes.push(make_node(i).await);
        }

        (local_node, remote_nodes)
    }

    async fn make_node(index: u8) -> Node {
        let ip = String::from("127.0.0.1").parse::<IpAddr>().unwrap();
        let port_start = 9000_u16;

        let mut id = [0_u8; 32];
        id[31] += index;
        let port = port_start + index as u16;

        let socket_addr = net::SocketAddr::new(ip, port);

        let peer = Peer {
            id,
            socket_addr: node::SocketAddr { addr: socket_addr },
        };

        Node::new(peer).await
    }

    // Run tests independently.  Tests fail when they're run together bc of address reuse.
    // TIP: If you don't give the thing a port, a free port is given automatically
    #[tokio::test]
    async fn add_redundant_node() {
        let (local_node, remote_nodes) = make_nodes(2).await;
        let local_table = &mut local_node.table.lock().unwrap();
        let remote_table = &remote_nodes[0].table.lock().unwrap();

        let result = local_table.add(remote_table.peer);
        let result2 = local_table.add(remote_table.peer);
        assert!(result);
        assert!(!result2);
    }

    #[tokio::test]
    async fn ping() {
        let mut local = make_node(0).await;
        let mut remote = make_node(1).await;
        local.table.lock().unwrap().add(remote.local_record);

        let _ = local.start().await;
        let _ = remote.start().await;
        tokio::time::sleep(Duration::from_secs(1)).await;
        let ping = local.ping(remote.id);
        assert!(ping.await);
        tokio::time::sleep(Duration::from_secs(1)).await;

        let dummy = make_node(2).await;
        let ping = local.ping(dummy.id);
        assert!(!ping.await);
    }

    // TODO:
    #[allow(warnings)]
    #[tokio::test]
    async fn find_node() {
        let mut local = make_node(0).await;
        let mut remote = make_node(1).await;
        local.table.lock().unwrap().add(remote.local_record);

        let _ = local.start().await;
        let _ = remote.start().await;

        // Populate remote's table
        let mut remote_table = remote.table.lock().unwrap();
        let remote_table = {
            for i in 2..10 {
                let node = make_node(i).await;
                remote_table.add(node.local_record);
            }
            remote_table
        };

        tokio::time::sleep(Duration::from_secs(1)).await;
        let node_to_find = make_node(7).await.local_record.id;
        let result = local.find_node(node_to_find).await;
        println!("Result");
        assert!(true);
        // Verify response from node
    }
}
