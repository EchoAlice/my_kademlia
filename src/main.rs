#![allow(unused)]

use sha2::{Digest, Sha256};
use crate::{kbucket::KbucketTable,
            helper::Node,
};


pub mod kbucket;
pub mod helper;


fn main() {
    // Routing Table Logic
    let local_node = Node {
        ip_address: "random",
        udp_port: "words",
        node_id: [0 as u8; 32],
    };
    let node_to_add = Node {
        ip_address: "more",
        udp_port: "words",
        node_id: [2 as u8; 32],
    };
    
    let local_nodes_rt = KbucketTable::new(local_node.node_id);

    // Testing XOR Logic for now
    let result = local_nodes_rt.add_node(node_to_add);
    println!("{:?}", result);
}
