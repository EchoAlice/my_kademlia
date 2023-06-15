use crate::helper::Identifier;
use crate::kbucket::{Bucket, KbucketTable, TableRecord};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::net::{Ipv4Addr, SocketAddrV4};
use std::sync::{Arc, Mutex};
use tokio::io;
use tokio::net::UdpSocket;
use tokio::time::Duration;

// The main Kademlia client struct.
// Provides user-level API for performing querie and interacting with the underlying service.
#[derive(Clone, Debug)]
pub struct Node {
    pub node_id: Identifier,
    pub table: Arc<Mutex<KbucketTable>>,
    pub store: HashMap<Vec<u8>, Vec<u8>>, // Same storage as Portal network to store samples
    pub socket: Arc<UdpSocket>,
}

impl Node {
    pub async fn new(node_id: Identifier, table_record: TableRecord) -> Self {
        Self {
            node_id,
            table: Arc::new(Mutex::new(KbucketTable::new(node_id, table_record))),
            store: Default::default(),
            socket: Arc::new(
                UdpSocket::bind(SocketAddrV4::new(
                    table_record.ip_address,
                    table_record.udp_port,
                ))
                .await
                .unwrap(),
            ),
        }
    }

    // Protocol's RPCs:
    // ---------------------------------------------------------------------------------------------------
    pub fn find_node(&self, id: &Identifier) -> HashMap<[u8; 32], TableRecord> {
        self.table.lock().unwrap().get_bucket_for(id)
        // self.table.get_bucket_for(id)
    }

    pub async fn ping(&self, node_to_ping: &Identifier) -> usize {
        let message = b"Ping";
        let table = self.table.lock().unwrap();

        match table.get(node_to_ping) {
            Some(remote_record) => {
                let remote_socket =
                    SocketAddrV4::new(remote_record.ip_address, remote_record.udp_port);
                self.socket.connect(remote_socket).await;
                self.socket.send(message).await.unwrap()
            }
            _ => unreachable!("Node wasn't found to ping"),
        }
    }

    async fn pong(&self, addr_to_pong: &SocketAddr) {
        let message = b"Pong";
        self.socket.send_to(message, addr_to_pong).await;
    }

    // TODO:
    // pub fn find_value() {}

    // TODO:
    // pub fn store(&mut self, key: Identifier, value: Vec<u8>) {}

    // ---------------------------------------------------------------------------------------------------
    pub async fn start_server(&self, mut buffer: [u8; 1024]) {
        loop {
            let Ok((size, sender_addr)) = self.socket.recv_from(&mut buffer).await else { todo!() };
            self.process(&buffer, &sender_addr).await;
        }
    }

    async fn process(&self, message: &[u8], sender_addr: &SocketAddr) {
        println!("Message: {:?}", message);
        if &message[0..4] == b"Ping" {
            self.pong(sender_addr).await;
        }
        if &message[0..4] == b"Pong" {
            println!("Message was pong");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{helper::PING_MESSAGE_SIZE, node};
    use std::sync::LockResult;

    async fn mk_nodes(n: u8) -> (Node, Vec<Node>) {
        let ip_address = String::from("127.0.0.1").parse::<Ipv4Addr>().unwrap();
        let port_start = 9000_u16;

        let local_node = mk_node(&ip_address, port_start, 0).await;
        let mut remote_nodes = Vec::new();

        for i in 1..n {
            remote_nodes.push(mk_node(&ip_address, port_start, i).await);
        }

        (local_node, remote_nodes)
    }

    async fn mk_node(ip_address: &Ipv4Addr, port_start: u16, index: u8) -> Node {
        let mut node_id = [0_u8; 32];
        node_id[31] += index;
        let udp_port = port_start + index as u16;

        let table_record = TableRecord {
            ip_address: *ip_address,
            udp_port,
        };

        Node::new(node_id, table_record).await
    }

    #[tokio::test]
    async fn add_redundant_node() {
        let (local_node, remote_nodes) = mk_nodes(2).await;
        let mut local_table = local_node.table.lock().unwrap();
        let remote_table = remote_nodes[0].table.lock().unwrap();

        let result = local_table.add(remote_table.id, remote_table.record);
        assert!(result);
        let result2 = local_table.add(remote_table.id, remote_table.record);
        assert!(!result2);
    }

    #[tokio::test]
    async fn find_node() {
        let (local_node, remote_nodes) = mk_nodes(10).await;
        let mut local_table = local_node.table.lock().unwrap();

        let node_to_find = &remote_nodes[1];
        let ntf_bucket_index = local_table.xor_bucket_index(&node_to_find.node_id);

        for node in &remote_nodes {
            let remote_table = node.table.lock().unwrap();
            local_table.add(remote_table.id, remote_table.record);
        }

        drop(local_table);
        let closest_nodes = local_node.find_node(&node_to_find.node_id);

        for node in closest_nodes {
            let bucket_index = local_node.table.lock().unwrap().xor_bucket_index(&node.0);
            println!("{}, {}", ntf_bucket_index, bucket_index);
            assert_eq!(ntf_bucket_index, bucket_index);
        }
    }

    #[tokio::test]
    async fn ping() {
        let (local_node, remote_nodes) = mk_nodes(2).await;
        let mut local_table = local_node.table.lock().unwrap();
        let remote_table = remote_nodes[0].table.lock().unwrap();
        local_table.add(remote_table.id, remote_table.record);

        drop(local_table);

        let remote_id = remote_nodes[0].node_id;
        let local_node_copy = local_node.clone();
        let remote_node_copy = remote_nodes[0].clone();

        tokio::spawn(async move {
            let mut buffer1 = [0u8; 1024];
            println!("Starting remote server");
            remote_node_copy.start_server(buffer1).await;
        });

        tokio::spawn(async move {
            let mut buffer2 = [0u8; 1024];
            println!("Starting local server");
            local_node_copy.start_server(buffer2).await;
        });

        let result = local_node.ping(&remote_id).await;
        tokio::time::sleep(Duration::from_secs(1)).await;
        // TODO:  Create assertion logic (utilizing a transcript?)
    }
}
