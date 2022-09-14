use image_format::{decode, encode};
use std::fs;
use std::time::Instant;

fn main() {
    let input = ["tile", "bevy", "bevy_small", "hartenstein"];
    // let input = ["tile", "bevy_small"];
    // let input = ["bevy_small", "hartenstein"];
    // let input = ["bevy_small"];
    // let input = ["bevy"];

    for name in &input {
        let data = image::open(format!("images/input/{}.png", name)).unwrap();

        let png_size = fs::metadata(format!("images/input/{}.png", name))
            .unwrap()
            .len();
        let source = data.as_luma16().unwrap().to_vec();

        let size = source.len() * 2;

        println!("Image: {}", name);
        println!("Size uncompressed: {}", size);

        let start = Instant::now();
        let encoded = encode(data.width(), data.height(), &source);
        let duration = start.elapsed();

        // let start = Instant::now();
        // let lz4_output = compress(cast_slice(&input));
        // let lz4_duration = start.elapsed();

        println!(
            "Size encoded: {} ({}%) in {}s",
            encoded.len(),
            encoded.len() as f32 / size as f32 * 100.0,
            duration.as_secs_f32()
        );
        println!(
            "Size png: {} ({}%)",
            png_size,
            png_size as f32 / size as f32 * 100.0
        );
        // println!(
        //     "Size lz4: {} ({}%) in {}s",
        //     lz4_output.len(),
        //     lz4_output.len() as f32 / size as f32 * 100.0,
        //     lz4_duration.as_secs_f32()
        // );
        println!();

        // fs::write(format!("images/output/{}.if", name), &output).unwrap();

        let decoded = decode(data.width(), data.height(), &encoded);

        assert_eq!(source.len(), decoded.len());
        assert_eq!(source, decoded);
    }
}
