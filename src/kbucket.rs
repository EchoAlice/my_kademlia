#![allow(unused)]

use std::string::String;
use uint::*;
use crate::helper::{Identifier, 
    Node,
    U256,
};

const BUCKET_SIZE: usize = 20;
const MAX_BUCKETS: usize = 256;

type Bucket = [Option<Node>; BUCKET_SIZE];

pub enum StoreValue {
    Node(Node),
    Sample(String),  // Define a sample, and change the type to a sample
}

/*
    Implementation details:
        - Each k-bucket is kept sorted by time last seen.  Least recently seen -> Most recently seen node
*/


// Bucket 0: Farthest peers from node in network 
// Bucket 255: Closest peers from node in network
#[derive(Debug)]
pub struct KbucketTable {
    pub local_node_id: Identifier,
    pub buckets: [Bucket; MAX_BUCKETS],
    store: std::collections::HashMap<Vec<u8>, Vec<u8>>,   // Same storage as portal network.

}

impl KbucketTable {
    pub fn new(local_node_id: Identifier) -> Self {
       let empty_bucket: [Option<Node>; BUCKET_SIZE] = [None; BUCKET_SIZE];
        
        Self {
            local_node_id: local_node_id,
            buckets: [empty_bucket; MAX_BUCKETS],
            store: std::collections::HashMap::new(),
        }
    }
    pub fn store(&mut self, key: Identifier, value: StoreValue) {
        match value {
            StoreValue::Node(value) => {
                println!("Store a node");
                self.add_node(value);
            }
            StoreValue::Sample(value) => {
                println!("Store a value");
                self.add_store();
            }
        }
    }
    pub fn find_node(&mut self, y: Node) -> Option<Node> {
        let bucket_index = self.find_bucket(y.node_id);
        let mut bucket = self.buckets[bucket_index];
        let result = self.search_bucket(bucket, y);

        match result.0 {
            true => {
                println!("Node[bucket_index]: {:?}", bucket[result.1]);
                let found_node = bucket[result.1];
                return found_node
            }
            false => {
                println!("Node is not stored");
                return None
            }
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
        let bucket_index = self.find_bucket(y.node_id);
        let mut bucket = self.buckets[bucket_index];
        let result = self.search_bucket(bucket, y);

        match result.0 {
            // Node was already stored
            true => {
                println!("Node was already stored");
                return 
            }
            // Node wasn't already stored
            false => {
                bucket[result.1] = Some(y);
                self.buckets[bucket_index] = bucket;
                println!("Node is now stored in routing table");
                return
            }
        }
    }

    // TODO:
    fn add_store(&self) {
    }

    fn find_bucket(&self, identifier: Identifier) -> usize {
        let x = U256::from(self.local_node_id);
        let y = U256::from(identifier);
        let xor_distance = x^y;
        
        let bucket_index = ((xor_distance.leading_zeros() - 1) as usize);
        println!("Xor distance leading zeros, {}", xor_distance.leading_zeros());
        println!("Bucket index for given key: {}", bucket_index);
        bucket_index
    }

    // How can i make this return value less confusing?
    fn search_bucket(&self, bucket: Bucket, node: Node) -> (bool, usize) {
        let mut last_empty_index = 0;
        for i in 0..BUCKET_SIZE { 
            match bucket[i] {
                Some(bucket_node) => {
                    // If node was already in bucket -->  return (it's index, true).
                    if bucket_node == node {
                        return (true, i)
                    }
                    else {continue};
                }
                None => {
                    last_empty_index = i;
                }
            }
        }
        // If node wasn't already in bucket -->  return (largest available index, false)
        println!("Last empty index: {}", last_empty_index);
        return (false, last_empty_index)
    }
}
