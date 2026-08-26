//! Feature-gated adapter from committed reference contacts to the SPEC-45
//! P0.5 presentation laboratory. The estimator is intentionally provisional:
//! current ContactEventV1 has no impulse/effective-mass payload yet.

use next_contracts::physics::{
    ClosedPhysicsContactBatchV1, ContactEventV1, ContactPhaseV1, PhysicsBodyIdV1,
    PhysicsCanonicalSnapshotV2,
};
use next_contracts::presentation::audio_scene::AudioSceneSnapshotV1;
use next_presentation::physical_sound_lab::{
    ExperimentalGlassProfile, ExperimentalPhysicalSoundMixer, PhysicalSoundExcitation,
    PhysicalSoundImpactPoint, PhysicalSoundMaterial,
};

const MAX_IMPACTS_PER_TICK: usize = 4;
const ENERGY_SPEED_FULL_SCALE_MICROMETRES_PER_SECOND: u64 = 3_000_000;
const MINIMUM_BEGIN_ENERGY_Q16: u64 = 8_192;
const PAN_RANGE_MICROMETRES: i64 = 5_000_000;

pub(super) fn demo_mixer() -> ExperimentalPhysicalSoundMixer {
    let profile = if cfg!(feature = "physical-sound-selected-glass") {
        ExperimentalGlassProfile::SelectedThinContainerQ30
    } else {
        ExperimentalGlassProfile::GlassH
    };
    ExperimentalPhysicalSoundMixer::with_glass_profile(profile)
}

pub(super) fn excitations_from_committed_contacts(
    previous: &PhysicsCanonicalSnapshotV2,
    current: &PhysicsCanonicalSnapshotV2,
    contacts: &ClosedPhysicsContactBatchV1,
    audio_scene: &AudioSceneSnapshotV1,
) -> Vec<PhysicalSoundExcitation> {
    contacts
        .events
        .iter()
        .filter(|contact| contact.phase == ContactPhaseV1::Begin)
        .take(MAX_IMPACTS_PER_TICK)
        .map(|contact| {
            let speed = relative_normal_speed(previous, contact)
                .max(relative_normal_speed(current, contact));
            let energy_q16 = speed
                .saturating_mul(65_536)
                .checked_div(ENERGY_SPEED_FULL_SCALE_MICROMETRES_PER_SECOND)
                .unwrap_or(65_536)
                .clamp(MINIMUM_BEGIN_ENERGY_Q16, 65_536) as u32;
            let listener_x = audio_scene.listener.transform.translation_micrometres[0];
            let relative_x = contact.point_micrometres[0].saturating_sub(listener_x);
            let pan_q16 = (i128::from(relative_x) * 65_536 / i128::from(PAN_RANGE_MICROMETRES))
                .clamp(-65_536, 65_536) as i32;
            PhysicalSoundExcitation::new(
                material_proxy(contact),
                impact_point_proxy(contact),
                energy_q16,
                pan_q16,
                strike_seed(contact),
            )
        })
        .collect()
}

fn relative_normal_speed(snapshot: &PhysicsCanonicalSnapshotV2, contact: &ContactEventV1) -> u64 {
    let low = body_velocity(snapshot, contact.participant_low.body_id);
    let high = body_velocity(snapshot, contact.participant_high.body_id);
    let dot_q30 = (0..3).fold(0_i128, |sum, index| {
        let relative = i128::from(high[index]) - i128::from(low[index]);
        sum + relative * i128::from(contact.normal_low_to_high_q1_30[index])
    });
    u64::try_from(dot_q30.unsigned_abs() >> 30).unwrap_or(u64::MAX)
}

fn body_velocity(snapshot: &PhysicsCanonicalSnapshotV2, body_id: PhysicsBodyIdV1) -> [i64; 3] {
    snapshot
        .sorted_body_states
        .get(&body_id)
        .map_or([0; 3], |body| body.linear_velocity_micrometres_per_second)
}

/// Temporary material proxy for the reference scene, whose canonical physics
/// catalog currently uses one zero-material descriptor for every shape.
fn material_proxy(contact: &ContactEventV1) -> PhysicalSoundMaterial {
    // The second explicit laboratory feature is an audition route: every
    // committed Begin contact demonstrates the selected glass voice. The
    // ordinary physical-sound-lab feature retains the three-way proxy below.
    if cfg!(feature = "physical-sound-selected-glass") {
        return PhysicalSoundMaterial::Glass;
    }
    let low = contact.participant_low.body_id.subject_id.as_bytes()[0];
    let high = contact.participant_high.body_id.subject_id.as_bytes()[0];
    let selector = low
        .wrapping_add(high)
        .wrapping_add(contact.participant_low.shape_slot as u8)
        .wrapping_add(contact.participant_high.shape_slot as u8)
        % 3;
    match selector {
        0 => PhysicalSoundMaterial::Steel,
        1 => PhysicalSoundMaterial::Wood,
        _ => PhysicalSoundMaterial::Glass,
    }
}

/// Temporary location classifier until cooked acoustic descriptors provide
/// normalized geometry coordinates/mode participation weights.
fn impact_point_proxy(contact: &ContactEventV1) -> PhysicalSoundImpactPoint {
    let spatial_bucket = contact.point_micrometres.iter().fold(0_u64, |sum, value| {
        sum.wrapping_add(value.unsigned_abs() / 250_000)
    });
    match (spatial_bucket + u64::from(contact.feature_low) + u64::from(contact.feature_high)) % 3 {
        0 => PhysicalSoundImpactPoint::Center,
        1 => PhysicalSoundImpactPoint::Edge,
        _ => PhysicalSoundImpactPoint::Corner,
    }
}

fn strike_seed(contact: &ContactEventV1) -> u32 {
    let bytes = contact.contact_id.as_bytes();
    u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]])
}

#[cfg(test)]
mod tests {
    use std::sync::atomic::{AtomicU64, Ordering};

    use next_assets::ContentStore;
    use next_presentation::physical_sound_lab::ExperimentalGlassProfile;

    use crate::live::ReferenceGameDriverV2;

    static TEST_COUNTER: AtomicU64 = AtomicU64::new(0);

    #[test]
    fn demo_glass_profile_matches_the_explicit_feature() {
        let expected = if cfg!(feature = "physical-sound-selected-glass") {
            ExperimentalGlassProfile::SelectedThinContainerQ30
        } else {
            ExperimentalGlassProfile::GlassH
        };
        assert_eq!(super::demo_mixer().glass_profile(), expected);
    }

    #[test]
    fn demo_contact_changes_only_presentation_pcm_and_repeats_exactly() {
        let root = std::env::temp_dir().join(format!(
            "nextengine-physical-sound-lab-{}-{}",
            std::process::id(),
            TEST_COUNTER.fetch_add(1, Ordering::Relaxed)
        ));
        let store = ContentStore::new(&root);
        let cooked =
            next_project::cook_project_v7(crate::project_source_v7().expect("reference source"))
                .expect("cook");
        store
            .publish(&cooked.publication().expect("publication"))
            .expect("publish");
        let package = next_project::activate_project_package(&store).expect("activate");
        let mut enabled = ReferenceGameDriverV2::new(package.clone(), true).expect("enabled");
        let mut repeated = ReferenceGameDriverV2::new(package.clone(), true).expect("repeated");
        let mut disabled = ReferenceGameDriverV2::new(package, true).expect("disabled");
        disabled.physical_sound_lab_enabled = false;

        enabled.advance(&[]).expect("enabled advance");
        repeated.advance(&[]).expect("repeated advance");
        disabled.advance(&[]).expect("disabled advance");

        if cfg!(feature = "physical-sound-selected-glass") {
            for _ in 1..120 {
                if enabled.physical_sound_lab.selected_glass_impacts() > 0 {
                    break;
                }
                enabled.advance(&[]).expect("enabled advance");
                repeated.advance(&[]).expect("repeated advance");
                disabled.advance(&[]).expect("disabled advance");
            }
        }

        assert!(enabled.physical_sound_lab.admitted_impacts() > 0);
        if cfg!(feature = "physical-sound-selected-glass") {
            assert!(enabled.physical_sound_lab.selected_glass_impacts() > 0);
        }
        assert_eq!(enabled.audio_mixed_pcm(), repeated.audio_mixed_pcm());
        assert_ne!(enabled.audio_mixed_pcm(), disabled.audio_mixed_pcm());
        let enabled_state = enabled.state().expect("enabled state");
        let disabled_state = disabled.state().expect("disabled state");
        assert_eq!(
            enabled_state.checkpoint.state_root,
            disabled_state.checkpoint.state_root
        );
        assert_eq!(
            enabled_state.checkpoint.runtime_snapshot,
            disabled_state.checkpoint.runtime_snapshot
        );
        assert_eq!(
            enabled_state.checkpoint.rpg_snapshot,
            disabled_state.checkpoint.rpg_snapshot
        );
        assert_eq!(
            enabled_state.checkpoint.physics_checkpoint,
            disabled_state.checkpoint.physics_checkpoint
        );
        std::fs::remove_dir_all(root).expect("cleanup");
    }
}
