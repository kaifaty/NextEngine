//! Scene look L1 (plan `look/01`): the host side of the HDR chain. The
//! exposure and the tone curve the `tonemap` suite applies, repeated here
//! for the developer capture of the linear scene target, and the half-float
//! decode that capture needs. Renderer-local; nothing here enters gameplay.

/// The exposure of a frame without B0 content (nothing lit to expose).
pub(crate) const HDR_EXPOSURE_FALLBACK: f32 = 1.0;

/// The ACES fitted curve (Narkowicz 2015), clamped to `0..=1`; the same
/// expression as `tonemap.frag`.
pub(crate) fn aces_fitted(x: f32) -> f32 {
    let x = x.max(0.0);
    ((x * (2.51 * x + 0.03)) / (x * (2.43 * x + 0.59) + 0.14)).clamp(0.0, 1.0)
}

/// Linear `0..=1` to the sRGB transfer function, quantised to a byte.
pub(crate) fn linear_to_srgb_u8(value: f32) -> u8 {
    let value = value.clamp(0.0, 1.0);
    let encoded = if value <= 0.003_130_8 {
        value * 12.92
    } else {
        1.055 * value.powf(1.0 / 2.4) - 0.055
    };
    (encoded * 255.0 + 0.5) as u8
}

/// IEEE half-precision bits to `f32` (subnormals and infinities included).
pub(crate) fn half_bits_to_f32(bits: u16) -> f32 {
    let sign = if bits & 0x8000 != 0 { -1.0 } else { 1.0 };
    let exponent = i32::from((bits >> 10) & 0x1f);
    let mantissa = f32::from(bits & 0x3ff);
    match exponent {
        0 => sign * mantissa * 2.0_f32.powi(-24),
        31 => {
            if mantissa == 0.0 {
                sign * f32::INFINITY
            } else {
                f32::NAN
            }
        }
        _ => sign * (1.0 + mantissa / 1024.0) * 2.0_f32.powi(exponent - 15),
    }
}

/// Converts one `R16G16B16A16_SFLOAT` pixel row buffer into tone-mapped sRGB
/// `RGBA8` (alpha scaled linearly), the capture path's equivalent of the
/// `tonemap` suite.
pub(crate) fn tonemap_half_rgba_to_rgba8(source: &[u8], exposure: f32) -> Vec<u8> {
    let mut out = Vec::with_capacity(source.len() / 2);
    for pixel in source.chunks_exact(8) {
        let lane = |index: usize| {
            half_bits_to_f32(u16::from_le_bytes([pixel[index * 2], pixel[index * 2 + 1]]))
        };
        for channel in 0..3 {
            let value = lane(channel);
            let value = if value.is_finite() { value } else { 0.0 };
            out.push(linear_to_srgb_u8(aces_fitted(value * exposure)));
        }
        let alpha = lane(3);
        let alpha = if alpha.is_finite() { alpha } else { 0.0 };
        out.push((alpha.clamp(0.0, 1.0) * 255.0 + 0.5) as u8);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Plan look/01 G3: the curve is monotone on `0..=16` and zero at zero.
    #[test]
    fn aces_is_monotone_and_zero_at_zero() {
        assert_eq!(aces_fitted(0.0), 0.0);
        let mut previous = 0.0;
        for step in 1..=1_600 {
            let x = step as f32 / 100.0;
            let value = aces_fitted(x);
            assert!(value >= previous, "{x}: {value} < {previous}");
            assert!(value <= 1.0);
            previous = value;
        }
        assert!((aces_fitted(0.8) - 0.7).abs() < 0.1);
    }

    #[test]
    fn half_floats_decode() {
        assert_eq!(half_bits_to_f32(0x3c00), 1.0);
        assert_eq!(half_bits_to_f32(0xc000), -2.0);
        assert_eq!(half_bits_to_f32(0x0000), 0.0);
        assert!((half_bits_to_f32(0x3555) - 0.333_25).abs() < 1e-4);
        assert!(half_bits_to_f32(0x7c00).is_infinite());
        assert!(half_bits_to_f32(0x0001) > 0.0);
    }

    #[test]
    fn capture_conversion_tone_maps_and_encodes() {
        // One pixel: (1.0, 0.0, 0.0, 1.0) in halves.
        let source = [0x00, 0x3c, 0, 0, 0, 0, 0x00, 0x3c];
        let out = tonemap_half_rgba_to_rgba8(&source, 1.0);
        assert_eq!(out.len(), 4);
        assert_eq!(out[3], 255);
        assert_eq!(out[1], 0);
        assert!(out[0] > 200, "red {}", out[0]);
        assert_eq!(linear_to_srgb_u8(0.0), 0);
        assert_eq!(linear_to_srgb_u8(1.0), 255);
    }
}
