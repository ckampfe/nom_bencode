use std::{borrow::Borrow, collections::BTreeMap};

mod decode;
mod encode;

#[derive(Clone, Debug, PartialEq, PartialOrd, Eq, Ord)]
pub enum Bencode {
    String(Vec<u8>),
    Integer(isize),
    List(Vec<Bencode>),
    Dictionary(BTreeMap<Vec<u8>, Bencode>),
}

impl<'a> Borrow<BencodeRef<'a>> for Bencode {
    fn borrow(&self) -> &BencodeRef<'a> {
        todo!()
    }
}

#[derive(Debug, PartialEq, PartialOrd, Eq, Ord)]
pub enum BencodeRef<'a> {
    String(&'a [u8]),
    Integer(isize),
    List(Vec<BencodeRef<'a>>),
    Dictionary(BTreeMap<&'a [u8], BencodeRef<'a>>),
}

impl ToOwned for BencodeRef<'_> {
    type Owned = Bencode;

    // TODO:
    // while rare in practice given that most .torrent
    // files are not that deep structurally,
    // this can likely overflow the stack.
    // figure out a way to do this in a stack-safe way,
    // maybe with a queue
    fn to_owned(&self) -> Self::Owned {
        match self {
            BencodeRef::String(s) => Bencode::String(s.to_vec()),
            BencodeRef::Integer(i) => Bencode::Integer(*i),
            BencodeRef::List(l) => Bencode::List(l.iter().map(|el| el.to_owned()).collect()),
            BencodeRef::Dictionary(d) => {
                Bencode::Dictionary(d.iter().map(|(k, v)| (k.to_vec(), v.to_owned())).collect())
            }
        }
    }
}

type DecodeResult<'a> = decode::DecodeResult<'a>;

pub fn decode(b: &[u8]) -> DecodeResult {
    decode::decode(b)
}

impl<'a> BencodeRef<'a> {
    pub fn encode(&self) -> Vec<u8> {
        encode::encode(self)
    }

    pub fn unwrap_string(&self) -> String {
        if let Self::String(s) = self {
            std::str::from_utf8(s).unwrap().to_string()
        } else {
            panic!()
        }
    }

    pub fn unwrap_bytes(&self) -> Vec<u8> {
        if let Self::String(s) = self {
            s.to_owned().to_vec()
        } else {
            panic!()
        }
    }

    pub fn unwrap_integer(&self) -> isize {
        if let Self::Integer(i) = self {
            *i
        } else {
            panic!()
        }
    }

    pub fn unwrap_dictionary(&self) -> &BTreeMap<&[u8], BencodeRef> {
        if let Self::Dictionary(d) = self {
            d.to_owned()
        } else {
            panic!()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Read;

    macro_rules! btreemap {
        () => {
            BTreeMap::new()
        };
        ( $($x:expr, $y:expr),* ) => {
            {
                let mut bm = BTreeMap::new();

                $(
                    bm.insert($x, $y);
                )*

                bm
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
    fn to_owned_ubuntu() {
        let mut f =
            std::fs::File::open("./fixtures/ubuntu-14.04.4-desktop-amd64.iso.torrent").unwrap();
        let mut buf = vec![];
        f.read_to_end(&mut buf).unwrap();

        let result = crate::decode(&buf);

        // it decodes
        assert!(result.is_ok());
        let bencode = result.unwrap();
        let owned_dictionary = bencode.to_owned();
        if let crate::Bencode::Dictionary(_) = owned_dictionary {
            ()
        } else {
            panic!()
        }
    }

    #[test]
    fn to_owned_string() {
        let buf = b"5:hello";
        let bencode_ref = crate::decode(buf).unwrap();
        let owned_string = bencode_ref.to_owned();
        assert_eq!(owned_string, crate::Bencode::String(b"hello".to_vec()));
    }

    #[test]
    fn to_owned_integer() {
        let buf = b"i4e";
        let bencode_ref = crate::decode(buf).unwrap();
        let owned_integer = bencode_ref.to_owned();
        assert_eq!(owned_integer, crate::Bencode::Integer(4));
    }

    #[test]
    fn to_owned_list() {
        let buf = b"l5:helloi-4ee";
        let bencode_ref = crate::decode(buf).unwrap();
        let owned_list = bencode_ref.to_owned();
        assert_eq!(
            owned_list,
            crate::Bencode::List(vec![
                crate::Bencode::String(b"hello".to_vec()),
                crate::Bencode::Integer(-4)
            ])
        );
    }

    #[test]
    fn to_owned_dictionary() {
        let buf = b"d5:applei9e4:betai-1e5:helloi5ee";
        let bencode_ref = crate::decode(buf).unwrap();
        let owned_dictionary = bencode_ref.to_owned();
        assert_eq!(
            owned_dictionary,
            crate::Bencode::Dictionary(btreemap![
                b"beta".to_vec(),
                crate::Bencode::Integer(-1),
                b"hello".to_vec(),
                crate::Bencode::Integer(5isize),
                b"apple".to_vec(),
                crate::Bencode::Integer(9isize)
            ])
        );
    }
}
