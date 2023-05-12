#![allow(unused)]

use std::string::String;
use uint::*;

type Bucket = [Option<Node>; BUCKET_SIZE];



// Move these lines to helper.rs
// -------------------------------------------
const BUCKET_SIZE: usize = 20;
const MAX_BUCKETS: usize = 256;

type Identifier = [u8; 32];
construct_uint! {
    /// 256-bit unsigned integer (little endian).
    pub struct U256(4);
}
#[derive(Clone, Copy, Debug)]
pub struct Node {
    pub ip_address: &'static str,
    pub udp_port: &'static str,
    pub node_id: Identifier,
}


// Where our node keeps up with peers in the network.
// Bucket 0: Farthest peers --> Bucket 255: Closest peers
#[derive(Debug)]
pub struct KbucketTable {
    pub local_node_id: Identifier,
    pub buckets: [Bucket; MAX_BUCKETS],

}

impl KbucketTable {
    pub fn new(local_node_id: Identifier) -> Self {
       let empty_bucket: [Option<Node>; BUCKET_SIZE] = [None; BUCKET_SIZE];
        
        Self {
            local_node_id: local_node_id,
            buckets: [empty_bucket; MAX_BUCKETS],
        }
    }

    // Could add/remove a node OR a sample! 
    pub fn add(&self, y: Node) {
        let i = self.find_bucket(y.node_id);
        println!("Bucket index for given key: {}", i);
        // place_node();
    }
    pub fn remove(&self, y: Node) {
        let i = self.find_bucket(y.node_id);
        // place_node();
    }

    pub fn find_bucket(&self, identifier: Identifier) -> u32 {
        let x = U256::from(self.local_node_id);
        let y = U256::from(identifier);
        let distance = x^y;
        distance.leading_zeros() - 1
    }
}

// TODO:
fn search_bucket(bucket: Bucket, key: Identifier) {

}
