pub mod protobuf {
    include!(concat!(env!("OUT_DIR"), "/fdrop_net.definitons.rs"));
}

use bytes::{BufMut, Bytes, BytesMut};
use prost::Message;
pub use protobuf::*;

#[repr(u8)]
pub enum MessageType {
    Authentication = 1 << 7,
}

pub fn encode(mtype: MessageType, message: impl Message) -> Bytes {
    let mut buf = BytesMut::with_capacity(1024);
    buf.put_u8(mtype as u8);
    let length = message.encoded_len() as u16;
    buf.put_u16(length);
    message.encode_length_delimited(&mut buf).unwrap();
    buf.freeze()
}
