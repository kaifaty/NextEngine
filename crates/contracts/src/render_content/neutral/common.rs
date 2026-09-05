use crate::canonical::CanonicalCursor;

use super::super::RenderContentContractError;

pub(super) const POSITION_LIMIT_MICROMETRES: i64 = 8_388_608_000_000;
const UV_TRANSFORM_LIMIT_Q16_16: i32 = 1024 * 65_536;

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct AabbI64V1 {
    min: [i64; 3],
    max: [i64; 3],
}

impl AabbI64V1 {
    pub fn new(min: [i64; 3], max: [i64; 3]) -> Result<Self, RenderContentContractError> {
        for axis in 0..3 {
            if min[axis] < -POSITION_LIMIT_MICROMETRES
                || max[axis] > POSITION_LIMIT_MICROMETRES
                || min[axis] >= max[axis]
            {
                return Err(RenderContentContractError::InvalidBounds);
            }
        }
        Ok(Self { min, max })
    }

    #[must_use]
    pub const fn min(self) -> [i64; 3] {
        self.min
    }

    #[must_use]
    pub const fn max(self) -> [i64; 3] {
        self.max
    }

    #[must_use]
    pub fn contains(&self, point: [i64; 3]) -> bool {
        (0..3).all(|axis| point[axis] >= self.min[axis] && point[axis] < self.max[axis])
    }

    pub(super) fn encode(self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(48);
        for value in self.min.into_iter().chain(self.max) {
            bytes.extend_from_slice(&value.to_le_bytes());
        }
        bytes
    }

    pub(super) fn decode(bytes: &[u8]) -> Result<Self, RenderContentContractError> {
        if bytes.len() != 48 {
            return Err(RenderContentContractError::InvalidPayload);
        }
        let mut cursor = CanonicalCursor::new(bytes);
        let mut values = [0_i64; 6];
        for value in &mut values {
            *value = i64::from_le_bytes(
                cursor
                    .read_exact(8)?
                    .try_into()
                    .map_err(|_| RenderContentContractError::InvalidPayload)?,
            );
        }
        cursor.finish()?;
        Self::new(
            [values[0], values[1], values[2]],
            [values[3], values[4], values[5]],
        )
    }
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct UvTransformV1 {
    coefficients_q16_16: [i32; 6],
}

impl UvTransformV1 {
    pub fn new(coefficients_q16_16: [i32; 6]) -> Result<Self, RenderContentContractError> {
        if coefficients_q16_16
            .iter()
            .any(|value| !(-UV_TRANSFORM_LIMIT_Q16_16..=UV_TRANSFORM_LIMIT_Q16_16).contains(value))
        {
            return Err(RenderContentContractError::InvalidMaterial);
        }
        Ok(Self {
            coefficients_q16_16,
        })
    }

    #[must_use]
    pub const fn identity() -> Self {
        Self {
            coefficients_q16_16: [65_536, 0, 0, 0, 65_536, 0],
        }
    }

    #[must_use]
    pub const fn coefficients_q16_16(&self) -> [i32; 6] {
        self.coefficients_q16_16
    }

    /// Scene look L5 (plan `look/05`): the scale of a pure uniform scale
    /// (`[s, 0, 0, 0, s, 0]` with `s > 0`), `None` for any other transform.
    #[must_use]
    pub const fn uniform_scale_q16_16(&self) -> Option<i32> {
        let [a, b, tx, c, d, ty] = self.coefficients_q16_16;
        if a > 0 && a == d && b == 0 && c == 0 && tx == 0 && ty == 0 {
            Some(a)
        } else {
            None
        }
    }

    pub(super) fn encode(self) -> [u8; 24] {
        let mut bytes = [0_u8; 24];
        for (index, value) in self.coefficients_q16_16.into_iter().enumerate() {
            let offset = index * 4;
            bytes[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
        }
        bytes
    }

    pub(super) fn decode(bytes: &[u8]) -> Result<Self, RenderContentContractError> {
        if bytes.len() != 24 {
            return Err(RenderContentContractError::InvalidPayload);
        }
        let mut values = [0_i32; 6];
        for (index, value) in values.iter_mut().enumerate() {
            let offset = index * 4;
            *value = i32::from_le_bytes(
                bytes[offset..offset + 4]
                    .try_into()
                    .map_err(|_| RenderContentContractError::InvalidPayload)?,
            );
        }
        Self::new(values)
    }
}
