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
            (PhysicalSoundMaterial::Wood, Self::Center) => [
                32_767, 24_000, -18_000, 28_000, 22_000, -20_000, 16_000, 26_000, -14_000, 12_000,
                -9_000, 7_000,
            ],
            (PhysicalSoundMaterial::Wood, Self::Edge) => [
                14_000, 32_767, 26_000, -18_000, 22_000, 30_000, -24_000, 18_000, 28_000, -20_000,
                16_000, -12_000,
            ],
            (PhysicalSoundMaterial::Wood, Self::Corner) => [
                9_000, 18_000, 32_767, 24_000, -28_000, 22_000, 30_000, -26_000, 18_000, 32_767,
                -22_000, 14_000,
            ],
            (PhysicalSoundMaterial::Glass, Self::Center) => [
                18_000, 14_000, 32_767, 28_000, 24_000, -22_000, 30_000, 26_000, -18_000, 22_000,
                -14_000, 12_000,
            ],
            (PhysicalSoundMaterial::Glass, Self::Edge) => [
                12_000, 22_000, 20_000, 32_767, -26_000, 30_000, 24_000, -18_000, 28_000, 32_000,
                -22_000, 18_000,
            ],
            (PhysicalSoundMaterial::Glass, Self::Corner) => [
                8_000, 16_000, 14_000, 24_000, 32_767, -28_000, 22_000, 30_000, -26_000, 18_000,
                32_000, -24_000,
            ],
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

// A dry hardwood-block candidate screened against two small external CC0
// impact packs. The short, frequency-dependent T20 values keep the body woody
// rather than metallic; impact position changes participation, never pitch.
const WOOD_MODES: [ModeProfile; MAX_MODE_COUNT] = [
    // 140 Hz / T20 90 ms
    ModeProfile {
        coefficient_a_q30: 2_145_978_928,
        coefficient_b_q30: 1_072_597_813,
        gain_q15: 7_000,
        initial_sine_q15: 600,
    },
    // 225 Hz / T20 82.5 ms
    ModeProfile {
        coefficient_a_q30: 2_145_304_529,
        coefficient_b_q30: 1_072_493_872,
        gain_q15: 12_000,
        initial_sine_q15: 965,
    },
    // 315 Hz / T20 75 ms
    ModeProfile {
        coefficient_a_q30: 2_144_286_398,
        coefficient_b_q30: 1_072_369_157,
        gain_q15: 12_000,
        initial_sine_q15: 1_351,
    },
    // 485 Hz / T20 65 ms
    ModeProfile {
        coefficient_a_q30: 2_141_576_283,
        coefficient_b_q30: 1_072_158_133,
        gain_q15: 18_000,
        initial_sine_q15: 2_079,
    },
    // 590 Hz / T20 60 ms
    ModeProfile {
        coefficient_a_q30: 2_139_371_261,
        coefficient_b_q30: 1_072_026_264,
        gain_q15: 18_000,
        initial_sine_q15: 2_528,
    },
    // 815 Hz / T20 55 ms
    ModeProfile {
        coefficient_a_q30: 2_133_413_083,
        coefficient_b_q30: 1_071_870_440,
        gain_q15: 16_000,
        initial_sine_q15: 3_489,
    },
    // 1,080 Hz / T20 50 ms
    ModeProfile {
        coefficient_a_q30: 2_124_020_830,
        coefficient_b_q30: 1_071_683_481,
        gain_q15: 16_000,
        initial_sine_q15: 4_617,
    },
    // 1,185 Hz / T20 47.5 ms
    ModeProfile {
        coefficient_a_q30: 2_119_558_454,
        coefficient_b_q30: 1_071_575_257,
        gain_q15: 14_000,
        initial_sine_q15: 5_062,
    },
    // 1,715 Hz / T20 42.5 ms
    ModeProfile {
        coefficient_a_q30: 2_091_235_312,
        coefficient_b_q30: 1_071_320_654,
        gain_q15: 12_000,
        initial_sine_q15: 7_295,
    },
    // 2,400 Hz / T20 35 ms
    ModeProfile {
        coefficient_a_q30: 2_039_580_979,
        coefficient_b_q30: 1_070_802_543,
        gain_q15: 8_000,
        initial_sine_q15: 10_126,
    },
    // 3,150 Hz / T20 30 ms
    ModeProfile {
        coefficient_a_q30: 1_964_355_106,
        coefficient_b_q30: 1_070_313_445,
        gain_q15: 5_000,
        initial_sine_q15: 13_132,
    },
    // 4,100 Hz / T20 25 ms
    ModeProfile {
        coefficient_a_q30: 1_842_023_310,
        coefficient_b_q30: 1_069_629_084,
        gain_q15: 3_000,
        initial_sine_q15: 16_754,
    },
];

// A thick glass-plate candidate screened against two small external CC0 glass
// packs. Its inharmonic upper modes and longer T20 distinguish it from the dry
// wood body without stretching the experimental voice to the old two seconds.
const GLASS_MODES: [ModeProfile; MAX_MODE_COUNT] = [
    // 1,172 Hz / T20 350 ms
    ModeProfile {
        coefficient_a_q30: 2_121_970_743,
        coefficient_b_q30: 1_073_447_533,
        gain_q15: 4_000,
        initial_sine_q15: 5_007,
    },
    // 1,645 Hz / T20 300 ms
    ModeProfile {
        coefficient_a_q30: 2_097_554_096,
        coefficient_b_q30: 1_073_398_493,
        gain_q15: 6_000,
        initial_sine_q15: 7_002,
    },
    // 2,208 Hz / T20 275 ms
    ModeProfile {
        coefficient_a_q30: 2_058_050_834,
        coefficient_b_q30: 1_073_367_286,
        gain_q15: 16_000,
        initial_sine_q15: 9_340,
    },
    // 2,732 Hz / T20 250 ms
    ModeProfile {
        coefficient_a_q30: 2_011_233_483,
        coefficient_b_q30: 1_073_329_839,
        gain_q15: 20_000,
        initial_sine_q15: 11_470,
    },
    // 3,533 Hz / T20 325 ms
    ModeProfile {
        coefficient_a_q30: 1_921_615_061,
        coefficient_b_q30: 1_073_424_899,
        gain_q15: 17_000,
        initial_sine_q15: 14_620,
    },
    // 3,838 Hz / T20 300 ms
    ModeProfile {
        coefficient_a_q30: 1_881_824_117,
        coefficient_b_q30: 1_073_398_493,
        gain_q15: 18_000,
        initial_sine_q15: 15_779,
    },
    // 4,638 Hz / T20 225 ms
    ModeProfile {
        coefficient_a_q30: 1_763_349_309,
        coefficient_b_q30: 1_073_284_073,
        gain_q15: 15_000,
        initial_sine_q15: 18_694,
    },
    // 5,212 Hz / T20 212.5 ms
    ModeProfile {
        coefficient_a_q30: 1_666_407_102,
        coefficient_b_q30: 1_073_257_153,
        gain_q15: 14_000,
        initial_sine_q15: 20_662,
    },
    // 6,171 Hz / T20 187.5 ms
    ModeProfile {
        coefficient_a_q30: 1_483_753_158,
        coefficient_b_q30: 1_073_192_546,
        gain_q15: 11_000,
        initial_sine_q15: 23_683,
    },
    // 7,059 Hz / T20 168.75 ms
    ModeProfile {
        coefficient_a_q30: 1_293_740_616,
        coefficient_b_q30: 1_073_131_533,
        gain_q15: 9_000,
        initial_sine_q15: 26_150,
    },
    // 7,629 Hz / T20 150 ms
    ModeProfile {
        coefficient_a_q30: 1_162_386_337,
        coefficient_b_q30: 1_073_055_271,
        gain_q15: 7_000,
        initial_sine_q15: 27_549,
    },
    // 8,875 Hz / T20 112.5 ms
    ModeProfile {
        coefficient_a_q30: 853_794_206,
        coefficient_b_q30: 1_072_826_517,
        gain_q15: 5_000,
        initial_sine_q15: 30_064,
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
            noise_gain_q15: 8_192,
            noise_frames: 480,
            duration_frames: 16_800,
        },
        PhysicalSoundMaterial::Glass => MaterialProfile {
            modes: &GLASS_MODES,
            noise_gain_q15: 32_767,
            noise_frames: 480,
            duration_frames: 33_600,
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
    fn screened_wood_and_glass_profiles_retain_exact_pcm_and_position_response() {
        let assert_profile = |material, seed, expected_len, expected_hash: &str| {
            let render = |impact_point| {
                render_physical_sound_impact(PhysicalSoundExcitation::new(
                    material,
                    impact_point,
                    49_152,
                    0,
                    seed,
                ))
            };
            let center = render(PhysicalSoundImpactPoint::Center);
            let edge = render(PhysicalSoundImpactPoint::Edge);
            let corner = render(PhysicalSoundImpactPoint::Corner);

            assert_eq!(center.len(), expected_len);
            assert_eq!(
                ContentHash::from_bytes(sha256(&samples_as_bytes(&center))).to_hex(),
                expected_hash
            );
            assert_ne!(center, edge);
            assert_ne!(center, corner);
            assert_ne!(edge, corner);
        };

        assert_profile(
            PhysicalSoundMaterial::Wood,
            0x700d_0101,
            35_200,
            "5c536dfda9e4529c0dda14e854b5718e50d1860dd6a2ca47fc58809762e7f186",
        );
        assert_profile(
            PhysicalSoundMaterial::Glass,
            0x61a5_0101,
            67_200,
            "5d0e59d0155cde71d70cdd51fd9df59e5c781ea188aef568246e6e6102a7b634",
        );
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
