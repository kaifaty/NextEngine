use std::error::Error;
use std::fmt::{Display, Formatter};

use next_contracts::{
    AuthoritativeNumericProfileV1, PhysicsQuantizationProfileV1, PhysicsQuantizationRuleV1,
    PhysicsSourceFormatV1,
};

#[derive(Clone, Debug)]
pub struct PhysicsQuantizer {
    profile: PhysicsQuantizationProfileV1,
    numeric: AuthoritativeNumericProfileV1,
}

impl PhysicsQuantizer {
    pub fn new(
        profile: &PhysicsQuantizationProfileV1,
        numeric: &AuthoritativeNumericProfileV1,
    ) -> Result<Self, PhysicsQuantizationError> {
        profile
            .validate()
            .map_err(|_| PhysicsQuantizationError::InvalidProfile)?;
        numeric
            .validate()
            .map_err(|_| PhysicsQuantizationError::InvalidProfile)?;
        let profile_hash = profile
            .profile_hash()
            .map_err(|_| PhysicsQuantizationError::InvalidProfile)?;
        if numeric.physics_quantization_profile_hash != profile_hash {
            return Err(PhysicsQuantizationError::ProfileMismatch);
        }
        Ok(Self {
            profile: profile.clone(),
            numeric: numeric.clone(),
        })
    }

    #[must_use]
    pub const fn profile(&self) -> &PhysicsQuantizationProfileV1 {
        &self.profile
    }

    #[must_use]
    pub const fn numeric_profile(&self) -> &AuthoritativeNumericProfileV1 {
        &self.numeric
    }

    pub fn quantize_f32_bits(
        &self,
        field_id: &str,
        bits: u32,
    ) -> Result<i64, PhysicsQuantizationError> {
        let rule = self.rule(field_id, PhysicsSourceFormatV1::Ieee754Binary32)?;
        let value = decode_ieee(
            u64::from(bits),
            IeeeLayout {
                exponent_bits: 8,
                fraction_bits: 23,
                exponent_bias: 127,
            },
        )?;
        self.quantize(rule, value)
    }

    pub fn quantize_f64_bits(
        &self,
        field_id: &str,
        bits: u64,
    ) -> Result<i64, PhysicsQuantizationError> {
        let rule = self.rule(field_id, PhysicsSourceFormatV1::Ieee754Binary64)?;
        let value = decode_ieee(
            bits,
            IeeeLayout {
                exponent_bits: 11,
                fraction_bits: 52,
                exponent_bias: 1023,
            },
        )?;
        self.quantize(rule, value)
    }

    fn rule(
        &self,
        field_id: &str,
        source_format: PhysicsSourceFormatV1,
    ) -> Result<&PhysicsQuantizationRuleV1, PhysicsQuantizationError> {
        let (_, rule) = self
            .profile
            .rules
            .iter()
            .find(|(id, _)| id.as_str() == field_id)
            .ok_or(PhysicsQuantizationError::MissingRule)?;
        if rule.source_format != source_format {
            return Err(PhysicsQuantizationError::SourceFormatMismatch);
        }
        Ok(rule)
    }

    fn quantize(
        &self,
        rule: &PhysicsQuantizationRuleV1,
        value: ExactIeee,
    ) -> Result<i64, PhysicsQuantizationError> {
        rule.validate()
            .map_err(|_| PhysicsQuantizationError::InvalidProfile)?;
        let descriptor = self
            .numeric
            .fixed_points
            .get(&rule.destination_fixed_point)
            .ok_or(PhysicsQuantizationError::MissingDescriptor)?;
        descriptor
            .validate()
            .map_err(|_| PhysicsQuantizationError::InvalidProfile)?;

        let mut numerator = value
            .significand
            .checked_mul(i128::from(rule.scale_numerator))
            .ok_or(PhysicsQuantizationError::NumericOverflow)?;
        let mut denominator = i128::from(rule.scale_denominator);
        let binary_shift = value
            .binary_exponent
            .checked_add(i32::from(descriptor.fractional_bits))
            .ok_or(PhysicsQuantizationError::NumericOverflow)?;
        apply_binary_shift(&mut numerator, &mut denominator, binary_shift)?;
        reduce(&mut numerator, &mut denominator);
        numerator = numerator
            .checked_add(
                i128::from(rule.offset_raw)
                    .checked_mul(denominator)
                    .ok_or(PhysicsQuantizationError::NumericOverflow)?,
            )
            .ok_or(PhysicsQuantizationError::NumericOverflow)?;
        let rounded = round_ties_to_even(numerator, denominator)?;
        let raw = i64::try_from(rounded).map_err(|_| PhysicsQuantizationError::NumericOverflow)?;
        if raw < descriptor.minimum_raw
            || raw > descriptor.maximum_raw
            || raw < rule.minimum_raw
            || raw > rule.maximum_raw
        {
            return Err(PhysicsQuantizationError::OutOfBounds);
        }
        Ok(raw)
    }
}

#[derive(Clone, Copy)]
struct IeeeLayout {
    exponent_bits: u32,
    fraction_bits: u32,
    exponent_bias: i32,
}

#[derive(Clone, Copy)]
struct ExactIeee {
    significand: i128,
    binary_exponent: i32,
}

fn decode_ieee(bits: u64, layout: IeeeLayout) -> Result<ExactIeee, PhysicsQuantizationError> {
    let sign_shift = layout
        .exponent_bits
        .checked_add(layout.fraction_bits)
        .ok_or(PhysicsQuantizationError::NumericOverflow)?;
    let exponent_mask = (1_u64 << layout.exponent_bits) - 1;
    let fraction_mask = (1_u64 << layout.fraction_bits) - 1;
    let negative = ((bits >> sign_shift) & 1) != 0;
    let exponent = (bits >> layout.fraction_bits) & exponent_mask;
    let fraction = bits & fraction_mask;
    if exponent == exponent_mask {
        return Err(PhysicsQuantizationError::NonFinite);
    }
    if exponent == 0 && fraction == 0 {
        return Ok(ExactIeee {
            significand: 0,
            binary_exponent: 0,
        });
    }
    let (significand, unbiased_exponent) = if exponent == 0 {
        (fraction, 1 - layout.exponent_bias)
    } else {
        (
            (1_u64 << layout.fraction_bits) | fraction,
            i32::try_from(exponent).map_err(|_| PhysicsQuantizationError::NumericOverflow)?
                - layout.exponent_bias,
        )
    };
    let significand = i128::from(significand) * if negative { -1 } else { 1 };
    Ok(ExactIeee {
        significand,
        binary_exponent: unbiased_exponent
            .checked_sub(
                i32::try_from(layout.fraction_bits)
                    .map_err(|_| PhysicsQuantizationError::NumericOverflow)?,
            )
            .ok_or(PhysicsQuantizationError::NumericOverflow)?,
    })
}

fn apply_binary_shift(
    numerator: &mut i128,
    denominator: &mut i128,
    shift: i32,
) -> Result<(), PhysicsQuantizationError> {
    if *numerator == 0 {
        *denominator = 1;
        return Ok(());
    }
    if shift >= 0 {
        let mut remaining =
            u32::try_from(shift).map_err(|_| PhysicsQuantizationError::NumericOverflow)?;
        while remaining > 0 && *denominator % 2 == 0 {
            *denominator /= 2;
            remaining -= 1;
        }
        *numerator = checked_mul_power_of_two(*numerator, remaining)?;
    } else {
        let mut remaining = shift.unsigned_abs();
        while remaining > 0 && *numerator % 2 == 0 {
            *numerator /= 2;
            remaining -= 1;
        }
        *denominator = checked_mul_power_of_two(*denominator, remaining)?;
    }
    Ok(())
}

fn checked_mul_power_of_two(value: i128, exponent: u32) -> Result<i128, PhysicsQuantizationError> {
    if exponent > 126 {
        return Err(PhysicsQuantizationError::NumericOverflow);
    }
    value
        .checked_mul(1_i128 << exponent)
        .ok_or(PhysicsQuantizationError::NumericOverflow)
}

fn reduce(numerator: &mut i128, denominator: &mut i128) {
    let divisor = gcd(numerator.unsigned_abs(), denominator.unsigned_abs());
    if divisor > 1 {
        let divisor = i128::try_from(divisor).expect("gcd of positive i128 values fits i128");
        *numerator /= divisor;
        *denominator /= divisor;
    }
}

fn gcd(mut left: u128, mut right: u128) -> u128 {
    while right != 0 {
        let remainder = left % right;
        left = right;
        right = remainder;
    }
    left
}

fn round_ties_to_even(
    numerator: i128,
    denominator: i128,
) -> Result<i128, PhysicsQuantizationError> {
    if denominator <= 0 {
        return Err(PhysicsQuantizationError::InvalidProfile);
    }
    let quotient = numerator / denominator;
    let remainder = numerator % denominator;
    let twice_remainder = remainder
        .unsigned_abs()
        .checked_mul(2)
        .ok_or(PhysicsQuantizationError::NumericOverflow)?;
    let denominator = denominator.unsigned_abs();
    let increment =
        twice_remainder > denominator || (twice_remainder == denominator && quotient % 2 != 0);
    if increment {
        quotient
            .checked_add(if numerator.is_negative() { -1 } else { 1 })
            .ok_or(PhysicsQuantizationError::NumericOverflow)
    } else {
        Ok(quotient)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum PhysicsQuantizationError {
    InvalidProfile,
    ProfileMismatch,
    MissingRule,
    MissingDescriptor,
    SourceFormatMismatch,
    NonFinite,
    NumericOverflow,
    OutOfBounds,
}

impl PhysicsQuantizationError {
    #[must_use]
    pub const fn stable_code(self) -> &'static str {
        match self {
            Self::InvalidProfile => "PHYSICS_QUANTIZATION_PROFILE_INVALID",
            Self::ProfileMismatch => "PHYSICS_PROFILE_MISMATCH",
            Self::MissingRule => "PHYSICS_QUANTIZATION_RULE_MISSING",
            Self::MissingDescriptor => "PHYSICS_FIXED_POINT_DESCRIPTOR_MISSING",
            Self::SourceFormatMismatch => "PHYSICS_SOURCE_FORMAT_MISMATCH",
            Self::NonFinite => "PHYSICS_NON_FINITE_SOURCE",
            Self::NumericOverflow => "PHYSICS_NUMERIC_OVERFLOW",
            Self::OutOfBounds => "PHYSICS_QUANTIZED_VALUE_OUT_OF_BOUNDS",
        }
    }
}

impl Display for PhysicsQuantizationError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.stable_code())
    }
}

impl Error for PhysicsQuantizationError {}

#[cfg(test)]
mod tests {
    use super::*;
    use next_contracts::{PHYSICS_CONTACT_NORMAL_X_FIELD_ID, PHYSICS_SWEEP_DISTANCE_FIELD_ID};

    fn quantizer() -> (PhysicsQuantizationProfileV1, AuthoritativeNumericProfileV1) {
        let profile = PhysicsQuantizationProfileV1::grounded_capsule_v2().expect("profile");
        let numeric =
            AuthoritativeNumericProfileV1::grounded_capsule_v2(&profile).expect("numeric");
        (profile, numeric)
    }

    #[test]
    fn f32_conversion_is_exact_and_ties_to_even() {
        let (profile, numeric) = quantizer();
        let quantizer = PhysicsQuantizer::new(&profile, &numeric).expect("quantizer");
        assert_eq!(
            quantizer
                .quantize_f32_bits(PHYSICS_SWEEP_DISTANCE_FIELD_ID, 1.25_f32.to_bits())
                .expect("distance"),
            1_250_000
        );
        assert_eq!(
            quantizer
                .quantize_f32_bits(PHYSICS_CONTACT_NORMAL_X_FIELD_ID, 0.5_f32.to_bits())
                .expect("normal"),
            1 << 29
        );
        let half_raw_unit = 2.0_f32.powi(-31);
        assert_eq!(
            quantizer
                .quantize_f32_bits(PHYSICS_CONTACT_NORMAL_X_FIELD_ID, half_raw_unit.to_bits())
                .expect("even lower tie"),
            0
        );
        assert_eq!(
            quantizer
                .quantize_f32_bits(
                    PHYSICS_CONTACT_NORMAL_X_FIELD_ID,
                    (3.0 * half_raw_unit).to_bits()
                )
                .expect("odd lower tie"),
            2
        );
        assert_eq!(
            quantizer
                .quantize_f32_bits(PHYSICS_CONTACT_NORMAL_X_FIELD_ID, (-0.0_f32).to_bits())
                .expect("negative zero"),
            0
        );
    }

    #[test]
    fn non_finite_missing_and_overflow_fail_closed() {
        let (profile, numeric) = quantizer();
        let quantizer = PhysicsQuantizer::new(&profile, &numeric).expect("quantizer");
        assert_eq!(
            quantizer.quantize_f32_bits(PHYSICS_SWEEP_DISTANCE_FIELD_ID, f32::NAN.to_bits()),
            Err(PhysicsQuantizationError::NonFinite)
        );
        assert_eq!(
            quantizer.quantize_f32_bits(PHYSICS_SWEEP_DISTANCE_FIELD_ID, f32::INFINITY.to_bits()),
            Err(PhysicsQuantizationError::NonFinite)
        );
        assert_eq!(
            quantizer.quantize_f32_bits("nextengine.physics.raw.unknown", 0),
            Err(PhysicsQuantizationError::MissingRule)
        );
        assert_eq!(
            quantizer.quantize_f64_bits(PHYSICS_SWEEP_DISTANCE_FIELD_ID, 1.0_f64.to_bits()),
            Err(PhysicsQuantizationError::SourceFormatMismatch)
        );
        assert_eq!(
            quantizer
                .quantize_f32_bits(PHYSICS_SWEEP_DISTANCE_FIELD_ID, 40_000_000.0_f32.to_bits()),
            Err(PhysicsQuantizationError::OutOfBounds)
        );
        assert_eq!(
            quantizer.quantize_f32_bits(PHYSICS_SWEEP_DISTANCE_FIELD_ID, f32::MAX.to_bits()),
            Err(PhysicsQuantizationError::NumericOverflow)
        );
    }

    #[test]
    fn f64_conversion_uses_the_same_exact_rational_path() {
        let (mut profile, mut numeric) = quantizer();
        profile
            .rules
            .get_mut(
                &next_contracts::SchemaId::new(PHYSICS_SWEEP_DISTANCE_FIELD_ID)
                    .expect("distance field ID"),
            )
            .expect("distance rule")
            .source_format = PhysicsSourceFormatV1::Ieee754Binary64;
        numeric.physics_quantization_profile_hash =
            profile.profile_hash().expect("updated profile hash");
        let quantizer = PhysicsQuantizer::new(&profile, &numeric).expect("quantizer");
        assert_eq!(
            quantizer
                .quantize_f64_bits(PHYSICS_SWEEP_DISTANCE_FIELD_ID, 1.25_f64.to_bits())
                .expect("distance"),
            1_250_000
        );
    }
}
