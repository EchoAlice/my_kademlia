use crate::helper::{Identifier, U256};
use crate::node::Peer;
use std::collections::HashMap;
use std::net::IpAddr;

const BUCKET_SIZE: usize = 20; // "k"
const MAX_BUCKETS: usize = 256;

// TODO: Remove struct
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TableRecord {
    pub ip_address: IpAddr,
    pub udp_port: u16,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Bucket {
    pub map: HashMap<Identifier, TableRecord>,
    pub limit: usize,
}

impl Bucket {
    fn new(limit: usize) -> Self {
        Bucket {
            map: HashMap::new(),
            limit,
        }
    }

    fn add(&mut self, peer: Peer) -> Option<TableRecord> {
        if self.map.len() <= BUCKET_SIZE {
            self.map.insert(peer.id, peer.record)
        } else {
            None
        }
    }
}

// Bucket 0: Closest peers to node in network.
// Bucket 255: Farthest peers from node in network
#[derive(Clone, Debug, PartialEq)]
pub struct KbucketTable {
    pub peer: Peer,
    pub buckets: Vec<Bucket>,
}

impl KbucketTable {
    pub fn new(peer: Peer) -> Self {
        Self {
            peer,
            buckets: vec![Default::default(); MAX_BUCKETS],
        }
    }

    pub fn add(&mut self, peer: Peer) -> bool {
        let bucket_index = self.xor_bucket_index(&peer.id);

        match self.buckets[bucket_index].add(peer).is_none() {
            true => true,
            false => false,
        }
    }

    pub fn get(&self, id: &Identifier) -> Option<&TableRecord> {
        let bucket_index = self.xor_bucket_index(id);
        let mut bucket = &self.buckets[bucket_index];
        bucket.map.get(id)
    }

    pub fn get_bucket_for(&self, id: &Identifier) -> Option<&HashMap<[u8; 32], TableRecord>> {
        let bucket_index = self.xor_bucket_index(id);
        if self.buckets[bucket_index].map.is_empty() {
            println!("BUCKET IS EMPTY");
            return None;
        }
        println!("BUCKET ISN't EMPTY");
        Some(&self.buckets[bucket_index].map)
    }

    // TODO: pub fn get_closest_nodes(id)

    pub fn xor_bucket_index(&self, id: &Identifier) -> usize {
        let x = U256::from(self.peer.id);
        let y = U256::from(id);
        let xor_distance = x ^ y;

        MAX_BUCKETS - (xor_distance.leading_zeros() as usize)
    }
}
