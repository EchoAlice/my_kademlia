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

    //  Add our node to the bucket if it's not already there.  Make function private once finished testing
    pub fn add_node(&mut self, y: Node) {
        let bucket_index = self.find_bucket(y.node_id);
        let mut bucket = self.buckets[bucket_index];
        let result = self.search_bucket(bucket, y);
        if result.1 == true {
            println!("Node was already stored");
            return 
        }
        else {
            bucket[result.0] = Some(y);
            // DEBUG!  How can I write to my routing table?
            self.buckets[bucket_index] = bucket;
            println!("Node is now stored in routing table");
            return
        }
        println!("Bucket: {:?}", bucket); 
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

    // TODO:  Why isn't my Some arm working?
    fn search_bucket(&self, bucket: Bucket, node: Node) -> (usize, bool) {
        let mut last_empty_index = 0;
        
        for i in 0..BUCKET_SIZE { 
            println!("Bucket index {} is {:?}", i, bucket[i]);
            match bucket[i] {
                Some(bucket_node) => {
                    println!("Node in routing table: {:?}", bucket_node); 
                    // If node was already in bucket -->  return (it's index, true)
                    // return (i, true)
                }
                None => {
                    // If bucket spot is empty, record larger empty index
                    // println!("Spot {} is empty", i);
                    last_empty_index = i;
                }
            }
        }
        // If node wasn't already in bucket -->  return (largest available index, false)
        return (last_empty_index, false)
    }
}
