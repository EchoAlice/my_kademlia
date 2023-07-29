pub mod helper;
pub mod kbucket;
pub mod message;
pub mod node;
pub mod service;
pub mod socket;

fn main() {
    // Bootstrapping protocol -
    // "To join the network, a node u ust have a contact (bootstrap node) to an already
    // participating node w. u inserts w into the appropriate k-bucket. u then performs
    // a node lookup for its own node ID.  Finally, u refreshes all k-buckets further away
    // than its closest neighbor."
    //
    // TODO:  Implement bootstrapping
    //
    println!("Let's build this thing");
}

/*

impl Node {
    fn ping(id: Identifier) -> Future<bool>;
    fn find_nodes(id: Identifier) -> Future<&[Peer]>;
    fn get_value(key: Key) -> Future<Value>;
}

fn main() {

    ... get the args -> ping to id

    let mut node = node::new();

    node.start(local_ip, port, local_id); // tokio

    if node.ping(id).await {
        println!("node responded!");
    } else {
        println!("node didn't respond :(")
    }
}

 */
