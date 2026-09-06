//! Scene look L5 (plan `look/05`): the PNG decoder of authored texture
//! sources and the mip chain builder. Eight-bit grey, grey-alpha, RGB and
//! RGBA images without interlacing; every row filter; the output is always
//! `RGBA8`. Mips are a `2 x 2` box filter, averaged in linear light for
//! sRGB textures.

use std::io::Read;

/// A decoded image: `RGBA8` texels, row-major, top row first.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct DecodedPngV1 {
    pub(crate) width: u32,
    pub(crate) height: u32,
    pub(crate) rgba8: Vec<u8>,
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum PngDecodeError {
    NotPng,
    Unsupported(&'static str),
    Corrupt(&'static str),
}

const SIGNATURE: [u8; 8] = [0x89, b'P', b'N', b'G', 0x0d, 0x0a, 0x1a, 0x0a];

/// Decodes a PNG into `RGBA8`.
/// Scene look L6a: a 16-bit grey PNG (height fields) to its samples, row
/// major, big-endian samples decoded to `u16`.
pub(crate) fn decode_png_grey16(bytes: &[u8]) -> Result<(u32, u32, Vec<u16>), PngDecodeError> {
    let (width, height, bit_depth, color_type, compressed) = parse_chunks(bytes)?;
    if bit_depth != 16 || color_type != 0 {
        return Err(PngDecodeError::Unsupported("not a 16-bit grey PNG"));
    }
    let mut raw = Vec::new();
    flate2::read::ZlibDecoder::new(compressed.as_slice())
        .read_to_end(&mut raw)
        .map_err(|_| PngDecodeError::Corrupt("zlib stream"))?;
    let stride = width as usize * 2;
    if raw.len() != (stride + 1) * height as usize {
        return Err(PngDecodeError::Corrupt("decompressed size"));
    }
    let mut previous = vec![0_u8; stride];
    let mut current = vec![0_u8; stride];
    let mut samples = Vec::with_capacity(width as usize * height as usize);
    for row in 0..height as usize {
        let start = row * (stride + 1);
        let filter = raw[start];
        current.copy_from_slice(&raw[start + 1..start + 1 + stride]);
        unfilter_row(filter, &mut current, &previous, 2)?;
        for sample in current.chunks_exact(2) {
            samples.push(u16::from_be_bytes([sample[0], sample[1]]));
        }
        std::mem::swap(&mut previous, &mut current);
    }
    Ok((width, height, samples))
}

/// The IHDR fields and the concatenated IDAT stream.
fn parse_chunks(bytes: &[u8]) -> Result<(u32, u32, u8, u8, Vec<u8>), PngDecodeError> {
    if bytes.len() < 8 || bytes[..8] != SIGNATURE {
        return Err(PngDecodeError::NotPng);
    }
    let mut position = 8;
    let mut header: Option<(u32, u32, u8, u8)> = None;
    let mut compressed = Vec::new();
    while position + 8 <= bytes.len() {
        let length = u32::from_be_bytes([
            bytes[position],
            bytes[position + 1],
            bytes[position + 2],
            bytes[position + 3],
        ]) as usize;
        let kind = &bytes[position + 4..position + 8];
        let body_start = position + 8;
        let body_end = body_start
            .checked_add(length)
            .ok_or(PngDecodeError::Corrupt("chunk length"))?;
        if body_end + 4 > bytes.len() {
            return Err(PngDecodeError::Corrupt("chunk overruns the file"));
        }
        let body = &bytes[body_start..body_end];
        match kind {
            b"IHDR" => {
                if body.len() != 13 {
                    return Err(PngDecodeError::Corrupt("IHDR length"));
                }
                header = Some((
                    u32::from_be_bytes([body[0], body[1], body[2], body[3]]),
                    u32::from_be_bytes([body[4], body[5], body[6], body[7]]),
                    body[8],
                    body[9],
                ));
            }
            b"IDAT" => compressed.extend_from_slice(body),
            b"IEND" => break,
            _ => {}
        }
        position = body_end + 4;
    }
    let (width, height, bit_depth, color_type) =
        header.ok_or(PngDecodeError::Corrupt("no IHDR"))?;
    Ok((width, height, bit_depth, color_type, compressed))
}

pub(crate) fn decode_png(bytes: &[u8]) -> Result<DecodedPngV1, PngDecodeError> {
    if bytes.len() < 8 || bytes[..8] != SIGNATURE {
        return Err(PngDecodeError::NotPng);
    }
    let mut position = 8;
    let mut header: Option<(u32, u32, u8, u8)> = None;
    let mut compressed = Vec::new();
    let mut palette: Option<Vec<[u8; 3]>> = None;
    while position + 8 <= bytes.len() {
        let length = u32::from_be_bytes([
            bytes[position],
            bytes[position + 1],
            bytes[position + 2],
            bytes[position + 3],
        ]) as usize;
        let kind = &bytes[position + 4..position + 8];
        let body_start = position + 8;
        let body_end = body_start
            .checked_add(length)
            .ok_or(PngDecodeError::Corrupt("chunk length"))?;
        if body_end + 4 > bytes.len() {
            return Err(PngDecodeError::Corrupt("chunk overruns the file"));
        }
        let body = &bytes[body_start..body_end];
        match kind {
            b"IHDR" => {
                if body.len() != 13 {
                    return Err(PngDecodeError::Corrupt("IHDR length"));
                }
                let width = u32::from_be_bytes([body[0], body[1], body[2], body[3]]);
                let height = u32::from_be_bytes([body[4], body[5], body[6], body[7]]);
                let bit_depth = body[8];
                let color_type = body[9];
                if body[12] != 0 {
                    return Err(PngDecodeError::Unsupported("interlaced image"));
                }
                // Scene look L6a (plan `look/06a`): 16-bit grey for height
                // fields; every other form is 8-bit.
                if bit_depth != 8 && !(bit_depth == 16 && color_type == 0) {
                    return Err(PngDecodeError::Unsupported("bit depth other than 8"));
                }
                if !matches!(color_type, 0 | 2 | 3 | 4 | 6) {
                    return Err(PngDecodeError::Unsupported("colour type"));
                }
                if width == 0 || height == 0 || width > 16_384 || height > 16_384 {
                    return Err(PngDecodeError::Unsupported("extent"));
                }
                header = Some((width, height, bit_depth, color_type));
            }
            b"PLTE" => {
                palette = Some(body.chunks_exact(3).map(|c| [c[0], c[1], c[2]]).collect());
            }
            b"IDAT" => compressed.extend_from_slice(body),
            b"IEND" => break,
            _ => {}
        }
        position = body_end + 4;
    }
    let (width, height, bit_depth, color_type) =
        header.ok_or(PngDecodeError::Corrupt("no IHDR"))?;
    if bit_depth == 16 {
        return Err(PngDecodeError::Unsupported(
            "16-bit samples need decode_png_grey16",
        ));
    }
    let channels = match color_type {
        0 => 1,
        2 => 3,
        3 => 1,
        4 => 2,
        6 => 4,
        _ => unreachable!("validated above"),
    };
    let mut raw = Vec::new();
    flate2::read::ZlibDecoder::new(compressed.as_slice())
        .read_to_end(&mut raw)
        .map_err(|_| PngDecodeError::Corrupt("zlib stream"))?;
    let stride = width as usize * channels;
    if raw.len() != (stride + 1) * height as usize {
        return Err(PngDecodeError::Corrupt("decompressed size"));
    }
    let mut previous = vec![0_u8; stride];
    let mut current = vec![0_u8; stride];
    let mut rgba8 = Vec::with_capacity(width as usize * height as usize * 4);
    for row in 0..height as usize {
        let start = row * (stride + 1);
        let filter = raw[start];
        current.copy_from_slice(&raw[start + 1..start + 1 + stride]);
        unfilter_row(filter, &mut current, &previous, channels)?;
        for texel in current.chunks_exact(channels) {
            match color_type {
                0 => rgba8.extend_from_slice(&[texel[0], texel[0], texel[0], 255]),
                2 => rgba8.extend_from_slice(&[texel[0], texel[1], texel[2], 255]),
                3 => {
                    let entry = palette
                        .as_ref()
                        .and_then(|palette| palette.get(usize::from(texel[0])))
                        .ok_or(PngDecodeError::Corrupt("palette index"))?;
                    rgba8.extend_from_slice(&[entry[0], entry[1], entry[2], 255]);
                }
                4 => rgba8.extend_from_slice(&[texel[0], texel[0], texel[0], texel[1]]),
                _ => rgba8.extend_from_slice(texel),
            }
        }
        std::mem::swap(&mut previous, &mut current);
    }
    Ok(DecodedPngV1 {
        width,
        height,
        rgba8,
    })
}

fn unfilter_row(
    filter: u8,
    current: &mut [u8],
    previous: &[u8],
    channels: usize,
) -> Result<(), PngDecodeError> {
    for index in 0..current.len() {
        let left = if index >= channels {
            current[index - channels]
        } else {
            0
        };
        let up = previous[index];
        let up_left = if index >= channels {
            previous[index - channels]
        } else {
            0
        };
        let predictor = match filter {
            0 => 0,
            1 => left,
            2 => up,
            3 => ((u16::from(left) + u16::from(up)) / 2) as u8,
            4 => paeth(left, up, up_left),
            _ => return Err(PngDecodeError::Corrupt("row filter")),
        };
        current[index] = current[index].wrapping_add(predictor);
    }
    Ok(())
}

fn paeth(left: u8, up: u8, up_left: u8) -> u8 {
    let p = i16::from(left) + i16::from(up) - i16::from(up_left);
    let pa = (p - i16::from(left)).abs();
    let pb = (p - i16::from(up)).abs();
    let pc = (p - i16::from(up_left)).abs();
    if pa <= pb && pa <= pc {
        left
    } else if pb <= pc {
        up
    } else {
        up_left
    }
}

fn srgb_to_linear(value: u8) -> f32 {
    let c = f32::from(value) / 255.0;
    if c <= 0.040_45 {
        c / 12.92
    } else {
        ((c + 0.055) / 1.055).powf(2.4)
    }
}

fn linear_to_srgb(value: f32) -> u8 {
    let value = value.clamp(0.0, 1.0);
    let encoded = if value <= 0.003_130_8 {
        value * 12.92
    } else {
        1.055 * value.powf(1.0 / 2.4) - 0.055
    };
    (encoded * 255.0 + 0.5) as u8
}

/// One mip level: extent and `RGBA8` texels.
pub(crate) type MipLevel = ([u32; 3], Vec<u8>);

/// The full mip chain of an `RGBA8` image by a `2 x 2` box filter (odd
/// extents clamp the last column or row); sRGB colour channels average in
/// linear light, alpha and linear data as they are.
pub(crate) fn mip_chain(width: u32, height: u32, rgba8: &[u8], srgb: bool) -> Vec<MipLevel> {
    let mut levels = vec![([width, height, 1], rgba8.to_vec())];
    let (mut w, mut h) = (width, height);
    while w > 1 || h > 1 {
        let (nw, nh) = ((w / 2).max(1), (h / 2).max(1));
        let source = &levels.last().expect("at least one level").1;
        let mut next = Vec::with_capacity((nw * nh * 4) as usize);
        for y in 0..nh {
            for x in 0..nw {
                let mut sum = [0.0_f32; 4];
                let mut count = 0.0;
                for dy in 0..2 {
                    for dx in 0..2 {
                        let sx = (x * 2 + dx).min(w - 1);
                        let sy = (y * 2 + dy).min(h - 1);
                        let base = ((sy * w + sx) * 4) as usize;
                        for channel in 0..4 {
                            let value = source[base + channel];
                            sum[channel] += if srgb && channel < 3 {
                                srgb_to_linear(value)
                            } else {
                                f32::from(value) / 255.0
                            };
                        }
                        count += 1.0;
                    }
                }
                for (channel, total) in sum.iter().enumerate() {
                    let mean = total / count;
                    next.push(if srgb && channel < 3 {
                        linear_to_srgb(mean)
                    } else {
                        (mean * 255.0 + 0.5) as u8
                    });
                }
            }
        }
        levels.push(([nw, nh, 1], next));
        w = nw;
        h = nh;
    }
    levels
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    fn encode_png(
        width: u32,
        height: u32,
        channels: usize,
        texels: &[u8],
        filters: &[u8],
    ) -> Vec<u8> {
        let stride = width as usize * channels;
        let mut raw = Vec::new();
        let mut previous = vec![0_u8; stride];
        for (row, filter) in (0..height as usize).zip(filters.iter().cycle()) {
            let current = &texels[row * stride..(row + 1) * stride];
            raw.push(*filter);
            for index in 0..stride {
                let left = if index >= channels {
                    current[index - channels]
                } else {
                    0
                };
                let up = previous[index];
                let up_left = if index >= channels {
                    previous[index - channels]
                } else {
                    0
                };
                let predictor = match filter {
                    0 => 0,
                    1 => left,
                    2 => up,
                    3 => ((u16::from(left) + u16::from(up)) / 2) as u8,
                    _ => paeth(left, up, up_left),
                };
                raw.push(current[index].wrapping_sub(predictor));
            }
            previous.copy_from_slice(current);
        }
        let mut encoder =
            flate2::write::ZlibEncoder::new(Vec::new(), flate2::Compression::default());
        encoder.write_all(&raw).expect("deflate");
        let compressed = encoder.finish().expect("finish");
        let mut png = SIGNATURE.to_vec();
        let mut chunk = |kind: &[u8], body: &[u8]| {
            png.extend_from_slice(&(body.len() as u32).to_be_bytes());
            png.extend_from_slice(kind);
            png.extend_from_slice(body);
            png.extend_from_slice(&[0, 0, 0, 0]);
        };
        let color_type = match channels {
            1 => 0,
            2 => 4,
            3 => 2,
            _ => 6,
        };
        let mut header = Vec::new();
        header.extend_from_slice(&width.to_be_bytes());
        header.extend_from_slice(&height.to_be_bytes());
        header.extend_from_slice(&[8, color_type, 0, 0, 0]);
        chunk(b"IHDR", &header);
        chunk(b"IDAT", &compressed);
        chunk(b"IEND", &[]);
        png
    }

    /// Plan look/05 G3: a 4 x 3 RGBA image round-trips through every filter.
    #[test]
    fn rgba_round_trips_through_every_filter() {
        let texels: Vec<u8> = (0..4 * 3 * 4).map(|i| (i * 37 % 251) as u8).collect();
        let png = encode_png(4, 3, 4, &texels, &[0, 1, 2, 3, 4]);
        let decoded = decode_png(&png).expect("decode");
        assert_eq!((decoded.width, decoded.height), (4, 3));
        assert_eq!(decoded.rgba8, texels);
        let rgb: Vec<u8> = (0..4 * 3 * 3).map(|i| (i * 53 % 251) as u8).collect();
        let decoded = decode_png(&encode_png(4, 3, 3, &rgb, &[4, 3])).expect("decode rgb");
        for (texel, source) in decoded.rgba8.chunks_exact(4).zip(rgb.chunks_exact(3)) {
            assert_eq!(&texel[..3], source);
            assert_eq!(texel[3], 255);
        }
        let grey: Vec<u8> = (0..12).map(|i| (i * 20) as u8).collect();
        let decoded = decode_png(&encode_png(4, 3, 1, &grey, &[1])).expect("decode grey");
        assert_eq!(decoded.rgba8[4..8], [20, 20, 20, 255]);
        assert_eq!(decode_png(b"not a png"), Err(PngDecodeError::NotPng));
    }

    /// Plan look/05 G3: a 4 x 4 texture has three levels whose means match
    /// the box filter (linear data) and average in linear light for sRGB.
    #[test]
    fn mip_chain_box_filters() {
        let mut texels = vec![0_u8; 4 * 4 * 4];
        for (index, texel) in texels.chunks_exact_mut(4).enumerate() {
            let v = if index % 2 == 0 { 0 } else { 255 };
            texel.copy_from_slice(&[v, v, v, 255]);
        }
        let linear = mip_chain(4, 4, &texels, false);
        assert_eq!(linear.len(), 3);
        assert_eq!(linear[1].0, [2, 2, 1]);
        assert_eq!(linear[2].0, [1, 1, 1]);
        assert_eq!(&linear[1].1[..4], &[128, 128, 128, 255]);
        let srgb = mip_chain(4, 4, &texels, true);
        // Half black, half white in linear light is 0.5 linear: sRGB 188.
        assert_eq!(&srgb[1].1[..4], &[188, 188, 188, 255]);
        let odd = mip_chain(
            3,
            1,
            &[10, 10, 10, 255, 20, 20, 20, 255, 30, 30, 30, 255],
            false,
        );
        assert_eq!(odd.len(), 2);
        assert_eq!(odd[1].0, [1, 1, 1]);
    }
}
