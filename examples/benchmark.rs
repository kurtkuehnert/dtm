use dtm::DTM;
use std::{fs, time::Instant};

fn main() {
    let input = ["tile", "bevy_small", "bevy", "hartenstein"];
    // let input = ["tile"];
    // let input = ["bevy_small"];
    // let input = ["bevy"];
    // let input = ["hartenstein"];

    for name in &input {
        let source = image::open(format!("images/input/{}.png", name)).unwrap();
        let data = source.as_bytes();

        let path_bin = format!("images/output/{}.bin", name);
        let path_dtm = format!("images/output/{}.dtm", name);
        let path_png = format!("images/output/{}.png", name);

        let start = Instant::now();
        fs::write(&path_bin, data).unwrap();
        let encode_bin = start.elapsed().as_secs_f32() * 1000.0;

        let start = Instant::now();
        DTM {
            pixel_size: 2,
            channel_count: 1,
            width: source.width(),
            height: source.height(),
        }
        .encode_file(&path_dtm, data)
        .unwrap();
        let encode_dtm = start.elapsed().as_secs_f32() * 1000.0;

        let start = Instant::now();
        image::save_buffer(
            &path_png,
            data,
            source.width(),
            source.height(),
            image::ColorType::L16,
        )
        .unwrap();
        let encode_png = start.elapsed().as_secs_f32() * 1000.0;

        let start = Instant::now();
        let _data_bin = fs::read(&path_bin).unwrap();
        let decode_bin = start.elapsed().as_secs_f32() * 1000.0;

        let start = Instant::now();
        let (_descriptor, _data_dtm) = DTM::decode_file(&path_dtm).unwrap();
        let decode_dtm = start.elapsed().as_secs_f32() * 1000.0;

        let start = Instant::now();
        let _data_png = image::open(&path_png).unwrap();
        let decode_png = start.elapsed().as_secs_f32() * 1000.0;

        let size_bin = fs::metadata(&path_bin).unwrap().len() as usize;
        let size_dtm = fs::metadata(&path_dtm).unwrap().len() as usize;
        let size_png = fs::metadata(&path_png).unwrap().len() as usize;

        println!("Image: {}", name);
        println!(
            "bin    size: {} bytes ({}%)\n       loaded and decoded in {} ms ({}%)\n       encoded and saved in {} ms ({}%)",
            size_bin,
            100.0,
            decode_bin,
            100.0,
            encode_bin,
            100.0
        );
        println!(
            "dtm    size: {} bytes ({}%)\n       loaded and decoded in {} ms ({}%)\n       encoded and saved in {} ms ({}%)",
            size_dtm,
            size_dtm as f32 / size_bin as f32 * 100.0,
            decode_dtm,
            decode_dtm / decode_bin * 100.0,
            encode_dtm,
            encode_dtm / encode_bin * 100.0
        );
        println!(
            "png    size: {} bytes ({}%)\n       loaded and decoded in {} ms ({}%)\n       encoded and saved in {} ms ({}%)",
            size_png,
            size_png as f32 / size_bin as f32 * 100.0,
            decode_png,
            decode_png / decode_bin * 100.0,
            encode_png,
            encode_png / encode_bin * 100.0
        );
        println!();
    }
}
