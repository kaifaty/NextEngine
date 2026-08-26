//! Bounded floating-point modal recurrence for external P0 calibration.
//!
//! This is an offline evidence path. It consumes already cooked mode data and
//! deliberately does not define a runtime asset, public content schema or
//! gameplay-visible result.

use std::error::Error;
use std::f64::consts::TAU;
use std::fmt::{Display, Formatter};

const MIN_SAMPLE_RATE_HZ: u32 = 8_000;
const MAX_SAMPLE_RATE_HZ: u32 = 192_000;
const MAX_DURATION_SECONDS: usize = 30;
const MAX_MODE_COUNT: usize = 128;

/// One already cooked damped mode used by the offline calibration renderer.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct OfflineModalMode {
    pub damped_frequency_hz: f64,
    pub damping_per_second: f64,
    pub amplitude: f64,
}

/// Validation failure for the bounded offline modal renderer.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OfflineModalRenderError {
    InvalidSampleRate,
    InvalidFrameCount,
    InvalidModeCount,
    InvalidMode { index: usize },
    InvalidTransient,
    SilentOutput,
}

impl Display for OfflineModalRenderError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidSampleRate => formatter.write_str("offline modal sample rate is invalid"),
            Self::InvalidFrameCount => formatter.write_str("offline modal frame count is invalid"),
            Self::InvalidModeCount => formatter.write_str("offline modal mode count is invalid"),
            Self::InvalidMode { index } => {
                write!(formatter, "offline modal mode {index} is invalid")
            }
            Self::InvalidTransient => formatter.write_str("offline modal transient is invalid"),
            Self::SilentOutput => formatter.write_str("offline modal render is silent"),
        }
    }
}

impl Error for OfflineModalRenderError {}

#[derive(Clone, Copy, Debug)]
struct RecurrenceState {
    previous: f64,
    current: f64,
    coefficient_a: f64,
    coefficient_b: f64,
    amplitude: f64,
}

/// Renders `sum(A_i * exp(-d_i*t) * sin(2*pi*f_i*t))`, adds the supplied
/// already bounded onset transient, then peak-normalizes to mono float32.
///
/// The oscillator advances as a second-order recurrence; no eigensolver,
/// training framework or external synthesis implementation is involved.
pub fn render_offline_modal_recurrence(
    sample_rate_hz: u32,
    frame_count: usize,
    modes: &[OfflineModalMode],
    transient: &[f64],
) -> Result<Vec<f32>, OfflineModalRenderError> {
    if !(MIN_SAMPLE_RATE_HZ..=MAX_SAMPLE_RATE_HZ).contains(&sample_rate_hz) {
        return Err(OfflineModalRenderError::InvalidSampleRate);
    }
    let maximum_frames = usize::try_from(sample_rate_hz)
        .map_err(|_| OfflineModalRenderError::InvalidSampleRate)?
        .saturating_mul(MAX_DURATION_SECONDS);
    if frame_count == 0 || frame_count > maximum_frames {
        return Err(OfflineModalRenderError::InvalidFrameCount);
    }
    if modes.is_empty() || modes.len() > MAX_MODE_COUNT {
        return Err(OfflineModalRenderError::InvalidModeCount);
    }
    if transient.len() > frame_count || transient.iter().any(|sample| !sample.is_finite()) {
        return Err(OfflineModalRenderError::InvalidTransient);
    }

    let sample_rate = f64::from(sample_rate_hz);
    let nyquist = sample_rate * 0.5;
    let mut states = Vec::with_capacity(modes.len());
    for (index, mode) in modes.iter().enumerate() {
        if !mode.damped_frequency_hz.is_finite()
            || mode.damped_frequency_hz <= 0.0
            || mode.damped_frequency_hz >= nyquist
            || !mode.damping_per_second.is_finite()
            || mode.damping_per_second < 0.0
            || !mode.amplitude.is_finite()
        {
            return Err(OfflineModalRenderError::InvalidMode { index });
        }
        let decay = (-mode.damping_per_second / sample_rate).exp();
        let phase = TAU * mode.damped_frequency_hz / sample_rate;
        states.push(RecurrenceState {
            previous: 0.0,
            current: decay * phase.sin(),
            coefficient_a: 2.0 * decay * phase.cos(),
            coefficient_b: decay * decay,
            amplitude: mode.amplitude,
        });
    }

    let mut output = vec![0.0_f64; frame_count];
    for sample in output.iter_mut().skip(1) {
        for state in &mut states {
            *sample += state.amplitude * state.current;
            let next = state
                .coefficient_a
                .mul_add(state.current, -state.coefficient_b * state.previous);
            state.previous = state.current;
            state.current = next;
        }
    }
    for (sample, transient) in output.iter_mut().zip(transient) {
        *sample += transient;
    }

    let peak = output
        .iter()
        .fold(0.0_f64, |peak, sample| peak.max(sample.abs()));
    if !peak.is_finite() || peak <= f64::EPSILON {
        return Err(OfflineModalRenderError::SilentOutput);
    }
    Ok(output
        .into_iter()
        .map(|sample| (sample / peak) as f32)
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recurrence_matches_the_closed_form_and_repeats_exactly() {
        let modes = [
            OfflineModalMode {
                damped_frequency_hz: 731.25,
                damping_per_second: 27.0,
                amplitude: 0.75,
            },
            OfflineModalMode {
                damped_frequency_hz: 1_903.0,
                damping_per_second: 63.0,
                amplitude: -0.2,
            },
        ];
        let transient = [0.04, -0.02, 0.01];
        let actual = render_offline_modal_recurrence(32_000, 4_000, &modes, &transient)
            .expect("valid modal render");
        assert_eq!(
            actual,
            render_offline_modal_recurrence(32_000, 4_000, &modes, &transient)
                .expect("repeat modal render")
        );

        let mut expected = (0..4_000)
            .map(|frame| {
                let time = frame as f64 / 32_000.0;
                modes.iter().fold(0.0, |sample, mode| {
                    sample
                        + mode.amplitude
                            * (-mode.damping_per_second * time).exp()
                            * (TAU * mode.damped_frequency_hz * time).sin()
                })
            })
            .collect::<Vec<_>>();
        for (sample, onset) in expected.iter_mut().zip(transient) {
            *sample += onset;
        }
        let peak = expected
            .iter()
            .fold(0.0_f64, |peak, sample| peak.max(sample.abs()));
        let maximum_error = actual
            .iter()
            .zip(expected)
            .map(|(actual, expected)| (f64::from(*actual) - expected / peak).abs())
            .fold(0.0_f64, f64::max);
        assert!(maximum_error < 1.0e-6, "maximum error {maximum_error}");
    }

    #[test]
    fn malformed_or_unbounded_inputs_fail_closed() {
        let valid = OfflineModalMode {
            damped_frequency_hz: 1_000.0,
            damping_per_second: 40.0,
            amplitude: 1.0,
        };
        assert_eq!(
            render_offline_modal_recurrence(7_999, 10, &[valid], &[]),
            Err(OfflineModalRenderError::InvalidSampleRate)
        );
        assert_eq!(
            render_offline_modal_recurrence(32_000, 0, &[valid], &[]),
            Err(OfflineModalRenderError::InvalidFrameCount)
        );
        assert_eq!(
            render_offline_modal_recurrence(32_000, 10, &[], &[]),
            Err(OfflineModalRenderError::InvalidModeCount)
        );
        assert_eq!(
            render_offline_modal_recurrence(
                32_000,
                10,
                &[OfflineModalMode {
                    damped_frequency_hz: 16_000.0,
                    ..valid
                }],
                &[],
            ),
            Err(OfflineModalRenderError::InvalidMode { index: 0 })
        );
        assert_eq!(
            render_offline_modal_recurrence(32_000, 10, &[valid], &[f64::NAN]),
            Err(OfflineModalRenderError::InvalidTransient)
        );
        assert_eq!(
            render_offline_modal_recurrence(
                32_000,
                10,
                &[OfflineModalMode {
                    amplitude: 0.0,
                    ..valid
                }],
                &[],
            ),
            Err(OfflineModalRenderError::SilentOutput)
        );
    }
}
