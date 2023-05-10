#![allow(unused)]

use std::string::String;


const BUCKET_SIZE: usize = 20;
const MAX_BUCKETS: usize = 256;

type Bucket = [Option<Node>; BUCKET_SIZE];
type Identifier = [u8; 32];



#[derive(Clone, Copy, Debug)]
pub struct Node {
    pub ip_address: &'static str,
    pub udp_port: &'static str,
    pub node_id: Identifier,
}

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
    
    fn add(&self, x: Node) {
        let i = xor_distance(x.node_id, self.local_node_id);
        // search_bucket(buckets[i]);
    }

    fn remove(&self, x: Node) {
        let i = xor_distance(x.node_id, self.local_node_id);
        // search_bucket(buckets[i]);
    }

}

// TODO:
pub fn xor_distance(x: Identifier, y: Identifier) -> usize {
    // let result = x^y;

    300
}


fn search_bucket(bucket: Bucket, key: Identifier) {

}