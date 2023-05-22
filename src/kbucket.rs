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
    /// the k closest nodes to some given node ID"
    ///     - Kademlia Paper
    ///
    /// Recieves an id request and returns node information on nodes within
    /// *its closest bucket* (instead of k-closest nodes) to that id.
    pub fn find_node(&mut self, id: Identifier) -> FindNodeResult {
        let result = self.search_table(id);
        let mut bucket = self.buckets[result.bucket_index];

        if result.found {
            // Returns Node
            FindNodeResult::Found(bucket[result.column_index])
        } else {
            let mut known_nodes = Vec::new();

            for node in bucket.iter() {
                if node.is_some() {
                    // Should I be dereferencing the node to send to others?  Or copy the node to share?
                    known_nodes.push(*node)
                }
            }
            // Returns nodes within local node's closest bucket to queried node
            FindNodeResult::NotFound(known_nodes)
        }
    }
    // TODO:
    pub fn find_value() {}

    // TODO:
    /// Instructs a node to store a key, value pair for later retrieval. "Most operations are implemented
    /// in terms of the lookup proceedure. To store a <key,value> pair, a participant locates the k closes
    /// nodes to the key and sends them store RPCs".
    pub fn store(&mut self, key: Identifier, value: Vec<u8>) {}

    // Non-RPCs:
    // ---------------------------------------------------------------------------------------------------
    pub fn add_node(&mut self, node: Node) {
        let result = self.search_table(node.node_id);
        let mut bucket = self.buckets[result.bucket_index];

        if !result.found {
            println!("Node is added");
            bucket[result.column_index] = Some(node)
        } else {
            println!("Node is already in our table")
        }
    }

    // Searches table for node specified.
    fn search_table(&self, id: Identifier) -> SearchResult {
        let mut last_empty_index = 0;
        let bucket_index = self.find_bucket_index(id);
        let mut bucket = self.buckets[bucket_index];

        for (i, node) in bucket.iter().enumerate() {
            match node {
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
                _ => {
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

/// TODO:  Implement real deal tests!
///
/// Test find_node()      **Requires adding nodes to our table**
/// Test search_table()
/// Test add_node()
#[cfg(test)]
mod tests {
    use super::*;
    use crate::helper::{create_dummy_nodes, create_test_table};

    #[test]
    fn add_node() {
        let dummy_nodes = create_dummy_nodes();
        let mut table = create_test_table(dummy_nodes.clone());
        println!("Empty Table {:?}", table);
        println!("\n");
        table.add_node(dummy_nodes[1]);
        println!("Node added to table {:?}", table);
    }

    #[test]
    fn test_search_table() {
        let dummy_nodes = create_dummy_nodes();
        let mut table = create_test_table(dummy_nodes.clone());
    }

    #[test]
    fn find_node() {
        let dummy_nodes = create_dummy_nodes();
        let table = create_test_table(dummy_nodes.clone());
    }
}
