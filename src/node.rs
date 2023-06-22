use crate::helper::{Identifier, PING_MESSAGE_SIZE};
use crate::kbucket::{Bucket, KbucketTable, TableRecord};
use crate::node;
use rand::Rng;
use std::collections::HashMap;
use std::net::{IpAddr, SocketAddr, SocketAddrV4};
use std::sync::{Arc, Mutex};
use tokio::io;
use tokio::net::UdpSocket;
use tokio::time::Duration;

const NODES_TO_QUERY: usize = 1; // "a"

#[derive(Debug)]
pub enum Message {
    Ping([u8; 1024]),
    Pong([u8; 1024]),
    // FindNode,
    // FoundNode,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Peer {
    pub id: Identifier,
    pub record: TableRecord,
}

// The main Kademlia client struct.
// Provides user-level API for performing querie and interacting with the underlying service.
// TODO:  Place all Arc<Mutex<things>> in a state struct
#[derive(Clone, Debug)]
pub struct Node {
    pub table: Arc<Mutex<KbucketTable>>,
    pub store: HashMap<Vec<u8>, Vec<u8>>, // Same storage as Portal network to store samples
    pub outbound_requests: Arc<Mutex<HashMap<Identifier, u8>>>,
    pub socket: Arc<UdpSocket>,
}

impl Node {
    pub async fn new(peer: Peer) -> Self {
        Self {
            table: Arc::new(Mutex::new(KbucketTable::new(peer))),
            store: Default::default(),
            outbound_requests: Default::default(),
            socket: Arc::new(
                UdpSocket::bind(SocketAddr::new(
                    peer.record.ip_address,
                    peer.record.udp_port,
                ))
                .await
                .unwrap(),
            ),
        }
    }

    // Protocol's RPCs:
    // ---------------------------------------------------------------------------------------------------

    /// "The most important procedure a Kademlia participant must perform is to locate the k closest nodes
    /// to some given node ID.  We call this procedure a **node lookup**".
    ///
    /// How is a node lookup different from the find_node() RPC?

    // TODO:  1. Set up networking communication for find_node()
    //        2. Create complete routing table logic (return K closest nodes instead of closest bucket)
    pub fn find_node(&self, id: &Identifier) -> HashMap<[u8; 32], TableRecord> {
        self.table.lock().unwrap().get_bucket_for(id).clone()
    }

    pub async fn ping(&mut self, node_to_ping: Identifier) -> usize {
        let session_number: u8 = rand::thread_rng().gen_range(0..=255);

        let (local_id, remote_socket) = {
            let table = self.table.lock().unwrap();
            let remote_record = table.get(&node_to_ping).unwrap();
            (
                table.peer.id,
                SocketAddr::new(remote_record.ip_address, remote_record.udp_port),
            )
        };

        self.socket.connect(remote_socket).await;
        let message = self.create_message(b"Ping", &local_id, session_number);
        let insert_session_result = self
            .outbound_requests
            .lock()
            .unwrap()
            .insert(node_to_ping, session_number);
        self.socket.send(&message).await.unwrap()
    }

    // TODO:
    // pub fn find_value() {}

    // TODO:
    // pub fn store(&mut self, key: Identifier, value: Vec<u8>) {}

    // ---------------------------------------------------------------------------------------------------
    fn create_message(
        &self,
        mtype: &[u8; 4],
        local_id: &Identifier,
        session_number: u8,
    ) -> [u8; 1024] {
        let mut message = [0u8; 1024];
        message[0..4].copy_from_slice(mtype);
        message[4..36].copy_from_slice(local_id);
        message[36] = session_number;
        message
    }

    async fn pong(&self, session_number: u8, addr_to_pong: &SocketAddr) {
        let local_id = {
            let table = self.table.lock().unwrap();
            table.peer.id
        };
        let message = self.create_message(b"Pong", &local_id, session_number);
        self.socket.send_to(&message, addr_to_pong).await;
    }

    pub async fn start_server(&mut self, mut buffer: [u8; 1024]) {
        loop {
            let Ok((size, sender_addr)) = self.socket.recv_from(&mut buffer).await else { todo!() };

            // Converts received socket bytes to message type
            if &buffer[0..4] == b"Ping" {
                self.process(&Message::Ping(buffer), &sender_addr).await;
            } else if &buffer[0..4] == b"Pong" {
                self.process(&Message::Pong(buffer), &sender_addr).await;
            } else {
                println!("Message wasn't ping or pong");
            }
        }
    }

    async fn process(&mut self, message: &Message, sender_addr: &SocketAddr) {
        match message {
            Message::Ping(datagram) => {
                let node_id = &datagram[4..36];
                let session_number = datagram[36];
                self.pong(session_number, sender_addr).await;
            }
            Message::Pong(datagram) => {
                let node_id = &datagram[4..36];
                if let Some(session_number) = self.outbound_requests.lock().unwrap().get(node_id) {
                    if session_number == &datagram[36] {
                        println!("Successful ping");
                    } else {
                        println!("Unsuccessful ping");
                    }
                } else {
                    println!("No session number for remote node");
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{helper::PING_MESSAGE_SIZE, node};
    use std::sync::LockResult;

    async fn mk_nodes(n: u8) -> (Node, Vec<Node>) {
        let ip_address = String::from("127.0.0.1").parse::<IpAddr>().unwrap();
        let port_start = 9000_u16;

        let local_node = mk_node(&ip_address, port_start, 0).await;
        let mut remote_nodes = Vec::new();

        for i in 1..n {
            remote_nodes.push(mk_node(&ip_address, port_start, i).await);
        }

        (local_node, remote_nodes)
    }

    async fn mk_node(ip_address: &IpAddr, port_start: u16, index: u8) -> Node {
        let mut id = [0_u8; 32];
        id[31] += index;
        let udp_port = port_start + index as u16;

        let record = TableRecord {
            ip_address: *ip_address,
            udp_port,
        };
        let peer = Peer { id, record };

        Node::new(peer).await
    }

    // Run tests independently.  Tests fail when they're run together bc of address reuse.
    // TIP: If you don't give the thing a port, a free port is given automatically
    #[tokio::test]
    async fn add_redundant_node() {
        let (local_node, remote_nodes) = mk_nodes(2).await;
        let mut local_table = local_node.table.lock().unwrap();
        let remote_table = remote_nodes[0].table.lock().unwrap();

        let result = local_table.add(remote_table.peer);
        let result2 = local_table.add(remote_table.peer);
        assert!(result);
        assert!(!result2);
    }

    #[tokio::test]
    async fn find_node() {
        let (local_node, remote_nodes) = mk_nodes(10).await;

        let (node_to_find, ntf_bucket_index) = {
            let mut local_table = local_node.table.lock().unwrap();
            let node_to_find = remote_nodes[1].table.lock().unwrap().peer.id;
            let ntf_bucket_index = local_table.xor_bucket_index(&node_to_find);

            for node in &remote_nodes {
                let remote_peer = node.table.lock().unwrap().peer;
                local_table.add(remote_peer);
            }
            (node_to_find, ntf_bucket_index)
        };

        let closest_nodes = local_node.find_node(&node_to_find);

        for node in closest_nodes {
            let bucket_index = local_node.table.lock().unwrap().xor_bucket_index(&node.0);
            assert_eq!(ntf_bucket_index, bucket_index);
        }
    }

    #[tokio::test]
    async fn ping() {
        let (mut local_node, mut remote_nodes) = mk_nodes(2).await;

        let remote_id = {
            let mut local_table = local_node.table.lock().unwrap();
            let remote_peer = remote_nodes[0].table.lock().unwrap().peer;
            local_table.add(remote_peer);
            remote_peer.id
        };

        let mut local_node_copy = local_node.clone();

        tokio::spawn(async move {
            let mut buffer = [0u8; 1024];
            remote_nodes[0].start_server(buffer).await;
        });
        tokio::spawn(async move {
            let mut buffer = [0u8; 1024];
            local_node_copy.start_server(buffer).await;
        });
        tokio::time::sleep(Duration::from_secs(1)).await;

        // TODO:  Keep track of ping/pong messages w/ session_number
        let result = local_node.ping(remote_id).await;

        // Need to sleep for servers to run
        tokio::time::sleep(Duration::from_secs(1)).await;

        // assert_eq!(result, PING_MESSAGE_SIZE);
    }
}
