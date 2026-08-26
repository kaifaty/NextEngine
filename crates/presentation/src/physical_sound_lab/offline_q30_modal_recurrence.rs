//! Bounded Q30 modal recurrence for external P0 calibration.
//!
//! The cooker converts validated floating-point calibration values to a
//! fixed-point bank. Rendering after that boundary uses integer coefficients,
//! state and transient samples only. The final float conversion is kept
//! separate so the same integer samples can be inspected before audition WAV
//! normalization.

use std::error::Error;
use std::f64::consts::TAU;
use std::fmt::{Display, Formatter};

use super::offline_modal_recurrence::OfflineModalMode;

const MIN_SAMPLE_RATE_HZ: u32 = 8_000;
const MAX_SAMPLE_RATE_HZ: u32 = 192_000;
const MAX_DURATION_SECONDS: usize = 30;
const MAX_MODE_COUNT: usize = 128;
const MAX_INPUT_MAGNITUDE: f64 = 16.0;
const Q30_SHIFT: u32 = 30;
const Q30_ONE_F64: f64 = 1_073_741_824.0;

/// Validation or arithmetic failure in the bounded Q30 calibration path.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OfflineQ30ModalRenderError {
    InvalidSampleRate,
    InvalidFrameCount,
    InvalidModeCount,
    InvalidMode { index: usize },
    InvalidTransient,
    ArithmeticOverflow,
    SilentOutput,
}

impl Display for OfflineQ30ModalRenderError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidSampleRate => formatter.write_str("offline Q30 sample rate is invalid"),
            Self::InvalidFrameCount => formatter.write_str("offline Q30 frame count is invalid"),
            Self::InvalidModeCount => formatter.write_str("offline Q30 mode count is invalid"),
            Self::InvalidMode { index } => write!(formatter, "offline Q30 mode {index} is invalid"),
            Self::InvalidTransient => formatter.write_str("offline Q30 transient is invalid"),
            Self::ArithmeticOverflow => formatter.write_str("offline Q30 arithmetic overflowed"),
            Self::SilentOutput => formatter.write_str("offline Q30 render is silent"),
        }
    }
}

impl Error for OfflineQ30ModalRenderError {}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct OfflineQ30Mode {
    coefficient_a_q30: i64,
    coefficient_b_q30: i64,
    initial_sample_q30: i64,
}

/// Fully cooked integer input for the offline Q30 recurrence.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OfflineQ30ModalBank {
    sample_rate_hz: u32,
    modes: Vec<OfflineQ30Mode>,
    transient_q30: Vec<i64>,
}

impl OfflineQ30ModalBank {
    #[must_use]
    pub fn sample_rate_hz(&self) -> u32 {
        self.sample_rate_hz
    }

    #[must_use]
    pub fn mode_count(&self) -> usize {
        self.modes.len()
    }

    #[must_use]
    pub fn transient_sample_count(&self) -> usize {
        self.transient_q30.len()
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
struct RecurrenceState {
    previous_q30: i64,
    current_q30: i64,
}

/// Cooks continuous damped-mode parameters and a bounded onset to signed Q30.
///
/// This is an offline conversion boundary. A future runtime/content path would
/// persist the integer result rather than repeat this floating-point step.
pub fn cook_offline_q30_modal_bank(
    sample_rate_hz: u32,
    modes: &[OfflineModalMode],
    transient: &[f64],
) -> Result<OfflineQ30ModalBank, OfflineQ30ModalRenderError> {
    if !(MIN_SAMPLE_RATE_HZ..=MAX_SAMPLE_RATE_HZ).contains(&sample_rate_hz) {
        return Err(OfflineQ30ModalRenderError::InvalidSampleRate);
    }
    if modes.is_empty() || modes.len() > MAX_MODE_COUNT {
        return Err(OfflineQ30ModalRenderError::InvalidModeCount);
    }
    if transient
        .iter()
        .any(|sample| !sample.is_finite() || sample.abs() > MAX_INPUT_MAGNITUDE)
    {
        return Err(OfflineQ30ModalRenderError::InvalidTransient);
    }

    let sample_rate = f64::from(sample_rate_hz);
    let nyquist = sample_rate * 0.5;
    let mut cooked_modes = Vec::with_capacity(modes.len());
    for (index, mode) in modes.iter().enumerate() {
        if !mode.damped_frequency_hz.is_finite()
            || mode.damped_frequency_hz <= 0.0
            || mode.damped_frequency_hz >= nyquist
            || !mode.damping_per_second.is_finite()
            || mode.damping_per_second < 0.0
            || !mode.amplitude.is_finite()
            || mode.amplitude.abs() > MAX_INPUT_MAGNITUDE
        {
            return Err(OfflineQ30ModalRenderError::InvalidMode { index });
        }
        let decay = (-mode.damping_per_second / sample_rate).exp();
        let phase = TAU * mode.damped_frequency_hz / sample_rate;
        cooked_modes.push(OfflineQ30Mode {
            coefficient_a_q30: quantize_q30(2.0 * decay * phase.cos())
                .ok_or(OfflineQ30ModalRenderError::InvalidMode { index })?,
            coefficient_b_q30: quantize_q30(decay * decay)
                .ok_or(OfflineQ30ModalRenderError::InvalidMode { index })?,
            initial_sample_q30: quantize_q30(mode.amplitude * decay * phase.sin())
                .ok_or(OfflineQ30ModalRenderError::InvalidMode { index })?,
        });
    }

    let transient_q30 = transient
        .iter()
        .map(|sample| quantize_q30(*sample).ok_or(OfflineQ30ModalRenderError::InvalidTransient))
        .collect::<Result<Vec<_>, _>>()?;
    Ok(OfflineQ30ModalBank {
        sample_rate_hz,
        modes: cooked_modes,
        transient_q30,
    })
}

/// Renders a cooked bank with integer-only recurrence and onset accumulation.
pub fn render_offline_q30_modal_recurrence(
    frame_count: usize,
    bank: &OfflineQ30ModalBank,
) -> Result<Vec<i64>, OfflineQ30ModalRenderError> {
    let maximum_frames = usize::try_from(bank.sample_rate_hz)
        .map_err(|_| OfflineQ30ModalRenderError::InvalidSampleRate)?
        .saturating_mul(MAX_DURATION_SECONDS);
    if frame_count == 0 || frame_count > maximum_frames {
        return Err(OfflineQ30ModalRenderError::InvalidFrameCount);
    }
    if bank.transient_q30.len() > frame_count {
        return Err(OfflineQ30ModalRenderError::InvalidTransient);
    }

    let mut states = bank
        .modes
        .iter()
        .map(|mode| RecurrenceState {
            previous_q30: 0,
            current_q30: mode.initial_sample_q30,
        })
        .collect::<Vec<_>>();
    let mut output = vec![0_i64; frame_count];
    for sample in output.iter_mut().skip(1) {
        let mut sum_q30 = 0_i128;
        for (state, mode) in states.iter_mut().zip(&bank.modes) {
            sum_q30 = sum_q30
                .checked_add(i128::from(state.current_q30))
                .ok_or(OfflineQ30ModalRenderError::ArithmeticOverflow)?;
            let next_q30 = ((i128::from(mode.coefficient_a_q30) * i128::from(state.current_q30))
                >> Q30_SHIFT)
                - ((i128::from(mode.coefficient_b_q30) * i128::from(state.previous_q30))
                    >> Q30_SHIFT);
            state.previous_q30 = state.current_q30;
            state.current_q30 = i64::try_from(next_q30)
                .map_err(|_| OfflineQ30ModalRenderError::ArithmeticOverflow)?;
        }
        *sample =
            i64::try_from(sum_q30).map_err(|_| OfflineQ30ModalRenderError::ArithmeticOverflow)?;
    }
    for (sample, transient) in output.iter_mut().zip(&bank.transient_q30) {
        *sample = sample
            .checked_add(*transient)
            .ok_or(OfflineQ30ModalRenderError::ArithmeticOverflow)?;
    }
    if output.iter().all(|sample| *sample == 0) {
        return Err(OfflineQ30ModalRenderError::SilentOutput);
    }
    Ok(output)
}

/// Peak-normalizes integer recurrence samples for an audition or comparison.
pub fn normalize_offline_q30_samples(
    samples_q30: &[i64],
) -> Result<Vec<f32>, OfflineQ30ModalRenderError> {
    let peak = samples_q30
        .iter()
        .map(|sample| sample.unsigned_abs())
        .max()
        .unwrap_or(0);
    if peak == 0 {
        return Err(OfflineQ30ModalRenderError::SilentOutput);
    }
    let peak = peak as f64;
    Ok(samples_q30
        .iter()
        .map(|sample| (*sample as f64 / peak) as f32)
        .collect())
}

fn quantize_q30(value: f64) -> Option<i64> {
    let scaled = value * Q30_ONE_F64;
    if !scaled.is_finite() || scaled < i64::MIN as f64 || scaled > i64::MAX as f64 {
        None
    } else {
        Some(scaled.round() as i64)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::physical_sound_lab::render_offline_modal_recurrence;

    #[test]
    fn cooked_q30_render_repeats_and_tracks_high_precision_reference() {
        let modes = [
            OfflineModalMode {
                damped_frequency_hz: 1_538.554_443_359_375,
                damping_per_second: 53.562_419_891_357_42,
                amplitude: -0.903_104_066_848_754_9,
            },
            OfflineModalMode {
                damped_frequency_hz: 1_541.762_939_453_125,
                damping_per_second: 53.553_016_662_597_656,
                amplitude: 1.0,
            },
            OfflineModalMode {
                damped_frequency_hz: 14_803.311_523_437_5,
                damping_per_second: 78.492_164_611_816_4,
                amplitude: 0.052_423_495_799_303_055,
            },
        ];
        let transient = [0.0, 0.05, -0.08, 0.03];
        let bank =
            cook_offline_q30_modal_bank(48_000, &modes, &transient).expect("cook valid Q30 bank");
        let first =
            render_offline_q30_modal_recurrence(24_000, &bank).expect("render valid Q30 bank");
        assert_eq!(
            first,
            render_offline_q30_modal_recurrence(24_000, &bank).expect("repeat valid Q30 bank")
        );

        let actual = normalize_offline_q30_samples(&first).expect("normalize Q30 render");
        let reference = render_offline_modal_recurrence(48_000, 24_000, &modes, &transient)
            .expect("render high precision reference");
        let error_energy = actual
            .iter()
            .zip(&reference)
            .map(|(actual, reference)| {
                let error = f64::from(*actual) - f64::from(*reference);
                error * error
            })
            .sum::<f64>();
        let rms = (error_energy / actual.len() as f64).sqrt();
        assert!(rms <= 1.0e-4, "Q30 RMS residual {rms}");
    }

    #[test]
    fn malformed_or_unbounded_q30_inputs_fail_closed() {
        let valid = OfflineModalMode {
            damped_frequency_hz: 1_000.0,
            damping_per_second: 40.0,
            amplitude: 1.0,
        };
        assert_eq!(
            cook_offline_q30_modal_bank(7_999, &[valid], &[]),
            Err(OfflineQ30ModalRenderError::InvalidSampleRate)
        );
        assert_eq!(
            cook_offline_q30_modal_bank(48_000, &[], &[]),
            Err(OfflineQ30ModalRenderError::InvalidModeCount)
        );
        assert_eq!(
            cook_offline_q30_modal_bank(
                48_000,
                &[OfflineModalMode {
                    damped_frequency_hz: 24_000.0,
                    ..valid
                }],
                &[],
            ),
            Err(OfflineQ30ModalRenderError::InvalidMode { index: 0 })
        );
        assert_eq!(
            cook_offline_q30_modal_bank(48_000, &[valid], &[f64::NAN]),
            Err(OfflineQ30ModalRenderError::InvalidTransient)
        );
        let bank =
            cook_offline_q30_modal_bank(48_000, &[valid], &[0.1]).expect("cook valid Q30 bank");
        assert_eq!(
            render_offline_q30_modal_recurrence(0, &bank),
            Err(OfflineQ30ModalRenderError::InvalidFrameCount)
        );
        assert_eq!(
            normalize_offline_q30_samples(&[]),
            Err(OfflineQ30ModalRenderError::SilentOutput)
        );
    }
}
