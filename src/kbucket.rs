#![allow(unused)]

use crate::helper::{Identifier, Node, U256};
use std::collections::HashMap;
use uint::*;

const BUCKET_SIZE: usize = 20;
const MAX_BUCKETS: usize = 256;

type Bucket = [Option<Node>; BUCKET_SIZE];

pub enum FindNodeResult {
    // I don't think this should be "Option<T>".  Fix later
    Found(Option<Node>),
    NotFound(Vec<Option<Node>>),
}
// Should this be an enum instead?
pub struct SearchResult {
    pub found: bool,
    pub bucket_index: usize,
    pub column_index: usize,
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
    /// the k closest nodes to some given node ID" - Kademlia Paper
    ///
    /// Recieves an id request and returns node information on nodes within
    /// *its closest bucket* to that id. *Slight modification*
    pub fn find_node(&mut self, id: Identifier) -> FindNodeResult {
        let result = self.search_table(id);
        let mut bucket = self.buckets[result.bucket_index];

        if result.found == true {
            // Returns Node
            FindNodeResult::Found(bucket[result.column_index])
        } else {
            let mut known_nodes = Vec::new();
            for i in 0..BUCKET_SIZE {
                if bucket[i].is_some() {
                    known_nodes.push(bucket[i].clone())
                }
            }
            // Returns nodes within local node's closest bucket
            FindNodeResult::NotFound(known_nodes)
        }
    }
    // TODO:
    pub fn find_value() {}

    /// Instructs a node to store a key, value pair for later retrieval. "Most operations are implemented
    /// in terms of the lookup proceedure. To store a <key,value> pair, a participant locates the k closes
    /// nodes to the key and sends them store RPCs".
    pub fn store(&mut self, key: Identifier, value: Vec<u8>) {}

    // Non-RPCs:
    // ---------------------------------------------------------------------------------------------------
    pub fn add_node(&mut self, node: Node) {}

    // Searches table for node specified.
    fn search_table(&self, id: Identifier) -> SearchResult {
        let mut last_empty_index = 0;
        let bucket_index = self.find_bucket_index(id);
        let mut bucket = self.buckets[bucket_index];

        for i in 0..BUCKET_SIZE {
            match bucket[i] {
                Some(bucket_node) => {
                    if bucket_node.node_id == id {
                        SearchResult {
                            found: true,
                            bucket_index,
                            column_index: i,
                        }
                    } else {
                        continue;
                    };
                }
                None => {
                    last_empty_index = i;
                }
            }
        }
        SearchResult {
            found: false,
            bucket_index,
            column_index: last_empty_index,
        }
    }

    fn find_bucket_index(&self, identifier: Identifier) -> usize {
        let x = U256::from(self.local_node_id);
        let y = U256::from(identifier);
        let xor_distance = x ^ y;

        let bucket_index = MAX_BUCKETS - (xor_distance.leading_zeros() as usize);
        println!(
            "Xor distance leading zeros, {}",
            xor_distance.leading_zeros()
        );
        println!("Bucket index for given key: {}", bucket_index);
        bucket_index
    }
}

#[cfg(test)]
mod tests {
    use super::*;
}
