#![allow(unused)]

use crate::helper::{Identifier, Node, U256};
use std::collections::HashMap;
use std::net::{Ipv4Addr, SocketAddrV4};
use tokio::runtime::Runtime;
use tokio::sync::mpsc;
use uint::*;

const BUCKET_SIZE: usize = 20;
const MAX_BUCKETS: usize = 256;

type Bucket = [Option<Node>; BUCKET_SIZE];

#[derive(Debug)]
pub enum FindNodeResult {
    Found(Option<Node>),
    NotFound(Vec<Option<Node>>),
}

#[derive(Debug)]
enum Search {
    Success(usize, usize),
    Failure(usize, usize),
}

// Bucket 0: Closest peers from node in network.
// Bucket 255: Farthest peers from node in network
#[derive(Debug)]
pub struct KbucketTable {
    pub local_node_id: Identifier,
    pub buckets: [Bucket; MAX_BUCKETS],
    store: HashMap<Vec<u8>, Vec<u8>>, // Same storage as portal network.
}

impl KbucketTable {
    pub fn new(local_node_id: Identifier) -> Self {
        Self {
            local_node_id,
            buckets: [Default::default(); MAX_BUCKETS],
            store: Default::default(),
        }
    }

    // Protocol's RPCs:
    // ---------------------------------------------------------------------------------------------------
    //
    /// Follow specs from Discv5.2:  https://github.com/ethereum/devp2p/blob/discv5-v5.2/discv5/discv5-wire.md.
    ///
    pub fn ping(&mut self, node: &Node, message_packet: String) {}

    /// "The most important procedure a Kademlia participant must perform is to locate
    /// the k closest nodes to some given node ID"
    ///     - Kademlia Paper
    ///
    /// Recieves an id request and returns node information on nodes within
    /// *its closest bucket* (instead of k-closest nodes) to that id.
    pub fn find_node(&mut self, id: Identifier) -> FindNodeResult {
        match self.search_table(id) {
            Search::Success(bucket_index, column_index) => {
                let bucket = self.buckets[bucket_index];
                FindNodeResult::Found(bucket[column_index])
            }
            Search::Failure(bucket_index, column_index) => {
                let bucket = self.buckets[bucket_index];
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
    // TODO:
    pub fn find_value() {}

    // TODO:
    /// Instructs a node to store a key, value pair for later retrieval.
    ///
    /// "Most operations are implemented in terms of the lookup proceedure. To store a
    /// <key,value> pair, a participant locates the k closes nodes to the key and sends them store RPCs".
    pub fn store(&mut self, key: Identifier, value: Vec<u8>) {}

    // Non-RPCs:
    // ---------------------------------------------------------------------------------------------------
    fn add_node(&mut self, node: &Node) -> bool {
        match self.search_table(node.node_id) {
            Search::Success(bucket_index, column_index) => false,
            Search::Failure(bucket_index, column_index) => {
                self.buckets[bucket_index][column_index] = Some(*node);
                true
            }
        }
    }

    fn search_table(&self, id: Identifier) -> Search {
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

    pub fn xor_bucket_index(&self, identifier: Identifier) -> usize {
        let x = U256::from(self.local_node_id);
        let y = U256::from(identifier);
        let xor_distance = x ^ y;

        MAX_BUCKETS - (xor_distance.leading_zeros() as usize)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Create Socket addresses for our nodes
    fn mk_nodes(n: u8) -> (Node, Vec<Node>) {
        let listen_addr = String::from("127.0.0.1").parse::<Ipv4Addr>().unwrap();
        let port_start = 9000_u16;

        let our_nodes: Vec<Node> = (0..n)
            .map(|i| mk_node(&listen_addr, port_start, i))
            .collect();

        if let Some((local_node, remote_nodes)) = our_nodes.split_first() {
            let remote_nodes = remote_nodes.to_vec();
            (*local_node, remote_nodes)
        } else {
            unreachable!("Nodes weren't created");
        }
    }

    fn mk_node(listen_addr: &Ipv4Addr, port_start: u16, index: u8) -> Node {
        let mut node_id = [0_u8; 32];
        node_id[31] += index;

        Node {
            node_id,
            ip_address: *listen_addr,
            udp_port: port_start + index as u16,
            socket: SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 1), port_start + index as u16),
        }
    }

    #[test]
    fn add_redundant_node() {
        let (local_node, remote_nodes) = mk_nodes(2);
        let mut table = KbucketTable::new(local_node.node_id);

        let result = table.add_node(&remote_nodes[0]);
        assert!(result);
        let result2 = table.add_node(&remote_nodes[0]);
        assert!(!result2);
    }

    #[test]
    fn find_node_present() {
        let (local_node, remote_nodes) = mk_nodes(5);
        let mut table = KbucketTable::new(local_node.node_id);
        let node_to_find = remote_nodes[1];
        for node in remote_nodes {
            table.add_node(&node);
        }

        match table.find_node(node_to_find.node_id) {
            FindNodeResult::Found(Some(node)) => {
                assert_eq!(node.node_id, node_to_find.node_id)
            }
            _ => unreachable!("Node should have been found"),
        }
    }

    #[test]
    fn find_node_absent() {
        let (local_node, remote_nodes) = mk_nodes(10);
        let absent_index = 4;
        let node_to_find = remote_nodes[absent_index];
        let mut table = KbucketTable::new(local_node.node_id);

        for (i, node) in remote_nodes.iter().enumerate() {
            if i == absent_index {
                continue;
            } else {
                table.add_node(node);
            }
        }

        match table.find_node(node_to_find.node_id) {
            FindNodeResult::NotFound(nodes_returned) => {
                let node_to_find_index = table.xor_bucket_index(node_to_find.node_id);

                for node in nodes_returned {
                    if let Some(node) = node {
                        let node_in_bucket_index = table.xor_bucket_index(node.node_id);
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

    #[test]
    fn run_ping() {
        let (local_node, remote_nodes) = mk_nodes(2);
        let mut table = KbucketTable::new(local_node.node_id);
        let message_packet = String::from("Alice");

        println!("Node's udp socket: {}", local_node.socket);
        table.ping(&local_node, message_packet);
    }
}
