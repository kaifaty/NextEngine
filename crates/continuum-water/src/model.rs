#![forbid(unsafe_code)]

use serde::Serialize;

use crate::error::{NONFINITE_VALUE, NUMERIC_OVERFLOW, WaterError};

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize)]
pub(crate) struct Vec3i {
    pub(crate) x: i64,
    pub(crate) y: i64,
    pub(crate) z: i64,
}

impl Vec3i {
    pub(crate) const fn new(x: i64, y: i64, z: i64) -> Self {
        Self { x, y, z }
    }

    pub(crate) fn checked_sub(self, other: Self) -> Result<Self, WaterError> {
        Ok(Self {
            x: self.x.checked_sub(other.x).ok_or_else(|| {
                WaterError::new(NUMERIC_OVERFLOW, "integer x displacement overflow")
            })?,
            y: self.y.checked_sub(other.y).ok_or_else(|| {
                WaterError::new(NUMERIC_OVERFLOW, "integer y displacement overflow")
            })?,
            z: self.z.checked_sub(other.z).ok_or_else(|| {
                WaterError::new(NUMERIC_OVERFLOW, "integer z displacement overflow")
            })?,
        })
    }

    pub(crate) fn squared_length_i128(self) -> Result<i128, WaterError> {
        let x = i128::from(self.x);
        let y = i128::from(self.y);
        let z = i128::from(self.z);
        let xx = x
            .checked_mul(x)
            .ok_or_else(|| WaterError::new(NUMERIC_OVERFLOW, "integer x square overflow"))?;
        let yy = y
            .checked_mul(y)
            .ok_or_else(|| WaterError::new(NUMERIC_OVERFLOW, "integer y square overflow"))?;
        let zz = z
            .checked_mul(z)
            .ok_or_else(|| WaterError::new(NUMERIC_OVERFLOW, "integer z square overflow"))?;
        xx.checked_add(yy)
            .and_then(|value| value.checked_add(zz))
            .ok_or_else(|| WaterError::new(NUMERIC_OVERFLOW, "integer distance overflow"))
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct Vec3f {
    pub(crate) x: f64,
    pub(crate) y: f64,
    pub(crate) z: f64,
}

impl Vec3f {
    pub(crate) const ZERO: Self = Self {
        x: 0.0,
        y: 0.0,
        z: 0.0,
    };

    pub(crate) const fn new(x: f64, y: f64, z: f64) -> Self {
        Self { x, y, z }
    }

    pub(crate) fn add(self, other: Self) -> Self {
        Self::new(self.x + other.x, self.y + other.y, self.z + other.z)
    }

    pub(crate) fn sub(self, other: Self) -> Self {
        Self::new(self.x - other.x, self.y - other.y, self.z - other.z)
    }

    pub(crate) fn scale(self, value: f64) -> Self {
        Self::new(self.x * value, self.y * value, self.z * value)
    }

    pub(crate) fn dot(self, other: Self) -> f64 {
        let xy = (self.x * other.x) + (self.y * other.y);
        xy + (self.z * other.z)
    }

    pub(crate) fn checked(self, phase: &str) -> Result<Self, WaterError> {
        if self.x.is_finite() && self.y.is_finite() && self.z.is_finite() {
            Ok(self)
        } else {
            Err(WaterError::new(
                NONFINITE_VALUE,
                format!("nonfinite vector after {phase}"),
            ))
        }
    }
}

pub(crate) fn checked_scalar(value: f64, phase: &str) -> Result<f64, WaterError> {
    if value.is_finite() {
        Ok(value)
    } else {
        Err(WaterError::new(
            NONFINITE_VALUE,
            format!("nonfinite scalar after {phase}"),
        ))
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub(crate) struct CanonicalSample {
    pub(crate) id: u32,
    pub(crate) position_um: Vec3i,
    pub(crate) velocity_um_s: Vec3i,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct Box3i {
    pub(crate) min: Vec3i,
    pub(crate) max: Vec3i,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct Aperture {
    pub(crate) wall_x_um: i64,
    pub(crate) y_min_um: i64,
    pub(crate) y_max_um: i64,
    pub(crate) z_min_um: i64,
    pub(crate) z_max_um: i64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct Geometry {
    pub(crate) bounds: Box3i,
    pub(crate) aperture: Option<Aperture>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum StorageOrder {
    Identity,
    Reverse,
    Affine,
}

impl StorageOrder {
    pub(crate) const fn label(self) -> &'static str {
        match self {
            Self::Identity => "identity",
            Self::Reverse => "reverse",
            Self::Affine => "affine-257k-plus-17-mod-1152",
        }
    }
}

#[derive(Clone, Debug)]
pub(crate) struct Scenario {
    pub(crate) id: &'static str,
    pub(crate) kind: &'static str,
    pub(crate) geometry: Geometry,
    pub(crate) dimensions: [u32; 3],
    pub(crate) first_position_um: Vec3i,
    pub(crate) initial_velocity_um_s: Vec3i,
    pub(crate) steps: u32,
    pub(crate) output_every: u32,
    pub(crate) smoke_only: bool,
}

#[derive(Clone, Debug, Serialize)]
pub(crate) struct StepSummary {
    pub(crate) step: u32,
    pub(crate) frame_root: String,
    pub(crate) density_iterations: u8,
    pub(crate) density_error_ppb: i64,
    pub(crate) density_kkt_error_ppb: Option<i64>,
    pub(crate) density_maximum_multiplier_bits: String,
    pub(crate) density_ratio_p50_ppb: i64,
    pub(crate) density_ratio_p95_ppb: i64,
    pub(crate) density_ratio_p99_ppb: i64,
    pub(crate) divergence_iterations: u8,
    pub(crate) divergence_error_ppb: i64,
    pub(crate) divergence_maximum_multiplier_bits: String,
    pub(crate) maximum_penetration_um: i64,
    pub(crate) centre_of_mass_um: Vec3i,
}

#[derive(Clone, Debug, Serialize)]
pub(crate) struct OutputMetric {
    pub(crate) step: u32,
    pub(crate) sample_count: u32,
    pub(crate) mass_mg: u64,
    pub(crate) centre_of_mass_um: Vec3i,
    pub(crate) q99_x_um: i64,
    pub(crate) q99_y_um: i64,
    pub(crate) receiver_count: u32,
    pub(crate) energy_residual_ppb: i64,
    pub(crate) momentum_residual_ppb: i64,
}

#[derive(Clone, Debug)]
pub(crate) struct AcceptedFrame {
    pub(crate) step: u32,
    pub(crate) samples: Vec<CanonicalSample>,
    pub(crate) frame_root: [u8; 32],
}
