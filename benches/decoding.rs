use criterion::{criterion_group, criterion_main, Criterion};
use dtm::DTM;
use image::GenericImageView;

fn decode_full_png() {
    let data_png = image::open("data/N265E425.png").unwrap();
    assert_eq!(data_png.bounds(), (0, 0, 5000, 5000));
}

fn decode_full_dtm() {
    let (descriptor, _data_dtm) = DTM::decode_file("data/N265E425.dtm").unwrap();
    assert_eq!(descriptor.width, 5000);
    assert_eq!(descriptor.height, 5000);
}

fn decode_full_tif() {
    let data_tif = image::open("data/N265E425.tif").unwrap();
    assert_eq!(data_tif.bounds(), (0, 0, 5000, 5000));
}

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("png full", |b| b.iter(|| decode_full_png()));
    c.bench_function("dtm full", |b| b.iter(|| decode_full_dtm()));
    c.bench_function("tif full", |b| b.iter(|| decode_full_tif()));
}

criterion_group! {
    name = benches;
    config = Criterion::default().sample_size(10);
    targets = criterion_benchmark
}
criterion_main!(benches);
