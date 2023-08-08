use crate::helper::Identifier;
use crate::kbucket::KbucketTable;
use crate::message::{Message, MessageBody};
use crate::service::Service;
use crate::socket::{self, SocketAddr};
use alloy_rlp::{RlpDecodable, RlpEncodable};
use rand::Rng;
use std::{
    collections::HashMap,
    future::Future,
    net,
    sync::{Arc, Mutex},
};
use tokio::sync::{mpsc, oneshot};

//  A == Parallel queries for node_lookup()
const A: usize = 3;
//  K == Max bucket size
//  Typically 20.  Only 5 for testing
pub const K: usize = 7;
pub const MAX_BUCKETS: usize = 256;

#[derive(Clone, Copy, Debug, PartialEq, RlpEncodable, RlpDecodable)]
pub struct Peer {
    pub id: Identifier,
    pub socket_addr: socket::SocketAddr,
}

// The main Kademlia client struct.
// Provides user-level API for performing querie and interacting with the underlying service.
#[derive(Debug)]
pub struct Node {
    pub id: Identifier,
    pub socket: SocketAddr,
    pub service_tx: Option<mpsc::Sender<Message>>,
    pub table: Arc<Mutex<KbucketTable>>,
    pub outbound_requests: HashMap<Identifier, Message>,
}

impl Node {
    pub fn new(id: Identifier, socket: net::SocketAddr) -> Self {
        Self {
            id,
            socket: SocketAddr { addr: socket },
            service_tx: None,
            table: Arc::new(Mutex::new(KbucketTable::new(id))),
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
                target.unwrap()
            };

            let (tx, rx) = oneshot::channel();

            let msg = Message {
                target: peer,
                session: (rand::thread_rng().gen_range(0..=255)),
                body: (MessageBody::Ping(self.id, Some(tx))),
            };

            let _ = self.service_tx.as_ref().unwrap().send(msg).await;
            rx.await.unwrap()
        }
    }

    // Should i check routing table for node_to_find before requesting?
    pub fn find_node(&mut self, id: Identifier) -> impl Future<Output = Option<Vec<Peer>>> + '_ {
        async move {
            let target = {
                let table = &self.table.lock().unwrap();
                let target = table.get(&id);

                if target.is_some() {
                    println!("Node is already in table!");
                    return None;
                }
                if let Some(target) = table.get_closest_nodes(&id, K) {
                    target[0]
                } else {
                    println!("No nodes in routing table");
                    return None;
                }
            };

            let (tx, rx) = oneshot::channel();
            let msg = Message {
                target,
                session: (rand::thread_rng().gen_range(0..=255)),
                body: (MessageBody::FindNode(self.id, id, Some(tx))),
            };

            let _ = self.service_tx.as_ref().unwrap().send(msg).await;
            rx.await.unwrap()
        }
    }

    pub fn node_lookup(&mut self, id: Identifier) {
        // What should max count be?
        // let mut count = 0;
        let targets = {
            let table = &self.table.lock().unwrap();
            if let Some(targets) = table.get_closest_nodes(&id, A) {
                targets
            } else {
                println!("No nodes in table");
                return;
            }
        };

        // while count < 15 {
        //     // 1. Grab "A" closest nodes from table.

        //     // 2. Send find_node request to each.

        //     // 3. Update table with responses

        //     count += 1;
        // }

        unimplemented!()
    }
    // ---------------------------------------------------------------------------------------------------

    pub async fn start(&mut self) -> Result<(), &'static str> {
        let local_record = Peer {
            id: self.id,
            socket_addr: self.socket,
        };
        if let Some(service_tx) = Service::spawn(local_record, self.table.clone()).await {
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
    use crate::helper::U256;
    use std::net::{IpAddr, SocketAddr};
    use tokio::time::Duration;

    #[tokio::test]
    async fn ping_rpc() {
        let mut local = Node::new(
            U256::from(0).into(),
            SocketAddr::new("127.0.0.1".parse::<IpAddr>().unwrap(), 6000),
        );
        let mut remote = Node::new(
            U256::from(1).into(),
            SocketAddr::new("127.0.0.1".parse::<IpAddr>().unwrap(), 6001),
        );
        local.table.lock().unwrap().add(Peer {
            id: remote.id,
            socket_addr: remote.socket,
        });

        let _ = local.start().await;
        let _ = remote.start().await;

        tokio::time::sleep(Duration::from_secs(1)).await;
        let ping = local.ping(remote.id);
        assert!(ping.await);
        tokio::time::sleep(Duration::from_secs(1)).await;

        let dummy = Node::new(
            U256::from(2).into(),
            SocketAddr::new("127.0.0.1".parse::<IpAddr>().unwrap(), 6002),
        );
        let ping = local.ping(dummy.id);
        assert!(!ping.await);
    }

    #[allow(warnings)]
    #[tokio::test]
    async fn find_node_rpc() {
        let mut local = Node::new(
            U256::from(0).into(),
            SocketAddr::new("127.0.0.1".parse::<IpAddr>().unwrap(), 6000),
        );
        let mut remote = Node::new(
            U256::from(1).into(),
            SocketAddr::new("127.0.0.1".parse::<IpAddr>().unwrap(), 6001),
        );

        local.table.lock().unwrap().add(Peer {
            id: remote.id,
            socket_addr: remote.socket,
        });

        // Populate remote's table
        {
            let mut remote_table = remote.table.lock().unwrap();
            let remote_table = {
                for i in 2..10 {
                    if i != 3 {
                        let port = "600".to_string() + &i.to_string();
                        let mut node = Node::new(
                            U256::from(i).into(),
                            SocketAddr::new(
                                "127.0.0.1".parse::<IpAddr>().unwrap(),
                                port.parse::<u16>().unwrap(),
                            ),
                        );

                        remote_table.add(Peer {
                            id: node.id,
                            socket_addr: node.socket,
                        });
                    }
                }
                remote_table
            };
        }
        let _ = local.start().await;
        let _ = remote.start().await;

        tokio::time::sleep(Duration::from_secs(1)).await;
        let mut node_to_find = Node::new(
            U256::from(3).into(),
            SocketAddr::new("127.0.0.1".parse::<IpAddr>().unwrap(), 6003),
        );

        let find_node = local.find_node(node_to_find.id);
        let result = find_node.await;
        println!("\n");
        println!("Peer: {:?}", result);

        // let expected result = ;
        // assert_eq!(find_node.await, );

        // Verify response from node
    }

    #[tokio::test]
    async fn node_lookup() {
        let mut local = Node::new(
            U256::from(0).into(),
            SocketAddr::new("127.0.0.1".parse::<IpAddr>().unwrap(), 6000),
        );
        let remote = Node::new(
            U256::from(1).into(),
            SocketAddr::new("127.0.0.1".parse::<IpAddr>().unwrap(), 6001),
        );
        let node_to_find = Node::new(
            U256::from(3).into(),
            SocketAddr::new("127.0.0.1".parse::<IpAddr>().unwrap(), 6003),
        );

        local.table.lock().unwrap().add(Peer {
            id: remote.id,
            socket_addr: remote.socket,
        });

        local.node_lookup(node_to_find.id);
    }
}
