#![allow(unused)]

use crate::helper::{Identifier, Node, U256};
use std::collections::HashMap;
use uint::*;

const BUCKET_SIZE: usize = 20;
const MAX_BUCKETS: usize = 256;

type Bucket = [Option<Node>; BUCKET_SIZE];

pub enum StoreValue {
    Node(Node),
    Sample(String), // Define a sample, and change the type to a sample
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
    pub fn store(&mut self, key: Identifier, value: StoreValue) {
        match value {
            StoreValue::Node(value) => {
                println!("Store a node");
                self.add_node(value);
            }
            // TODO:
            StoreValue::Sample(value) => {
                println!("Store a value");
                self.add_store();
            }
        }
    }

    // Fix this function:
    // Should take an Identifier as an arguement and return node info for K closest noodes:  Option<Vec<Node>>
    pub fn find_node(&mut self, y: Node) -> Option<Node> {
        let bucket_index = self.find_bucket(y.node_id);
        let mut bucket = self.buckets[bucket_index];
        let result = self.search_bucket(bucket, y);

        if result.0 {
            println!("Node[bucket_index]: {:?}", bucket[result.1]);
            bucket[result.1]
        } else {
            println!("Node is not stored");
            None
        }
    }
    // TODO:
    pub fn find_value() {}
    pub fn ping() {}

    // Don't expose functions from here down.
    // ---------------------------------------------------------------------------------------------------

    //  Add our node to the bucket if it's not already there.
    pub fn add_node(&mut self, y: Node) {
        // TODO: Replace these 3 lines w/ find_node().  Kind of complex to do... Maybe later

        // Get rid of these lines
        let bucket_index = self.find_bucket(y.node_id);
        let mut bucket = self.buckets[bucket_index];
        let result = self.search_bucket(bucket, y);

        if result.0 {
            // Node was already stored
            println!("Node was already stored");
        } else {
            // Node wasn't already stored
            bucket[result.1] = Some(y);
            self.buckets[bucket_index] = bucket;
            println!("Node is now stored in routing table");
        }
    }

    // TODO:
    fn add_store(&self) {}

    // find_bucket_index
    fn find_bucket(&self, identifier: Identifier) -> usize {
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

    // How can i make this return value less confusing?
    // Make it clearer l*r
    // Checks to see if node is present in bucket.  If not, return last index
    fn search_bucket(&self, bucket: Bucket, node: Node) -> (bool, usize) {
        let mut last_empty_index = 0;
        for i in 0..BUCKET_SIZE {
            match bucket[i] {
                Some(bucket_node) => {
                    // If node was already in bucket  -->  return (it's true, index).
                    if bucket_node == node {
                        return (true, i);
                    } else {
                        continue;
                    };
                }
                None => {
                    last_empty_index = i;
                }
            }
        }
        // If node wasn't already in bucket -->  return (largest available false, last_empty_index)
        println!("Last empty index: {}", last_empty_index);
        return (false, last_empty_index);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
}
