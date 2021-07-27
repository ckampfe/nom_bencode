use std::collections::BTreeMap;

mod decode;
mod encode;

#[derive(Clone, Debug, PartialEq)]
pub enum Bencode {
    String(Vec<u8>),
    Integer(isize),
    List(Vec<Bencode>),
    Dictionary(BTreeMap<Vec<u8>, Bencode>),
}

impl Eq for Bencode {}

type DecodeResult<'a> = decode::DecodeResult<'a>;

pub fn decode(b: &[u8]) -> DecodeResult {
    decode::decode(b)
}

impl Bencode {
    pub fn encode(&self) -> Vec<u8> {
        encode::encode(&self)
    }
}
