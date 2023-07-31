use crate::helper::{Identifier, U256};
use crate::node::Peer;
use crate::socket::SocketAddr;
use std::collections::HashMap;

const BUCKET_SIZE: usize = 20; // "k"
const MAX_BUCKETS: usize = 256;

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Bucket {
    pub map: HashMap<Identifier, SocketAddr>,
    pub limit: usize,
}

impl Bucket {
    fn add(&mut self, peer: Peer) -> Option<SocketAddr> {
        // let
        if self.map.len() <= BUCKET_SIZE {
            self.map.insert(peer.id, peer.socket_addr)
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

    pub fn get(&self, id: &Identifier) -> Option<&SocketAddr> {
        let bucket_index = self.xor_bucket_index(id);
        let bucket = &self.buckets[bucket_index];
        bucket.map.get(id)
    }

    // TODO: Figure out this algorithm.
    pub fn get_closest_node(&self, id: &Identifier) -> Option<Peer> {
        println!("Getting closest node");
        if let Some(socket_addr) = self.get(id) {
            return Some(Peer {
                id: *id,
                socket_addr: *socket_addr,
            });
        }
        let bucket_index = self.xor_bucket_index(id);

        for bucket in self.buckets.iter().skip(bucket_index) {
            if !bucket.map.is_empty() {
                let k = bucket.map.keys().next().unwrap();
                let (k, v) = bucket.map.get_key_value(k).unwrap();
                return Some(Peer {
                    id: *k,
                    socket_addr: *v,
                });
            }
        }
        // Loops around table.  Not great...  Should I be oscilating between bucket[i+1] and bucket[i-1]?
        for i in (0..bucket_index).rev() {
            if !self.buckets[i].map.is_empty() {
                let k = self.buckets[i].map.keys().next().unwrap();
                let (k, v) = self.buckets[i].map.get_key_value(k).unwrap();
                return Some(Peer {
                    id: *k,
                    // socket_addr: *v,
                    socket_addr: *v,
                });
            }
        }
        None
    }

    // TODO: Create complete routing table logic (return K closest nodes instead of indexed bucket)
    // pub fn get_closest_nodes() {}

    pub fn get_bucket_for(&self, id: &Identifier) -> Option<&HashMap<[u8; 32], SocketAddr>> {
        let bucket_index = self.xor_bucket_index(id);
        if self.buckets[bucket_index].map.is_empty() {
            println!("BUCKET IS EMPTY");
            return None;
        }
        println!("BUCKET ISN'T EMPTY");
        Some(&self.buckets[bucket_index].map)
    }

    // TODO: Move to helper.rs  xor(id1, id2)
    pub fn xor_bucket_index(&self, id: &Identifier) -> usize {
        let x = U256::from(self.peer.id);
        let y = U256::from(id);
        let xor_distance = x ^ y;

        MAX_BUCKETS - (xor_distance.leading_zeros() as usize)
    }
}

// TODO: Create tests for bucket logic
