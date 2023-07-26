use core::slice::SlicePattern;

use crate::helper::Identifier;
use crate::node::Peer;
use bytes::BytesMut;
use fastrlp::{encode_list, Decodable, DecodeError, Encodable};
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

// TODO: Delete MessageInner
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

// TODO: Write tests!
impl fastrlp::Encodable for MessageBody {
    fn encode(&self, out: &mut dyn bytes::BufMut) {
        match self {
            Self::Ping(id, _) => {
                let mut enc: [&dyn Encodable; 2] = [b""; 2];
                enc[0] = &0_u8;
                enc[1] = &id;
                encode_list::<dyn Encodable, _>(&enc, out);
            }
            Self::Pong(id) => {
                let mut enc: [&dyn Encodable; 2] = [b""; 2];
                enc[0] = &1_u8;
                enc[1] = &id;
                encode_list::<dyn Encodable, _>(&enc, out);
            }
            Self::FindNode(req_id, node_to_find, _) => {
                let mut enc: [&dyn Encodable; 3] = [b""; 3];
                enc[0] = &2_u8;
                enc[1] = &req_id;
                enc[2] = &node_to_find;
                encode_list::<dyn Encodable, _>(&enc, out);
            }
            Self::FoundNode(req_id, total_nodes, closest_nodes) => {
                // I have a list of elements in closest nodes that implements
                // the trait Encodable, but for some reason I can't get it added
                // to an array of items which implment the Encodable trait. It
                // seems to be something with not being able to convince the
                // complier that closest_nodes is a &[T] where T implements
                // Encodable and is ?Sized.
                let mut enc: [&dyn Encodable; 4] = [b""; 4];
                enc[0] = &3_u8;
                enc[1] = &req_id;
                enc[2] = &total_nodes;
                enc[3] = closest_nodes.iter().collect();
                encode_list(enc, out);
            }
        }
    }
}



impl fastrlp::Decodable for MessageBody {
    fn decode(data: &mut &[u8]) -> Result<Self, DecodeError> {
        let mut stream = fastrlp::Rlp::new(data)?;
        let typ = stream.get_next::<u8>()?.unwrap();

        let id: Identifier = data[2..34].try_into().expect("Invalid slice length");
        let body = data[34..].as_ref();

        let msg = match typ {
            b'1' => {
                let id = stream.get_next::<[u8; 32]>()?.unwrap();
                MessageBody::Ping(id, None)
            }
            b'2' => {
                let id = stream.get_next::<[u8; 32]>()?.unwrap();
                MessageBody::Pong(id)
            }
            b'3' => {
                let id = stream.get_next::<[u8; 32]>()?.unwrap();
                let target = stream.get_next::<[u8; 32]>()?.unwrap();
                MessageBody::FindNode(id, target, None)
            }
            b'4' => {
                let id = stream.get_next::<[u8; 32]>()?.unwrap();
                let total = stream.get_next::<u8>()?.unwrap();
                let peers = Vec::new();

                loop {
                    match stream.get_next()::<Peer>() {
                        Ok(Some(peer)) => {

                        }
                        Ok(None) => {
                            break
                        }
                        Err(error) => {

                        }
                    }
                }

                MessageBody::FoundNode(id, total, peers)

                // let total = body[0];
                // if body.len() != 1 + PEER_LENGTH * total as usize {
                //     return Err(DecodeError::UnexpectedLength);
                // }

                // let mut peers = Vec::new();
                // let mut dil_index = 1;
                // let peer_data = &mut &body[dil_index..dil_index + 32];
                // for _ in 0..total {
                //     if let Ok(peer) = Peer::decode(peer_data) {
                //         peers.push(peer);

                //         match body[dil_index] {
                //             0 => {
                //                 dil_index += 38;
                //             }
                //             1 => {
                //                 dil_index += 50;
                //             }
                //             _ => panic!(),
                //         }
                //     } else {
                //         panic!()
                //     }
                // }
                // MessageBody::FoundNode(id, total, peers)
            }
            _ => panic!(),
        };
        Ok(msg)
    }
}

//  TODO: Delete!
//
//  TODO: impl "from" for Identifier
//  TODO: Move id to msg inner
//
//  +----------+---------+---------+----------+
//  | msg type | session | node_id |   body   |
//  +----------+---------+---------+----------+
//  |  1 byte  |  1 byte | 32 bytes|    'n'   |
//  +----------+---------+---------+----------+
pub fn decode(data: &mut &[u8]) -> Result<MessageInner, DecoderError> {
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
            MessageInner {
                session,
                body: MessageBody::FoundNode(id, total, peers),
            }
        }
        _ => return Err(DecoderError::Malformed),
    };
    Ok(msg)
}

pub fn construct_msg(data: &mut &[u8], target: Peer) -> Message {
    if let Ok(inner) = decode(data) {
        let msg = Message { target, inner };
        return msg;
    }
    panic!("Couldn't convert data to msg")
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn msg_body_serialization_ping() {
        // TODO: Create msg body
        let id = [0u8; 32];
        let body = MessageBody::Ping(id, None);

        let mut out = BytesMut::new();
        body.encode(&mut out);
        println!("Out: {:?}", out);
        let result = MessageBody::decode(&mut out.to_vec().as_slice());
        println!("Result: {:?}", result);
        assert!(result.is_ok());
    }

    #[test]
    fn msg_body_serialization_find_node() {}
}
