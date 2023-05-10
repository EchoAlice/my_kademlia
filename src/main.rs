#![allow(unused)]

use sha2::Sha256;
use crate::kbucket::{KbucketTable, Node};


pub mod kbucket;

fn main() {
    // let x = Sha256(4844);
    // let y = Sha256(4444);
    // let result = kbucket::xor_distance(x, y);
    // println!("{:?}", result);

    let local_node = Node {
        ip_address: "random",
        udp_port: "words",
        node_id: [0 as u8; 32],
    };

    let routing_table = KbucketTable::new(local_node.node_id);
    println!("routing table:  {:?}", routing_table);
    
    /*
    In the end, expose these functionalities:
        ping()
        store()
        find_node()
        find_value()
     */
}
