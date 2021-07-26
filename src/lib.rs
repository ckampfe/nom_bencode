use nom::branch::alt;
use nom::bytes::complete::{tag, take};
use nom::character::complete::digit1;
use nom::combinator::{complete, map, opt};
use nom::multi::{fold_many0, many0};
use nom::sequence::{pair, preceded, terminated};
use nom::*;
use std::collections::BTreeMap;

#[derive(Clone, Debug, PartialEq)]
pub enum Bencode<'a> {
    String(&'a [u8]),
    Integer(isize),
    List(Vec<Bencode<'a>>),
    Dictionary(BTreeMap<&'a [u8], Bencode<'a>>),
}

impl<'a> Eq for Bencode<'a> {}

type DecodeResult<'a> = Result<(&'a [u8], Bencode<'a>), nom::Err<nom::error::Error<&'a [u8]>>>;

pub fn decode(b: &[u8]) -> DecodeResult {
    any(b)
}

fn string(s: &[u8]) -> IResult<&[u8], Bencode> {
    let (s, prefix) = map(digit1, |bytes| {
        let n: usize = std::str::from_utf8(bytes)
            .expect("not utf8")
            .parse()
            .expect("not a number");
        n
    })(s)?;

    let (s, _) = tag(":")(s)?;
    let (s, bytes) = take(prefix)(s)?;

    Ok((s, Bencode::String(bytes)))
}

fn any(s: &[u8]) -> IResult<&[u8], Bencode> {
    let (s, b) = complete(alt((string, int, list, dict)))(s)?;

    Ok((s, b))
}

fn int(s: &[u8]) -> IResult<&[u8], Bencode> {
    let (s, int) = map(
        preceded(tag("i"), terminated(pair(opt(tag("-")), digit1), tag("e"))),
        |(sign_maybe, bytes): (Option<&[u8]>, &[u8])| {
            if let Some(sign) = sign_maybe {
                {
                    let c = &[sign, bytes].concat();
                    std::str::from_utf8(c)
                        .expect("not utf8")
                        .parse()
                        .expect("not an int")
                }
            } else {
                std::str::from_utf8(bytes)
                    .expect("not utf8")
                    .parse()
                    .expect("not an int")
            }
        },
    )(s)?;

    Ok((s, Bencode::Integer(int)))
}

fn list(s: &[u8]) -> IResult<&[u8], Bencode> {
    let (s, list) = preceded(tag("l"), terminated(many0(any), tag("e")))(s)?;

    Ok((s, Bencode::List(list)))
}

fn dict(s: &[u8]) -> IResult<&[u8], Bencode> {
    let (s, dict) = preceded(
        tag("d"),
        terminated(
            fold_many0(
                pair(string, any),
                BTreeMap::new(),
                |mut acc: BTreeMap<_, _>, (k, v)| {
                    if let Bencode::String(s) = k {
                        acc.insert(s, v);
                        acc
                    } else {
                        unreachable!("String keys in dicts are illegal, so something is definitely worng with the given torrent file")
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
    fn strings() {
        assert_eq!(string(b"3:cow"), Ok((vec![].as_bytes(), String(b"cow"))));
        assert_eq!(
            string(b"7:piglets"),
            Ok((vec![].as_bytes(), String(b"piglets")))
        );
    }

    #[test]
    fn ints() {
        assert_eq!(int(b"i0e"), Ok((vec![].as_bytes(), Integer(0))));
        assert_eq!(int(b"i-42e"), Ok((vec![].as_bytes(), Integer(-42))));
        assert_eq!(int(b"i42e"), Ok((vec![].as_bytes(), Integer(42))));
    }

    #[test]
    fn lists() {
        assert_eq!(
            list(b"li0ee"),
            Ok((vec![].as_bytes(), List(vec![Integer(0)])))
        );
        assert_eq!(list(b"le"), Ok((vec![].as_bytes(), List(vec![]))));
    }

    #[test]
    fn dicts() {
        assert_eq!(
            dict(b"de"),
            Ok((vec![].as_bytes(), Dictionary(btreemap![])))
        );
        assert_eq!(
            dict(b"d3:cow7:pigletse"),
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
        assert!(decode(&buf).is_ok())
    }
}
