use crate::helper::Identifier;
use crate::kbucket::{Bucket, KbucketTable, TableRecord};
use std::collections::HashMap;
use std::net::{Ipv4Addr, SocketAddrV4};
use tokio::io;
use tokio::net::UdpSocket;
use tokio::time::Duration;

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
    pub fn find_node(&mut self, id: &Identifier) -> HashMap<[u8; 32], TableRecord> {
        self.table.get_bucket_for(id)
    }

    pub async fn ping(&mut self, local_socket: &UdpSocket, node_to_ping: &Identifier) -> usize {
        let message_packet = b"Ping";

        match self.table.get(node_to_ping) {
            Some(remote_record) => {
                let remote_socket =
                    SocketAddrV4::new(remote_record.ip_address, remote_record.udp_port);
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
        let table_record = self.table.record;
        let socket_addr = SocketAddrV4::new(table_record.ip_address, table_record.udp_port);
        let socket = UdpSocket::bind(socket_addr).await;
        socket
    }

    // TODO: start_server() -> Result<(), Box<dyn std::error::Error>> {
    pub async fn start_server(&self, mut buffer: [u8; 1024], socket: UdpSocket) {
        println!("Starting server");
        loop {
            let Ok((size, addr)) = socket.recv_from(&mut buffer).await else { todo!() };
            let message = buffer.clone().to_vec();
            self.process(message).await;
        }
    }

    async fn process(&self, message: Vec<u8>) {
        println!("Message: {:?}", message);
    }
}

#[cfg(test)]
mod tests {
    use std::sync::LockResult;

    use crate::{helper::PING_MESSAGE_SIZE, node};

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
            .add(remote_nodes[0].table.id, remote_nodes[0].table.record);
        assert!(result);
        let result2 = local_node
            .table
            .add(remote_nodes[0].table.id, remote_nodes[0].table.record);
        assert!(!result2);
    }

    #[test]
    fn find_node() {
        let (mut local_node, remote_nodes) = mk_nodes(10);
        let node_to_find = &remote_nodes[1];
        let ntf_bucket_index = local_node.table.xor_bucket_index(&node_to_find.table.id);

        for node in &remote_nodes {
            local_node.table.add(node.table.id, node.table.record);
        }

        let closest_nodes = local_node.find_node(&node_to_find.node_id);
        for node in closest_nodes {
            let bucket_index = local_node.table.xor_bucket_index(&node.0);
            assert_eq!(ntf_bucket_index, bucket_index);
        }
    }

    #[tokio::test]
    async fn ping() {
        let (mut local_node, remote_nodes) = mk_nodes(2);
        local_node
            .table
            .add(remote_nodes[0].table.id, remote_nodes[0].table.record);

        let local_socket = local_node.socket().await.unwrap();
        let remote_socket = remote_nodes[0].socket().await.unwrap();
        let mut buffer = [0u8; 1024];
        let mut remote_id = [0u8; 32];
        remote_id.copy_from_slice(&remote_nodes[0].node_id);

        let handle = tokio::spawn(async move {
            remote_nodes[0].start_server(buffer, remote_socket).await;
        });

        tokio::time::sleep(Duration::from_secs(1)).await;
        let result = local_node.ping(&local_socket, &remote_id).await;
        tokio::time::sleep(Duration::from_secs(1)).await;
        assert_eq!(result, PING_MESSAGE_SIZE)
    }
}
