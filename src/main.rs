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
    let first_node_to_add = Node {
        ip_address: "first",
        udp_port: "node",
        node_id: [1 as u8; 32],
    };
    let second_node_to_add = Node {
        ip_address: "second",
        udp_port: "node",
        node_id: [2 as u8; 32],
    };
    let third_node_to_add = Node {
        ip_address: "second",
        udp_port: "node",
        node_id: [3 as u8; 32],
    };
    
    let mut local_nodes_rt = KbucketTable::new(local_node.node_id);

    // TODO:  Implement real deal tests!

    // Testing node is added only once
    let result = local_nodes_rt.add_node(first_node_to_add);
    println!("\n");
    let result = local_nodes_rt.add_node(second_node_to_add);
    println!("\n");
    let result = local_nodes_rt.add_node(second_node_to_add);
    println!("\n");
    let result = local_nodes_rt.add_node(first_node_to_add);
    println!("\n");

    // Testing find_node()
    let result = local_nodes_rt.find_node(first_node_to_add);
    println!("\n");
    let result = local_nodes_rt.find_node(third_node_to_add);
    println!("\n");
    
    // Verify Table at a glance
    println!("Node's routing table: {:?}", local_nodes_rt.buckets);


}
