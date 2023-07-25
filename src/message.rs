use crate::helper::Identifier;
use crate::node::{encoded, Peer};
use fastrlp::{Decodable, DecodeError, Encodable};

use tokio::sync::oneshot;
const PEER_LENGTH: usize = 46;
type TotalNodes = u8;

pub enum DecoderError {
    Malformed,
}

#[derive(Debug, fastrlp_derive::Encodable)]
pub struct Message {
    pub target: Peer,
    pub inner: MessageInner,
}

// Get rid of MessageInner
#[derive(Debug, fastrlp_derive::Encodable)]
pub struct MessageInner {
    pub session: u8,
    pub body: MessageBody,
}

#[derive(Debug)]
pub enum MessageBody {
    Ping(Identifier, Option<oneshot::Sender<bool>>), // b"01"
    Pong(Identifier),                                // b"02"
    FindNode(
        Identifier,
        Identifier,
        Option<oneshot::Sender<Option<Vec<Peer>>>>,
    ), // b"03"
    FoundNode(Identifier, TotalNodes, Vec<Peer>),    // b"04"
}

impl fastrlp::Encodable for MessageBody {
    fn encode(&self, out: &mut dyn bytes::BufMut) {
        match self {
            Self::Ping(id, _) => id.encode(out),
            Self::Pong(id) => id.encode(out),
            Self::FindNode(req_id, node_to_find, _) => [req_id, node_to_find].encode(out),
            // Find a way to RLP encode all the things. We can't put it in a format to .encode()
            Self::FoundNode(req_id, total_nodes, closest_nodes) => {
                req_id.encode(out);
                total_nodes.encode(out);
                for node in closest_nodes {
                    node.encode(out)
                }
            }
        }
    }
}
// TODO: Implement msg body.  Delete msg inner!
impl fastrlp::Decodable for MessageBody {
    fn decode(data: &mut &[u8]) -> Result<Self, DecodeError> {
        let msg_type = data[0];
        let session = data[1];
        let id: Identifier = data[2..34].try_into().expect("Invalid slice length");
        let body = data[34..].as_ref();
        let msg = match msg_type {
            b'1' => MessageBody::Ping(id, None),
            b'2' => MessageBody::Pong(id),
            b'3' => {
                let target = body[0..32].try_into().expect("Invalid slice length");
                MessageBody::FindNode(id, target, None)
            }
            b'4' => {
                let total = body[0];
                if body.len() != 1 + PEER_LENGTH * total as usize {
                    return Err(DecodeError::UnexpectedLength);
                }

                let mut peers = Vec::new();
                let mut dil_index = 1;
                let peer_data = &mut &body[dil_index..dil_index + 32];
                for _ in 0..total {
                    if let Ok(peer) = Peer::decode(peer_data) {
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
                MessageBody::FoundNode(id, total, peers)
            }
        };
        Ok(msg)
    }
}
// impl MessageInner {
//     pub fn encode(&self) -> Vec<u8> {
//         let mut out = Vec::new();
//         match &self.body {
//             MessageBody::Ping(requester_id, _) => {
//                 out.extend_from_slice(b"1");
//                 out.push(self.session);
//                 out.extend_from_slice(requester_id);
//             }
//             MessageBody::Pong(requester_id) => {
//                 out.extend_from_slice(b"2");
//                 out.push(self.session);
//                 out.extend_from_slice(requester_id);
//             }
//             MessageBody::FindNode(requester_id, node_to_find, _) => {
//                 out.extend_from_slice(b"3");
//                 out.push(self.session);
//                 out.extend_from_slice(requester_id);
//                 out.extend_from_slice(node_to_find);
//             }
//             MessageBody::FoundNode(requester_id, total_nodes, closest_nodes) => {
//                 out.extend_from_slice(b"4");
//                 out.push(self.session);
//                 out.extend_from_slice(requester_id);
//                 out.push(*total_nodes); // 35th byte.  Number of nodes in vector.

//                 for node in closest_nodes.iter() {
//                     out.extend_from_slice(encoded(&node).as_ref());
//                 }
//             }
//         }
//         out
//     }

//     // This is cursed...
//     pub fn decode(&self, data: &[u8]) -> Result<Self, DecoderError> {
//         decode(data)
//     }
// }

//  TODO: impl "from" for Identifier
//  TODO: Move id to msg inner
//
//  +----------+---------+---------+----------+
//  | msg type | session | node_id |   body   |
//  +----------+---------+---------+----------+
//  |  1 byte  |  1 byte | 32 bytes|    'n'   |
//  +----------+---------+---------+----------+
pub fn decode(data: &[u8]) -> Result<MessageInner, DecoderError> {
    if data.len() < 34 {
        return Err(DecoderError::Malformed);
    }
    let msg_type = data[0];
    let session = data[1];
    let id: Identifier = data[2..34].try_into().expect("Invalid slice length");
    let body = data[34..].as_ref();
    println!("Msg type: {}", msg_type);
    let msg = match msg_type {
        b'1' => MessageInner {
            session,
            body: MessageBody::Ping(id, None),
        },
        b'2' => MessageInner {
            session,
            body: MessageBody::Pong(id),
        },
        b'3' => {
            let target = body[0..32].try_into().expect("Invalid slice length");
            MessageInner {
                session,
                body: MessageBody::FindNode(id, target, None),
            }
        }
        b'4' => {
            let total = body[0];
            if body.len() != 1 + PEER_LENGTH * total as usize {
                return Err(DecoderError::Malformed);
            }

            let mut peers = Vec::new();
            let mut dil_index = 1;
            let peer_data = &body[dil_index..dil_index + 32];
            for _ in 0..total {
                if let Ok(peer) = Peer::decode(&mut data) {
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
            MessageInner {
                session,
                body: MessageBody::FoundNode(id, total, peers),
            }
        }
        _ => return Err(DecoderError::Malformed),
    };
    Ok(msg)
}
pub fn construct_msg(data: &[u8], target: Peer) -> Message {
    if let Ok(inner) = decode(data) {
        let msg = Message { target, inner };
        return msg;
    }
    panic!("Couldn't convert data to msg")
}
