use dtm::DTM;

fn main() {
    let (descriptor, data) = DTM::decode_file("images/input/180236.dtm").unwrap();

    // let descriptor1 = DTM {
    //     pixel_size: 2,
    //     channel_count: 1,
    //     width: 16,
    //     height: 16,
    // };
    // let data1 = vec![0u16; descriptor1.image_pixel_count()];
    //
    // descriptor1.encode_file("image.dtm", &data1).unwrap();
    // let (descriptor2, data2) = DTM::decode_file("image.dtm").unwrap();
    //
    // assert_eq!(descriptor1, descriptor2);
    // assert_eq!(data1, data2);
}
