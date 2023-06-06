use crate::helper::Identifier;
use crate::kbucket::KbucketTable;
use std::collections::HashMap;
use std::net::{Ipv4Addr, SocketAddrV4};
use tokio::net::UdpSocket;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TableRecord {
    pub node_id: Identifier,
    pub ip_address: Ipv4Addr,
    pub udp_port: u16,
}

// The main Kademlia client struct.
// Provides user-level API for performing querie and interacting with the underlying service.
// #[derive(Clone, Copy, Debug, PartialEq)]
#[derive(Debug, PartialEq)]
pub struct Node {
    pub node_id: Identifier,
    pub table_record: TableRecord,
    pub socket_addr: SocketAddrV4,
    pub table: KbucketTable,
    pub store: HashMap<Vec<u8>, Vec<u8>>, // Same storage as Portal network to store samples
}

impl Node {
    pub fn new(node_id: Identifier, table_record: TableRecord) -> Self {
        Self {
            node_id,
            table_record,
            socket_addr: SocketAddrV4::new(table_record.ip_address, table_record.udp_port),
            table: KbucketTable::new(node_id),
            store: Default::default(),
        }
    }

    // pub async fn socket() -> UdpSocket {}

    // pub async fn ping(&self, node_to_ping: &Node) -> io::Result<()> {}

    // pub fn find_node() {}

    // pub fn find_value() {}

    // pub fn store() {}
}
