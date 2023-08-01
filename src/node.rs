use crate::helper::Identifier;
use crate::kbucket::KbucketTable;
use crate::message::{Message, MessageBody};
use crate::service::Service;
use crate::socket;
use alloy_rlp::{RlpDecodable, RlpEncodable};
use rand::Rng;
use std::{
    collections::HashMap,
    future::Future,
    sync::{Arc, Mutex},
};
use tokio::sync::{mpsc, oneshot};

#[derive(Clone, Copy, Debug, PartialEq, RlpEncodable, RlpDecodable)]
pub struct Peer {
    pub id: Identifier,
    pub socket_addr: socket::SocketAddr,
}

// TODO: Handle errors properly

// The main Kademlia client struct.
// Provides user-level API for performing querie and interacting with the underlying service.
#[derive(Debug)]
pub struct Node {
    pub id: Identifier,
    pub local_record: Peer,
    pub service_tx: Option<mpsc::Sender<Message>>,
    pub table: Arc<Mutex<KbucketTable>>,
    pub outbound_requests: HashMap<Identifier, Message>,
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
                session: (rand::thread_rng().gen_range(0..=255)),
                body: (MessageBody::Ping(self.id, Some(tx))),
            };

            let _ = self.service_tx.as_ref().unwrap().send(msg).await;
            rx.await.unwrap()
        }
    }

    // Should i check routing table for node_to_find before requesting?
    pub fn find_node(
        &mut self,
        node_to_find: Identifier,
    ) -> impl Future<Output = Option<Vec<Peer>>> + '_ {
        async move {
            let target = {
                let table = &self.table.lock().unwrap();
                let target = table.get(&node_to_find);

                if !target.is_none() {
                    println!("Node is already in table!");
                    return None;
                }
                if let Some(target) = table.get_closest_node(&node_to_find) {
                    target
                } else {
                    println!("No nodes in routing table");
                    return None;
                }
            };

            let (tx, rx) = oneshot::channel();
            let msg = Message {
                target,
                session: (rand::thread_rng().gen_range(0..=255)),
                body: (MessageBody::FindNode(self.id, node_to_find, Some(tx))),
            };

            let _ = self.service_tx.as_ref().unwrap().send(msg).await;
            rx.await.unwrap()
        }
    }
    // ---------------------------------------------------------------------------------------------------

    pub async fn start(&mut self) -> Result<(), &'static str> {
        if let Some(service_tx) = Service::spawn(self.local_record, self.table.clone()).await {
            self.service_tx = Some(service_tx);
            Ok(())
        } else {
            Err("Service wasn't created")
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::helper::{make_node, make_nodes};
    use tokio::time::Duration;

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
    async fn ping_rpc() {
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

    #[allow(warnings)]
    #[tokio::test]
    async fn find_node_rpc() {
        let mut local = make_node(0).await;
        let mut remote = make_node(1).await;
        local.table.lock().unwrap().add(remote.local_record);

        // Populate remote's table
        {
            let mut remote_table = remote.table.lock().unwrap();
            let remote_table = {
                for i in 2..10 {
                    if i != 3 {
                        let node = make_node(i).await;
                        remote_table.add(node.local_record);
                    }
                }
                remote_table
            };
        }
        let _ = local.start().await;
        let _ = remote.start().await;

        tokio::time::sleep(Duration::from_secs(1)).await;
        let node_to_find = make_node(3).await.local_record;
        let find_node = local.find_node(node_to_find.id);
        let result = find_node.await;
        println!("\n");
        println!("Peer: {:?}", result);

        // let expected result = ;
        // assert_eq!(find_node.await, );

        // Verify response from node
    }
}
