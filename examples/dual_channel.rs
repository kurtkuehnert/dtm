use dtm::DTM;

fn main() {
    let source = image::open("images/input/minmax.png").unwrap();

    let descriptor1 = DTM {
        pixel_size: 2,
        channel_count: 2,
        width: source.width(),
        height: source.height(),
    };

    let data1 = Vec::from(source.as_bytes());

    descriptor1
        .encode_file("images/output/minmax.dtm", &data1)
        .unwrap();

    let (descriptor2, data2) = DTM::decode_file("images/output/minmax.dtm").unwrap();

    assert_eq!(descriptor1, descriptor2);
    assert_eq!(data1, data2);
}
