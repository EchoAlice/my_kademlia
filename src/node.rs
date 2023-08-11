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

//  Typically 20.  Only 7 for testing
pub const K: usize = 7; // Max bucket size
const A: usize = 3; // Parallel queries for node_lookup()
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

    // Protocol's Exposed functions:
    // ---------------------------------------------------------------------------------------------------
    /// The node_lookup function iteratively calls our find_node rpc to query the "a" closest nodes to an id.
    /// With each response, our local node updates its routing table and calls the next closest peers etc...
    ///
    /// WIP:
    ///  - Wrap query logic within a "query_depth = 5" loop.
    ///  - Should I have 3 seperate indexes for the 3 parallel lookups (each with a max of "?
    pub async fn node_lookup(&mut self, id: Identifier) {
        // 1. Grab "A" closest peers from table.
        let targets = {
            let table = &self.table.lock().unwrap();
            if let Some(targets) = table.get_closest_nodes(&id, A) {
                targets
            } else {
                println!("No nodes in table");
                return;
            }
        };
        println!("Targets: {:?}", targets);

        // 2. Send find_node request to each peer.
        for peer in targets {
            let rx = self.find_node_targeted(id, peer).await;
            tokio::spawn(async move {
                if let Some(peers) = rx.await.unwrap() {
                    // How do i update the table with newly received peers?
                    println!("Peers received: {:?}", peers);
                    println!("\n");
                }
            });
        }

        // 3. Update table with responses
    }

    /// This function is async because the service processes inbound reqs from rpcs one at a time.  
    /// service_tx.send() doesn't require a response to happen immediately!  Access rx response by assigning fn a
    /// variable.
    pub async fn find_node_targeted(
        &mut self,
        id: Identifier,
        target: Peer,
    ) -> oneshot::Receiver<Option<Vec<Peer>>> {
        // ) -> impl Future<Output = Option<Vec<Peer>>> + '_ {
        // async move {
        let (tx, rx) = oneshot::channel();
        let msg = Message {
            target,
            session: (rand::thread_rng().gen_range(0..=255)),
            body: (MessageBody::FindNode(self.id, id, Some(tx))),
        };

        let _ = self.service_tx.as_ref().unwrap().send(msg).await;
        // rx.await.unwrap; }
        rx
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

/// Run tests individually.  Some error occurs because of shared IP addresses
/// between tests.
///
/// Tests are explicitely verbose to provide all context needed in one source.
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
        let node_to_find = Node::new(
            U256::from(13).into(),
            SocketAddr::new("127.0.0.1".parse::<IpAddr>().unwrap(), 6003),
        );

        local.table.lock().unwrap().add(Peer {
            id: remote.id,
            socket_addr: remote.socket,
        });

        // Populate remote's table
        {
            let mut remote_table = remote.table.lock().unwrap();
            let remote_table = {
                for i in 2..30 {
                    if i == 13 {
                        continue;
                    }
                    let port = "600".to_string() + &i.to_string();
                    let peer = Peer {
                        id: U256::from(i).into(),
                        socket_addr: socket::SocketAddr {
                            addr: SocketAddr::new(
                                "127.0.0.1".parse::<IpAddr>().unwrap(),
                                port.parse::<u16>().unwrap(),
                            ),
                        },
                    };
                    remote_table.add(peer);
                }
                remote_table
            };
        }

        // Creates our expected response
        let mut expected_peers: Vec<Peer> = Vec::new();
        for i in 8..16 {
            if i == 13 {
                continue;
            }
            let port = "600".to_string() + &i.to_string();
            let peer = Peer {
                id: U256::from(i).into(),
                socket_addr: socket::SocketAddr {
                    addr: SocketAddr::new(
                        "127.0.0.1".parse::<IpAddr>().unwrap(),
                        port.parse::<u16>().unwrap(),
                    ),
                },
            };
            expected_peers.push(peer);
        }

        let _ = local.start().await;
        let _ = remote.start().await;

        tokio::time::sleep(Duration::from_secs(1)).await;
        let find_node = local.find_node(node_to_find.id);
        if let Some(mut closest_nodes) = find_node.await {
            closest_nodes.sort_by(|a, b| a.id.partial_cmp(&b.id).unwrap());
            assert_eq!(closest_nodes, expected_peers);
        } else {
            panic!()
        }
    }

    #[tokio::test]
    async fn find_node_targeted() {
        let mut local = Node::new(
            U256::from(0).into(),
            SocketAddr::new("127.0.0.1".parse::<IpAddr>().unwrap(), 6000),
        );
        let mut remote = Node::new(
            U256::from(1).into(),
            SocketAddr::new("127.0.0.1".parse::<IpAddr>().unwrap(), 6001),
        );
        let remote_peer = Peer {
            id: remote.id,
            socket_addr: remote.socket,
        };

        let node_to_find = Node::new(
            U256::from(13).into(),
            SocketAddr::new("127.0.0.1".parse::<IpAddr>().unwrap(), 6003),
        );

        // TODO: Add more peers to table so we can call "a" nodes simultaneously.
        local.table.lock().unwrap().add(Peer {
            id: remote.id,
            socket_addr: remote.socket,
        });

        // Populate remote's table
        {
            let mut remote_table = remote.table.lock().unwrap();
            for i in 2..30 {
                if i == 13 {
                    continue;
                }
                let port = "600".to_string() + &i.to_string();
                let peer = Peer {
                    id: U256::from(i).into(),
                    socket_addr: socket::SocketAddr {
                        addr: SocketAddr::new(
                            "127.0.0.1".parse::<IpAddr>().unwrap(),
                            port.parse::<u16>().unwrap(),
                        ),
                    },
                };
                remote_table.add(peer);
            }
        }

        // Creates our expected response
        let mut expected_peers: Vec<Peer> = Vec::new();
        for i in 8..16 {
            if i == 13 {
                continue;
            }
            let port = "600".to_string() + &i.to_string();
            let peer = Peer {
                id: U256::from(i).into(),
                socket_addr: socket::SocketAddr {
                    addr: SocketAddr::new(
                        "127.0.0.1".parse::<IpAddr>().unwrap(),
                        port.parse::<u16>().unwrap(),
                    ),
                },
            };
            expected_peers.push(peer);
        }

        let _ = local.start().await;
        let _ = remote.start().await;
        tokio::time::sleep(Duration::from_secs(1)).await;

        let rx = local.find_node_targeted(node_to_find.id, remote_peer).await;

        if let Some(mut closest_nodes) = rx.await.unwrap() {
            closest_nodes.sort_by(|a, b| a.id.partial_cmp(&b.id).unwrap());
            assert_eq!(closest_nodes, expected_peers);
        } else {
            panic!()
        }
    }

    #[tokio::test]
    async fn node_lookup() {
        // TODO: Request the node_to_find from a node who doesn't have the node.
        //       ie. Require two hops for successful lookup.
        let mut local = Node::new(
            U256::from(0).into(),
            SocketAddr::new("127.0.0.1".parse::<IpAddr>().unwrap(), 6000),
        );
        let node_to_find = Node::new(
            U256::from(3).into(),
            SocketAddr::new("127.0.0.1".parse::<IpAddr>().unwrap(), 6003),
        );

        // Here we create nodes to add to local's routing table.
        let mut remote_nodes = Vec::new();
        let mut remote1 = Node::new(
            U256::from(1).into(),
            SocketAddr::new("127.0.0.1".parse::<IpAddr>().unwrap(), 6001),
        );
        remote_nodes.push(&remote1);
        let mut remote5 = Node::new(
            U256::from(5).into(),
            SocketAddr::new("127.0.0.1".parse::<IpAddr>().unwrap(), 6005),
        );
        remote_nodes.push(&remote5);
        let mut remote7 = Node::new(
            U256::from(7).into(),
            SocketAddr::new("127.0.0.1".parse::<IpAddr>().unwrap(), 6007),
        );
        remote_nodes.push(&remote7);
        let mut remote20 = Node::new(
            U256::from(20).into(),
            SocketAddr::new("127.0.0.1".parse::<IpAddr>().unwrap(), 6020),
        );
        remote_nodes.push(&remote20);

        tokio::time::sleep(Duration::from_secs(1)).await;

        // Populate local's and remotes' tables.
        {
            let mut local_table = local.table.lock().unwrap();
            for node in remote_nodes {
                local_table.add(Peer {
                    id: node.id,
                    socket_addr: node.socket,
                });

                let mut remote_table = node.table.lock().unwrap();
                for i in 2..30 {
                    if i == 13 {
                        continue;
                    }
                    let port = "600".to_string() + &i.to_string();
                    let peer = Peer {
                        id: U256::from(i).into(),
                        socket_addr: socket::SocketAddr {
                            addr: SocketAddr::new(
                                "127.0.0.1".parse::<IpAddr>().unwrap(),
                                port.parse::<u16>().unwrap(),
                            ),
                        },
                    };
                    remote_table.add(peer);
                }
            }
        }

        // To test the communication between nodes, we need to instantiate
        // each of their servers.
        let _ = local.start().await;
        let _ = remote1.start().await;
        let _ = remote5.start().await;
        let _ = remote7.start().await;
        let _ = remote20.start().await;

        local.node_lookup(node_to_find.id).await;
    }
}
