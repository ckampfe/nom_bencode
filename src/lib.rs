use std::{collections::BTreeMap, fmt::Display};

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
        encode::encode(self)
    }
}

impl Display for Bencode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Bencode::String(bytes) => match std::str::from_utf8(bytes) {
                Ok(s) => {
                    write!(f, "{}", s)
                }
                Err(_) => write!(f, "{:?}", bytes),
            },
            Bencode::Integer(i) => write!(f, "{}", i),
            Bencode::List(l) => {
                for element in l {
                    write!(f, "{}", element.to_string())?;
                }
                Ok(())
            }
            Bencode::Dictionary(map) => {
                for (k, v) in map {
                    match std::str::from_utf8(k) {
                        Ok(s) => {
                            write!(f, "({} => {})", s.to_string(), v.to_string())?;
                        }
                        Err(_) => {
                            write!(f, "({:?} => {})", k, v.to_string())?;
                        }
                    }
                }
                Ok(())
            }
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
