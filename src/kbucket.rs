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
        let mut closest_nodes = Vec::new();
        let target_index = xor_bucket_index(&self.id, &id) as i32;
        let mut prev_index = -1;
        let mut current_index = target_index;
        let mut bucket = &self.buckets[target_index as usize];
        let mut count = 0;
        let mut radius = 0;

        // TOOD: Implement functionality that removes visiting a bucket twice in a row.
        //
        //  Utilize left and right cursors.  Think of this as left and right of a number line:
        //      0, 1, 2, ... target, ... 254, 255
        // let mut l_cursor: i32 = 0;
        // let mut r_cursor: i32 = 0;

        // TODO: Oscilates around bucket index.  Break IF node 256 buckets have been checked
        while closest_nodes.len() < K && count <= MAX_BUCKETS {
            if !bucket.map.is_empty() {
                // TODO: Grab as many peers from the bucket as possible
                let k = bucket.map.keys().next().unwrap();
                let (k, v) = bucket.map.get_key_value(k).unwrap();

                let peer = Peer {
                    id: *k,
                    socket_addr: *v,
                };
                closest_nodes.push(peer);
            }

            // Increases index
            if count % 2 == 0 {
                radius += 1;
                // Valid
                if target_index + radius <= 255 {
                    current_index = target_index + radius;
                }
            }
            // Decreases index
            if count % 2 == 1 {
                // Valid
                if target_index - radius >= 0 {
                    current_index = target_index - radius;
                }
            }

            println!("Index: {:?}", current_index);
            bucket = &self.buckets[current_index as usize];
            count += 1;
        }
        if closest_nodes.is_empty() {
            return None;
        }
        return Some(closest_nodes);
    }

    // TOOD: Delete this when I've implemented closest_nodes()
    pub fn get_bucket_for(&self, id: &Identifier) -> Option<&HashMap<[u8; 32], SocketAddr>> {
        let bucket_index = xor_bucket_index(&self.id, id);
        if self.buckets[bucket_index].map.is_empty() {
            println!("BUCKET IS EMPTY");
            return None;
        }
        println!("BUCKET ISN'T EMPTY");
        Some(&self.buckets[bucket_index].map)
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
        println!("Closest nodes: {:?}", closest_nodes);
    }
}
