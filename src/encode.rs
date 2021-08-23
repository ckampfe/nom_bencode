use std::collections::BTreeMap;

use crate::BencodeRef;

pub(crate) fn encode(bencode: &BencodeRef) -> Vec<u8> {
    let mut buf = vec![];

    do_encode(bencode, &mut buf);

    buf
}

fn do_encode(bencode: &BencodeRef, buf: &mut Vec<u8>) {
    match bencode {
        BencodeRef::String(s) => encode_string(s, buf),
        BencodeRef::Integer(i) => encode_integer(i, buf),
        BencodeRef::List(l) => encode_list(l, buf),
        BencodeRef::Dictionary(d) => encode_dictionary(d, buf),
    }
}

fn encode_string(s: &[u8], buf: &mut Vec<u8>) {
    let len = s.len();
    buf.extend_from_slice(format!("{}", len).as_bytes());
    buf.push(b":"[0]);
    buf.extend_from_slice(s);
}

fn encode_integer(i: &isize, buf: &mut Vec<u8>) {
    buf.push(b"i"[0]);
    buf.extend_from_slice(format!("{}", i).as_bytes());
    buf.push(b"e"[0]);
}

fn encode_list(l: &[BencodeRef], buf: &mut Vec<u8>) {
    buf.push(b"l"[0]);

    for element in l {
        do_encode(element, buf);
    }

    buf.push(b"e"[0]);
}

fn encode_dictionary(d: &BTreeMap<&[u8], BencodeRef>, buf: &mut Vec<u8>) {
    buf.push(b"d"[0]);

    for (k, v) in d {
        encode_string(k, buf);
        do_encode(v, buf);
    }

    buf.push(b"e"[0]);
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
            encode(&crate::BencodeRef::String(b"hello"))
        )
    }

    #[test]
    fn integer() {
        assert_eq!(b"i5e".to_vec(), encode(&crate::BencodeRef::Integer(5isize)));
        assert_eq!(b"i-5e".to_vec(), encode(&crate::BencodeRef::Integer(-5isize)))
    }

    #[test]
    fn list() {
        assert_eq!(b"le".to_vec(), encode(&crate::BencodeRef::List(vec![])));
        assert_eq!(
            b"li5ee".to_vec(),
            encode(&crate::BencodeRef::List(vec![crate::BencodeRef::Integer(5isize)]))
        );
        assert_eq!(
            b"li5e5:helloe".to_vec(),
            encode(&crate::BencodeRef::List(vec![
                crate::BencodeRef::Integer(5isize),
                crate::BencodeRef::String(b"hello")
            ]))
        )
    }

    #[test]
    fn dictionary() {
        assert_eq!(
            b"de".to_vec(),
            encode(&crate::BencodeRef::Dictionary(btreemap![]))
        );
        assert_eq!(
            b"d5:helloi5ee".to_vec(),
            encode(&crate::BencodeRef::Dictionary(btreemap![
                b"hello".as_ref(),
                crate::BencodeRef::Integer(5isize)
            ]))
        );
        assert_eq!(
            b"d5:applei9e4:betai-1e5:helloi5ee".to_vec(),
            encode(&crate::BencodeRef::Dictionary(btreemap![
                // according to the spec, keys shall appear in sorted order
                b"beta".as_ref(),
                crate::BencodeRef::Integer(-1),
                b"hello".as_ref(),
                crate::BencodeRef::Integer(5isize),
                b"apple".as_ref(),
                crate::BencodeRef::Integer(9isize)
            ]))
        )
    }
}
