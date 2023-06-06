use crate::helper::Identifier;
use crate::kbucket::KbucketTable;
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

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TableRecord {
    pub node_id: Identifier,
    pub ip_address: Ipv4Addr,
    pub udp_port: u16,
    pub socket_addr: SocketAddrV4,
}

// The main Kademlia client struct.
// Provides user-level API for performing querie and interacting with the underlying service.
// #[derive(Clone, Copy, Debug, PartialEq)]
#[derive(Debug, PartialEq)]
pub struct Node {
    pub node_id: Identifier,
    pub table_record: TableRecord,
    pub table: KbucketTable,
    pub store: HashMap<Vec<u8>, Vec<u8>>, // Same storage as Portal network to store samples
}

impl Node {
    pub fn new(node_id: Identifier, table_record: TableRecord) -> Self {
        Self {
            node_id,
            table_record,
            table: KbucketTable::new(node_id),
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
    pub fn find_node(&mut self, node_id: Identifier) -> FindNodeResult {
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
                        // Should I be dereferencing the node to send to others?  Or copy the node to share?
                        known_nodes.push(*node)
                    }
                }
                FindNodeResult::NotFound(known_nodes)
            }
        }
    }

    pub async fn ping(&self, local_socket: &UdpSocket, node_to_ping: &SocketAddrV4) -> usize {
        let message_packet = b"Ping";
        local_socket.connect(node_to_ping).await;
        local_socket.send(message_packet).await.unwrap()
    }

    // TODO:
    // pub fn find_value() {}

    // TODO:
    /// Instructs a node to store a key, value pair for later retrieval.
    ///
    /// "Most operations are implemented in terms of the lookup proceedure. To store a
    /// <key,value> pair, a participant locates the k closes nodes to the key and sends them store RPCs".
    // pub fn store(&mut self, key: Identifier, value: Vec<u8>) {}

    // ---------------------------------------------------------------------------------------------------
    pub async fn socket(&self) -> io::Result<UdpSocket> {
        let socket = UdpSocket::bind(self.table_record.socket_addr).await;
        socket
    }
}

#[cfg(test)]
mod tests {
    use crate::helper::PING_MESSAGE_SIZE;

    use super::*;

    fn mk_nodes(n: u8) -> (Node, Vec<TableRecord>) {
        let ip_address = String::from("127.0.0.1").parse::<Ipv4Addr>().unwrap();
        let port_start = 9000_u16;

        let local_node_record = mk_node_record(&ip_address, port_start, 0);
        let local_node = Node::new(local_node_record.node_id, local_node_record);

        let remote_node_records: Vec<TableRecord> = (1..n)
            .map(|i| mk_node_record(&ip_address, port_start, i))
            .collect();

        (local_node, remote_node_records)
    }

    fn mk_node_record(ip_address: &Ipv4Addr, port_start: u16, index: u8) -> TableRecord {
        let mut node_id = [0_u8; 32];
        node_id[31] += index;
        let udp_port = port_start + index as u16;

        let table_record = TableRecord {
            node_id,
            ip_address: *ip_address,
            udp_port,
            socket_addr: SocketAddrV4::new(*ip_address, udp_port),
        };

        return table_record;
    }

    #[test]
    fn add_redundant_node() {
        let (mut local_node, remote_nodes) = mk_nodes(2);

        let result = local_node.table.add_node(&remote_nodes[0]);
        assert!(result);
        let result2 = local_node.table.add_node(&remote_nodes[0]);
        assert!(!result2);
    }

    #[test]
    fn find_node_present() {
        let (mut local_node, remote_nodes) = mk_nodes(5);

        let node_to_find = remote_nodes[1];
        for node in remote_nodes {
            local_node.table.add_node(&node);
        }

        match local_node.find_node(node_to_find.node_id) {
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
        let node_to_find = remote_nodes[absent_index];

        for (i, node) in remote_nodes.iter().enumerate() {
            if i == absent_index {
                continue;
            } else {
                local_node.table.add_node(&node);
            }
        }

        match local_node.find_node(node_to_find.node_id) {
            FindNodeResult::NotFound(nodes_returned) => {
                let node_to_find_index = local_node.table.xor_bucket_index(node_to_find.node_id);

                for node in nodes_returned {
                    if let Some(node) = node {
                        let node_in_bucket_index = local_node.table.xor_bucket_index(node.node_id);
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
        let (local_node, remote_nodes) = mk_nodes(2);

        let local_socket = local_node.socket().await;
        let remote_socket = UdpSocket::bind(remote_nodes[0].socket_addr).await;

        match (local_socket, remote_socket) {
            (Ok(local_socket), Ok(remote_socket)) => {
                let result = local_node
                    .ping(&local_socket, &remote_nodes[0].socket_addr)
                    .await;
                assert_eq!(result, PING_MESSAGE_SIZE)
            }
            _ => unreachable!("Both nodes should have UDP sockets"),
        }
    }
}
