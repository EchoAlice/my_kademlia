use crate::helper::Identifier;
use crate::kbucket::KbucketTable;
use crate::node;
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
    // pub fn store() {}

    // ---------------------------------------------------------------------------------------------------
    pub async fn socket(&self) -> io::Result<UdpSocket> {
        let socket = UdpSocket::bind(self.table_record.socket_addr).await;
        socket
    }
}
