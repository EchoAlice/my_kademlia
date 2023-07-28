use crate::helper::Identifier;
use crate::node::Peer;
use alloy_rlp::{encode_list, Decodable, Encodable, Error, Rlp, RlpDecodable, RlpEncodable};
use tokio::sync::oneshot;
const PEER_LENGTH: usize = 46;
type TotalNodes = u8;

#[derive(Debug)]
pub enum DecoderError {
    Malformed,
}

#[derive(Debug, RlpEncodable, RlpDecodable)]
pub struct Message {
    pub target: Peer,
    pub session: u8,
    pub body: MessageBody,
}

#[derive(Debug)]
pub enum MessageBody {
    Ping(Identifier, Option<oneshot::Sender<bool>>), // 0
    Pong(Identifier),                                // 1
    FindNode(
        Identifier,
        Identifier,
        Option<oneshot::Sender<Option<Vec<Peer>>>>,
    ), // 2
    FoundNode(Identifier, TotalNodes, Vec<Peer>),    // 3
}

//  TODO: Move id to msg inner
//
//  +----------+---------+---------+----------+
//  | msg type | session | node_id |   body   |
//  +----------+---------+---------+----------+
//  |  1 byte  |  1 byte | 32 bytes|    'n'   |
//  +----------+---------+---------+----------+
// TODO: Write tests!
impl Encodable for MessageBody {
    fn encode(&self, out: &mut dyn bytes::BufMut) {
        match self {
            Self::Ping(id, _) => {
                let mut enc: [&dyn Encodable; 2] = [b""; 2];
                enc[0] = &0_u8;
                enc[1] = id;
                encode_list::<_, dyn Encodable>(&enc, out);
            }
            Self::Pong(id) => {
                let mut enc: [&dyn Encodable; 2] = [b""; 2];
                enc[0] = &1_u8;
                enc[1] = &id;
                encode_list::<_, dyn Encodable>(&enc, out);
            }
            Self::FindNode(req_id, node_to_find, _) => {
                let mut enc: [&dyn Encodable; 3] = [b""; 3];
                enc[0] = &2_u8;
                enc[1] = &req_id;
                enc[2] = &node_to_find;
                encode_list::<_, dyn Encodable>(&enc, out);
            }
            Self::FoundNode(req_id, total_nodes, closest_nodes) => {
                let mut enc: [&dyn Encodable; 4] = [b""; 4];
                enc[0] = &3_u8;
                enc[1] = &req_id;
                enc[2] = &total_nodes;
                enc[3] = closest_nodes;
                encode_list::<_, dyn Encodable>(&enc, out);
            }
        }
    }
}

impl Decodable for MessageBody {
    fn decode(data: &mut &[u8]) -> Result<Self, Error> {
        println!("Decode msg body");
        let mut stream = Rlp::new(data)?;
        let typ = stream.get_next::<u8>()?.unwrap();
        let msg = match typ {
            0 => {
                let id = stream.get_next::<[u8; 32]>()?.unwrap();
                MessageBody::Ping(id, None)
            }
            1 => {
                let id = stream.get_next::<[u8; 32]>()?.unwrap();
                MessageBody::Pong(id)
            }
            2 => {
                let id = stream.get_next::<[u8; 32]>()?.unwrap();
                let target = stream.get_next::<[u8; 32]>()?.unwrap();
                MessageBody::FindNode(id, target, None)
            }
            3 => {
                let id = stream.get_next::<[u8; 32]>()?.unwrap();
                let total = stream.get_next::<u8>()?.unwrap();
                println!("Parsing stream for peers");
                let peers = stream.get_next::<Vec<Peer>>()?.unwrap();
                println!("Peers: {:?}", peers);

                MessageBody::FoundNode(id, total, peers)
            }
            _ => panic!(),
        };
        Ok(msg)
    }
}

//  TODO: Delete!
pub fn decode(data: &mut &[u8], target: Peer) -> Result<Message, DecoderError> {
    if data.len() < 34 {
        return Err(DecoderError::Malformed);
    }
    let msg_type = data[0];
    let session = data[1];
    let id: Identifier = data[2..34].try_into().expect("Invalid slice length");
    let body = data[34..].as_ref();
    let msg = match msg_type {
        b'1' => Message {
            target,
            session,
            body: MessageBody::Ping(id, None),
        },
        b'2' => Message {
            target,
            session,
            body: MessageBody::Pong(id),
        },
        b'3' => {
            let node_to_find = body[0..32].try_into().expect("Invalid slice length");
            Message {
                target,
                session,
                body: MessageBody::FindNode(id, node_to_find, None),
            }
        }
        b'4' => {
            let total = body[0];
            if body.len() != 1 + PEER_LENGTH * total as usize {
                return Err(DecoderError::Malformed);
            }

            let mut peers = Vec::new();
            let mut dil_index = 1;
            for _ in 0..total {
                if let Ok(peer) = Peer::decode(data) {
                    peers.push(peer);

                    match body[dil_index] {
                        0 => {
                            dil_index += 38;
                        }
                        1 => {
                            dil_index += 50;
                        }
                        _ => panic!(),
                    }
                } else {
                    panic!()
                }
            }
            Message {
                target,
                session,
                body: MessageBody::FoundNode(id, total, peers),
            }
        }
        _ => return Err(DecoderError::Malformed),
    };
    Ok(msg)
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::helper;
    use bytes::BytesMut;
    use std::net::{self, IpAddr};

    #[test]
    fn serialize_ping() {
        let id = [0u8; 32];
        let body = MessageBody::Ping(id, None);
        println!("Body: {:?}", body);

        let mut out = BytesMut::new();
        body.encode(&mut out);
        println!("Out: {:?}", out);
        let result = MessageBody::decode(&mut out.to_vec().as_slice());
        println!("Result: {:?}", result);
        assert!(result.is_ok());
    }

    #[test]
    fn serialize_pong() {
        let id = [0u8; 32];
        let body = MessageBody::Pong(id);
        println!("Body: {:?}", body);

        let mut out = BytesMut::new();
        body.encode(&mut out);
        println!("Out: {:?}", out);
        let result = MessageBody::decode(&mut out.to_vec().as_slice());
        println!("Result: {:?}", result);
        assert!(result.is_ok());
    }
    #[test]
    fn serialize_find_node() {
        let id = [0u8; 32];
        let target = [1u8; 32];
        let body = MessageBody::FindNode(id, target, None);
        println!("Body: {:?}", body);

        let mut out = BytesMut::new();
        body.encode(&mut out);
        println!("Out: {:?}", out);
        let result = MessageBody::decode(&mut out.to_vec().as_slice());
        println!("Result: {:?}", result);
        assert!(result.is_ok());
    }
    #[test]
    fn serialize_found_node() {
        let local_id = [0u8; 32];
        let total = 2;
        let mut closest_peers = Vec::new();

        let id = [1u8; 32];
        let ip = String::from("127.0.0.1").parse::<IpAddr>().unwrap();
        let port = 68;
        let socket_addr = net::SocketAddr::new(ip, port);
        let peer1 = Peer {
            id,
            socket_addr: helper::SocketAddr { addr: socket_addr },
        };
        closest_peers.push(peer1);

        let id = [2u8; 32];
        let ip = String::from("127.0.0.1").parse::<IpAddr>().unwrap();
        let port = 69;
        let socket_addr = net::SocketAddr::new(ip, port);
        let peer2 = Peer {
            id,
            socket_addr: helper::SocketAddr { addr: socket_addr },
        };
        closest_peers.push(peer2);

        let body = MessageBody::FoundNode(local_id, total, closest_peers);
        println!("Body: {:?}", body);

        let mut out = BytesMut::new();
        body.encode(&mut out);
        println!("Out: {:?}", out);
        let result = MessageBody::decode(&mut out.to_vec().as_slice());
        println!("Result: {:?}", result);
        assert!(result.is_ok());
    }
}
