#[macro_use]
extern crate criterion;

use criterion::Criterion;
use nom_bencode;
use std::io::Read;

fn ubuntu(c: &mut Criterion) {
    let mut torrent =
        std::fs::File::open("./fixtures/ubuntu-14.04.4-desktop-amd64.iso.torrent").unwrap();

    let mut buf = Vec::new();

    torrent.read_to_end(&mut buf).unwrap();

    c.bench_function("ubuntu", move |b| b.iter(|| nom_bencode::parse_bytes(&buf)));
}

criterion_group!(benches, ubuntu);
criterion_main!(benches);
