//! Pre-cooked 48 kHz Q30 glass-container candidate for the demo laboratory.
//!
//! The constants are a clean-room transfer of the product-owner-selected
//! external calibration profile whose source-profile SHA-256 is
//! `7939326bb4b1e09f9b6b89c3a2f4099708edbe35c953d710db3c100c6bd007f7`.
//! No DiffSound source, dependency, generated PCM or model state is embedded.

use super::Q16_ONE;

const Q30_SHIFT: u32 = 30;
const MODE_COUNT: usize = 16;
const TRANSIENT_SAMPLE_COUNT: usize = 144;
pub(super) const DURATION_FRAMES: u32 = 24_000;

// Peak of the unnormalized 24,000-frame integer recurrence, including onset.
const RAW_PEAK_Q30: i128 = 516_069_029;
// The accepted external audition used 0.9 full-scale S16 per channel. The
// common mixer halves a centered mono voice, so the pre-pan target is doubled.
const CENTER_CHANNEL_PEAK: i128 = 29_490;

#[derive(Clone, Copy, Debug)]
struct SelectedGlassQ30Mode {
    coefficient_a_q30: i64,
    coefficient_b_q30: i64,
    initial_sample_q30: i64,
}

/// Immutable research snapshot of one cooked mode in the selected Q30
/// benchmark. This is deliberately a presentation-lab type, not content or a
/// public engine contract.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SelectedGlassQ30ModeSnapshot {
    pub coefficient_a_q30: i64,
    pub coefficient_b_q30: i64,
    pub initial_sample_q30: i64,
}

/// Immutable research snapshot used by external benchmark tooling. Runtime
/// playback continues to use the private constants below directly.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SelectedGlassQ30ProfileSnapshot {
    pub revision: &'static str,
    pub source_profile_sha256: &'static str,
    pub sample_rate_hz: u32,
    pub channel_count: u32,
    pub duration_frames: u32,
    pub raw_peak_q30: i128,
    pub center_channel_peak: i128,
    pub modes: [SelectedGlassQ30ModeSnapshot; MODE_COUNT],
    pub transient_q30: &'static [i64],
}

/// Returns the exact already-cooked selected-glass profile for external
/// baseline export. No floating-point inverse reconstruction is performed.
#[must_use]
pub const fn selected_glass_q30_profile_snapshot() -> SelectedGlassQ30ProfileSnapshot {
    let mut modes = [SelectedGlassQ30ModeSnapshot {
        coefficient_a_q30: 0,
        coefficient_b_q30: 0,
        initial_sample_q30: 0,
    }; MODE_COUNT];
    let mut index = 0;
    while index < MODE_COUNT {
        modes[index] = SelectedGlassQ30ModeSnapshot {
            coefficient_a_q30: SELECTED_GLASS_Q30_MODES[index].coefficient_a_q30,
            coefficient_b_q30: SELECTED_GLASS_Q30_MODES[index].coefficient_b_q30,
            initial_sample_q30: SELECTED_GLASS_Q30_MODES[index].initial_sample_q30,
        };
        index += 1;
    }
    SelectedGlassQ30ProfileSnapshot {
        revision: "selected-thin-container-q30-v1",
        source_profile_sha256: "7939326bb4b1e09f9b6b89c3a2f4099708edbe35c953d710db3c100c6bd007f7",
        sample_rate_hz: 48_000,
        channel_count: 2,
        duration_frames: DURATION_FRAMES,
        raw_peak_q30: RAW_PEAK_Q30,
        center_channel_peak: CENTER_CHANNEL_PEAK,
        modes,
        transient_q30: &SELECTED_GLASS_TRANSIENT_Q30,
    }
}

#[derive(Clone, Copy, Debug, Default)]
struct SelectedGlassQ30State {
    previous_q30: i64,
    current_q30: i64,
}

#[derive(Clone, Debug)]
pub(super) struct SelectedGlassQ30Voice {
    states: [SelectedGlassQ30State; MODE_COUNT],
    energy_q16: u32,
    pan_q16: i32,
    frames_elapsed: u32,
}

impl SelectedGlassQ30Voice {
    pub(super) fn new(energy_q16: u32, pan_q16: i32) -> Self {
        let mut states = [SelectedGlassQ30State::default(); MODE_COUNT];
        for (state, mode) in states.iter_mut().zip(SELECTED_GLASS_Q30_MODES) {
            state.current_q30 = mode.initial_sample_q30;
        }
        Self {
            states,
            energy_q16,
            pan_q16,
            frames_elapsed: 0,
        }
    }

    pub(super) const fn pan_q16(&self) -> i32 {
        self.pan_q16
    }

    pub(super) const fn frames_remaining(&self) -> u32 {
        DURATION_FRAMES.saturating_sub(self.frames_elapsed)
    }

    pub(super) fn next_mono_sample(&mut self) -> i64 {
        if self.frames_elapsed >= DURATION_FRAMES {
            return 0;
        }

        let mut sample_q30 = 0_i128;
        // The calibration recurrence defines frame zero as onset-only. Its
        // initialized modal state is first emitted at frame one.
        if self.frames_elapsed > 0 {
            for (state, mode) in self.states.iter_mut().zip(SELECTED_GLASS_Q30_MODES) {
                sample_q30 += i128::from(state.current_q30);
                let next_q30 = ((i128::from(mode.coefficient_a_q30)
                    * i128::from(state.current_q30))
                    >> Q30_SHIFT)
                    - ((i128::from(mode.coefficient_b_q30) * i128::from(state.previous_q30))
                        >> Q30_SHIFT);
                state.previous_q30 = state.current_q30;
                state.current_q30 = i64::try_from(next_q30)
                    .expect("pre-cooked selected-glass recurrence stays inside i64");
            }
        }
        if let Some(transient_q30) = SELECTED_GLASS_TRANSIENT_Q30.get(self.frames_elapsed as usize)
        {
            sample_q30 += i128::from(*transient_q30);
        }
        self.frames_elapsed = self.frames_elapsed.saturating_add(1);

        let scaled = sample_q30 * (CENTER_CHANNEL_PEAK * 2) * i128::from(self.energy_q16)
            / RAW_PEAK_Q30
            / i128::from(Q16_ONE);
        i64::try_from(scaled).expect("pre-cooked selected-glass output stays inside i64")
    }
}

const SELECTED_GLASS_Q30_MODES: [SelectedGlassQ30Mode; MODE_COUNT] = [
    SelectedGlassQ30Mode {
        coefficient_a_q30: 2_101_732_581,
        coefficient_b_q30: 1_071_348_154,
        initial_sample_q30: -193_760_257,
    },
    SelectedGlassQ30Mode {
        coefficient_a_q30: 2_101_552_590,
        coefficient_b_q30: 1_071_348_574,
        initial_sample_q30: 214_990_580,
    },
    SelectedGlassQ30Mode {
        coefficient_a_q30: 1_882_255_937,
        coefficient_b_q30: 1_070_878_543,
        initial_sample_q30: 7_709_785,
    },
    SelectedGlassQ30Mode {
        coefficient_a_q30: 1_881_914_112,
        coefficient_b_q30: 1_070_878_395,
        initial_sample_q30: 1_257_294,
    },
    SelectedGlassQ30Mode {
        coefficient_a_q30: 1_360_539_237,
        coefficient_b_q30: 1_070_707_027,
        initial_sample_q30: 19_734_409,
    },
    SelectedGlassQ30Mode {
        coefficient_a_q30: 1_360_308_328,
        coefficient_b_q30: 1_070_706_964,
        initial_sample_q30: 13_273_713,
    },
    SelectedGlassQ30Mode {
        coefficient_a_q30: 483_304_696,
        coefficient_b_q30: 1_070_501_451,
        initial_sample_q30: 68_573_783,
    },
    SelectedGlassQ30Mode {
        coefficient_a_q30: 482_858_158,
        coefficient_b_q30: 1_070_501_355,
        initial_sample_q30: 115_912_991,
    },
    SelectedGlassQ30Mode {
        coefficient_a_q30: 209_914_636,
        coefficient_b_q30: 1_070_443_669,
        initial_sample_q30: 57_113_794,
    },
    SelectedGlassQ30Mode {
        coefficient_a_q30: -51_192_240,
        coefficient_b_q30: 1_070_389_179,
        initial_sample_q30: 114_232_115,
    },
    SelectedGlassQ30Mode {
        coefficient_a_q30: -77_549_139,
        coefficient_b_q30: 1_070_383_683,
        initial_sample_q30: 72_266_890,
    },
    SelectedGlassQ30Mode {
        coefficient_a_q30: -552_443_200,
        coefficient_b_q30: 1_070_283_398,
        initial_sample_q30: -44_756_283,
    },
    SelectedGlassQ30Mode {
        coefficient_a_q30: -646_930_154,
        coefficient_b_q30: 1_070_262_884,
        initial_sample_q30: -10_432_135,
    },
    SelectedGlassQ30Mode {
        coefficient_a_q30: -648_975_128,
        coefficient_b_q30: 1_070_262_437,
        initial_sample_q30: -14_486_883,
    },
    SelectedGlassQ30Mode {
        coefficient_a_q30: -743_823_100,
        coefficient_b_q30: 1_070_241_533,
        initial_sample_q30: 27_866_424,
    },
    SelectedGlassQ30Mode {
        coefficient_a_q30: -769_199_431,
        coefficient_b_q30: 1_070_235_880,
        initial_sample_q30: 52_455_991,
    },
];

const SELECTED_GLASS_TRANSIENT_Q30: [i64; TRANSIENT_SAMPLE_COUNT] = [
    6_215_864,
    14_116_818,
    44_042_984,
    95_994_360,
    30_129_704,
    580_027,
    7_345_329,
    11_586_009,
    -7_422_419,
    -49_679_956,
    -27_481_158,
    -5_342_391,
    16_736_345,
    -6_150_488,
    -10_985_162,
    2_232_322,
    -25_252_682,
    -616_976,
    76_139_440,
    40_062_547,
    27_537_863,
    38_565_388,
    77_853_700,
    49_628_587,
    -46_109_952,
    -23_406_189,
    -4_709_305,
    9_980_700,
    -42_454_567,
    -60_587_067,
    -44_416_800,
    24_409_240,
    43_592_648,
    13_133_423,
    35_934_536,
    27_002_311,
    -13_663_252,
    22_974_697,
    11_280_656,
    -48_745_376,
    -40_292_469,
    -11_925_999,
    36_354_036,
    -70_751_140,
    -84_888_337,
    -6_057_554,
    -12_590_387,
    -36_042_602,
    -76_414_200,
    -15_877_883,
    20_235_337,
    31_925_458,
    -25_741_146,
    -19_432_513,
    50_851_356,
    22_787_251,
    -14_481_743,
    -60_955_624,
    -30_688_312,
    -26_025_132,
    -46_966_084,
    -28_686_104,
    -17_158_890,
    -12_384_442,
    -122_219,
    1_691_012,
    -6_944_748,
    -9_620_922,
    1_081_411,
    25_162_252,
    17_340_883,
    -1_338_335,
    -30_875_404,
    11_231_185,
    13_099_608,
    -25_270_136,
    -24_490_697,
    -15_527_697,
    1_618_864,
    -17_555_632,
    -19_216_531,
    -3_363_833,
    12_992_498,
    24_999_010,
    32_655_702,
    19_675_021,
    14_534_249,
    17_233_388,
    11_704_351,
    5_468_667,
    -1_473_662,
    -8_776_150,
    -2_267_135,
    18_053_382,
    -5_492_797,
    -10_893_303,
    1_851_864,
    17_417_452,
    12_610_653,
    -12_568_532,
    3_842_182,
    11_926_592,
    11_684_697,
    2_587_975,
    925_690,
    6_697_842,
    5_323_084,
    -674_216,
    -11_294_058,
    -6_797_097,
    -4_127_343,
    -3_284_795,
    -2_160_800,
    -1_294_344,
    -685_428,
    1_391_973,
    2_946_327,
    3_977_633,
    435_217,
    -84_872,
    2_417_366,
    772_681,
    -1_383_713,
    -4_051_814,
    -1_071_288,
    -45_160,
    -973_432,
    -1_845_604,
    -1_655_715,
    -403_765,
    -705_679,
    -645_531,
    -223_324,
    -14_703,
    185_157,
    376_258,
    70_873,
    -79_435,
    -74_664,
    -27_992,
    -9_660,
    -19_668,
    -6_556,
    0,
];
