use crate::helper::Identifier;
use crate::node::Peer;
use alloy_rlp::{encode_list, Decodable, Encodable, Error, RlpDecodable, RlpEncodable};
use tokio::sync::oneshot;
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

// TODO: Impl PartialEq for MessageBody so we can verify serialization
//       and deserialization within tests
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

//  +----------+---------+---------+----------+
//  | msg type | session | node_id |   body   |
//  +----------+---------+---------+----------+
//  |  1 byte  |  1 byte | 32 bytes|    'n'   |
//  +----------+---------+---------+----------+
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
        let mut payload = alloy_rlp::Header::decode_bytes(data, true)?;

        let typ = u8::decode(&mut payload)?;
        let msg = match typ {
            0 => {
                let id = <[u8; 32]>::decode(&mut payload)?;
                MessageBody::Ping(id, None)
            }
            1 => {
                let id = <[u8; 32]>::decode(&mut payload)?;
                MessageBody::Pong(id)
            }
            2 => {
                let id = <[u8; 32]>::decode(&mut payload)?;
                let target = <[u8; 32]>::decode(&mut payload)?;
                MessageBody::FindNode(id, target, None)
            }
            3 => {
                let id = <[u8; 32]>::decode(&mut payload)?;
                let total = <u8>::decode(&mut payload)?;
                let peers = <Vec<Peer>>::decode(&mut payload)?;
                MessageBody::FoundNode(id, total, peers)
            }
            _ => panic!(),
        };
        Ok(msg)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::helper::make_peer;
    use bytes::BytesMut;

    // One shot channels don't allow us to #[derive(PartialEq)] on msg body.
    // Printing statements instead.
    #[test]
    fn serialize_ping() {
        let id = [0u8; 32];
        let body = MessageBody::Ping(id, None);
        println!("Body: {:?}", body);

        let mut out = BytesMut::new();
        body.encode(&mut out);
        let result = MessageBody::decode(&mut out.to_vec().as_slice());
        println!("Result: {:?}", result);
        assert!(result.is_ok());
        println!("\n");
    }

    #[test]
    fn serialize_pong() {
        let id = [0u8; 32];
        let body = MessageBody::Pong(id);
        println!("Body: {:?}", body);

        let mut out = BytesMut::new();
        body.encode(&mut out);
        let result = MessageBody::decode(&mut out.to_vec().as_slice());
        println!("Result: {:?}", result);
        assert!(result.is_ok());
        println!("\n");
    }

    #[test]
    fn serialize_find_node() {
        let id = [0u8; 32];
        let target = [1u8; 32];
        let body = MessageBody::FindNode(id, target, None);
        println!("Body: {:?}", body);

        let mut out = BytesMut::new();
        body.encode(&mut out);
        let result = MessageBody::decode(&mut out.to_vec().as_slice());
        println!("Result: {:?}", result);
        assert!(result.is_ok());
        println!("\n");
    }

    #[test]
    fn serialize_found_node() {
        let local_id = [0u8; 32];

        let total = 2;
        let mut closest_peers = Vec::new();
        let peer = make_peer(1);
        closest_peers.push(peer);
        let peer = make_peer(2);
        closest_peers.push(peer);

        let body = MessageBody::FoundNode(local_id, total, closest_peers);
        println!("Body: {:?}", body);

        let mut out = BytesMut::new();
        body.encode(&mut out);
        let result = MessageBody::decode(&mut out.to_vec().as_slice());
        println!("Result: {:?}", result);
        assert!(result.is_ok());
        println!("\n");
    }
}
