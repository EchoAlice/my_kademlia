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

    pub fn store(&self, key: Identifier, value: StoreValue) {
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

    //  Add our node to the bucket if it's not already there.  Make function private once finished testing
    pub fn add_node(&self, y: Node) {
        let bucket_index = self.find_bucket(y.node_id);
        let mut bucket = self.buckets[bucket_index];
        let result = self.search_bucket(bucket, y);
        if result.1 == true{
            println!("Node was already stored");
            return 
        }
        else {
            bucket[result.0] = Some(y);
            println!("Node is now stored in routing table");
            println!("Bucket: {:?}", bucket); 
            return
        }
    }

    // TODO:
    fn add_store(&self) {
    }

    // Should these functions be outside of impl KbucketTable?
    fn find_bucket(&self, identifier: Identifier) -> usize {
        let x = U256::from(self.local_node_id);
        let y = U256::from(identifier);
        let xor_distance = x^y;
        
        let bucket_index = ((xor_distance.leading_zeros() - 1) as usize);
        println!("Xor distance leading zeros, {}", xor_distance.leading_zeros());
        println!("Bucket index for given key: {}", bucket_index);
        bucket_index
    }

    // TODO:
    fn search_bucket(&self, bucket: Bucket, node: Node) -> (usize, bool) {
        let mut last_empty_index = 0;
        
        // If node was already in bucket -->  return (it's index, true)
        for i in 0..BUCKET_SIZE { 
            // if node.node_id == bucket[i].node_id {
            //     // Node is already in the routing table, return where it's located
            //     return (i, true)
            // }
            // Check if bucket spot is empty
        }
        // If node wasn't already in bucket -->  return (largest available index, false)
        return (last_empty_index, false)
    }
}
