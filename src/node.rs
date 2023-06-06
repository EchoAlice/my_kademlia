use crate::helper::Identifier;
use crate::kbucket::KbucketTable;
use std::collections::HashMap;
use std::net::{Ipv4Addr, SocketAddrV4};
use tokio::io;
use tokio::net::UdpSocket;

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

    pub async fn socket(&self) -> io::Result<UdpSocket> {
        let socket = UdpSocket::bind(self.table_record.socket_addr).await;
        socket
    }

    pub async fn ping(&self, local_socket: &UdpSocket, node_to_ping: &SocketAddrV4) -> usize {
        let message_packet = b"Ping";
        local_socket.connect(node_to_ping).await;
        local_socket.send(message_packet).await.unwrap()
    }

    // pub fn find_node() {}

    // pub fn find_value() {}

    // pub fn store() {}
}
