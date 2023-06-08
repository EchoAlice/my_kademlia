use crate::helper::Identifier;
use crate::kbucket::{KbucketTable, TableRecord};
use std::collections::HashMap;
use std::net::{Ipv4Addr, SocketAddrV4};
use tokio::io;
use tokio::net::UdpSocket;

#[derive(Debug)]
pub enum FindNodeResult {
    Found(Option<TableRecord>),
    NotFound(Vec<Option<TableRecord>>),
}

#[derive(Debug)]
pub enum Search {
    Success(usize, usize),
    Failure(usize, usize),
}

// The main Kademlia client struct.
// Provides user-level API for performing querie and interacting with the underlying service.
#[derive(Debug, PartialEq)]
pub struct Node {
    pub node_id: Identifier,
    pub table: KbucketTable,
    pub store: HashMap<Vec<u8>, Vec<u8>>, // Same storage as Portal network to store samples
}

impl Node {
    pub fn new(node_id: Identifier, table_record: TableRecord) -> Self {
        Self {
            node_id,
            table: KbucketTable::new(node_id, table_record),
            store: Default::default(),
        }
    }

    // Protocol's RPCs:
    // ---------------------------------------------------------------------------------------------------
    /// "The most important procedure a Kademlia participant must perform is to locate
    /// the k closest nodes to some given node ID"
    ///     - Kademlia Paper
    ///
    /// Recieves an id request and returns node information on nodes within
    /// *its closest bucket* (instead of k-closest nodes) to that id.
    pub fn find_node(&mut self, node_id: &Identifier) -> FindNodeResult {
        match KbucketTable::search_table(&self.table, node_id) {
            Search::Success(bucket_index, column_index) => {
                let bucket = self.table.buckets[bucket_index];
                FindNodeResult::Found(bucket[column_index])
            }
            Search::Failure(bucket_index, column_index) => {
                let bucket = self.table.buckets[bucket_index];
                let mut known_nodes = Vec::new();

                for node in bucket.iter() {
                    if node.is_some() {
                        known_nodes.push(*node)
                    }
                }
                FindNodeResult::NotFound(known_nodes)
            }
        }
    }

    pub async fn ping(&mut self, local_socket: &UdpSocket, node_to_ping: &Identifier) -> usize {
        let message_packet = b"Ping";

        match self.find_node(node_to_ping) {
            FindNodeResult::Found(Some(node_record)) => {
                let remote_socket = SocketAddrV4::new(node_record.ip_address, node_record.udp_port);
                local_socket.connect(remote_socket).await;
                local_socket.send(message_packet).await.unwrap()
            }
            _ => unreachable!("Node wasn't found to ping"),
        }
    }

    // TODO:
    // pub fn find_value() {}

    /// "Most operations are implemented in terms of the lookup proceedure. To store a
    /// <key,value> pair, a participant locates the k closes nodes to the key and sends them store RPCs".
    ///
    // TODO: Instructs a node to store a key, value pair for later retrieval.
    // pub fn store(&mut self, key: Identifier, value: Vec<u8>) {}

    // ---------------------------------------------------------------------------------------------------
    pub async fn socket(&self) -> io::Result<UdpSocket> {
        let table_record = self.table.local_record;
        let socket_addr = SocketAddrV4::new(table_record.ip_address, table_record.udp_port);
        let socket = UdpSocket::bind(socket_addr).await;
        socket
    }
}

#[cfg(test)]
mod tests {
    use crate::helper::PING_MESSAGE_SIZE;

    use super::*;

    fn mk_nodes(n: u8) -> (Node, Vec<Node>) {
        let ip_address = String::from("127.0.0.1").parse::<Ipv4Addr>().unwrap();
        let port_start = 9000_u16;

        let local_node = mk_node(&ip_address, port_start, 0);
        let remote_nodes: Vec<Node> = (1..n)
            .map(|i| mk_node(&ip_address, port_start, i))
            .collect();

        (local_node, remote_nodes)
    }

    fn mk_node(ip_address: &Ipv4Addr, port_start: u16, index: u8) -> Node {
        let mut node_id = [0_u8; 32];
        node_id[31] += index;
        let udp_port = port_start + index as u16;

        let table_record = TableRecord {
            node_id,
            ip_address: *ip_address,
            udp_port,
        };

        Node::new(node_id, table_record)
    }

    #[test]
    fn add_redundant_node() {
        let (mut local_node, remote_nodes) = mk_nodes(2);

        let result = local_node
            .table
            .add_node(&remote_nodes[0].table.local_record);
        assert!(result);
        let result2 = local_node
            .table
            .add_node(&remote_nodes[0].table.local_record);
        assert!(!result2);
    }

    #[test]
    fn find_node_present() {
        let (mut local_node, remote_nodes) = mk_nodes(10);

        let node_to_find = remote_nodes[1].table.local_record;
        for node in remote_nodes {
            local_node.table.add_node(&node.table.local_record);
        }

        match local_node.find_node(&node_to_find.node_id) {
            FindNodeResult::Found(Some(node)) => {
                assert_eq!(node.node_id, node_to_find.node_id)
            }
            _ => unreachable!("Node should have been found"),
        }
    }

    #[test]
    fn find_node_absent() {
        let (mut local_node, remote_nodes) = mk_nodes(10);
        let absent_index = 4;
        let node_to_find = remote_nodes[absent_index].table.local_record;

        for (i, node) in remote_nodes.iter().enumerate() {
            if i == absent_index {
                continue;
            } else {
                local_node.table.add_node(&node.table.local_record);
            }
        }

        match local_node.find_node(&node_to_find.node_id) {
            FindNodeResult::NotFound(nodes_returned) => {
                let node_to_find_index = local_node.table.xor_bucket_index(&node_to_find.node_id);

                for node in nodes_returned {
                    if let Some(node) = node {
                        let node_in_bucket_index = local_node.table.xor_bucket_index(&node.node_id);
                        assert_ne!(node_to_find, node);
                        assert_eq!(node_to_find_index, node_in_bucket_index);
                    } else {
                        panic!("find_node() returned an empty index")
                    }
                }
            }
            _ => unreachable!("FindNodeResult shouldn't == Found"),
        }
    }

    #[tokio::test]
    async fn run_ping() {
        let (mut local_node, remote_nodes) = mk_nodes(2);
        local_node
            .table
            .add_node(&remote_nodes[0].table.local_record);

        // Create a server for our node.  Improper way first, then proper.
        let local_socket = local_node.socket().await; // .unwrap()
        let remote_socket = remote_nodes[0].socket().await;

        match (local_socket, remote_socket) {
            (Ok(local_socket), Ok(remote_socket)) => {
                let result = local_node
                    .ping(&local_socket, &remote_nodes[0].node_id)
                    .await;
                assert_eq!(result, PING_MESSAGE_SIZE)
            }
            _ => unreachable!("Both nodes should have UDP sockets"),
        }
    }
}
