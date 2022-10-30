use criterion::{criterion_group, criterion_main, Criterion};
use dtm::DTM;
use image::GenericImageView;
use std::fs::File;
use tiff::decoder::{Decoder, DecodingResult};
use tiff::ColorType;

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

fn decode_tif_tiles() {
    let img_file = File::open("data/N265E425.tif").expect("Cannot find test image!");
    let mut decoder = Decoder::new(img_file).expect("Cannot create decoder");
    assert_eq!(decoder.colortype().unwrap(), ColorType::Gray(16));

    let tiles = decoder.tile_count().unwrap();
    assert_eq!(tiles as usize, 100);

    for tile in 0..tiles {
        match decoder.read_chunk(tile).unwrap() {
            DecodingResult::U16(res) => {
                let sum: u64 = res.into_iter().map(<u64>::from).sum();
                if tile == 0 {
                    assert_eq!(sum, 173214606);
                }
            }
            _ => panic!("Wrong bit depth"),
        }
    }
}

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("png full", |b| b.iter(|| decode_full_png()));
    c.bench_function("dtm full", |b| b.iter(|| decode_full_dtm()));
    c.bench_function("tif full", |b| b.iter(|| decode_full_tif()));
    c.bench_function("tif tiles", |b| b.iter(|| decode_tif_tiles()));
}

criterion_group! {
    name = benches;
    config = Criterion::default().sample_size(10);
    targets = criterion_benchmark
}
criterion_main!(benches);
