use std::collections::BTreeMap;

use crate::Bencode;

const COLON_BYTE: u8 = b':';
const D_BYTE: u8 = b'd';
const E_BYTE: u8 = b'e';
const I_BYTE: u8 = b'i';
const L_BYTE: u8 = b'l';

pub(crate) fn encode(bencode: &Bencode) -> Vec<u8> {
    let mut buf = vec![];

    do_encode(bencode, &mut buf);

    buf
}

fn do_encode(bencode: &Bencode, buf: &mut Vec<u8>) {
    match bencode {
        Bencode::String(s) => encode_string(s, buf),
        Bencode::Integer(i) => encode_integer(i, buf),
        Bencode::List(l) => encode_list(l, buf),
        Bencode::Dictionary(d) => encode_dictionary(d, buf),
    }
}

fn encode_string(s: &[u8], buf: &mut Vec<u8>) {
    let len = s.len();
    itoa::write(&mut *buf, len).unwrap();
    buf.push(COLON_BYTE);
    buf.extend_from_slice(s);
}

fn encode_integer(i: &isize, buf: &mut Vec<u8>) {
    buf.push(I_BYTE);
    itoa::write(&mut *buf, *i).unwrap();
    buf.push(E_BYTE);
}

fn encode_list(l: &[Bencode], buf: &mut Vec<u8>) {
    buf.push(L_BYTE);

    for element in l {
        do_encode(element, buf);
    }

    buf.push(E_BYTE);
}

fn encode_dictionary(d: &BTreeMap<&[u8], Bencode>, buf: &mut Vec<u8>) {
    buf.push(D_BYTE);

    for (k, v) in d {
        encode_string(k, buf);
        do_encode(v, buf);
    }

    buf.push(E_BYTE);
}

#[cfg(test)]
mod tests {
    use super::*;

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
    fn string() {
        assert_eq!(
            b"5:hello".to_vec(),
            encode(&crate::Bencode::String(b"hello"))
        )
    }

    #[test]
    fn integer() {
        assert_eq!(b"i5e".to_vec(), encode(&crate::Bencode::Integer(5isize)));
        assert_eq!(b"i-5e".to_vec(), encode(&crate::Bencode::Integer(-5isize)))
    }

    #[test]
    fn list() {
        assert_eq!(b"le".to_vec(), encode(&crate::Bencode::List(vec![])));
        assert_eq!(
            b"li5ee".to_vec(),
            encode(&crate::Bencode::List(vec![crate::Bencode::Integer(5isize)]))
        );
        assert_eq!(
            b"li5e5:helloe".to_vec(),
            encode(&crate::Bencode::List(vec![
                crate::Bencode::Integer(5isize),
                crate::Bencode::String(b"hello")
            ]))
        )
    }

    #[test]
    fn dictionary() {
        assert_eq!(
            b"de".to_vec(),
            encode(&crate::Bencode::Dictionary(btreemap![]))
        );
        assert_eq!(
            b"d5:helloi5ee".to_vec(),
            encode(&crate::Bencode::Dictionary(btreemap![
                b"hello".as_ref(),
                crate::Bencode::Integer(5isize)
            ]))
        );
        assert_eq!(
            b"d5:applei9e4:betai-1e5:helloi5ee".to_vec(),
            encode(&crate::Bencode::Dictionary(btreemap![
                // according to the spec, keys shall appear in sorted order
                b"beta".as_ref(),
                crate::Bencode::Integer(-1),
                b"hello".as_ref(),
                crate::Bencode::Integer(5isize),
                b"apple".as_ref(),
                crate::Bencode::Integer(9isize)
            ]))
        )
    }
}
