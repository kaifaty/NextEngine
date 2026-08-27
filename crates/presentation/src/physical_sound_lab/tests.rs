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
fn steel_search_control_matches_current_fixed_point_profile_exactly() {
    let expected = render_physical_sound_impact(PhysicalSoundExcitation::new(
        PhysicalSoundMaterial::Steel,
        PhysicalSoundImpactPoint::Center,
        49_152,
        0,
        0x51ee_0101,
    ));
    let actual = render_experimental_steel_search_impact(
        ExperimentalSteelSearchProfile::current_control(),
        PhysicalSoundImpactPoint::Center,
        49_152,
        0x51ee_0101,
    )
    .expect("bounded steel search control");

    assert_eq!(actual, expected);
    assert_eq!(
        render_experimental_steel_search_impact(
            ExperimentalSteelSearchProfile::current_control(),
            PhysicalSoundImpactPoint::Center,
            49_152,
            0x51ee_0101,
        )
        .expect("repeated steel search control"),
        actual
    );
}

#[test]
fn steel_search_rejects_profiles_outside_the_frozen_bounds() {
    let invalid = ExperimentalSteelSearchProfile {
        frequency_scale_permille: 649,
        ..ExperimentalSteelSearchProfile::current_control()
    };
    assert_eq!(
        render_experimental_steel_search_impact(
            invalid,
            PhysicalSoundImpactPoint::Center,
            49_152,
            1,
        ),
        Err(ExperimentalSteelSearchError::ProfileOutsideBounds)
    );
}

#[test]
fn accepted_wood_and_hybrid_glass_profiles_retain_exact_pcm_and_position_response() {
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
        16_000,
        "4cd5e61da43d34e71715bef6d11a51522cfd5ab4ac9b0ec3d25920b8d79bb34c",
    );
}

#[test]
fn glass_clink_keeps_microbursts_inside_one_fused_onset() {
    let profile = material_profile(PhysicalSoundMaterial::Glass);
    assert_eq!(profile.modes.len(), 4);
    let StrikeTransientProfile::FusedGlassClink { pulses } = profile.strike_transient else {
        panic!("glass must use the bounded fused-clink transient");
    };
    assert!(pulses.iter().all(|pulse| pulse.duration_frames > 0));
    assert!(
        pulses.windows(2).all(|pair| {
            pair[0].offset_frames + pair[0].duration_frames <= pair[1].offset_frames
        })
    );
    let final_frame = pulses
        .last()
        .map(|pulse| pulse.offset_frames + pulse.duration_frames)
        .expect("glass clink pulse");
    assert!(final_frame <= 72, "onset must end within 1.5 ms at 48 kHz");
}

#[test]
fn glass_body_variants_repeat_exactly_and_stay_materially_separated() {
    let render = |variant| render_glass_body_variant_impact(variant, 49_152, 0x61a5_b0d1);
    let thin_goblet = render(GlassBodyVariant::ThinGoblet);
    let bottle = render(GlassBodyVariant::Bottle);
    let thick_jar = render(GlassBodyVariant::ThickJar);

    assert_eq!(thin_goblet, render(GlassBodyVariant::ThinGoblet));
    assert_eq!(bottle, render(GlassBodyVariant::Bottle));
    assert_eq!(thick_jar, render(GlassBodyVariant::ThickJar));
    assert_eq!(thin_goblet.len(), 48_000);
    assert_eq!(bottle.len(), 22_400);
    assert_eq!(thick_jar.len(), 16_000);
    assert_eq!(
        ContentHash::from_bytes(sha256(&samples_as_bytes(&thin_goblet))).to_hex(),
        "47fc42095a7bffcd6a1562d9e2095265a76ff3e1e3ac10fc2ac79f444f145bb5"
    );
    assert_eq!(
        ContentHash::from_bytes(sha256(&samples_as_bytes(&bottle))).to_hex(),
        "07c0de35ad35ee9f8760165f94a9245cef3ed84cab145e57a379936a22564515"
    );
    assert_eq!(
        ContentHash::from_bytes(sha256(&samples_as_bytes(&thick_jar))).to_hex(),
        "acb94f263d9de25e50ab2e5dcead69c129dc457c05852cc19cc143b29a1e10c7"
    );
    assert_ne!(thin_goblet, bottle);
    assert_ne!(thin_goblet, thick_jar);
    assert_ne!(bottle, thick_jar);
    assert!(thin_goblet.iter().any(|sample| *sample != 0));
    assert!(bottle.iter().any(|sample| *sample != 0));
    assert!(thick_jar.iter().any(|sample| *sample != 0));
}

#[test]
fn selected_glass_q30_is_explicit_exact_and_bounded() {
    let first = render_selected_glass_q30_impact(65_536);
    let second = render_selected_glass_q30_impact(65_536);

    assert_eq!(
        ExperimentalPhysicalSoundMixer::default().glass_profile(),
        ExperimentalGlassProfile::GlassH
    );
    assert_eq!(first, second);
    assert_eq!(first.len(), 48_000);
    assert_eq!(
        first.iter().map(|sample| sample.unsigned_abs()).max(),
        Some(29_490)
    );
    assert_eq!(
        ContentHash::from_bytes(sha256(&samples_as_bytes(&first))).to_hex(),
        "a85dee33d2379484e6a074b1588d5b1e858980684af45ed36eb886385f4abde0"
    );
    assert_ne!(
        first,
        render_physical_sound_impact(PhysicalSoundExcitation::new(
            PhysicalSoundMaterial::Glass,
            PhysicalSoundImpactPoint::Center,
            65_536,
            0,
            0x61a5_0101,
        ))
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
