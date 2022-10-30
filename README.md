# DTM Image Format

Fast encoder/decoder for the DTM image format.

The DTM image format is a 16-bit lossless image format supporting one to four channels.
Its purpose is to serve as a (5x - 10x) faster png alternative with comparable compression.

This format was developed to compress large terrain heightmaps also known as digital terrain models, hence the name.

## Example
```rust
use dtm::DTM;

fn main() {
    let descriptor1 = DTM {
        pixel_size: 2,
        channel_count: 1,
        width: 16,
        height: 16,
    };
    let data1 = vec![0u8; descriptor1.image_size()];

    descriptor1.encode_file("image.dtm", &data1).unwrap();
    let (descriptor2, data2) = DTM::decode_file("image.dtm").unwrap();

    assert_eq!(descriptor1, descriptor2);
    assert_eq!(data1, data2);
}
```



## Format

The DTM format is inspired by the QOI format and utilizes four simple compression ideas.
Additionally, the paeth filter is used to achieve better local reuse.

**Note:**
This format is in now way stable or formally specified.
I might extend it to support 8 and 32 bit images as well.

```
uncompressed |         11111111 | byte1 | byte2 |  
mru cache    | 00 |       index |                   cache-size:  64
single dif   | 01 |         dif |                   6 bit dif:   [-32, 31] 
double dif   | 10 | dif1 | dif2 |                   3 bit dif:   [ -4,  3]
run lenght   | 11 |         run |                   run-length:  [  1, 63]
```

## License
DTM Image Format is dual-licensed under either

* MIT License (LICENSE-MIT or http://opensource.org/licenses/MIT)
* Apache License, Version 2.0 (LICENSE-APACHE or http://www.apache.org/licenses/LICENSE-2.0)

at your option.