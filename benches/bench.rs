use criterion::{criterion_group, criterion_main, Criterion};
use nom_bencode;
use std::io::Read;

fn decode_ubuntu_ref(c: &mut Criterion) {
    let mut torrent =
        std::fs::File::open("./fixtures/ubuntu-14.04.4-desktop-amd64.iso.torrent").unwrap();

    let mut buf = Vec::new();

    torrent.read_to_end(&mut buf).unwrap();

    c.bench_function("decode ubuntu_ref", move |b| {
        b.iter(|| nom_bencode::decode(&buf).unwrap())
    });
}

fn decode_string(c: &mut Criterion) {
    let buf = b"8:abcdefgh";
    c.bench_function("decode_string", move |b| {
        b.iter(|| nom_bencode::decode(buf).unwrap())
    });
}

fn decode_integer(c: &mut Criterion) {
    let buf = b"i-204156622e";
    c.bench_function("decode_integer", move |b| {
        b.iter(|| nom_bencode::decode(buf).unwrap())
    });
}

fn encode_ubuntu_ref(c: &mut Criterion) {
    let mut torrent =
        std::fs::File::open("./fixtures/ubuntu-14.04.4-desktop-amd64.iso.torrent").unwrap();

    let mut buf = Vec::new();

    torrent.read_to_end(&mut buf).unwrap();

    let edn = nom_bencode::decode(&buf).unwrap();

    c.bench_function("encode ubuntu_ref", move |b| b.iter(|| edn.encode()));
}

criterion_group!(
    benches,
    decode_ubuntu_ref,
    decode_string,
    decode_integer,
    encode_ubuntu_ref
);
criterion_main!(benches);
