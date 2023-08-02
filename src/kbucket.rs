use crate::helper::{xor_bucket_index, Identifier};
use crate::node::Peer;
use crate::socket::SocketAddr;
use std::collections::HashMap;

const BUCKET_SIZE: usize = 20; // "k"
pub const MAX_BUCKETS: usize = 256;

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Bucket {
    pub map: HashMap<Identifier, SocketAddr>,
    pub limit: usize,
}

impl Bucket {
    fn add(&mut self, peer: Peer) -> Option<SocketAddr> {
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

    // TODO: pub fn k_closest_nodes() {}
    pub fn get_closest_node(&self, id: &Identifier) -> Option<Peer> {
        let bucket_index = xor_bucket_index(&self.id, &id);

        // Searches table for closest (single) peer
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

        // Loops around table. Should I be oscilating between bucket[i+1] and bucket[i-1]?
        for i in (0..bucket_index).rev() {
            if !self.buckets[i].map.is_empty() {
                let k = self.buckets[i].map.keys().next().unwrap();
                let (k, v) = self.buckets[i].map.get_key_value(k).unwrap();

                return Some(Peer {
                    id: *k,
                    socket_addr: *v,
                });
            }
        }
        println!("Node still wasn't found");
        None
    }

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
    use crate::helper::U256;
    use crate::node::Node;
    use crate::socket;
    use std::net::{IpAddr, SocketAddr};

    #[test]
    fn get_closest_node() {
        let local = Node::new(
            U256::from(0).into(),
            SocketAddr::new("127.0.0.1".parse::<IpAddr>().unwrap(), 6000),
        );
        let mut table = KbucketTable::new(local.id);

        for i in 2..10 {
            if i != 3 {
                let port = "600".to_string() + &i.to_string();
                println!("{:?}", port);

                table.add(Peer {
                    id: U256::from(i).into(),
                    socket_addr: socket::SocketAddr {
                        addr: SocketAddr::new(
                            "127.0.0.1".parse::<IpAddr>().unwrap(),
                            port.parse::<u16>().unwrap(),
                        ),
                    },
                });
            }
        }

        let node_to_find = Node::new(
            U256::from(3).into(),
            SocketAddr::new("127.0.0.1".parse::<IpAddr>().unwrap(), 6003),
        );
        let closest_node = table.get_closest_node(&node_to_find.id).unwrap();
        let expected_node = Peer {
            id: U256::from(2).into(),
            socket_addr: socket::SocketAddr {
                addr: SocketAddr::new("127.0.0.1".parse::<IpAddr>().unwrap(), 6002),
            },
        };

        assert_eq!(closest_node, expected_node);
    }

    #[test]
    fn get_closest_nodes() {
        let local = Node::new(
            U256::from(0).into(),
            SocketAddr::new("127.0.0.1".parse::<IpAddr>().unwrap(), 6000),
        );
        let mut table = KbucketTable::new(local.id);
        let peers_added = Vec::new();

        for i in 2..10 {
            if i != 3 {
                let port = "600".to_string() + &i.to_string();
                println!("{:?}", port);

                table.add(Peer {
                    id: U256::from(i).into(),
                    socket_addr: socket::SocketAddr {
                        addr: SocketAddr::new(
                            "127.0.0.1".parse::<IpAddr>().unwrap(),
                            port.parse::<u16>().unwrap(),
                        ),
                    },
                });
            }
        }

        // Figure out xor distances for each node.
        // Create expected_nodes based on this info!
        let node_to_find = Node::new(
            U256::from(3).into(),
            SocketAddr::new("127.0.0.1".parse::<IpAddr>().unwrap(), 6003),
        );
        for peer in peers_added {
            let distance = xor_bucket_index(&node_to_find.id, &peer);
            println!("Node: {:?}, Distance: {:?} ", peer[31], distance);
        }

        // let _closest_nodes = table.get_closest_nodes(&node_to_find.id).unwrap();
    }
}
