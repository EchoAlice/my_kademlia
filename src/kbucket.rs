use crate::helper::{Identifier, U256};
use std::collections::HashMap;
use std::net::Ipv4Addr;

const BUCKET_SIZE: usize = 20;
const MAX_BUCKETS: usize = 256;

// Should I create a HashMap with a generic K, V or have my hashmap take in an id and TR?
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Bucket {
    pub map: HashMap<Identifier, TableRecord>,
    pub limit: usize,
}

impl Bucket {
    fn new(&self, limit: usize) -> Self {
        Bucket {
            map: HashMap::new(),
            limit,
        }
    }

    fn add(&mut self, key: Identifier, value: TableRecord) -> Option<TableRecord> {
        if self.map.len() <= BUCKET_SIZE {
            self.map.insert(key, value)
        } else {
            println!("TODO: Implement record replacement logic!");
            None
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TableRecord {
    pub node_id: Identifier,
    pub ip_address: Ipv4Addr,
    pub udp_port: u16,
}

// Bucket 0: Closest peers from node in network.
// Bucket 255: Farthest peers from node in network
#[derive(Debug, PartialEq)]
pub struct KbucketTable {
    pub local_node_id: Identifier,
    pub local_record: TableRecord,
    pub buckets: Vec<Bucket>,
}

impl KbucketTable {
    pub fn new(local_node_id: Identifier, local_record: TableRecord) -> Self {
        Self {
            local_node_id,
            local_record,
            buckets: vec![Default::default(); MAX_BUCKETS],
        }
    }

    // TODO: Remove bool return statement (used in tests rn)
    pub fn add_node(&mut self, record: TableRecord) -> bool {
        let bucket_index = self.xor_bucket_index(&record.node_id);
        self.buckets[bucket_index].add(record.node_id, record);
        true
    }

    pub fn search(&self, id: &Identifier) -> Option<TableRecord> {
        let mut last_empty_index = 0;
        let bucket_index = self.xor_bucket_index(&id);
        let mut bucket = self.buckets[bucket_index].clone();

        for (i, node) in bucket.map.iter().enumerate() {
            if node.0 == id {
                let record = node.1.clone();
                return Some(record);
            }
        }
        None
    }

    pub fn xor_bucket_index(&self, identifier: &Identifier) -> usize {
        let x = U256::from(self.local_node_id);
        let y = U256::from(identifier);
        let xor_distance = x ^ y;

        MAX_BUCKETS - (xor_distance.leading_zeros() as usize)
    }
}
