#![forbid(unsafe_code)]

use std::{borrow::Borrow, collections::BTreeMap};

mod decode;
mod encode;

#[derive(Debug, PartialEq, PartialOrd, Eq, Ord)]
pub enum Bencode<'a> {
    String(&'a [u8]),
    Integer(isize),
    List(Vec<Bencode<'a>>),
    Dictionary(BTreeMap<&'a [u8], Bencode<'a>>),
}

pub enum BencodeOwned {
    String(Vec<u8>),
    Integer(isize),
    List(Vec<BencodeOwned>),
    Dictionary(BTreeMap<Vec<u8>, BencodeOwned>),
}

type DecodeResult<'a> = decode::DecodeResult<'a>;

pub fn decode(b: &[u8]) -> DecodeResult {
    decode::decode(b)
}

impl<'a> Bencode<'a> {
    pub fn encode(&self) -> Vec<u8> {
        encode::encode(self)
    }

    pub fn unwrap_string(&self) -> String {
        if let Self::String(s) = self {
            std::str::from_utf8(s).unwrap().to_string()
        } else {
            panic!("Value must be a Bencode::String to unwrap as String")
        }
    }

    pub fn unwrap_bytes(&self) -> Vec<u8> {
        if let Self::String(s) = self {
            s.to_vec()
        } else {
            panic!("Value must be a Bencode::String to unwrap as Vec<u8>")
        }
    }

    pub fn unwrap_integer(&self) -> isize {
        if let Self::Integer(i) = self {
            *i
        } else {
            panic!("Value must be a Bencode::Integer to unwrap as isize")
        }
    }

    pub fn unwrap_list(&self) -> &Vec<Bencode> {
        if let Self::List(v) = self {
            v
        } else {
            panic!("Value must be a Bencode::List to unwrap as a Vec<Bencode>")
        }
    }

    pub fn unwrap_dictionary(&self) -> &BTreeMap<&[u8], Bencode> {
        if let Self::Dictionary(d) = self {
            d
        } else {
            panic!("Value must be a Bencode::Dictionary to unwrap as BTreeMap<&[u8], Bencode>")
        }
    }
}

impl<'a> Borrow<Bencode<'a>> for BencodeOwned {
    fn borrow(&self) -> &Bencode<'a> {
        unimplemented!(
            "This might be possible, but I can't figure out how to get it to typecheck yet."
        )
    }
}

impl<'a> ToOwned for Bencode<'a> {
    type Owned = BencodeOwned;

    fn to_owned(&self) -> Self::Owned {
        match self {
            Bencode::String(s) => BencodeOwned::String(s.to_vec()),
            Bencode::Integer(i) => BencodeOwned::Integer(*i),
            Bencode::List(v) => BencodeOwned::List(v.iter().map(|el| el.to_owned()).collect()),
            Bencode::Dictionary(d) => BencodeOwned::Dictionary(
                d.iter().map(|(k, v)| (k.to_vec(), v.to_owned())).collect(),
            ),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::io::Read;

    #[test]
    fn roundtrip() {
        let mut f =
            std::fs::File::open("./fixtures/ubuntu-14.04.4-desktop-amd64.iso.torrent").unwrap();
        let mut buf = vec![];
        f.read_to_end(&mut buf).unwrap();

        let result = crate::decode(&buf);

        // it decodes
        assert!(result.is_ok());
        let bencode = result.unwrap();

        // and the reencoded for is the same as the original bytes
        assert_eq!(bencode.encode(), buf);
    }
}
