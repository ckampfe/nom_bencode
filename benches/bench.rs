use criterion::{criterion_group, criterion_main, Criterion};
use nom_bencode;
use std::io::Read;

fn ubuntu_ref(c: &mut Criterion) {
    let mut torrent =
        std::fs::File::open("./fixtures/ubuntu-14.04.4-desktop-amd64.iso.torrent").unwrap();

    let mut buf = Vec::new();

    torrent.read_to_end(&mut buf).unwrap();

    c.bench_function("ubuntu_ref", move |b| {
        b.iter(|| nom_bencode::decode(&buf).unwrap())
    });
}

fn ubuntu_owned(c: &mut Criterion) {
    let mut torrent =
        std::fs::File::open("./fixtures/ubuntu-14.04.4-desktop-amd64.iso.torrent").unwrap();

    let mut buf = Vec::new();

    torrent.read_to_end(&mut buf).unwrap();

    c.bench_function("ubuntu_owned", move |b| {
        b.iter(|| nom_bencode::decode(&buf).unwrap().to_owned())
    });
}

fn decode_integer(c: &mut Criterion) {
    let buf = b"i-204156622e";
    c.bench_function("decode_integer", move |b| {
        b.iter(|| nom_bencode::decode(buf).unwrap())
    });
}

criterion_group!(benches, ubuntu_ref, ubuntu_owned, decode_integer);
criterion_main!(benches);
