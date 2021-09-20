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

#[derive(Debug, PartialEq, PartialOrd, Eq, Ord)]
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

#[macro_export]
macro_rules! get_in {
    ($bencode:expr, $($key:expr),*) => {
        {
            let mut r = Some(&$bencode);

            $(
                r = if let Some($crate::Bencode::Dictionary(ref d)) = r {
                    d.get($key)
                } else {
                    None
                };
            )*

            r
        }
    };
}

#[macro_export]
macro_rules! get_in_owned {
    ($bencode:expr, $($key:expr),*) => {
        {
            let mut r = Some(&$bencode);

            $(
                r = if let Some($crate::BencodeOwned::Dictionary(ref d)) = r {
                    d.get($key)
                } else {
                    None
                };
            )*

            r
        }
    };
}

#[cfg(test)]
mod tests {
    use std::io::Read;

    use crate::Bencode;
    use crate::BencodeOwned;

    macro_rules! map {
        () => {
            crate::Map::new()
        };
        ($($key:expr, $value:expr),*) => {
            {
                use std::collections::BTreeMap;
                let mut m = BTreeMap::new();

                $(
                    m.insert($key, $value);
                )*

                m
            }
        };
    }

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

    #[test]
    fn get_in_owned_macro() {
        let nested = BencodeOwned::Dictionary(map![
            b"info".to_vec(),
            BencodeOwned::Dictionary(map![
                b"size".to_vec(),
                BencodeOwned::Integer(8),
                b"type".to_vec(),
                BencodeOwned::String(b"torrent".to_vec()),
                b"deeper".to_vec(),
                BencodeOwned::Dictionary(map![b"secret".to_vec(), BencodeOwned::Integer(29)])
            ])
        ]);

        let res = get_in_owned![
            nested,
            b"info".as_ref(),
            b"deeper".as_ref(),
            b"secret".as_ref()
        ];

        assert_eq!(res, Some(&BencodeOwned::Integer(29)));
    }

    #[test]
    fn get_in_macro() {
        let nested = Bencode::Dictionary(map![
            b"info".as_ref(),
            Bencode::Dictionary(map![
                b"size".as_ref(),
                Bencode::Integer(8),
                b"type".as_ref(),
                Bencode::String(b"torrent".as_ref()),
                b"deeper".as_ref(),
                Bencode::Dictionary(map![b"secret".as_ref(), Bencode::Integer(29)])
            ])
        ]);

        let res = get_in![
            nested,
            b"info".as_ref(),
            b"deeper".as_ref(),
            b"secret".as_ref()
        ];

        assert_eq!(res, Some(&Bencode::Integer(29)));
    }
}
