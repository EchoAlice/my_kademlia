use crate::helper::{Identifier, PING_MESSAGE_SIZE};
use crate::kbucket::{Bucket, KbucketTable, TableRecord};
use crate::message::{Message, MessageBody};
use crate::node;

use core::panic;
use rand::Rng;
use std::collections::HashMap;
use std::net::{IpAddr, SocketAddr, SocketAddrV4};
use std::sync::{Arc, Mutex};
use tokio::io;
use tokio::net::UdpSocket;
use tokio::time::Duration;

const NODES_TO_QUERY: usize = 1; // "a"

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Peer {
    pub id: Identifier,
    pub record: TableRecord,
}

#[derive(Clone, Debug)]
pub struct State {
    pub table: KbucketTable,
    pub outbound_requests: HashMap<Identifier, u8>,
}

// The main Kademlia client struct.
// Provides user-level API for performing querie and interacting with the underlying service.
#[derive(Clone, Debug)]
pub struct Node {
    pub id: Identifier,
    pub local_record: Peer,
    pub socket: Arc<UdpSocket>,
    pub messages: Arc<Mutex<Vec<Message>>>,
    pub state: Arc<Mutex<State>>,
}

impl Node {
    pub async fn new(local_record: Peer) -> Self {
        Self {
            id: local_record.id,
            local_record,
            // TODO: Bind socket when you start server
            socket: Arc::new(
                UdpSocket::bind(SocketAddr::new(
                    local_record.record.ip_address,
                    local_record.record.udp_port,
                ))
                .await
                .unwrap(),
            ),
            messages: Default::default(),
            state: Arc::new(Mutex::new(State {
                table: (KbucketTable::new(local_record)),
                outbound_requests: (Default::default()),
            })),
        }
    }

    // TODO: node_lookup(self, id) -> peer {}

    // Protocol's RPCs:
    // ---------------------------------------------------------------------------------------------------
    pub async fn ping(&mut self, id: Identifier, target: &Peer) -> u8 {
        let msg = Message {
            session: rand::thread_rng().gen_range(0..=255),
            body: MessageBody::Ping(self.id),
        };
        self.send_message(&msg, target).await
    }

    /// "The most important procedure a Kademlia participant must perform is to locate the k closest nodes
    /// to some given node ID.  We call this procedure a **node lookup**".
    ///
    /// TODO:  1. Set up networking communication for find_node() **with non-empty bucket**.
    ///        2. Create complete routing table logic (return K closest nodes instead of indexed bucket)
    pub async fn find_node(&mut self, id: &Identifier, target: &Peer) -> u8 {
        let msg = Message {
            session: rand::thread_rng().gen_range(0..=255),
            body: MessageBody::FindNode([self.id, *id]),
        };
        self.send_message(&msg, target).await
    }

    // TODO:
    // pub fn find_value() {}

    // TODO:
    // pub fn store(&mut self, key: Identifier, value: Vec<u8>) {}

    // ---------------------------------------------------------------------------------------------------
    async fn send_message(&self, msg: &Message, target: &Peer) -> u8 {
        let dest = SocketAddr::new(target.record.ip_address, target.record.udp_port);
        self.socket.connect(dest).await;

        // TODO: One pending message per target (for now)
        self.state
            .lock()
            .unwrap()
            .outbound_requests
            .insert(target.id, msg.session);

        self.messages.lock().unwrap().push(msg.clone());
        let message_bytes = msg.to_bytes();

        self.socket.send(&message_bytes).await.unwrap();
        msg.session
    }

    async fn pong(&self, session: u8, target: &Peer) -> u8 {
        let msg = Message {
            session,
            body: MessageBody::Pong(self.id),
        };
        self.send_message(&msg, target).await
    }

    pub async fn start_server(&mut self, mut buffer: [u8; 1024]) {
        loop {
            let Ok((size, sender_addr)) = self.socket.recv_from(&mut buffer).await else { todo!() };
            let requester_id: [u8; 32] = buffer[3..35].try_into().expect("Invalid slice length");

            match &buffer[0..2] {
                b"01" => {
                    let message = Message {
                        session: buffer[2],
                        body: MessageBody::Ping(requester_id),
                    };
                    self.process(message, &sender_addr).await;
                }
                b"02" => {
                    let message = Message {
                        session: buffer[2],
                        body: MessageBody::Pong(requester_id),
                    };
                    self.process(message, &sender_addr).await;
                }
                b"03" => {
                    let message = Message {
                        session: buffer[2],
                        body: MessageBody::FindNode([
                            requester_id,
                            buffer[35..67].try_into().expect("Invalid slice length"),
                        ]),
                    };
                    self.process(message, &sender_addr).await;
                }
                _ => {
                    println!("Message wasn't legitimate");
                }
            }
        }
    }

    async fn process(&mut self, message: Message, sender_addr: &SocketAddr) {
        match message.body {
            MessageBody::Ping(datagram) => {
                self.messages.lock().unwrap().push(message);

                let session_number = datagram[2];
                let node_id = &datagram[3..35];

                self.pong(session_number, sender_addr).await;
            }
            MessageBody::Pong(datagram) => {
                let mut outbound_requests = &mut self.state.lock().unwrap().outbound_requests;
                let mut messages = self.messages.lock().unwrap();
                let node_id = &datagram[3..35];

                if let Some(session_number) = outbound_requests.get(node_id) {
                    if session_number == &datagram[2] {
                        println!("Successful ping. Removing k,v");
                        messages.push(message);
                        outbound_requests.remove(node_id); // Warning: This removes all outbound reqs to an individual node.
                    } else {
                        println!("Unsuccessful ping");
                    }
                } else {
                    println!("No session number for remote node");
                }
            }
            MessageBody::FindNode(datagram) => {
                println!("FindNode datagram: {:?}", datagram)
            }
            _ => println!("Message was not ping, pong, or FindNode"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{helper::PING_MESSAGE_SIZE, node};
    use std::sync::LockResult;

    async fn make_nodes(n: u8) -> (Node, Vec<Node>) {
        let ip_address = String::from("127.0.0.1").parse::<IpAddr>().unwrap();
        let port_start = 9000_u16;

        let local_node = make_node(0).await;
        let mut remote_nodes = Vec::new();

        for i in 1..n {
            remote_nodes.push(make_node(i).await);
        }

        (local_node, remote_nodes)
    }

    async fn make_node(index: u8) -> Node {
        let ip_address = String::from("127.0.0.1").parse::<IpAddr>().unwrap();
        let port_start = 9000_u16;

        let mut id = [0_u8; 32];
        id[31] += index;
        let udp_port = port_start + index as u16;

        let record = TableRecord {
            // ip_address: *ip_address,
            ip_address,
            udp_port,
        };
        let peer = Peer { id, record };

        Node::new(peer).await
    }

    // Run tests independently.  Tests fail when they're run together bc of address reuse.
    // TIP: If you don't give the thing a port, a free port is given automatically
    #[tokio::test]
    async fn add_redundant_node() {
        let (local_node, remote_nodes) = make_nodes(2).await;
        let mut local_table = &mut local_node.state.lock().unwrap().table;
        let remote_table = &remote_nodes[0].state.lock().unwrap().table;

        let result = local_table.add(remote_table.peer);
        let result2 = local_table.add(remote_table.peer);
        assert!(result);
        assert!(!result2);
    }
    /*
        // TODO: Figure out where to place messaging logic
        #[tokio::test]
        async fn get_bucket_for() {
            let (local_node, remote_nodes) = mk_nodes(10).await;

            // Populates local table and gather necessary data from inner scope
            let (node_to_find, ntf_bucket_index, closest_nodes) = {
                let mut local_table = &mut local_node.state.lock().unwrap().table;
                let node_to_find = remote_nodes[1].node_id;
                let ntf_bucket_index = local_table.xor_bucket_index(&node_to_find);
                let closest_nodes = local_table.get_bucket_for(&node_to_find).clone();

                for node in &remote_nodes {
                    let remote_peer = node.state.lock().unwrap().table.peer;
                    local_table.add(remote_peer);
                }
                (node_to_find, ntf_bucket_index, closest_nodes)
            };

            // Verifies that nodes returned from *local* find_node query are correct.
            for node in closest_nodes {
                let bucket_index = local_node
                    .state
                    .lock()
                    .unwrap()
                    .table
                    .xor_bucket_index(&node.0);
                assert_eq!(ntf_bucket_index, bucket_index);
            }
        }
    */
    // TODO: Compress networking testing logic
    #[tokio::test]
    async fn ping() {
        // let (mut local_node, mut remote_nodes) = make_nodes(2).await;
        // let mut local_node_copy = local_node.clone();
        // let mut remote_node_copy = remote_nodes[0].clone();

        // let remote_id = {
        //     let mut local_table = &mut local_node.state.lock().unwrap().table;
        //     let remote_peer = remote_nodes[0].state.lock().unwrap().table.peer;
        //     local_table.add(remote_peer);
        //     remote_peer.id
        // };

        // tokio::spawn(async move {
        //     let mut buffer = [0u8; 1024];
        //     remote_node_copy.start_server(buffer).await;
        // });
        // tokio::spawn(async move {
        //     let mut buffer = [0u8; 1024];
        //     local_node_copy.start_server(buffer).await;
        // });

        // tokio::time::sleep(Duration::from_secs(1)).await;
        // let session_number = local_node.ping(remote_id).await;

        // tokio::time::sleep(Duration::from_secs(1)).await;
        // let local_messages = local_node.messages.lock().unwrap();
        // let remote_messages = remote_nodes[0].messages.lock().unwrap();
        // assert_eq!(local_messages[0], remote_messages[0]);
        // assert_eq!(local_messages[1], remote_messages[1]);

        let mut local = make_node(0).await;
        let mut local_copy = local.clone();
        let mut remote = make_node(1).await;
        let mut remote_copy = remote.clone();

        // add remote peer to local node
        local.state.lock().unwrap().table.add(remote.local_record);

        // start local node
        tokio::spawn(async move {
            let mut buffer = [0u8; 1024];
            local_copy.start_server(buffer).await;
        });
        // start remote
        tokio::spawn(async move {
            let mut buffer = [0u8; 1024];
            remote_copy.start_server(buffer).await;
        });

        // assert!(local.ping(remote.local_record.id).await, true);
        // local.start_server(buffer)
    }

    // TODO: Compress networking testing logic
    #[tokio::test]
    async fn find_node() {
        // let (mut local_node, mut remote_nodes) = make_nodes(4).await;
        // // let requesting_peer = remote_nodes[0];
        // let mut local_node_copy = local_node.clone();
        // let mut remote_node_copy = remote_nodes[0].clone();

        // let requesting_peer = {
        //     let mut local_table = &mut local_node.state.lock().unwrap().table;
        //     let requesting_peer = remote_nodes[0].state.lock().unwrap().table.peer;
        //     let table_peer = remote_nodes[1].state.lock().unwrap().table.peer;
        //     local_table.add(table);

        //     requesting_peer
        // };

        // tokio::spawn(async move {
        //     println!("Starting remote server");
        //     let mut buffer = [0u8; 1024];
        //     remote_node_copy.start_server(buffer).await;
        // });
        // tokio::spawn(async move {
        //     println!("Starting local server");
        //     let mut buffer = [0u8; 1024];
        //     local_node_copy.start_server(buffer).await;
        // });

        // tokio::time::sleep(Duration::from_secs(1)).await;

        // // Resolve errors.  Fix test.
        // let asks_for = remote_nodes[2];
        // let session_number = local_node.find_node(remote_id).await;

        // tokio::time::sleep(Duration::from_secs(1)).await;
        // let local_messages = local_node.messages.lock().unwrap();
        // let remote_messages = remote_nodes[0].messages.lock().unwrap();

        // TODO:  try to make_node() not await
        let mut local = make_node(0).await;
        let mut local_copy = local.clone();
        let mut remote = make_node(1).await;
        let mut remote_copy = remote.clone();

        // add remote peer to local node
        local.state.lock().unwrap().table.add(remote.local_record);

        let node_to_store = make_node(2).await;
        remote
            .state
            .lock()
            .unwrap()
            .table
            .add(node_to_store.local_record);

        // start local node
        tokio::spawn(async move {
            println!("Starting local server");
            let mut buffer = [0u8; 1024];
            local_copy.start_server(buffer).await;
        });
        // start remote
        tokio::spawn(async move {
            println!("Starting remote server");
            let mut buffer = [0u8; 1024];
            local_copy.start_server(buffer).await;
        });

        // assert!(local.find_node(id).await, &vec![id]);
    }
}
