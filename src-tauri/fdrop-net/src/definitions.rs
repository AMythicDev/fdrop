pub mod protobuf {
    include!(concat!(env!("OUT_DIR"), "/fdrop_net.definitons.rs"));
}

use bytes::{BufMut, Bytes, BytesMut};
use prost::Message;
pub use protobuf::*;

#[repr(u8)]
#[derive(Debug, PartialEq)]
pub enum MessageType {
    Link = 1 << 7,
    TextMessage = 0x01,
}

impl TryFrom<u8> for MessageType {
    type Error = std::io::Error;
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            128 => Ok(Self::Link),
            1 => Ok(Self::TextMessage),
            _ => Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "invalid value given to convert to message type",
            )),
        }
    }
}

pub(crate) fn encode(mtype: MessageType, message: impl Message) -> Bytes {
    let mut buf = BytesMut::with_capacity(1024);
    buf.put_u8(mtype as u8);
    let length = message.encoded_len() as u16;
    buf.put_u16(length);
    message.encode(&mut buf).unwrap();
    buf.freeze()
}
