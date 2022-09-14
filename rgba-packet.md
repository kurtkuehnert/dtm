# Packet Encodings

## Alpha Change

Does not output any pixels. Simply changes the "remembered" value
for the A channel. The next pixel will use this A value.

Small difference:
```
Bytes: |BYTE 0|
 Bits: 76543210
Value: 1101aaaa
```

 - `a`: A difference: -7 .. 7 (pattern `0000` disallowed)

Reset:
```
Bytes: |BYTE 0||BYTE 1|
 Bits: 7654321076543210
Value: 11010000aaaaaaaa
```

 - `a`: new A value to set

## Run-Length

Represents a run of many pixels of the same value.

The value is copied from the previous pixel.

The last known A value is used.

```
Bytes: |BYTE 0|
 Bits: 76543210
Value: 111nnnnn
```
(`n` between 1-30, bias 1)

The run length to output is `n`.

```
Bytes: |BYTE 0||BYTE 1|
 Bits: 7654321076543210
Value: 11100000nnnnnnnn
```
(31-286 range)

The run length to output is `n + 31`.

## Copy Pixels

Represents a sequence of many pixels.

The pixels are copied from a range that was previously encountered in the file.

All ARGB channels are copied.

The copied pixels are *not* inserted into the palette.

After the copy operation, the *last* pixel of the sequence is treated as if
it was encoded normally: its RGBA values are remembered and it is inserted
into the palette.

```
Bytes: |BYTE 0||BYTE 1|
 Bits: 7654321076543210
Value: 1100llll0eeeeeee
```
(`l` range 2-16, bias 1)
(`e` range 1-128, bias 1)

```
Bytes: |BYTE 0||BYTE 1||BYTE 2|
 Bits: 765432107654321076543210
Value: 1100llll1eeeeeeeeeeeeeee
```
(`l` range 2-16, bias 1)
(`e` range 128-32896, bias 128)

- `e` is the end offset (bias 1)
- `l` is the sequence length (number of pixels to copy) (bias 1)

The pixels to copy are:
 - starting at `current - e - l`
 - ending at `current - e`

`l` must not be zero.

## Previous Pixel

Represents a single pixel.

The RGB channels are copied from the previously encountered pixel.

The last known A value is used.

```
Bytes: |BYTE 0|
 Bits: 76543210
Value: 11000000
```

## Small Difference (Paeth)

Represents a single pixel.

The Paeth filter is computed:

```rust
let a = left; // current - 1
let b = above; // current - row_len
let c = diagonal; // current - row_len - 1

let p = left + above - diagonal; // wrapping arithmetic

pa = p.abs_diff(a);
pb = p.abs_diff(b);
pc = p.abs_diff(c);

if pa <= pb && pa <= pc {
    a
} else if pb <= pc {
    b
} else {
    c
}
```

The above computation is done separately for each R/G/B channel.

The RGB channels for the current pixel are represented as deltas from the
RGB channels from the Paeth computation above.

If the encoder/decoder has not reached far enough in the image for the
diagonal access to be in-bounds (`current >= row_len - 1`), then the Paeth
computation is not performed, and the previous pixel value is used instead.

The last known A value is used.

```
Bytes: |BYTE 0|
 Bits: 76543210
Value: 10rrggbb
```

 - `r`: R difference: -2 .. 1
 - `g`: G difference: -2 .. 1
 - `b`: B difference: -2 .. 1

## Long Difference (Luma-like)

Represents a single pixel.

The RGB channels for the current pixel are represented as deltas from the
previous pixel.

```
Bytes: |BYTE 0||BYTE 1|
 Bits: 7654321076543210
Value: 01ggggggrrrrbbbb
```

 - `g`: G difference: -32 .. 31
 - `r`: G+R difference: -8 .. 7
 - `b`: G+B difference: -8 .. 7

`r`/`b` represent the R/B channel delta, but their range
is computed from `g`.

## Palette Reference

Represents a single pixel.

The RGB channels are copied from the global palette/dictionary.

The last known A value is used.

```
Bytes: |BYTE 0|
 Bits: 76543210
Value: 00iiiiii
```

`i` must be `< 64`.

### RGB Literal

Represents a single pixel.

Each of the RGB channels is set to a new value.

The last known A value is kept.

```
Bytes: |BYTE 0||BYTE 1||BYTE 2||BYTE 3|
 Bits: 76543210765432107654321076543210
Value: 11111110rrrrrrrrggggggggbbbbbbbb
```

### ARGB Literal

Represents a single pixel.

Each of the ARGB channels is set to a new value.

```
Bytes: |BYTE 0||BYTE 1||BYTE 2||BYTE 3||BYTE 4|
 Bits: 7654321076543210765432107654321076543210
Value: 11111111aaaaaaaarrrrrrrrggggggggbbbbbbbb
```
