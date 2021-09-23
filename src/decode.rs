use crate::Bencode;
use atoi::{FromRadix10, FromRadix10Signed};
use nom::branch::alt;
use nom::bytes::complete::{tag, take};
use nom::multi::{fold_many0, many0};
use nom::sequence::{pair, preceded, terminated};
use nom::*;
use std::collections::BTreeMap;

pub(crate) fn decode(b: &[u8]) -> Result<Bencode, nom::error::Error<&[u8]>> {
    let (_s, o) = any(b).finish()?;
    Ok(o)
}

fn decode_string(s: &[u8]) -> IResult<&[u8], Bencode> {
    let (prefix, rest_index) = usize::from_radix_10(s);
    if prefix == 0 && rest_index == 0 {
        return Err(nom::Err::Error(nom::error::Error {
            input: s,
            code: nom::error::ErrorKind::Digit,
        }));
    }
    let s = &s[rest_index..];

    let (s, _) = tag(":")(s)?;
    let (s, bytes) = take(prefix)(s)?;

    Ok((s, Bencode::String(bytes)))
}

fn any(s: &[u8]) -> IResult<&[u8], Bencode> {
    let (s, b) = alt((
        decode_string,
        decode_integer,
        decode_list,
        decode_dictionary,
    ))(s)?;

    Ok((s, b))
}

fn decode_integer(s: &[u8]) -> IResult<&[u8], Bencode> {
    let (s, _) = tag("i")(s)?;
    let (int, rest_index) = isize::from_radix_10_signed(s);
    if int == 0 && rest_index == 0 {
        return Err(nom::Err::Error(nom::error::Error {
            input: s,
            code: nom::error::ErrorKind::Digit,
        }));
    }
    let s = &s[rest_index..];
    let (s, _) = tag("e")(s)?;

    Ok((s, Bencode::Integer(int)))
}

fn decode_list(s: &[u8]) -> IResult<&[u8], Bencode> {
    let (s, list) = preceded(tag("l"), terminated(many0(any), tag("e")))(s)?;

    Ok((s, Bencode::List(list)))
}

fn decode_dictionary(s: &[u8]) -> IResult<&[u8], Bencode> {
    let (s, dict) = preceded(
        tag("d"),
        terminated(
            fold_many0(
                pair(decode_string, any),
                BTreeMap::new,
                |mut acc: BTreeMap<_, _>, (k, v)| {
                    if let Bencode::String(s) = k {
                        acc.insert(s, v);
                        acc
                    } else {
                        unreachable!("Non-string keys in dicts are illegal, so something is definitely wrong with the given torrent file")
                    }
                },
            ),
            tag("e"),
        ),
    )(s)?;

    Ok((s, Bencode::Dictionary(dict)))
}

#[cfg(test)]
mod tests {
    use std::io::Read;

    use super::*;
    use Bencode::*;

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
            decode_string(b"3:cow"),
            Ok((vec![].as_bytes(), String(b"cow")))
        );
        assert_eq!(
            decode_string(b"7:piglets"),
            Ok((vec![].as_bytes(), String(b"piglets")))
        );
    }

    #[test]
    fn integer() {
        assert_eq!(decode_integer(b"i0e"), Ok((vec![].as_bytes(), Integer(0))));
        assert_eq!(
            decode_integer(b"i-42e"),
            Ok((vec![].as_bytes(), Integer(-42)))
        );
        assert_eq!(
            decode_integer(b"i42e"),
            Ok((vec![].as_bytes(), Integer(42)))
        );
    }

    #[test]
    fn list() {
        assert_eq!(
            decode_list(b"li0ee"),
            Ok((vec![].as_bytes(), List(vec![Integer(0)])))
        );
        assert_eq!(decode_list(b"le"), Ok((vec![].as_bytes(), List(vec![]))));
    }

    #[test]
    fn dictionary() {
        assert_eq!(
            decode_dictionary(b"de"),
            Ok((vec![].as_bytes(), Dictionary(btreemap![])))
        );
        assert_eq!(
            decode_dictionary(b"d3:cow7:pigletse"),
            Ok((
                vec![].as_bytes(),
                Dictionary(btreemap![b"cow".as_ref(), String(b"piglets")])
            ))
        );
    }

    #[test]
    fn file() {
        let mut f =
            std::fs::File::open("./fixtures/ubuntu-14.04.4-desktop-amd64.iso.torrent").unwrap();
        let mut buf = vec![];
        f.read_to_end(&mut buf).unwrap();
        assert!(crate::decode(&buf).is_ok())
    }
}
