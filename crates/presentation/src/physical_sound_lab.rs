//! Experimental fixed-point modal sound laboratory for SPEC-45 P0/P0.5.
//!
//! This module is deliberately presentation-only and is not a shipped asset,
//! save, replay or gameplay contract. It provides a byte-exact reference
//! synthesizer that can be auditioned offline and, behind an explicit Cargo
//! feature in the reference game, mixed after the ordinary baseline mixer.

const SAMPLE_RATE_HZ: u32 = 48_000;
const FRAMES_PER_TICK: usize = 1_600;
const CHANNEL_COUNT: usize = 2;
const MAX_MODE_COUNT: usize = 12;
const Q15_ONE: i64 = 32_768;
const Q16_ONE: i64 = 65_536;
const MAX_VOICES: usize = 16;
const STRIKE_AMPLITUDE: i64 = 10_000;
const FINAL_FADE_FRAMES: u32 = 4_800;

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum PhysicalSoundMaterial {
    Steel,
    Wood,
    Glass,
}

impl PhysicalSoundMaterial {
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Steel => "steel",
            Self::Wood => "wood",
            Self::Glass => "glass",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum PhysicalSoundImpactPoint {
    Center,
    Edge,
    Corner,
}

impl PhysicalSoundImpactPoint {
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Center => "center",
            Self::Edge => "edge",
            Self::Corner => "corner",
        }
    }

    const fn factors_q15(self, material: PhysicalSoundMaterial) -> [i16; MAX_MODE_COUNT] {
        match (material, self) {
            (PhysicalSoundMaterial::Steel, Self::Center) => [
                32_767, 18_000, -22_000, 14_000, 28_000, 24_000, -26_000, 30_000, 22_000, -28_000,
                32_767, 26_000,
            ],
            (PhysicalSoundMaterial::Steel, Self::Edge) => [
                16_000, 32_767, 25_000, -30_000, 18_000, 32_767, 24_000, -22_000, 30_000, 26_000,
                -18_000, 32_767,
            ],
            (PhysicalSoundMaterial::Steel, Self::Corner) => [
                10_000, 22_000, 32_767, 28_000, -20_000, -18_000, 30_000, 26_000, -26_000, 32_767,
                28_000, -24_000,
            ],
            (_, Self::Center) => [32_767, 8_192, -22_938, 6_554, 18_022, 0, 0, 0, 0, 0, 0, 0],
            (_, Self::Edge) => [14_746, 32_767, 11_469, -26_214, 21_299, 0, 0, 0, 0, 0, 0, 0],
            (_, Self::Corner) => [9_830, 24_576, 32_767, 19_661, -16_384, 0, 0, 0, 0, 0, 0, 0],
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PhysicalSoundExcitation {
    pub material: PhysicalSoundMaterial,
    pub impact_point: PhysicalSoundImpactPoint,
    /// Presentation-only impact strength in Q16.16, clamped to `[0, 1]`.
    pub energy_q16: u32,
    /// Stereo pan in Q16.16, clamped to `[-1, 1]`.
    pub pan_q16: i32,
    /// Stable seed for the short strike-noise component.
    pub strike_seed: u32,
}

impl PhysicalSoundExcitation {
    #[must_use]
    pub const fn new(
        material: PhysicalSoundMaterial,
        impact_point: PhysicalSoundImpactPoint,
        energy_q16: u32,
        pan_q16: i32,
        strike_seed: u32,
    ) -> Self {
        Self {
            material,
            impact_point,
            energy_q16: if energy_q16 > Q16_ONE as u32 {
                Q16_ONE as u32
            } else {
                energy_q16
            },
            pan_q16: if pan_q16 < -(Q16_ONE as i32) {
                -(Q16_ONE as i32)
            } else if pan_q16 > Q16_ONE as i32 {
                Q16_ONE as i32
            } else {
                pan_q16
            },
            strike_seed,
        }
    }
}

#[derive(Clone, Copy, Debug)]
struct ModeProfile {
    coefficient_a_q30: i64,
    coefficient_b_q30: i64,
    gain_q15: i64,
    initial_sine_q15: i64,
}

#[derive(Clone, Copy, Debug, Default)]
struct ModeState {
    previous: i64,
    current: i64,
}

#[derive(Clone, Debug)]
struct ModalVoice {
    modes: [ModeState; MAX_MODE_COUNT],
    profiles: &'static [ModeProfile],
    pan_q16: i32,
    energy_q16: u32,
    noise_gain_q15: i64,
    noise_state: u32,
    noise_frames_remaining: u32,
    noise_frames_total: u32,
    frames_remaining: u32,
}

impl ModalVoice {
    fn new(excitation: PhysicalSoundExcitation) -> Self {
        let profile = material_profile(excitation.material);
        let point_factors = excitation.impact_point.factors_q15(excitation.material);
        let mut modes = [ModeState::default(); MAX_MODE_COUNT];
        for (index, mode) in modes.iter_mut().take(profile.modes.len()).enumerate() {
            let modal_amplitude = STRIKE_AMPLITUDE
                * i64::from(excitation.energy_q16)
                * profile.modes[index].gain_q15
                * i64::from(point_factors[index])
                / Q16_ONE
                / Q15_ONE
                / Q15_ONE;
            mode.current = modal_amplitude * profile.modes[index].initial_sine_q15 / Q15_ONE;
        }
        let seed = if excitation.strike_seed == 0 {
            0x9e37_79b9
        } else {
            excitation.strike_seed
        };
        Self {
            modes,
            profiles: profile.modes,
            pan_q16: excitation.pan_q16,
            energy_q16: excitation.energy_q16,
            noise_gain_q15: profile.noise_gain_q15,
            noise_state: seed,
            noise_frames_remaining: profile.noise_frames,
            noise_frames_total: profile.noise_frames,
            frames_remaining: profile.duration_frames,
        }
    }

    fn next_mono_sample(&mut self) -> i64 {
        let mut sample = 0_i64;
        for (state, profile) in self.modes.iter_mut().zip(self.profiles) {
            sample += state.current;
            let next = ((profile.coefficient_a_q30 * state.current) >> 30)
                - ((profile.coefficient_b_q30 * state.previous) >> 30);
            state.previous = state.current;
            state.current = next;
        }
        if self.noise_frames_remaining > 0 {
            self.noise_state ^= self.noise_state << 13;
            self.noise_state ^= self.noise_state >> 17;
            self.noise_state ^= self.noise_state << 5;
            let noise = i64::from(self.noise_state as i16);
            sample += noise
                * i64::from(self.energy_q16)
                * self.noise_gain_q15
                * i64::from(self.noise_frames_remaining)
                / Q16_ONE
                / Q15_ONE
                / i64::from(self.noise_frames_total);
            self.noise_frames_remaining -= 1;
        }
        if self.frames_remaining < FINAL_FADE_FRAMES {
            sample = sample * i64::from(self.frames_remaining) / i64::from(FINAL_FADE_FRAMES);
        }
        self.frames_remaining = self.frames_remaining.saturating_sub(1);
        sample
    }
}

struct MaterialProfile {
    modes: &'static [ModeProfile],
    noise_gain_q15: i64,
    noise_frames: u32,
    duration_frames: u32,
}

// A bounded plate-like steel profile fitted to the modal/decay envelope of a
// small external CC0 metal-impact screen. Frequencies stay fixed across impact
// positions; only participation changes. This remains experimental P0 data.
const STEEL_MODES: [ModeProfile; MAX_MODE_COUNT] = [
    // 215.3 Hz / T20 270 ms
    ModeProfile {
        coefficient_a_q30: 2_146_249_514,
        coefficient_b_q30: 1_073_360_351,
        gain_q15: 6_554,
        initial_sine_q15: 923,
    },
    // 409.1 Hz / T20 240 ms
    ModeProfile {
        coefficient_a_q30: 2_143_976_622,
        coefficient_b_q30: 1_073_312_677,
        gain_q15: 8_192,
        initial_sine_q15: 1_754,
    },
    // 710.6 Hz / T20 195 ms
    ModeProfile {
        coefficient_a_q30: 2_137_674_153,
        coefficient_b_q30: 1_073_213_667,
        gain_q15: 10_486,
        initial_sine_q15: 3_044,
    },
    // 807.5 Hz / T20 180 ms
    ModeProfile {
        coefficient_a_q30: 2_134_929_062,
        coefficient_b_q30: 1_073_169_666,
        gain_q15: 9_830,
        initial_sine_q15: 3_457,
    },
    // 1,076.7 Hz / T20 150 ms
    ModeProfile {
        coefficient_a_q30: 2_125_510_270,
        coefficient_b_q30: 1_073_055_271,
        gain_q15: 18_022,
        initial_sine_q15: 4_603,
    },
    // 1,453.5 Hz / T20 135 ms
    ModeProfile {
        coefficient_a_q30: 2_107_982_241,
        coefficient_b_q30: 1_072_979_014,
        gain_q15: 15_729,
        initial_sine_q15: 6_197,
    },
    // 1,857.2 Hz / T20 120 ms
    ModeProfile {
        coefficient_a_q30: 2_083_503_335,
        coefficient_b_q30: 1_072_883_701,
        gain_q15: 13_763,
        initial_sine_q15: 7_888,
    },
    // 2,153.3 Hz / T20 135 ms
    ModeProfile {
        coefficient_a_q30: 2_062_006_809,
        coefficient_b_q30: 1_072_979_014,
        gain_q15: 20_316,
        initial_sine_q15: 9_114,
    },
    // 2,659.4 Hz / T20 112.5 ms
    ModeProfile {
        coefficient_a_q30: 2_017_811_680,
        coefficient_b_q30: 1_072_826_517,
        gain_q15: 18_022,
        initial_sine_q15: 11_178,
    },
    // 3,203.1 Hz / T20 97.5 ms
    ModeProfile {
        coefficient_a_q30: 1_960_504_514,
        coefficient_b_q30: 1_072_685_770,
        gain_q15: 16_384,
        initial_sine_q15: 13_340,
    },
    // 3,644.5 Hz / T20 82.5 ms
    ModeProfile {
        coefficient_a_q30: 1_906_601_530,
        coefficient_b_q30: 1_072_493_872,
        gain_q15: 14_746,
        initial_sine_q15: 15_046,
    },
    // 5,695.5 Hz / T20 60 ms
    ModeProfile {
        coefficient_a_q30: 1_576_543_008,
        coefficient_b_q30: 1_072_026_264,
        gain_q15: 10_486,
        initial_sine_q15: 22_229,
    },
];

const WOOD_MODES: [ModeProfile; 5] = [
    ModeProfile {
        coefficient_a_q30: 2_146_788_181,
        coefficient_b_q30: 1_073_642_408,
        gain_q15: 23_593,
        initial_sine_q15: 772,
    },
    ModeProfile {
        coefficient_a_q30: 2_144_111_380,
        coefficient_b_q30: 1_073_614_005,
        gain_q15: 15_729,
        initial_sine_q15: 1_801,
    },
    ModeProfile {
        coefficient_a_q30: 2_135_264_964,
        coefficient_b_q30: 1_073_582_053,
        gain_q15: 10_158,
        initial_sine_q15: 3_468,
    },
    ModeProfile {
        coefficient_a_q30: 2_108_697_596,
        coefficient_b_q30: 1_073_518_151,
        gain_q15: 6_226,
        initial_sine_q15: 6_182,
    },
    ModeProfile {
        coefficient_a_q30: 2_034_963_840,
        coefficient_b_q30: 1_073_369_062,
        gain_q15: 3_604,
        initial_sine_q15: 10_452,
    },
];

const GLASS_MODES: [ModeProfile; 5] = [
    ModeProfile {
        coefficient_a_q30: 2_136_272_041,
        coefficient_b_q30: 1_073_713_862,
        gain_q15: 17_039,
        initial_sine_q15: 3_340,
    },
    ModeProfile {
        coefficient_a_q30: 2_090_709_929,
        coefficient_b_q30: 1_073_706_033,
        gain_q15: 15_729,
        initial_sine_q15: 7_483,
    },
    ModeProfile {
        coefficient_a_q30: 1_957_178_328,
        coefficient_b_q30: 1_073_692_115,
        gain_q15: 11_469,
        initial_sine_q15: 13_485,
    },
    ModeProfile {
        coefficient_a_q30: 1_610_795_325,
        coefficient_b_q30: 1_073_667_261,
        gain_q15: 7_209,
        initial_sine_q15: 21_670,
    },
    ModeProfile {
        coefficient_a_q30: 847_659_503,
        coefficient_b_q30: 1_073_624_096,
        gain_q15: 3_932,
        initial_sine_q15: 30_107,
    },
];

const fn material_profile(material: PhysicalSoundMaterial) -> MaterialProfile {
    match material {
        PhysicalSoundMaterial::Steel => MaterialProfile {
            modes: &STEEL_MODES,
            noise_gain_q15: 8_192,
            noise_frames: 960,
            duration_frames: 24_000,
        },
        PhysicalSoundMaterial::Wood => MaterialProfile {
            modes: &WOOD_MODES,
            noise_gain_q15: 6_554,
            noise_frames: 288,
            duration_frames: 48_000,
        },
        PhysicalSoundMaterial::Glass => MaterialProfile {
            modes: &GLASS_MODES,
            noise_gain_q15: 2_621,
            noise_frames: 96,
            duration_frames: 96_000,
        },
    }
}

/// Bounded, cloneable presentation state used by both offline audition and
/// the feature-gated reference demo integration.
#[derive(Clone, Debug, Default)]
pub struct ExperimentalPhysicalSoundMixer {
    voices: Vec<ModalVoice>,
    admitted_impacts: u64,
    dropped_impacts: u64,
    clipped_samples: u64,
}

impl ExperimentalPhysicalSoundMixer {
    #[must_use]
    pub const fn sample_rate_hz() -> u32 {
        SAMPLE_RATE_HZ
    }

    #[must_use]
    pub const fn frames_per_tick() -> usize {
        FRAMES_PER_TICK
    }

    #[must_use]
    pub const fn active_voice_count(&self) -> usize {
        self.voices.len()
    }

    #[must_use]
    pub const fn admitted_impacts(&self) -> u64 {
        self.admitted_impacts
    }

    #[must_use]
    pub const fn dropped_impacts(&self) -> u64 {
        self.dropped_impacts
    }

    #[must_use]
    pub const fn clipped_samples(&self) -> u64 {
        self.clipped_samples
    }

    /// Admits this tick's immutable excitations and returns one interleaved
    /// stereo S16 window. Oldest voices are deterministically preempted at the
    /// fixed laboratory bound.
    pub fn mix_tick(&mut self, excitations: &[PhysicalSoundExcitation]) -> Vec<i16> {
        for excitation in excitations {
            if excitation.energy_q16 == 0 {
                self.dropped_impacts = self.dropped_impacts.saturating_add(1);
                continue;
            }
            if self.voices.len() == MAX_VOICES {
                self.voices.remove(0);
                self.dropped_impacts = self.dropped_impacts.saturating_add(1);
            }
            self.voices.push(ModalVoice::new(*excitation));
            self.admitted_impacts = self.admitted_impacts.saturating_add(1);
        }

        let mut output = Vec::with_capacity(FRAMES_PER_TICK * CHANNEL_COUNT);
        for _ in 0..FRAMES_PER_TICK {
            let mut left = 0_i64;
            let mut right = 0_i64;
            for voice in &mut self.voices {
                let mono = voice.next_mono_sample();
                let pan = i64::from(voice.pan_q16);
                left += mono * (Q16_ONE - pan) / (Q16_ONE * 2);
                right += mono * (Q16_ONE + pan) / (Q16_ONE * 2);
            }
            for sample in [left, right] {
                let clamped = sample.clamp(i64::from(i16::MIN), i64::from(i16::MAX));
                if clamped != sample {
                    self.clipped_samples = self.clipped_samples.saturating_add(1);
                }
                output.push(clamped as i16);
            }
        }
        self.voices.retain(|voice| voice.frames_remaining > 0);
        output
    }
}

/// Adds a laboratory stereo window after the ordinary presentation mixer.
/// Length mismatch is a bounded no-op; no caller can affect gameplay state.
pub fn mix_physical_sound_in_place(baseline: &mut [i16], physical: &[i16]) {
    if baseline.len() != physical.len() {
        return;
    }
    for (baseline, physical) in baseline.iter_mut().zip(physical) {
        *baseline = i32::from(*baseline)
            .saturating_add(i32::from(*physical))
            .clamp(i32::from(i16::MIN), i32::from(i16::MAX)) as i16;
    }
}

/// Renders one complete laboratory impact, including the bounded decay tail.
#[must_use]
pub fn render_physical_sound_impact(excitation: PhysicalSoundExcitation) -> Vec<i16> {
    let mut mixer = ExperimentalPhysicalSoundMixer::default();
    let mut samples = mixer.mix_tick(std::slice::from_ref(&excitation));
    while mixer.active_voice_count() > 0 {
        samples.extend(mixer.mix_tick(&[]));
    }
    samples
}

/// Six-second comparison sequence used by the independent audition command.
#[must_use]
pub fn render_physical_sound_lab_sequence() -> Vec<i16> {
    let mut mixer = ExperimentalPhysicalSoundMixer::default();
    let mut output = Vec::with_capacity(SAMPLE_RATE_HZ as usize * 6 * CHANNEL_COUNT);
    for tick in 0..180_u32 {
        let excitation = match tick {
            3 => Some(PhysicalSoundExcitation::new(
                PhysicalSoundMaterial::Steel,
                PhysicalSoundImpactPoint::Center,
                49_152,
                -32_768,
                0x51ee_1001,
            )),
            63 => Some(PhysicalSoundExcitation::new(
                PhysicalSoundMaterial::Wood,
                PhysicalSoundImpactPoint::Edge,
                57_344,
                0,
                0x700d_2002,
            )),
            108 => Some(PhysicalSoundExcitation::new(
                PhysicalSoundMaterial::Glass,
                PhysicalSoundImpactPoint::Corner,
                45_875,
                32_768,
                0x61a5_3003,
            )),
            _ => None,
        };
        output.extend(match excitation {
            Some(excitation) => mixer.mix_tick(std::slice::from_ref(&excitation)),
            None => mixer.mix_tick(&[]),
        });
    }
    output
}

#[cfg(test)]
mod tests {
    use next_contracts::canonical::sha256;
    use next_contracts::ids::ContentHash;

    use super::*;

    #[test]
    fn fixed_point_render_is_byte_exact_and_profiles_are_distinct() {
        let render = |material| {
            render_physical_sound_impact(PhysicalSoundExcitation::new(
                material,
                PhysicalSoundImpactPoint::Edge,
                49_152,
                0,
                7,
            ))
        };
        let steel = render(PhysicalSoundMaterial::Steel);
        let wood = render(PhysicalSoundMaterial::Wood);
        let glass = render(PhysicalSoundMaterial::Glass);
        assert_eq!(steel, render(PhysicalSoundMaterial::Steel));
        assert_ne!(
            sha256(&samples_as_bytes(&steel)),
            sha256(&samples_as_bytes(&wood))
        );
        assert_ne!(
            sha256(&samples_as_bytes(&steel)),
            sha256(&samples_as_bytes(&glass))
        );
        assert_ne!(
            sha256(&samples_as_bytes(&wood)),
            sha256(&samples_as_bytes(&glass))
        );
        assert!(steel.iter().any(|sample| *sample != 0));
        assert!(wood.iter().any(|sample| *sample != 0));
        assert!(glass.iter().any(|sample| *sample != 0));
    }

    #[test]
    fn calibrated_steel_profile_retains_exact_pcm_and_position_response() {
        let render = |impact_point| {
            render_physical_sound_impact(PhysicalSoundExcitation::new(
                PhysicalSoundMaterial::Steel,
                impact_point,
                49_152,
                0,
                0x51ee_0101,
            ))
        };
        let center = render(PhysicalSoundImpactPoint::Center);
        let edge = render(PhysicalSoundImpactPoint::Edge);
        let corner = render(PhysicalSoundImpactPoint::Corner);

        assert_eq!(center.len(), 48_000);
        assert_eq!(
            ContentHash::from_bytes(sha256(&samples_as_bytes(&center))).to_hex(),
            "319befa843e591e936ccf8da531626146312721ac820e56029a07fed5bae896a"
        );
        assert_ne!(center, edge);
        assert_ne!(center, corner);
        assert_ne!(edge, corner);
    }

    #[test]
    fn excitation_and_voice_bounds_are_fail_bounded() {
        let excitation = PhysicalSoundExcitation::new(
            PhysicalSoundMaterial::Glass,
            PhysicalSoundImpactPoint::Corner,
            u32::MAX,
            i32::MAX,
            0,
        );
        assert_eq!(excitation.energy_q16, 65_536);
        assert_eq!(excitation.pan_q16, 65_536);
        let mut mixer = ExperimentalPhysicalSoundMixer::default();
        let window = mixer.mix_tick(&vec![excitation; MAX_VOICES + 2]);
        assert_eq!(window.len(), FRAMES_PER_TICK * CHANNEL_COUNT);
        assert_eq!(mixer.active_voice_count(), MAX_VOICES);
        assert_eq!(mixer.dropped_impacts(), 2);
    }

    fn samples_as_bytes(samples: &[i16]) -> Vec<u8> {
        samples
            .iter()
            .flat_map(|sample| sample.to_le_bytes())
            .collect()
    }
}
