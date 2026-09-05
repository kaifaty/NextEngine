//! Scene look L1 (plan `look/01`): the analytic sky and the lighting block.
//! The Preetham model (turbidity, sun elevation) evaluated on the host for
//! the spherical-harmonic irradiance, the fog colour and the exposure, and
//! packed into the `LightingUniforms` block (set 0, binding 1) the lit
//! programs and the sky program read. Units: the sky's irradiance on an
//! upward face is `1.0` in luminance; the sun's irradiance on a face normal
//! to it is `SUN_TO_SKY_IRRADIANCE`. Renderer-local; nothing here enters
//! gameplay, persistence or replay authority.

use std::f32::consts::PI;

/// `inverse_view_projection` (64), `sun_radiance` (16), nine `sky_sh`
/// (144), `fog` (16), `sky_zenith` (16), three Perez pairs (96),
/// `sky_params` (16), three shadow cascades (192, plan `look/02`),
/// `shadow_extents` (16).
pub(crate) const LIGHTING_UNIFORM_SIZE: u64 = 576;
/// Atmospheric turbidity of the reference sky (clear).
pub(crate) const SKY_TURBIDITY: f32 = 2.5;
/// The ground's albedo under the sky, for the lower SH hemisphere.
pub(crate) const GROUND_ALBEDO: f32 = 0.25;
/// Direct sun irradiance (normal to the sun) over the sky's upward
/// irradiance: the daylight ratio.
pub(crate) const SUN_TO_SKY_IRRADIANCE: f32 = 5.0;
/// Middle grey: the albedo of the reference face and where it lands on the
/// display after the tone curve (revision 1 of plan `look/01`).
pub(crate) const MIDDLE_GREY: f32 = 0.18;
/// Cosine of the sun disc's half angle (`0.35°`), and of its soft edge.
pub(crate) const SUN_DISC_COS_INNER: f32 = 0.999_981_3;
pub(crate) const SUN_DISC_COS_OUTER: f32 = 0.999_945_2;

/// The Preetham sky for one sun direction and turbidity.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct SkyModel {
    /// Unit vector towards the sun.
    sun: [f32; 3],
    perez_x: [f32; 5],
    perez_y: [f32; 5],
    perez_luminance: [f32; 5],
    /// Zenith chromaticity and luminance of the raw model.
    zenith: [f32; 3],
    /// Multiplier that makes the upward luminance irradiance `1.0`.
    scale: f32,
}

fn normalize(v: [f32; 3]) -> [f32; 3] {
    let length = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt().max(1e-6);
    [v[0] / length, v[1] / length, v[2] / length]
}

fn dot(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

fn perez(coefficients: [f32; 5], cos_theta: f32, gamma: f32) -> f32 {
    let [a, b, c, d, e] = coefficients;
    (1.0 + a * (b / cos_theta.max(0.01)).exp())
        * (1.0 + c * (d * gamma).exp() + e * gamma.cos().powi(2))
}

fn xyy_to_linear_srgb(x: f32, y: f32, luminance: f32) -> [f32; 3] {
    let y = y.max(1e-4);
    let big_x = x * luminance / y;
    let big_z = (1.0 - x - y) * luminance / y;
    let big_y = luminance;
    [
        (3.2406 * big_x - 1.5372 * big_y - 0.4986 * big_z).max(0.0),
        (-0.9689 * big_x + 1.8758 * big_y + 0.0415 * big_z).max(0.0),
        (0.0557 * big_x - 0.2040 * big_y + 1.0570 * big_z).max(0.0),
    ]
}

fn luminance(rgb: [f32; 3]) -> f32 {
    0.2126 * rgb[0] + 0.7152 * rgb[1] + 0.0722 * rgb[2]
}

/// Directions and solid angles of a `columns x rows` grid over the sphere
/// (uniform in azimuth and in the cosine of the polar angle from `+y`).
fn sphere_samples(columns: usize, rows: usize) -> Vec<([f32; 3], f32)> {
    let mut samples = Vec::with_capacity(columns * rows);
    let weight = 4.0 * PI / (columns * rows) as f32;
    for row in 0..rows {
        let cos_theta = 1.0 - 2.0 * (row as f32 + 0.5) / rows as f32;
        let sin_theta = (1.0 - cos_theta * cos_theta).max(0.0).sqrt();
        for column in 0..columns {
            let phi = 2.0 * PI * (column as f32 + 0.5) / columns as f32;
            samples.push((
                [sin_theta * phi.cos(), cos_theta, sin_theta * phi.sin()],
                weight,
            ));
        }
    }
    samples
}

/// The nine real SH basis functions in the order `L00, L1-1, L10, L11,
/// L2-2, L2-1, L20, L21, L22` (Ramamoorthi and Hanrahan 2001).
pub(crate) fn sh_basis(direction: [f32; 3]) -> [f32; 9] {
    let [x, y, z] = direction;
    [
        0.282_095,
        0.488_603 * y,
        0.488_603 * z,
        0.488_603 * x,
        1.092_548 * x * y,
        1.092_548 * y * z,
        0.315_392 * (3.0 * z * z - 1.0),
        1.092_548 * x * z,
        0.546_274 * (x * x - y * y),
    ]
}

/// Projects a radiance function onto SH2 over the sphere.
pub(crate) fn project_sh(radiance: impl Fn([f32; 3]) -> [f32; 3]) -> [[f32; 3]; 9] {
    let mut coefficients = [[0.0_f32; 3]; 9];
    for (direction, weight) in sphere_samples(64, 32) {
        let value = radiance(direction);
        let basis = sh_basis(direction);
        for (coefficient, basis) in coefficients.iter_mut().zip(basis) {
            for channel in 0..3 {
                coefficient[channel] += value[channel] * basis * weight;
            }
        }
    }
    coefficients
}

/// Irradiance at a normal from SH2 coefficients (the same expression as the
/// `sh_irradiance` function of the lit programs).
#[cfg(test)]
pub(crate) fn sh_irradiance(coefficients: &[[f32; 3]; 9], normal: [f32; 3]) -> [f32; 3] {
    let [x, y, z] = normalize(normal);
    const C1: f32 = 0.429_043;
    const C2: f32 = 0.511_664;
    const C3: f32 = 0.743_125;
    const C4: f32 = 0.886_227;
    const C5: f32 = 0.247_708;
    let mut result = [0.0_f32; 3];
    for (channel, value) in result.iter_mut().enumerate() {
        let l = |index: usize| coefficients[index][channel];
        *value = C1 * l(8) * (x * x - y * y) + C3 * l(6) * z * z + C4 * l(0) - C5 * l(6)
            + 2.0 * C1 * (l(4) * x * y + l(7) * x * z + l(5) * y * z)
            + 2.0 * C2 * (l(3) * x + l(1) * y + l(2) * z);
    }
    result
}

impl SkyModel {
    /// `sun_direction` points from the world towards the sun.
    pub(crate) fn new(sun_direction: [f32; 3], turbidity: f32) -> Self {
        let sun = normalize(sun_direction);
        let t = turbidity;
        let theta_s = sun[1].clamp(-1.0, 1.0).acos();
        let perez_luminance = [
            0.1787 * t - 1.4630,
            -0.3554 * t + 0.4275,
            -0.0227 * t + 5.3251,
            0.1206 * t - 2.5771,
            -0.0670 * t + 0.3703,
        ];
        let perez_x = [
            -0.0193 * t - 0.2592,
            -0.0665 * t + 0.0008,
            -0.0004 * t + 0.2125,
            -0.0641 * t - 0.8989,
            -0.0033 * t + 0.0452,
        ];
        let perez_y = [
            -0.0167 * t - 0.2608,
            -0.0950 * t + 0.0092,
            -0.0079 * t + 0.2102,
            -0.0441 * t - 1.6537,
            -0.0109 * t + 0.0529,
        ];
        let chi = (4.0 / 9.0 - t / 120.0) * (PI - 2.0 * theta_s);
        let zenith_luminance = ((4.0453 * t - 4.9710) * chi.tan() - 0.2155 * t + 2.4192).max(0.001);
        let powers = [theta_s.powi(3), theta_s.powi(2), theta_s, 1.0];
        let turbidities = [t * t, t, 1.0];
        let chroma = |matrix: [[f32; 4]; 3]| {
            let mut value = 0.0;
            for (row, tt) in matrix.iter().zip(turbidities) {
                for (entry, power) in row.iter().zip(powers) {
                    value += tt * entry * power;
                }
            }
            value
        };
        let zenith_x = chroma([
            [0.001_66, -0.003_75, 0.002_09, 0.0],
            [-0.029_03, 0.063_77, -0.032_02, 0.003_94],
            [0.116_93, -0.211_96, 0.060_52, 0.258_86],
        ]);
        let zenith_y = chroma([
            [0.002_75, -0.006_10, 0.003_17, 0.0],
            [-0.042_14, 0.089_70, -0.041_53, 0.005_16],
            [0.153_46, -0.267_56, 0.066_70, 0.266_88],
        ]);
        let mut model = Self {
            sun,
            perez_x,
            perez_y,
            perez_luminance,
            zenith: [zenith_x, zenith_y, zenith_luminance],
            scale: 1.0,
        };
        let mut irradiance = 0.0;
        for (direction, weight) in sphere_samples(64, 32) {
            if direction[1] <= 0.0 {
                continue;
            }
            irradiance += luminance(model.radiance(direction)) * direction[1] * weight;
        }
        model.scale = 1.0 / irradiance.max(1e-6);
        model
    }

    #[cfg(test)]
    pub(crate) const fn sun_direction(&self) -> [f32; 3] {
        self.sun
    }

    /// The model's radiance along `direction` (linear sRGB), scaled so the
    /// upward luminance irradiance is `1.0`; below the horizon the horizon's.
    pub(crate) fn radiance(&self, direction: [f32; 3]) -> [f32; 3] {
        let direction = normalize(direction);
        let cos_theta = direction[1].max(0.01);
        let gamma = dot(direction, self.sun).clamp(-1.0, 1.0).acos();
        let theta_s = self.sun[1].clamp(-1.0, 1.0).acos();
        let ratio = |coefficients: [f32; 5]| {
            perez(coefficients, cos_theta, gamma) / perez(coefficients, 1.0, theta_s)
        };
        let x = self.zenith[0] * ratio(self.perez_x);
        let y = self.zenith[1] * ratio(self.perez_y);
        let big_y = self.zenith[2] * ratio(self.perez_luminance);
        let rgb = xyy_to_linear_srgb(x, y, big_y.max(0.0));
        [
            rgb[0] * self.scale,
            rgb[1] * self.scale,
            rgb[2] * self.scale,
        ]
    }

    /// The upward irradiance per channel (the luminance one is `1.0`).
    fn irradiance_up(&self) -> [f32; 3] {
        let mut result = [0.0_f32; 3];
        for (direction, weight) in sphere_samples(64, 32) {
            if direction[1] <= 0.0 {
                continue;
            }
            let value = self.radiance(direction);
            for channel in 0..3 {
                result[channel] += value[channel] * direction[1] * weight;
            }
        }
        result
    }

    /// SH2 of the sky over the ground: the model above the horizon, the
    /// ground's reflection of its irradiance (the sky and the sun) below it.
    pub(crate) fn sh_coefficients(&self, ground_albedo: f32) -> [[f32; 3]; 9] {
        let irradiance = self.irradiance_up();
        let sun_on_ground = SUN_TO_SKY_IRRADIANCE * self.sun[1].max(0.0);
        let ground = irradiance.map(|value| ground_albedo * (value + sun_on_ground) / PI);
        project_sh(|direction| {
            if direction[1] > 0.0 {
                self.radiance(direction)
            } else {
                ground
            }
        })
    }

    /// The mean radiance two degrees above the horizon: the fog's colour.
    pub(crate) fn fog_colour(&self) -> [f32; 3] {
        let elevation = 2.0_f32.to_radians();
        let mut result = [0.0_f32; 3];
        let count = 64;
        for column in 0..count {
            let phi = 2.0 * PI * (column as f32 + 0.5) / count as f32;
            let value = self.radiance([
                elevation.cos() * phi.cos(),
                elevation.sin(),
                elevation.cos() * phi.sin(),
            ]);
            for channel in 0..3 {
                result[channel] += value[channel] / count as f32;
            }
        }
        result
    }

    /// The exposure that maps a middle-grey upward face in full sun to
    /// middle grey on the display, through the ACES curve.
    pub(crate) fn exposure(&self) -> f32 {
        let sun_on_up = SUN_TO_SKY_IRRADIANCE * self.sun[1].max(0.0);
        let radiance = MIDDLE_GREY / PI * (1.0 + sun_on_up);
        aces_inverse(MIDDLE_GREY) / radiance.max(1e-6)
    }

    /// The `LightingUniforms` bytes for one frame.
    pub(crate) fn lighting_uniform_bytes(
        &self,
        inverse_view_projection: [f32; 16],
        fog_density: f32,
        shadow_cascades: &[[f32; 16]; 3],
        shadow_extents_metres: [f32; 3],
    ) -> [u8; LIGHTING_UNIFORM_SIZE as usize] {
        let mut bytes = [0_u8; LIGHTING_UNIFORM_SIZE as usize];
        let mut offset = 0;
        let mut push = |value: f32| {
            bytes[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
            offset += 4;
        };
        for value in inverse_view_projection {
            push(value);
        }
        for _ in 0..3 {
            push(SUN_TO_SKY_IRRADIANCE);
        }
        push(self.exposure());
        for coefficient in self.sh_coefficients(GROUND_ALBEDO) {
            for value in coefficient {
                push(value);
            }
            push(0.0);
        }
        for value in self.fog_colour() {
            push(value);
        }
        push(fog_density);
        for value in self.zenith {
            push(value);
        }
        push(SUN_DISC_COS_INNER);
        for coefficients in [self.perez_x, self.perez_y, self.perez_luminance] {
            for value in coefficients {
                push(value);
            }
            push(0.0);
            push(0.0);
            push(0.0);
        }
        push(GROUND_ALBEDO);
        push(SKY_TURBIDITY);
        push(self.scale);
        push(SUN_DISC_COS_OUTER);
        // Plan look/02: the cascades at 368..560 and their extents at 560.
        for cascade in shadow_cascades {
            for value in cascade {
                push(*value);
            }
        }
        for value in shadow_extents_metres {
            push(value);
        }
        push(0.0);
        bytes
    }
}

/// The scene value the ACES fitted curve maps to `display` (bisection on
/// the monotone curve over `0..=16`).
pub(crate) fn aces_inverse(display: f32) -> f32 {
    let (mut low, mut high) = (0.0_f32, 16.0_f32);
    for _ in 0..48 {
        let middle = 0.5 * (low + high);
        if crate::hdr::aces_fitted(middle) < display {
            low = middle;
        } else {
            high = middle;
        }
    }
    0.5 * (low + high)
}

/// General 4x4 inverse of a matrix in `[f32; 16]` (any consistent layout);
/// the identity when the matrix is singular.
pub(crate) fn invert_matrix(m: [f32; 16]) -> [f32; 16] {
    let mut inv = [0.0_f32; 16];
    inv[0] = m[5] * m[10] * m[15] - m[5] * m[11] * m[14] - m[9] * m[6] * m[15]
        + m[9] * m[7] * m[14]
        + m[13] * m[6] * m[11]
        - m[13] * m[7] * m[10];
    inv[4] = -m[4] * m[10] * m[15] + m[4] * m[11] * m[14] + m[8] * m[6] * m[15]
        - m[8] * m[7] * m[14]
        - m[12] * m[6] * m[11]
        + m[12] * m[7] * m[10];
    inv[8] = m[4] * m[9] * m[15] - m[4] * m[11] * m[13] - m[8] * m[5] * m[15]
        + m[8] * m[7] * m[13]
        + m[12] * m[5] * m[11]
        - m[12] * m[7] * m[9];
    inv[12] = -m[4] * m[9] * m[14] + m[4] * m[10] * m[13] + m[8] * m[5] * m[14]
        - m[8] * m[6] * m[13]
        - m[12] * m[5] * m[10]
        + m[12] * m[6] * m[9];
    inv[1] = -m[1] * m[10] * m[15] + m[1] * m[11] * m[14] + m[9] * m[2] * m[15]
        - m[9] * m[3] * m[14]
        - m[13] * m[2] * m[11]
        + m[13] * m[3] * m[10];
    inv[5] = m[0] * m[10] * m[15] - m[0] * m[11] * m[14] - m[8] * m[2] * m[15]
        + m[8] * m[3] * m[14]
        + m[12] * m[2] * m[11]
        - m[12] * m[3] * m[10];
    inv[9] = -m[0] * m[9] * m[15] + m[0] * m[11] * m[13] + m[8] * m[1] * m[15]
        - m[8] * m[3] * m[13]
        - m[12] * m[1] * m[11]
        + m[12] * m[3] * m[9];
    inv[13] = m[0] * m[9] * m[14] - m[0] * m[10] * m[13] - m[8] * m[1] * m[14]
        + m[8] * m[2] * m[13]
        + m[12] * m[1] * m[10]
        - m[12] * m[2] * m[9];
    inv[2] = m[1] * m[6] * m[15] - m[1] * m[7] * m[14] - m[5] * m[2] * m[15]
        + m[5] * m[3] * m[14]
        + m[13] * m[2] * m[7]
        - m[13] * m[3] * m[6];
    inv[6] = -m[0] * m[6] * m[15] + m[0] * m[7] * m[14] + m[4] * m[2] * m[15]
        - m[4] * m[3] * m[14]
        - m[12] * m[2] * m[7]
        + m[12] * m[3] * m[6];
    inv[10] = m[0] * m[5] * m[15] - m[0] * m[7] * m[13] - m[4] * m[1] * m[15]
        + m[4] * m[3] * m[13]
        + m[12] * m[1] * m[7]
        - m[12] * m[3] * m[5];
    inv[14] = -m[0] * m[5] * m[14] + m[0] * m[6] * m[13] + m[4] * m[1] * m[14]
        - m[4] * m[2] * m[13]
        - m[12] * m[1] * m[6]
        + m[12] * m[2] * m[5];
    inv[3] = -m[1] * m[6] * m[11] + m[1] * m[7] * m[10] + m[5] * m[2] * m[11]
        - m[5] * m[3] * m[10]
        - m[9] * m[2] * m[7]
        + m[9] * m[3] * m[6];
    inv[7] = m[0] * m[6] * m[11] - m[0] * m[7] * m[10] - m[4] * m[2] * m[11]
        + m[4] * m[3] * m[10]
        + m[8] * m[2] * m[7]
        - m[8] * m[3] * m[6];
    inv[11] = -m[0] * m[5] * m[11] + m[0] * m[7] * m[9] + m[4] * m[1] * m[11]
        - m[4] * m[3] * m[9]
        - m[8] * m[1] * m[7]
        + m[8] * m[3] * m[5];
    inv[15] = m[0] * m[5] * m[10] - m[0] * m[6] * m[9] - m[4] * m[1] * m[10]
        + m[4] * m[2] * m[9]
        + m[8] * m[1] * m[6]
        - m[8] * m[2] * m[5];
    let determinant = m[0] * inv[0] + m[1] * inv[4] + m[2] * inv[8] + m[3] * inv[12];
    if determinant.abs() < 1e-12 {
        let mut identity = [0.0_f32; 16];
        identity[0] = 1.0;
        identity[5] = 1.0;
        identity[10] = 1.0;
        identity[15] = 1.0;
        return identity;
    }
    inv.map(|value| value / determinant)
}

#[cfg(test)]
mod tests {
    use super::*;

    const SUN: [f32; 3] = [0.45, 0.82, 0.35];

    /// Plan look/01 G3: a constant sky projects to a constant irradiance.
    #[test]
    fn sh_of_a_constant_sky_is_pi_times_its_radiance() {
        let coefficients = project_sh(|_| [0.5, 1.0, 2.0]);
        for normal in [[0.0, 1.0, 0.0], [1.0, 0.0, 0.0], [0.3, -0.5, 0.8]] {
            let irradiance = sh_irradiance(&coefficients, normal);
            for (value, radiance) in irradiance.iter().zip([0.5, 1.0, 2.0]) {
                let expected = PI * radiance;
                assert!(
                    (value - expected).abs() / expected < 0.01,
                    "{value} vs {expected}"
                );
            }
        }
    }

    /// Plan look/01 G3: the zenith luminance is positive from 5 to 90 degrees.
    #[test]
    fn zenith_luminance_is_positive_for_daylight_elevations() {
        for degrees in 5..=90 {
            let elevation = (degrees as f32).to_radians();
            let model = SkyModel::new([elevation.cos(), elevation.sin(), 0.0], SKY_TURBIDITY);
            assert!(model.zenith[2] > 0.0, "{degrees}: {}", model.zenith[2]);
            let up = model.radiance([0.0, 1.0, 0.0]);
            assert!(up.iter().all(|value| value.is_finite() && *value >= 0.0));
        }
    }

    /// The scale makes the upward luminance irradiance one; the sky is
    /// bluer at the zenith than at the horizon; the horizon is brighter.
    #[test]
    fn the_model_is_normalised_and_reads_as_a_clear_sky() {
        let model = SkyModel::new(SUN, SKY_TURBIDITY);
        let irradiance = model.irradiance_up();
        assert!((luminance(irradiance) - 1.0).abs() < 0.02, "{irradiance:?}");
        let zenith = model.radiance([0.0, 1.0, 0.0]);
        let horizon = model.radiance([0.0, 0.05, -1.0]);
        assert!(zenith[2] > zenith[0], "zenith is blue: {zenith:?}");
        assert!(
            luminance(horizon) > luminance(zenith),
            "horizon brighter: {horizon:?} vs {zenith:?}"
        );
        let fog = model.fog_colour();
        assert!(
            fog.iter().all(|value| *value > 0.0 && *value < 5.0),
            "{fog:?}"
        );
    }

    /// Plan look/01 G3 (revision 1): the exposure maps middle grey in full
    /// sun to middle grey on the display through the curve.
    #[test]
    fn exposure_maps_middle_grey() {
        let model = SkyModel::new(SUN, SKY_TURBIDITY);
        let sun = model.sun_direction();
        let grey_up = MIDDLE_GREY * (1.0 + SUN_TO_SKY_IRRADIANCE * sun[1]) / PI;
        let display = crate::hdr::aces_fitted(grey_up * model.exposure());
        assert!((display - MIDDLE_GREY).abs() < 1e-3, "{display}");
        assert!((aces_inverse(crate::hdr::aces_fitted(0.7)) - 0.7).abs() < 1e-4);
        let mut cascade = [0.0_f32; 16];
        cascade[0] = 7.0;
        let bytes =
            model.lighting_uniform_bytes([0.0; 16], 0.035, &[cascade; 3], [12.0, 36.0, 108.0]);
        assert_eq!(bytes.len(), 576);
        // Plan look/02 G3: the cascades at 368, the extents at 560.
        assert_eq!(
            f32::from_le_bytes([bytes[368], bytes[369], bytes[370], bytes[371]]),
            7.0
        );
        assert_eq!(
            f32::from_le_bytes([bytes[560], bytes[561], bytes[562], bytes[563]]),
            12.0
        );
        assert_eq!(
            f32::from_le_bytes([bytes[568], bytes[569], bytes[570], bytes[571]]),
            108.0
        );
        let exposure = f32::from_le_bytes([bytes[76], bytes[77], bytes[78], bytes[79]]);
        assert!((exposure - model.exposure()).abs() < 1e-6);
    }

    #[test]
    fn matrix_inverse_round_trips() {
        let m = [
            2.0, 0.0, 0.0, 0.0, 0.0, 3.0, 0.0, 0.0, 0.0, 0.0, 4.0, 0.0, 1.0, 2.0, 3.0, 1.0,
        ];
        let inv = invert_matrix(m);
        // (column-major) product m * inv == identity
        for row in 0..4 {
            for column in 0..4 {
                let mut sum = 0.0;
                for k in 0..4 {
                    sum += m[k * 4 + row] * inv[column * 4 + k];
                }
                let expected = if row == column { 1.0 } else { 0.0 };
                assert!((sum - expected).abs() < 1e-5, "{row},{column}: {sum}");
            }
        }
    }
}
