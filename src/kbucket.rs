#![allow(unused)]

use crate::helper::{Identifier, Node, U256};
use std::collections::HashMap;
use std::net::Ipv4Addr;
use uint::*;

const BUCKET_SIZE: usize = 20;
const MAX_BUCKETS: usize = 256;

type Bucket = [Option<Node>; BUCKET_SIZE];

#[derive(Debug)]
pub enum FindNodeResult {
    // I don't think this should be "Option<T>".  Fix later
    Found(Option<Node>),
    NotFound(Vec<Option<Node>>),
}

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
    // TODO:
    // Probes a node to see if it's online
    pub fn ping() {}

    /// "The most important procedure a Kademlia participant must perform is to locate
    /// the k closest nodes to some given node ID"
    ///     - Kademlia Paper
    ///
    /// Recieves an id request and returns node information on nodes within
    /// *its closest bucket* (instead of k-closest nodes) to that id.
    pub fn find_node(&mut self, id: Identifier) -> FindNodeResult {
        let result = self.search_table(id);

        match result {
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
    fn add_node(&mut self, node: Node) {
        let result = self.search_table(node.node_id);

        match result {
            Search::Success(bucket_index, column_index) => {
                println!("Node is already in our table");
            }
            Search::Failure(bucket_index, column_index) => {
                self.buckets[bucket_index][column_index] = Some(node);
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
                        Search::Success(bucket_index, i)
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

        println!("Identifier: {:?}", identifier);
        println!("Leading zeros: {}", xor_distance.leading_zeros());
        println!("\n");
        MAX_BUCKETS - ((xor_distance.leading_zeros() - 1) as usize)
    }
}

// TODO:  Make better assertions
#[cfg(test)]
mod tests {
    use super::*;

    // Add parameter for number of nodes
    pub fn mk_nodes() -> Vec<Node> {
        // Should these nodes have different IP addresses?
        let listen_addr = String::from("127.0.0.1").parse::<Ipv4Addr>().unwrap();
        let port_start = 9000_u16;

        let our_nodes: Vec<Node> = (0..5)
            .into_iter()
            .map(|i| mk_node(&listen_addr, port_start, i))
            .collect();

        our_nodes
    }

    fn mk_node(listen_addr: &Ipv4Addr, port_start: u16, index: usize) -> Node {
        Node {
            ip_address: listen_addr.clone(),
            udp_port: port_start + index as u16,
            node_id: [index as u8; 32],
        }
    }

    #[test]
    fn add_redundant_node() {
        let dummy_nodes = mk_nodes();
        let mut table = KbucketTable::new(dummy_nodes[0].node_id);
        let result = table.add_node(dummy_nodes[1]);
        println!("Updated table: {:?}", table);
        println!("\n");

        let result = table.add_node(dummy_nodes[1]);
    }

    #[test]
    fn search_table() {
        let dummy_nodes = mk_nodes();
        let mut table = KbucketTable::new(dummy_nodes[0].node_id);
        table.add_node(dummy_nodes[1]);

        let result = table.search_table(dummy_nodes[1].node_id);
    }

    // TODO: Create assertion for test.  Get rid of print.
    //       Why does [1, 1... ] print twice?
    #[test]
    fn find_node_present() {
        let dummy_nodes = mk_nodes();
        let mut table = KbucketTable::new(dummy_nodes[0].node_id);

        for i in 1..dummy_nodes.len() {
            table.add_node(dummy_nodes[i]);
        }

        let result = table.find_node(dummy_nodes[1].node_id);
        // Result returns node as expected
        println!("result: {:?}", result);
    }

    // TODO:  Make it more obvious that the correct node(s) are being returned
    #[test]
    fn find_node_absent() {
        let dummy_nodes = mk_nodes();
        let mut table = KbucketTable::new(dummy_nodes[0].node_id);

        for i in 1..dummy_nodes.len() {
            if i == 3 {
                break;
            } else {
                table.add_node(dummy_nodes[i]);
            }
        }

        let result = table.find_node(dummy_nodes[3].node_id);
        // Result returns dummy_nodes[2] (they'd share the same bucket) as expected.
        println!("result: {:?}", result);
    }

    // TODO?:  XOR Test
}
