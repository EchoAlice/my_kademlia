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

    fn insert(&mut self, key: Identifier, value: TableRecord) -> Option<TableRecord> {
        if self.map.len() <= BUCKET_SIZE {
            self.map.insert(key, value)
        } else {
            None
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TableRecord {
    pub ip_address: Ipv4Addr,
    pub udp_port: u16,
}

// Bucket 0: Closest peers from node in network.
// Bucket 255: Farthest peers from node in network
#[derive(Debug, PartialEq)]
pub struct KbucketTable {
    pub id: Identifier,
    pub record: TableRecord,
    pub buckets: Vec<Bucket>,
}

impl KbucketTable {
    pub fn new(id: Identifier, record: TableRecord) -> Self {
        Self {
            id,
            record,
            buckets: vec![Default::default(); MAX_BUCKETS],
        }
    }

    // TODO: Remove bool return statement (used in tests rn)
    pub fn add(&mut self, id: Identifier, record: TableRecord) -> bool {
        let bucket_index = self.xor_bucket_index(&id);
        match self.buckets[bucket_index].insert(id, record) {
            Some(_) => false,
            None => true,
        }
    }

    pub fn get(&self, id: &Identifier) -> Option<&TableRecord> {
        let bucket_index = self.xor_bucket_index(&id);
        let mut bucket = &self.buckets[bucket_index];
        bucket.map.get(id)
    }

    pub fn get_bucket_for(&self, id: &Identifier) -> HashMap<[u8; 32], TableRecord> {
        let bucket_index = self.xor_bucket_index(id);
        self.buckets[bucket_index].map.clone()
    }

    pub fn xor_bucket_index(&self, id: &Identifier) -> usize {
        let x = U256::from(self.id);
        let y = U256::from(id);
        let xor_distance = x ^ y;

        MAX_BUCKETS - (xor_distance.leading_zeros() as usize)
    }
}
