//! Offline-only bounded steel calibration search.
//!
//! The ordinary laboratory/demo voices retain their fixed 12-state storage.
//! This module alone admits a heap-backed 36-mode counterfactual and never
//! defines a cooked asset, public content schema or runtime default.

use std::error::Error;
use std::f64::consts::TAU;
use std::fmt::{Display, Formatter};

use super::{
    ExperimentalPhysicalSoundMixer, ExperimentalVoice, FINAL_FADE_FRAMES, MAX_MODE_COUNT,
    ModeProfile, ModeState, PhysicalSoundExcitation, PhysicalSoundImpactPoint,
    PhysicalSoundMaterial, Q15_ONE, Q16_ONE, SAMPLE_RATE_HZ, STEEL_MODES, STRIKE_AMPLITUDE,
    StrikeTransientProfile,
};

const STEEL_FREQUENCIES_HZ: [f64; MAX_MODE_COUNT] = [
    215.3, 409.1, 710.6, 807.5, 1_076.7, 1_453.5, 1_857.2, 2_153.3, 2_659.4, 3_203.1, 3_644.5,
    5_695.5,
];
const STEEL_T20_MILLISECONDS: [f64; MAX_MODE_COUNT] = [
    270.0, 240.0, 195.0, 180.0, 150.0, 135.0, 120.0, 135.0, 112.5, 97.5, 82.5, 60.0,
];
const Q30_ONE_F64: f64 = 1_073_741_824.0;

/// One bounded, offline-only point in the steel calibration search space.
/// Integer permille controls keep experiment manifests exact and reviewable.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct ExperimentalSteelSearchProfile {
    pub frequency_scale_permille: u16,
    pub decay_scale_permille: u16,
    pub high_mode_gain_permille: u16,
    pub transient_gain_q15: u16,
    pub transient_frames: u16,
    /// Sideband offset as permille of the critical bandwidth. The cited
    /// perceptual maximum near one quarter of a critical band is `250`.
    pub roughness_sideband_fraction_permille: u16,
    /// AM-equivalent sideband strength in permille, where each sideband starts
    /// from half of this index before energy normalization.
    pub roughness_index_permille: u16,
    /// Gain of one deterministic broadband residual in Q1.15. This remains an
    /// offline counterfactual; zero disables the residual exactly.
    pub stochastic_residual_gain_q15: u16,
    /// Amplitude T20 of the broadband residual in milliseconds. It must be
    /// zero exactly when the residual gain is zero.
    pub stochastic_residual_t20_ms: u16,
}

impl ExperimentalSteelSearchProfile {
    #[must_use]
    pub const fn current_control() -> Self {
        Self {
            frequency_scale_permille: 1_000,
            decay_scale_permille: 1_000,
            high_mode_gain_permille: 1_000,
            transient_gain_q15: 8_192,
            transient_frames: 960,
            roughness_sideband_fraction_permille: 0,
            roughness_index_permille: 0,
            stochastic_residual_gain_q15: 0,
            stochastic_residual_t20_ms: 0,
        }
    }

    fn validate(self) -> Result<(), ExperimentalSteelSearchError> {
        if !(650..=1_400).contains(&self.frequency_scale_permille)
            || !(750..=4_000).contains(&self.decay_scale_permille)
            || !(400..=1_800).contains(&self.high_mode_gain_permille)
            || !(2_048..=16_384).contains(&self.transient_gain_q15)
            || !(120..=1_440).contains(&self.transient_frames)
            || self.roughness_sideband_fraction_permille > 500
            || self.roughness_index_permille > 1_000
            || (self.roughness_sideband_fraction_permille == 0)
                != (self.roughness_index_permille == 0)
            || self.stochastic_residual_gain_q15 > 2_048
            || (self.stochastic_residual_gain_q15 == 0) != (self.stochastic_residual_t20_ms == 0)
            || (self.stochastic_residual_gain_q15 > 0
                && !(150..=2_000).contains(&self.stochastic_residual_t20_ms))
        {
            return Err(ExperimentalSteelSearchError::ProfileOutsideBounds);
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ExperimentalSteelSearchError {
    ProfileOutsideBounds,
    CookedCoefficientOutsideBounds,
}

impl Display for ExperimentalSteelSearchError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ProfileOutsideBounds => {
                formatter.write_str("experimental steel search profile is outside its bounds")
            }
            Self::CookedCoefficientOutsideBounds => {
                formatter.write_str("experimental steel search coefficient is outside i64")
            }
        }
    }
}

impl Error for ExperimentalSteelSearchError {}

/// Heap-backed only for the offline search command.
#[derive(Clone, Debug)]
pub(super) struct SearchModalVoice {
    modes: Vec<ModeState>,
    profiles: Box<[ModeProfile]>,
    pan_q16: i32,
    energy_q16: u32,
    strike_transient: StrikeTransientProfile,
    stochastic_residual: StochasticResidualProfile,
    stochastic_residual_envelope_q30: i64,
    noise_state: u32,
    frames_elapsed: u32,
    frames_remaining: u32,
}

#[derive(Clone, Copy, Debug)]
enum StochasticResidualProfile {
    None,
    DecayingWhite {
        gain_q15: i64,
        decay_coefficient_q30: i64,
    },
}

impl SearchModalVoice {
    fn new(
        excitation: PhysicalSoundExcitation,
        profiles: Box<[ModeProfile]>,
        point_factors: &[i16],
        strike_transient: StrikeTransientProfile,
        stochastic_residual: StochasticResidualProfile,
        duration_frames: u32,
    ) -> Self {
        debug_assert_eq!(profiles.len(), point_factors.len());
        let mut modes = vec![ModeState::default(); profiles.len()];
        for (index, mode) in modes.iter_mut().enumerate() {
            let modal_amplitude = STRIKE_AMPLITUDE
                * i64::from(excitation.energy_q16)
                * profiles[index].gain_q15
                * i64::from(point_factors[index])
                / Q16_ONE
                / Q15_ONE
                / Q15_ONE;
            mode.current = modal_amplitude * profiles[index].initial_sine_q15 / Q15_ONE;
        }
        let seed = if excitation.strike_seed == 0 {
            0x9e37_79b9
        } else {
            excitation.strike_seed
        };
        Self {
            modes,
            profiles,
            pan_q16: excitation.pan_q16,
            energy_q16: excitation.energy_q16,
            strike_transient,
            stochastic_residual,
            stochastic_residual_envelope_q30: Q30_ONE_F64 as i64,
            noise_state: seed,
            frames_elapsed: 0,
            frames_remaining: duration_frames,
        }
    }

    pub(super) fn next_mono_sample(&mut self) -> i64 {
        let mut sample = 0_i64;
        for (state, profile) in self.modes.iter_mut().zip(&self.profiles) {
            sample += state.current;
            let next = ((profile.coefficient_a_q30 * state.current) >> 30)
                - ((profile.coefficient_b_q30 * state.previous) >> 30);
            state.previous = state.current;
            state.current = next;
        }
        if let StrikeTransientProfile::WhiteNoise { gain_q15, frames } = self.strike_transient
            && self.frames_elapsed < frames
        {
            let noise = self.next_noise_sample();
            let frames_remaining = frames - self.frames_elapsed;
            sample += noise * i64::from(self.energy_q16) * gain_q15 * i64::from(frames_remaining)
                / Q16_ONE
                / Q15_ONE
                / i64::from(frames);
        }
        if let StochasticResidualProfile::DecayingWhite {
            gain_q15,
            decay_coefficient_q30,
        } = self.stochastic_residual
        {
            let noise = self.next_noise_sample();
            let residual = noise * i64::from(self.energy_q16) / Q16_ONE * gain_q15 / Q15_ONE;
            sample += residual * self.stochastic_residual_envelope_q30 / Q30_ONE_F64 as i64;
            self.stochastic_residual_envelope_q30 =
                self.stochastic_residual_envelope_q30 * decay_coefficient_q30 / Q30_ONE_F64 as i64;
        }
        if self.frames_remaining < FINAL_FADE_FRAMES {
            sample = sample * i64::from(self.frames_remaining) / i64::from(FINAL_FADE_FRAMES);
        }
        self.frames_elapsed = self.frames_elapsed.saturating_add(1);
        self.frames_remaining = self.frames_remaining.saturating_sub(1);
        sample
    }

    pub(super) const fn pan_q16(&self) -> i32 {
        self.pan_q16
    }

    pub(super) const fn frames_remaining(&self) -> u32 {
        self.frames_remaining
    }

    fn next_noise_sample(&mut self) -> i64 {
        self.noise_state ^= self.noise_state << 13;
        self.noise_state ^= self.noise_state >> 17;
        self.noise_state ^= self.noise_state << 5;
        i64::from(self.noise_state as i16)
    }
}

/// Renders one steel search candidate through the same integer recurrence and
/// mixer semantics used by the current laboratory profile.
pub fn render_experimental_steel_search_impact(
    search_profile: ExperimentalSteelSearchProfile,
    impact_point: PhysicalSoundImpactPoint,
    energy_q16: u32,
    strike_seed: u32,
) -> Result<Vec<i16>, ExperimentalSteelSearchError> {
    search_profile.validate()?;
    let (profiles, point_factors) = cook_profiles(search_profile, impact_point)?;
    let stochastic_residual = cook_stochastic_residual(search_profile)?;
    let excitation = PhysicalSoundExcitation::new(
        PhysicalSoundMaterial::Steel,
        impact_point,
        energy_q16,
        0,
        strike_seed,
    );
    let duration_frames =
        (24_000_u32 * u32::from(search_profile.decay_scale_permille) / 1_000).clamp(24_000, 96_000);
    let mut mixer = ExperimentalPhysicalSoundMixer::default();
    mixer
        .voices
        .push(ExperimentalVoice::SearchModal(SearchModalVoice::new(
            excitation,
            profiles.into_boxed_slice(),
            &point_factors,
            StrikeTransientProfile::WhiteNoise {
                gain_q15: i64::from(search_profile.transient_gain_q15),
                frames: u32::from(search_profile.transient_frames),
            },
            stochastic_residual,
            duration_frames,
        )));
    let mut samples = mixer.mix_tick(&[]);
    while mixer.active_voice_count() > 0 {
        samples.extend(mixer.mix_tick(&[]));
    }
    Ok(samples)
}

fn cook_stochastic_residual(
    search_profile: ExperimentalSteelSearchProfile,
) -> Result<StochasticResidualProfile, ExperimentalSteelSearchError> {
    if search_profile.stochastic_residual_gain_q15 == 0 {
        return Ok(StochasticResidualProfile::None);
    }
    let t20_seconds = f64::from(search_profile.stochastic_residual_t20_ms) / 1_000.0;
    let damping_per_second = 10.0_f64.ln() / t20_seconds;
    let decay = (-damping_per_second / f64::from(SAMPLE_RATE_HZ)).exp();
    Ok(StochasticResidualProfile::DecayingWhite {
        gain_q15: i64::from(search_profile.stochastic_residual_gain_q15),
        decay_coefficient_q30: quantize(decay, Q30_ONE_F64)?,
    })
}

fn cook_profiles(
    search_profile: ExperimentalSteelSearchProfile,
    impact_point: PhysicalSoundImpactPoint,
) -> Result<(Vec<ModeProfile>, Vec<i16>), ExperimentalSteelSearchError> {
    let frequency_scale = f64::from(search_profile.frequency_scale_permille) / 1_000.0;
    let decay_scale = f64::from(search_profile.decay_scale_permille) / 1_000.0;
    let last_mode_index = (MAX_MODE_COUNT - 1) as i64;
    let roughness_index = f64::from(search_profile.roughness_index_permille) / 1_000.0;
    let gain_normalization = (1.0 + 0.5 * roughness_index * roughness_index).sqrt();
    let base_point_factors = impact_point.factors_q15(PhysicalSoundMaterial::Steel);
    let sideband_count = usize::from(search_profile.roughness_index_permille > 0) * 2;
    let mut profiles = Vec::with_capacity(MAX_MODE_COUNT * (1 + sideband_count));
    let mut point_factors = Vec::with_capacity(profiles.capacity());

    for (index, base) in STEEL_MODES.iter().enumerate() {
        let frequency_hz = STEEL_FREQUENCIES_HZ[index] * frequency_scale;
        let t20_seconds = STEEL_T20_MILLISECONDS[index] * decay_scale / 1_000.0;
        let high_gain_delta = i64::from(search_profile.high_mode_gain_permille) - 1_000;
        let gain_scale_permille =
            1_000 + high_gain_delta * i64::try_from(index).unwrap_or_default() / last_mode_index;
        let tilted_gain = base.gain_q15 * gain_scale_permille / 1_000;
        let center_gain = (tilted_gain as f64 / gain_normalization).round() as i64;
        profiles.push(cook_mode(frequency_hz, t20_seconds, center_gain)?);
        point_factors.push(base_point_factors[index]);

        if search_profile.roughness_index_permille > 0 {
            let frequency_khz = frequency_hz / 1_000.0;
            let critical_bandwidth_hz =
                25.0 + 75.0 * (1.0 + 1.4 * frequency_khz * frequency_khz).powf(0.69);
            let sideband_offset_hz = critical_bandwidth_hz
                * f64::from(search_profile.roughness_sideband_fraction_permille)
                / 1_000.0;
            let sideband_gain =
                (tilted_gain as f64 * roughness_index * 0.5 / gain_normalization).round() as i64;
            for sideband_frequency_hz in [
                frequency_hz - sideband_offset_hz,
                frequency_hz + sideband_offset_hz,
            ] {
                profiles.push(cook_mode(
                    sideband_frequency_hz,
                    t20_seconds,
                    sideband_gain,
                )?);
                point_factors.push(base_point_factors[index]);
            }
        }
    }
    Ok((profiles, point_factors))
}

fn cook_mode(
    frequency_hz: f64,
    t20_seconds: f64,
    gain_q15: i64,
) -> Result<ModeProfile, ExperimentalSteelSearchError> {
    let damping_per_second = 10.0_f64.ln() / t20_seconds;
    let decay = (-damping_per_second / f64::from(SAMPLE_RATE_HZ)).exp();
    let phase = TAU * frequency_hz / f64::from(SAMPLE_RATE_HZ);
    Ok(ModeProfile {
        coefficient_a_q30: quantize(2.0 * decay * phase.cos(), Q30_ONE_F64)?,
        coefficient_b_q30: quantize(decay * decay, Q30_ONE_F64)?,
        gain_q15,
        initial_sine_q15: quantize(phase.sin(), Q15_ONE as f64)?,
    })
}

fn quantize(value: f64, scale: f64) -> Result<i64, ExperimentalSteelSearchError> {
    let scaled = value * scale;
    if !scaled.is_finite() || scaled < i64::MIN as f64 || scaled > i64::MAX as f64 {
        Err(ExperimentalSteelSearchError::CookedCoefficientOutsideBounds)
    } else {
        Ok(scaled.round() as i64)
    }
}
