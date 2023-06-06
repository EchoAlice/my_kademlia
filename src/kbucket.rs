#![allow(unused)]

use crate::helper::{Identifier, U256};
use crate::node::{FindNodeResult, Node, Search, TableRecord};
use std::io;
use std::net::{Ipv4Addr, SocketAddrV4};
use std::sync::Arc;
use tokio::net::UdpSocket;
use tokio::runtime::Runtime;
use tokio::sync::mpsc;
use uint::*;

const BUCKET_SIZE: usize = 20;
const MAX_BUCKETS: usize = 256;

// TODO:
type Bucket = [Option<TableRecord>; BUCKET_SIZE];
/*
// MOVED
#[derive(Debug)]
pub enum FindNodeResult {
    Found(Option<TableRecord>),
    NotFound(Vec<Option<TableRecord>>),
}
// MOVED
#[derive(Debug)]
enum Search {
    Success(usize, usize),
    Failure(usize, usize),
}
*/

// Bucket 0: Closest peers from node in network.
// Bucket 255: Farthest peers from node in network
#[derive(Debug, PartialEq)]
pub struct KbucketTable {
    pub local_node_id: Identifier,
    pub buckets: [Bucket; MAX_BUCKETS],
}

impl KbucketTable {
    pub fn new(local_node_id: Identifier) -> Self {
        Self {
            local_node_id,
            buckets: [Default::default(); MAX_BUCKETS],
        }
    }

    fn add_node(&mut self, record: &TableRecord) -> bool {
        match self.search_table(record.node_id) {
            Search::Success(bucket_index, column_index) => false,
            Search::Failure(bucket_index, column_index) => {
                self.buckets[bucket_index][column_index] = Some(*record);
                true
            }
        }
    }

    pub fn search_table(&self, id: Identifier) -> Search {
        let mut last_empty_index = 0;
        let bucket_index = self.xor_bucket_index(id);
        let mut bucket = self.buckets[bucket_index];

        for (i, node) in bucket.iter().enumerate() {
            match node {
                Some(bucket_node) => {
                    if bucket_node.node_id == id {
                        return Search::Success(bucket_index, i);
                    } else {
                        continue;
                    };
                }
                _ => {
                    last_empty_index = i;
                }
            }
        }
        Search::Failure(bucket_index, last_empty_index)
    }

    fn xor_bucket_index(&self, identifier: Identifier) -> usize {
        let x = U256::from(self.local_node_id);
        let y = U256::from(identifier);
        let xor_distance = x ^ y;

        MAX_BUCKETS - (xor_distance.leading_zeros() as usize)
    }
}

#[cfg(test)]
mod tests {
    use crate::{helper::PING_MESSAGE_SIZE, node};

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
