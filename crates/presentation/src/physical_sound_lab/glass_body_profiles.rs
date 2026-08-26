use super::{
    GLASS_CLINK_PULSES, GlassBodyVariant, MaterialProfile, ModeProfile, StrikeTransientProfile,
};

// A deliberately separated glass-object audition set. The thin-goblet modes
// follow the measured impulse peaks reported for a medium wineglass
// (630/1,563/2,927/4,640 Hz). The bottle modes retain the measured dominant
// 3,200/4,448/5,632 Hz beer-bottle clink and one short upper residual. The
// thick jar is an explicitly heuristic, more heavily damped counterfactual.
const THIN_GOBLET_MODES: [ModeProfile; 4] = [
    // 630 Hz / T20 420 ms
    ModeProfile {
        coefficient_a_q30: 2_139_941_081,
        coefficient_b_q30: 1_073_496_576,
        gain_q15: 16_000,
        initial_sine_q15: 2_699,
    },
    // 1,563 Hz / T20 260 ms
    ModeProfile {
        coefficient_a_q30: 2_102_305_882,
        coefficient_b_q30: 1_073_345_682,
        gain_q15: 22_000,
        initial_sine_q15: 6_658,
    },
    // 2,927 Hz / T20 150 ms
    ModeProfile {
        coefficient_a_q30: 1_991_141_530,
        coefficient_b_q30: 1_073_055_271,
        gain_q15: 15_000,
        initial_sine_q15: 12_250,
    },
    // 4,640 Hz / T20 90 ms
    ModeProfile {
        coefficient_a_q30: 1_762_464_845,
        coefficient_b_q30: 1_072_597_813,
        gain_q15: 7_000,
        initial_sine_q15: 18_701,
    },
];

const BOTTLE_MODES: [ModeProfile; 4] = [
    // 3,200 Hz / T20 120 ms
    ModeProfile {
        coefficient_a_q30: 1_961_039_841,
        coefficient_b_q30: 1_072_883_701,
        gain_q15: 22_000,
        initial_sine_q15: 13_328,
    },
    // 4,448 Hz / T20 85 ms
    ModeProfile {
        coefficient_a_q30: 1_792_635_018,
        coefficient_b_q30: 1_072_530_556,
        gain_q15: 18_000,
        initial_sine_q15: 18_019,
    },
    // 5,632 Hz / T20 65 ms
    ModeProfile {
        coefficient_a_q30: 1_588_685_413,
        coefficient_b_q30: 1_072_158_133,
        gain_q15: 14_000,
        initial_sine_q15: 22_028,
    },
    // 7,600 Hz / T20 35 ms
    ModeProfile {
        coefficient_a_q30: 1_168_001_477,
        coefficient_b_q30: 1_070_802_543,
        gain_q15: 5_000,
        initial_sine_q15: 27_482,
    },
];

const THICK_JAR_MODES: [ModeProfile; 5] = [
    // 1,050 Hz / T20 70 ms
    ModeProfile {
        coefficient_a_q30: 2_125_774_183,
        coefficient_b_q30: 1_072_271_176,
        gain_q15: 20_000,
        initial_sine_q15: 4_490,
    },
    // 2,100 Hz / T20 55 ms
    ModeProfile {
        coefficient_a_q30: 2_065_054_973,
        coefficient_b_q30: 1_071_870_440,
        gain_q15: 24_000,
        initial_sine_q15: 8_895,
    },
    // 3,600 Hz / T20 40 ms
    ModeProfile {
        coefficient_a_q30: 1_911_128_620,
        coefficient_b_q30: 1_071_169_512,
        gain_q15: 16_000,
        initial_sine_q15: 14_876,
    },
    // 5,400 Hz / T20 28 ms
    ModeProfile {
        coefficient_a_q30: 1_630_164_132,
        coefficient_b_q30: 1_070_068_980,
        gain_q15: 9_000,
        initial_sine_q15: 21_281,
    },
    // 7,600 Hz / T20 18 ms
    ModeProfile {
        coefficient_a_q30: 1_166_490_544,
        coefficient_b_q30: 1_068_033_943,
        gain_q15: 3_500,
        initial_sine_q15: 27_482,
    },
];

pub(super) const fn glass_body_profile(variant: GlassBodyVariant) -> MaterialProfile {
    let (modes, duration_frames): (&'static [ModeProfile], u32) = match variant {
        GlassBodyVariant::ThinGoblet => (&THIN_GOBLET_MODES, 24_000),
        GlassBodyVariant::Bottle => (&BOTTLE_MODES, 11_200),
        GlassBodyVariant::ThickJar => (&THICK_JAR_MODES, 8_000),
    };
    MaterialProfile {
        modes,
        strike_transient: StrikeTransientProfile::FusedGlassClink {
            pulses: &GLASS_CLINK_PULSES,
        },
        duration_frames,
    }
}
