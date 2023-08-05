use crate::helper::{xor_bucket_index, Identifier};
use crate::node::Peer;
use crate::socket::SocketAddr;
use std::collections::HashMap;

//  K == Max bucket size
//  Typically 20.  Only 5 for testing
const K: usize = 5;
pub const MAX_BUCKETS: usize = 256;

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Bucket {
    pub map: HashMap<Identifier, SocketAddr>,
    pub limit: usize,
}

impl Bucket {
    fn add(&mut self, peer: Peer) -> Option<SocketAddr> {
        if self.map.len() <= K {
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
    pub id: Identifier,
    pub buckets: Vec<Bucket>,
}

impl KbucketTable {
    pub fn new(id: Identifier) -> Self {
        Self {
            id,
            buckets: vec![Default::default(); MAX_BUCKETS],
        }
    }

    pub fn add(&mut self, peer: Peer) -> bool {
        let bucket_index = xor_bucket_index(&self.id, &peer.id);
        match self.buckets[bucket_index].add(peer).is_none() {
            true => true,
            false => false,
        }
    }

    pub fn get(&self, id: &Identifier) -> Option<Peer> {
        let bucket_index = xor_bucket_index(&self.id, &id);
        let bucket = &self.buckets[bucket_index];
        if let Some(socket_addr) = bucket.map.get(id) {
            return Some(Peer {
                id: *id,
                socket_addr: *socket_addr,
            });
        } else {
            None
        }
    }

    pub fn get_closest_nodes(&self, id: &Identifier) -> Option<Vec<Peer>> {
        // Diff in cursors keep the index from repeating in first iteration of function
        let mut l_cursor: i32 = 1;
        let mut r_cursor: i32 = 0;

        //  Utilize left and right cursors.  Think of this as left and right of a number line:
        //      0, 1, 2, ... target, ... 254, 255
        let mut closest_peers: Vec<Peer> = Vec::new();
        let target_index = xor_bucket_index(&self.id, &id) as i32;
        let mut current_index = target_index;

        for _ in 0..256 {
            if target_index + r_cursor < 256 {
                current_index = target_index + r_cursor;
                if let Some(peers) = self.bucket_peers(current_index) {
                    for peer in peers {
                        closest_peers.push(peer);
                    }
                }
                r_cursor += 1;
            }
            if target_index - l_cursor >= 0 {
                current_index = target_index - l_cursor;
                if let Some(peers) = self.bucket_peers(current_index) {
                    for peer in peers {
                        closest_peers.push(peer);
                    }
                }
                l_cursor += 1;
            }
        }
        if closest_peers.is_empty() {
            return None;
        }
        Some(closest_peers)
    }

    fn bucket_peers(&self, i: i32) -> Option<Vec<Peer>> {
        let bucket = &self.buckets[i as usize];
        let mut bucket_peers = Vec::new();

        // Cycle through bucket.
        for (k, v) in bucket.map.iter() {
            let peer = Peer {
                id: *k,
                socket_addr: *v,
            };
            bucket_peers.push(peer);
        }
        if bucket_peers.is_empty() {
            return None;
        }
        Some(bucket_peers)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::helper::{xor_bucket_index, U256};
    use crate::node::Node;
    use crate::socket;
    use std::net::{IpAddr, SocketAddr};

    #[test]
    fn get_closest_nodes() {
        let local = Node::new(
            U256::from(0).into(),
            SocketAddr::new("127.0.0.1".parse::<IpAddr>().unwrap(), 6000),
        );
        let node_to_find = Node::new(
            U256::from(13).into(),
            SocketAddr::new("127.0.0.1".parse::<IpAddr>().unwrap(), 6013),
        );
        let mut table = KbucketTable::new(local.id);
        let mut peers_added = Vec::new();
        let mut expected_nodes: Vec<Peer> = Vec::new();

        for i in 2..30 {
            if i == 13 {
                continue;
            }
            let port = "600".to_string() + &i.to_string();
            let peer = Peer {
                id: U256::from(i).into(),
                socket_addr: socket::SocketAddr {
                    addr: SocketAddr::new(
                        "127.0.0.1".parse::<IpAddr>().unwrap(),
                        port.parse::<u16>().unwrap(),
                    ),
                },
            };
            table.add(peer);
            peers_added.push(peer);

            // How i derive expected nodes
            // let distance = xor_bucket_index(&node_to_find.id, &peer.id);
            // println!("Node: {:?}, Distance: {:?} ", peer.id[31], distance);
        }

        // expected_nodes.extend_from_slice(&peers_added[..K]);

        let closest_nodes = table.get_closest_nodes(&node_to_find.id).unwrap();
        println!("Test Closest nodes: {:?}", closest_nodes);
    }
}
