use crate::helper::Identifier;
use crate::kbucket::KbucketTable;
use crate::message::{Message, MessageBody, MessageInner};
use crate::service::Service;

use rand::Rng;
use std::{
    collections::HashMap,
    future::Future,
    net::SocketAddr,
    sync::{Arc, Mutex},
};
use tokio::sync::{mpsc, oneshot};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Peer {
    pub id: Identifier,
    pub socket_addr: SocketAddr,
}

// The main Kademlia client struct.
// Provides user-level API for performing querie and interacting with the underlying service.
#[derive(Debug)]
pub struct Node {
    pub id: Identifier,
    pub local_record: Peer,
    pub service_tx: Option<mpsc::Sender<Message>>,
    pub table: Arc<Mutex<KbucketTable>>,
    pub outbound_requests: HashMap<Identifier, MessageInner>,
}

impl Node {
    pub async fn new(local_record: Peer) -> Self {
        Self {
            id: local_record.id,
            local_record,
            service_tx: None,
            table: Arc::new(Mutex::new(KbucketTable::new(local_record))),
            outbound_requests: (Default::default()),
        }
    }

    /// "The most important procedure a Kademlia participant must perform is to locate the k closest nodes
    /// to some given node ID.  We call this procedure a **node lookup**".
    ///
    // TODO: node_lookup(self, id) -> peer {}

    // Protocol's Exposed functions:
    // ---------------------------------------------------------------------------------------------------

    pub fn ping(&mut self, id: Identifier) -> impl Future<Output = bool> + '_ {
        async move {
            let peer = {
                let table = &self.table.lock().unwrap();
                let target = table.get(&id);
                if target.is_none() {
                    return false;
                }
                let socket_addr = *target.unwrap();
                Peer { id, socket_addr }
            };

            let (tx, rx) = oneshot::channel();

            let msg = Message {
                target: peer,
                inner: MessageInner {
                    session: (rand::thread_rng().gen_range(0..=255)),
                    body: (MessageBody::Ping(self.id, Some(tx))),
                },
            };

            let _ = self.service_tx.as_ref().unwrap().send(msg).await;
            rx.await.unwrap()
        }
    }

    /// TODO:  1. Set up networking communication for find_node() **with non-empty bucket**.
    ///        2. Create complete routing table logic (return K closest nodes instead of indexed bucket)
    // pub fn find_node(&mut self, id: &Identifier) -> impl Future<Output = bool> + '_ {}

    // TODO: Later
    // pub fn find_value() {}

    // TODO: Later
    // pub fn store(&mut self, value: Vec<u8>) {}

    // ---------------------------------------------------------------------------------------------------

    pub async fn start(&mut self) -> Result<(), &'static str> {
        // TODO: if let this thing
        let service_tx = Service::spawn(self.local_record, self.table.clone()).await;
        self.service_tx = Some(service_tx);

        if self.service_tx.is_none() {
            return Err("Service tx wasn't created");
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::IpAddr;
    use tokio::time::Duration;

    async fn make_nodes(n: u8) -> (Node, Vec<Node>) {
        let local_node = make_node(0).await;
        let mut remote_nodes = Vec::new();

        for i in 1..n {
            remote_nodes.push(make_node(i).await);
        }

        (local_node, remote_nodes)
    }

    async fn make_node(index: u8) -> Node {
        let ip = String::from("127.0.0.1").parse::<IpAddr>().unwrap();
        let port_start = 9000_u16;

        let mut id = [0_u8; 32];
        id[31] += index;
        let port = port_start + index as u16;

        let socket_addr = SocketAddr::new(ip, port);
        let peer = Peer { id, socket_addr };

        Node::new(peer).await
    }

    // Run tests independently.  Tests fail when they're run together bc of address reuse.
    // TIP: If you don't give the thing a port, a free port is given automatically
    #[tokio::test]
    async fn add_redundant_node() {
        let (local_node, remote_nodes) = make_nodes(2).await;
        let local_table = &mut local_node.table.lock().unwrap();
        let remote_table = &remote_nodes[0].table.lock().unwrap();

        let result = local_table.add(remote_table.peer);
        let result2 = local_table.add(remote_table.peer);
        assert!(result);
        assert!(!result2);
    }

    #[tokio::test]
    async fn ping() {
        let mut local = make_node(0).await;
        let mut remote = make_node(1).await;
        local.table.lock().unwrap().add(remote.local_record);

        let _ = local.start().await;
        let _ = remote.start().await;
        tokio::time::sleep(Duration::from_secs(1)).await;
        let ping = local.ping(remote.id);
        assert!(ping.await);
        tokio::time::sleep(Duration::from_secs(1)).await;

        let dummy = make_node(2).await;
        let ping = local.ping(dummy.id);
        assert!(!ping.await);
    }
}
