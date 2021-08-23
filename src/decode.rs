use crate::BencodeRef;
use nom::branch::alt;
use nom::bytes::complete::{tag, take};
use nom::character::complete::digit1;
use nom::combinator::{complete, map, opt};
use nom::multi::{fold_many0, many0};
use nom::sequence::{pair, preceded, terminated};
use nom::*;
use std::collections::BTreeMap;

pub type DecodeResult<'a> = Result<BencodeRef<'a>, nom::error::Error<&'a [u8]>>;

pub(crate) fn decode(b: &[u8]) -> DecodeResult {
    let (_s, o) = any(b).finish()?;
    Ok(o)
}

fn decode_string(s: &[u8]) -> IResult<&[u8], BencodeRef> {
    let (s, prefix) = map(digit1, |bytes| {
        let n: usize = std::str::from_utf8(bytes)
            .expect("not utf8")
            .parse()
            .expect("not a number");
        n
    })(s)?;

    let (s, _) = tag(":")(s)?;
    let (s, bytes) = take(prefix)(s)?;

    Ok((s, BencodeRef::String(bytes)))
}

fn any(s: &[u8]) -> IResult<&[u8], BencodeRef> {
    let (s, b) = complete(alt((
        decode_string,
        decode_integer,
        decode_list,
        decode_dictionary,
    )))(s)?;

    Ok((s, b))
}

fn decode_integer(s: &[u8]) -> IResult<&[u8], BencodeRef> {
    let (s, int) = map(
        preceded(tag("i"), terminated(pair(opt(tag("-")), digit1), tag("e"))),
        |(sign_maybe, bytes): (Option<&[u8]>, &[u8])| {
            let n: isize = std::str::from_utf8(bytes)
                .expect("not utf8")
                .parse()
                .expect("not an int");

            if sign_maybe.is_some() {
                -n
            } else {
                n
            }
        },
    )(s)?;

    Ok((s, BencodeRef::Integer(int)))
}

fn decode_list(s: &[u8]) -> IResult<&[u8], BencodeRef> {
    let (s, list) = preceded(tag("l"), terminated(many0(any), tag("e")))(s)?;

    Ok((s, BencodeRef::List(list)))
}

fn decode_dictionary(s: &[u8]) -> IResult<&[u8], BencodeRef> {
    let (s, dict) = preceded(
        tag("d"),
        terminated(
            fold_many0(
                pair(decode_string, any),
                BTreeMap::new,
                |mut acc: BTreeMap<_, _>, (k, v)| {
                    if let BencodeRef::String(s) = k {
                        acc.insert(s, v);
                        acc
                    } else {
                        unreachable!("Non-string keys in dicts are illegal, so something is definitely worng with the given torrent file")
                    }
                },
            ),
            tag("e"),
        ),
    )(s)?;

    Ok((s, BencodeRef::Dictionary(dict)))
}

#[cfg(test)]
mod tests {
    use std::io::Read;

    use super::*;
    use BencodeRef::*;

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
