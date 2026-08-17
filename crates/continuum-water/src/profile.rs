#![forbid(unsafe_code)]

use std::env;
use std::hint::black_box;

use crate::error::{
    FLOAT_ENVIRONMENT_MISMATCH, NONFINITE_VALUE, NUMERIC_OVERFLOW, PROFILE_MISMATCH, WaterError,
};

pub(crate) const W0B_DOCUMENT_ROOT_HEX: &str =
    "d357bca64983fbd2074961a462743a4fb5d3fedc2af09631d32ec178bd299550";
pub(crate) const FLOAT_PROFILE_ROOT_HEX: &str =
    "d6152c575fd88bb53d0d63d0e1e8b2e86a82465268fc8d102ebdb1d77c092d63";
pub(crate) const CORPUS_ROOT_HEX: &str =
    "cb091e0f3a04f2c052b614aeab72034bddd3c0b32430fba85854ede2d183aa91";

pub(crate) const MAXIMUM_SAMPLES: usize = 50_000;
pub(crate) const MAXIMUM_BOUNDARY_SAMPLES: usize = 16_384;
pub(crate) const MAXIMUM_NEIGHBORS_PER_FLUID_ROW: usize = 128;
pub(crate) const MAXIMUM_DIRECTED_FLUID_NEIGHBORS: usize = 6_400_000;
pub(crate) const MAXIMUM_NEIGHBORS_PER_BOUNDARY_ROW: usize = 128;
pub(crate) const MAXIMUM_DIRECTED_BOUNDARY_NEIGHBORS: usize = 2_097_152;
pub(crate) const MAXIMUM_STEPS: u32 = 7_200;
pub(crate) const MAXIMUM_REPORT_BYTES: usize = 16_777_216;
pub(crate) const MAXIMUM_REFERENCE_INPUT_BYTES: usize = 33_554_432;
pub(crate) const MAXIMUM_DECODED_HEAP_BYTES: usize = 536_870_912;

pub(crate) const POSITION_LIMIT_UM: i64 = 16_000_000;
pub(crate) const VELOCITY_LIMIT_UM_S: i64 = 64_000_000;
pub(crate) const GRID_CELL_WIDTH_UM: i64 = 100_000;
pub(crate) const SUPPORT_RADIUS_UM: i64 = 100_000;
pub(crate) const PARTICLE_RADIUS_UM: i64 = 25_000;
pub(crate) const MINIMUM_CLEARANCE_UM: i64 = 22_500;
pub(crate) const LATTICE_SPACING_UM: i64 = 50_000;

pub(crate) const RHO0: f64 = f64::from_bits(0x408f_4000_0000_0000);
pub(crate) const PARTICLE_RADIUS: f64 = f64::from_bits(0x3f99_9999_9999_999a);
pub(crate) const LATTICE_SPACING: f64 = f64::from_bits(0x3fa9_9999_9999_999a);
pub(crate) const SUPPORT_RADIUS: f64 = f64::from_bits(0x3fb9_9999_9999_999a);
pub(crate) const UNIFORM_MASS: f64 = f64::from_bits(0x3fc0_0000_0000_0000);
pub(crate) const REST_VOLUME: f64 = f64::from_bits(0x3f20_624d_d2f1_a9fc);
pub(crate) const DT: f64 = f64::from_bits(0x3f71_1111_1111_1111);
pub(crate) const GRAVITY_MAGNITUDE: f64 = f64::from_bits(0x4023_9eb8_51eb_851f);
pub(crate) const PI: f64 = f64::from_bits(0x4009_21fb_5444_2d18);
pub(crate) const SOLVER_EPSILON: f64 = f64::from_bits(0x3ee4_f8b5_88e3_68f1);
pub(crate) const RELAXATION: f64 = f64::from_bits(0x3fe0_0000_0000_0000);
pub(crate) const MICROMETRES_PER_METRE: f64 = f64::from_bits(0x412e_8480_0000_0000);

pub(crate) const H2: f64 = f64::from_bits(0x3f84_7ae1_47ae_147c);
pub(crate) const H3: f64 = f64::from_bits(0x3f50_624d_d2f1_a9fd);
pub(crate) const KERNEL_DENOMINATOR: f64 = f64::from_bits(0x3f69_bc65_b68b_71c4);
pub(crate) const KERNEL_K: f64 = f64::from_bits(0x40a3_e4f5_4b37_0dcf);
pub(crate) const KERNEL_L: f64 = f64::from_bits(0x40cd_d76f_f0d2_94b6);
pub(crate) const DT2: f64 = f64::from_bits(0x3ef2_3456_789a_bcdf);
pub(crate) const INV_DT: f64 = f64::from_bits(0x406e_0000_0000_0000);
pub(crate) const INV_DT2: f64 = f64::from_bits(0x40ec_2000_0000_0000);

const EXPECTED_TARGETS: [&str; 2] = ["x86_64-unknown-linux-gnu", "x86_64-pc-windows-msvc"];
const EXPECTED_RUSTFLAGS: [&str; 3] = [
    "-Ctarget-cpu=x86-64",
    "-Ctarget-feature=-sse3,-ssse3,-sse4.1,-sse4.2,-avx,-avx2,-fma",
    "-Cllvm-args=-fp-contract=off",
];

pub(crate) fn validate_execution_profile() -> Result<(), WaterError> {
    if env!("WATER_BUILD_PROFILE") != "water-oracle" {
        return profile_error(format!(
            "build profile {:?}, expected water-oracle",
            env!("WATER_BUILD_PROFILE")
        ));
    }
    if !EXPECTED_TARGETS.contains(&env!("WATER_BUILD_TARGET")) {
        return profile_error(format!(
            "target {:?} is not in the W0B target set",
            env!("WATER_BUILD_TARGET")
        ));
    }
    if env!("WATER_BUILD_OPT_LEVEL") != "3" || env!("WATER_BUILD_DEBUG") != "false" {
        return profile_error(format!(
            "compiled profile values are opt-level={:?}, debug={:?}",
            env!("WATER_BUILD_OPT_LEVEL"),
            env!("WATER_BUILD_DEBUG")
        ));
    }
    let rustc = env!("WATER_BUILD_RUSTC_VV");
    for required in [
        "commit-hash: 8bab26f4f68e0e26f0bb7960be334d5b520ea452",
        "release: 1.97.1",
        "LLVM version: 22.1.6",
    ] {
        if !rustc.split('|').any(|line| line == required) {
            return profile_error(format!("rustc -vV is missing {required:?}"));
        }
    }
    validate_flags(env!("WATER_BUILD_RUSTFLAGS"), "build rustflags")?;
    validate_ambient_environment()?;
    validate_frozen_constant_bits()?;
    validate_derived_constants()?;
    Ok(())
}

fn validate_frozen_constant_bits() -> Result<(), WaterError> {
    let actual = [
        RHO0.to_bits(),
        PARTICLE_RADIUS.to_bits(),
        LATTICE_SPACING.to_bits(),
        SUPPORT_RADIUS.to_bits(),
        UNIFORM_MASS.to_bits(),
        REST_VOLUME.to_bits(),
        DT.to_bits(),
        GRAVITY_MAGNITUDE.to_bits(),
        PI.to_bits(),
        SOLVER_EPSILON.to_bits(),
        RELAXATION.to_bits(),
        MICROMETRES_PER_METRE.to_bits(),
        H2.to_bits(),
        H3.to_bits(),
        KERNEL_DENOMINATOR.to_bits(),
        KERNEL_K.to_bits(),
        KERNEL_L.to_bits(),
        DT2.to_bits(),
        INV_DT.to_bits(),
        INV_DT2.to_bits(),
    ];
    let expected = [
        0x408f_4000_0000_0000,
        0x3f99_9999_9999_999a,
        0x3fa9_9999_9999_999a,
        0x3fb9_9999_9999_999a,
        0x3fc0_0000_0000_0000,
        0x3f20_624d_d2f1_a9fc,
        0x3f71_1111_1111_1111,
        0x4023_9eb8_51eb_851f,
        0x4009_21fb_5444_2d18,
        0x3ee4_f8b5_88e3_68f1,
        0x3fe0_0000_0000_0000,
        0x412e_8480_0000_0000,
        0x3f84_7ae1_47ae_147c,
        0x3f50_624d_d2f1_a9fd,
        0x3f69_bc65_b68b_71c4,
        0x40a3_e4f5_4b37_0dcf,
        0x40cd_d76f_f0d2_94b6,
        0x3ef2_3456_789a_bcdf,
        0x406e_0000_0000_0000,
        0x40ec_2000_0000_0000,
    ];
    if actual == expected {
        Ok(())
    } else {
        profile_error(format!("frozen constant bits mismatch: {actual:016x?}"))
    }
}

fn validate_ambient_environment() -> Result<(), WaterError> {
    if let Some(value) = env::var_os("RUSTFLAGS") {
        validate_flags(&value.to_string_lossy(), "ambient RUSTFLAGS")?;
    }
    if let Some(value) = env::var_os("CARGO_ENCODED_RUSTFLAGS") {
        validate_flags(&value.to_string_lossy(), "ambient CARGO_ENCODED_RUSTFLAGS")?;
    }
    if env::var_os("RUSTDOCFLAGS").is_some_and(|value| !value.is_empty()) {
        return profile_error("ambient RUSTDOCFLAGS is not allowed".to_owned());
    }
    if env::var_os("CARGO_ENCODED_RUSTDOCFLAGS").is_some_and(|value| !value.is_empty()) {
        return profile_error("ambient CARGO_ENCODED_RUSTDOCFLAGS is not allowed".to_owned());
    }

    let allowed_overrides = [
        ("CARGO_PROFILE_WATER_ORACLE_OPT_LEVEL", "3"),
        ("CARGO_PROFILE_WATER_ORACLE_CODEGEN_UNITS", "1"),
        ("CARGO_PROFILE_WATER_ORACLE_LTO", "false"),
        ("CARGO_PROFILE_WATER_ORACLE_INCREMENTAL", "false"),
        ("CARGO_PROFILE_WATER_ORACLE_OVERFLOW_CHECKS", "true"),
        ("CARGO_PROFILE_WATER_ORACLE_DEBUG_ASSERTIONS", "false"),
        ("CARGO_PROFILE_WATER_ORACLE_PANIC", "abort"),
    ];
    for (name, value) in env::vars().filter(|(name, _)| name.starts_with("CARGO_PROFILE_")) {
        let expected = allowed_overrides
            .iter()
            .find_map(|(allowed_name, allowed_value)| {
                (*allowed_name == name).then_some(*allowed_value)
            });
        if expected != Some(value.as_str()) {
            return profile_error(format!("ambient profile override {name}={value:?}"));
        }
    }
    Ok(())
}

fn validate_flags(value: &str, source: &str) -> Result<(), WaterError> {
    let actual: Vec<&str> = if value.contains('\u{1f}') {
        value
            .split('\u{1f}')
            .filter(|item| !item.is_empty())
            .collect()
    } else {
        value.split_ascii_whitespace().collect()
    };
    if actual == EXPECTED_RUSTFLAGS {
        Ok(())
    } else {
        profile_error(format!("{source} mismatch: {actual:?}"))
    }
}

fn validate_derived_constants() -> Result<(), WaterError> {
    let lattice_spacing = PARTICLE_RADIUS * 2.0;
    let support_radius = LATTICE_SPACING * 2.0;
    let h2 = SUPPORT_RADIUS * SUPPORT_RADIUS;
    let h3 = h2 * SUPPORT_RADIUS;
    let denominator = PI * h3;
    let kernel_k = 8.0 / denominator;
    let kernel_l = 48.0 / denominator;
    let rest_volume = UNIFORM_MASS / RHO0;
    let dt2 = DT * DT;
    let inv_dt = 1.0 / DT;
    let inv_dt2 = 1.0 / dt2;
    let actual = [
        lattice_spacing.to_bits(),
        support_radius.to_bits(),
        h2.to_bits(),
        h3.to_bits(),
        denominator.to_bits(),
        kernel_k.to_bits(),
        kernel_l.to_bits(),
        rest_volume.to_bits(),
        dt2.to_bits(),
        inv_dt.to_bits(),
        inv_dt2.to_bits(),
    ];
    let expected = [
        LATTICE_SPACING.to_bits(),
        SUPPORT_RADIUS.to_bits(),
        H2.to_bits(),
        H3.to_bits(),
        KERNEL_DENOMINATOR.to_bits(),
        KERNEL_K.to_bits(),
        KERNEL_L.to_bits(),
        REST_VOLUME.to_bits(),
        DT2.to_bits(),
        INV_DT.to_bits(),
        INV_DT2.to_bits(),
    ];
    if actual == expected {
        Ok(())
    } else {
        profile_error(format!("derived constants mismatch: {actual:016x?}"))
    }
}

pub(crate) fn validate_float_environment() -> Result<(), WaterError> {
    let one = black_box(f64::from_bits(0x3ff0_0000_0000_0000));
    let two_neg_53 = black_box(f64::from_bits(0x3ca0_0000_0000_0000));
    let three_two_neg_53 = black_box(f64::from_bits(0x3cb8_0000_0000_0000));
    check_probe("1 + 2^-53", one + two_neg_53, 0x3ff0_0000_0000_0000)?;
    check_probe("1 + 3*2^-53", one + three_two_neg_53, 0x3ff0_0000_0000_0002)?;
    check_probe(
        "MIN_POSITIVE * 0.5",
        black_box(f64::MIN_POSITIVE) * black_box(0.5),
        0x0008_0000_0000_0000,
    )?;
    let doubled_subnormal = black_box(f64::from_bits(1)) * black_box(2.0);
    check_probe(
        "(MIN_SUBNORMAL * 2) / 2",
        doubled_subnormal / black_box(2.0),
        0x0000_0000_0000_0001,
    )?;
    let delta = black_box(f64::from_bits(0x3e40_0000_0000_0000));
    let plus = one + delta;
    let minus = one - delta;
    let product = plus * minus;
    check_probe("non-contraction", product - one, 0x0000_0000_0000_0000)?;
    for (label, input, expected) in [
        ("sqrt(0)", 0.0, 0x0000_0000_0000_0000),
        ("sqrt(1)", 1.0, 0x3ff0_0000_0000_0000),
        ("sqrt(2)", 2.0, 0x3ff6_a09e_667f_3bcd),
        ("sqrt(4)", 4.0, 0x4000_0000_0000_0000),
    ] as [(&str, f64, u64); 4]
    {
        check_probe(label, black_box(input).sqrt(), expected)?;
    }
    Ok(())
}

fn check_probe(label: &str, actual: f64, expected: u64) -> Result<(), WaterError> {
    if actual.to_bits() == expected {
        Ok(())
    } else {
        Err(WaterError::new(
            FLOAT_ENVIRONMENT_MISMATCH,
            format!(
                "{label} produced 0x{:016x}, expected 0x{expected:016x}",
                actual.to_bits()
            ),
        ))
    }
}

fn profile_error<T>(detail: String) -> Result<T, WaterError> {
    Err(WaterError::new(PROFILE_MISMATCH, detail))
}

pub(crate) fn decode_micrometres(value: i64) -> Result<f64, WaterError> {
    if value.unsigned_abs() > POSITION_LIMIT_UM as u64 {
        return Err(WaterError::new(
            NUMERIC_OVERFLOW,
            format!("position {value} exceeds the W0B laboratory bound"),
        ));
    }
    Ok((value as f64) / MICROMETRES_PER_METRE)
}

pub(crate) fn decode_velocity(value: i64) -> Result<f64, WaterError> {
    if value.unsigned_abs() > VELOCITY_LIMIT_UM_S as u64 {
        return Err(WaterError::new(
            NUMERIC_OVERFLOW,
            format!("velocity {value} exceeds the W0B laboratory bound"),
        ));
    }
    Ok((value as f64) / MICROMETRES_PER_METRE)
}

pub(crate) fn quantize_micrometres(value: f64) -> Result<i64, WaterError> {
    let result = quantize_scaled(value, 1_000_000)?;
    if result.unsigned_abs() > POSITION_LIMIT_UM as u64 {
        return Err(WaterError::new(
            NUMERIC_OVERFLOW,
            format!("published position {result} exceeds the W0B laboratory bound"),
        ));
    }
    Ok(result)
}

pub(crate) fn quantize_velocity(value: f64) -> Result<i64, WaterError> {
    let result = quantize_scaled(value, 1_000_000)?;
    if result.unsigned_abs() > VELOCITY_LIMIT_UM_S as u64 {
        return Err(WaterError::new(
            NUMERIC_OVERFLOW,
            format!("published velocity {result} exceeds the W0B laboratory bound"),
        ));
    }
    Ok(result)
}

pub(crate) fn quantize_ppb(value: f64) -> Result<i64, WaterError> {
    quantize_scaled(value, 1_000_000_000)
}

pub(crate) fn quantize_scaled(value: f64, scale: u64) -> Result<i64, WaterError> {
    if !value.is_finite() {
        return Err(WaterError::new(
            NONFINITE_VALUE,
            "cannot publish a nonfinite binary64 value",
        ));
    }
    let bits = value.to_bits();
    let negative = (bits >> 63) != 0;
    let raw_exponent = ((bits >> 52) & 0x7ff) as i32;
    let fraction = bits & 0x000f_ffff_ffff_ffff;
    if raw_exponent == 0 && fraction == 0 {
        return Ok(0);
    }
    let (significand, exponent) = if raw_exponent == 0 {
        (u128::from(fraction), -1074)
    } else {
        (
            u128::from((1_u64 << 52) | fraction),
            raw_exponent - 1023 - 52,
        )
    };
    let scaled = significand
        .checked_mul(u128::from(scale))
        .ok_or_else(|| WaterError::new(NUMERIC_OVERFLOW, "publication multiply overflow"))?;
    let magnitude = if exponent >= 0 {
        let shift = exponent as u32;
        if shift >= u128::BITS || scaled > (u128::MAX >> shift) {
            return Err(WaterError::new(
                NUMERIC_OVERFLOW,
                "publication left shift overflow",
            ));
        }
        scaled << shift
    } else {
        round_divide_power_of_two(scaled, exponent.unsigned_abs())
    };
    signed_i64(negative, magnitude)
}

fn round_divide_power_of_two(numerator: u128, shift: u32) -> u128 {
    if shift > 128 {
        return 0;
    }
    if shift == 128 {
        let half = 1_u128 << 127;
        return u128::from(numerator > half);
    }
    let divisor = 1_u128 << shift;
    let quotient = numerator / divisor;
    let remainder = numerator % divisor;
    let half = divisor >> 1;
    if remainder > half || (remainder == half && (quotient & 1) != 0) {
        quotient + 1
    } else {
        quotient
    }
}

fn signed_i64(negative: bool, magnitude: u128) -> Result<i64, WaterError> {
    if negative {
        let minimum_magnitude = 1_u128 << 63;
        if magnitude > minimum_magnitude {
            return Err(WaterError::new(
                NUMERIC_OVERFLOW,
                "negative publication result is below i64::MIN",
            ));
        }
        if magnitude == minimum_magnitude {
            Ok(i64::MIN)
        } else {
            let positive = i64::try_from(magnitude).map_err(|_| {
                WaterError::new(NUMERIC_OVERFLOW, "publication result conversion overflow")
            })?;
            Ok(-positive)
        }
    } else {
        i64::try_from(magnitude)
            .map_err(|_| WaterError::new(NUMERIC_OVERFLOW, "publication result exceeds i64::MAX"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exact_ties_round_to_even() {
        assert_eq!(quantize_scaled(0.5, 1).unwrap(), 0);
        assert_eq!(quantize_scaled(1.5, 1).unwrap(), 2);
        assert_eq!(quantize_scaled(2.5, 1).unwrap(), 2);
        assert_eq!(quantize_scaled(3.5, 1).unwrap(), 4);
        assert_eq!(quantize_scaled(-1.5, 1).unwrap(), -2);
        assert_eq!(quantize_scaled(-2.5, 1).unwrap(), -2);
    }

    #[test]
    fn publication_normalizes_negative_zero_and_rejects_bad_values() {
        assert_eq!(quantize_scaled(-0.0, 1_000_000).unwrap(), 0);
        assert_eq!(
            quantize_scaled(f64::NAN, 1).unwrap_err().code(),
            NONFINITE_VALUE
        );
        assert_eq!(
            quantize_scaled(f64::INFINITY, 1).unwrap_err().code(),
            NONFINITE_VALUE
        );
        assert_eq!(
            quantize_scaled(f64::MAX, u64::MAX).unwrap_err().code(),
            NUMERIC_OVERFLOW
        );
    }

    #[test]
    fn exact_i64_boundaries_are_supported() {
        let two_pow_63 = f64::from_bits(((1023 + 63) as u64) << 52);
        assert_eq!(quantize_scaled(-two_pow_63, 1).unwrap(), i64::MIN);
        assert_eq!(
            quantize_scaled(two_pow_63, 1).unwrap_err().code(),
            NUMERIC_OVERFLOW
        );
        let two_pow_128 = f64::from_bits(((1023 + 128) as u64) << 52);
        assert_eq!(
            quantize_scaled(two_pow_128, 1).unwrap_err().code(),
            NUMERIC_OVERFLOW
        );
    }

    #[test]
    fn startup_float_probes_pass_on_supported_host() {
        validate_float_environment().unwrap();
        validate_frozen_constant_bits().unwrap();
        validate_derived_constants().unwrap();
    }
}
