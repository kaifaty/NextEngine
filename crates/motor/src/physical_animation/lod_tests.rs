use super::lod::{
    PhysicalAnimationLodDecisionV1, PhysicalAnimationLodLevelV1, PhysicalAnimationLodProfileV1,
    PhysicalAnimationLodPublicationModeV1, PhysicalAnimationLodRequestV1,
    PhysicalAnimationLodResourcesV1,
};
use super::tests::fixture;
use super::*;

fn request(
    requested_lod: PhysicalAnimationLodLevelV1,
    resources: PhysicalAnimationLodResourcesV1,
) -> PhysicalAnimationLodRequestV1 {
    PhysicalAnimationLodRequestV1 {
        requested_lod,
        resources,
    }
}

fn advance_without_motion(
    owner: &mut PhysicalAnimationOwnerV1,
    physics: &mut PhysicsCanonicalSnapshotV2,
) {
    let previous = physics.clone();
    physics.physics_tick += 1;
    owner
        .advance(
            &previous,
            physics,
            owner.snapshot().next_simulation_tick + 1,
        )
        .expect("advance animation owner");
}

#[test]
fn fixed_lod_profile_rejects_unbounded_cadence_or_hold() {
    assert_eq!(
        PhysicalAnimationLodProfileV1::new(0, 4),
        Err(PhysicalAnimationOwnerErrorV1::LodProfileInvalid)
    );
    assert_eq!(
        PhysicalAnimationLodProfileV1::new(4, 2),
        Err(PhysicalAnimationOwnerErrorV1::LodProfileInvalid)
    );
    assert_eq!(
        PhysicalAnimationLodProfileV1::new(17, 17),
        Err(PhysicalAnimationOwnerErrorV1::LodProfileInvalid)
    );
    assert_eq!(
        PhysicalAnimationLodProfileV1::new(2, 4).expect("bounded profile"),
        PhysicalAnimationLodProfileV1::REFERENCE_R5
    );
}

#[test]
fn lod_selection_covers_sample_hold_bind_and_no_pose_without_authority() {
    let fixture = fixture();
    let subject_id = fixture.bindings[0].subject_id;
    let mut physics = fixture.physics.clone();
    let mut owner = PhysicalAnimationOwnerV1::activate(
        fixture.profile,
        fixture.skeleton,
        fixture.idle,
        fixture.locomotion,
        fixture.bindings.clone(),
        &physics,
    )
    .expect("activate");
    let profile = PhysicalAnimationLodProfileV1::REFERENCE_R5;

    let owner_before = owner.snapshot().clone();
    let physics_before = physics.clone();
    let full = owner
        .project_pose_lod(
            subject_id,
            &physics,
            Some(0),
            profile,
            PhysicalAnimationLodRequestV1::full_pose(),
            None,
        )
        .expect("full projection");
    assert_eq!(full.decision(), PhysicalAnimationLodDecisionV1::Sampled);
    assert_eq!(
        full.publication_mode(),
        PhysicalAnimationLodPublicationModeV1::Sampled
    );
    assert_eq!(full.pose_source_animation_tick(), Some(0));
    assert_eq!(owner.snapshot(), &owner_before);
    assert_eq!(physics, physics_before);

    let previous_root = full.pose().expect("full pose").skeleton_root_pose;
    let previous_physics = physics.clone();
    physics.physics_tick += 1;
    physics
        .sorted_body_states
        .get_mut(&fixture.bindings[0].body_id)
        .expect("body")
        .pose
        .translation_micrometres[0] += 100;
    owner
        .advance(&previous_physics, &physics, 1)
        .expect("advance to off-cadence tick");
    let held_for_cadence = owner
        .project_pose_lod(
            subject_id,
            &physics,
            Some(0),
            profile,
            request(
                PhysicalAnimationLodLevelV1::ReducedPose,
                PhysicalAnimationLodResourcesV1::FULL,
            ),
            Some(&full),
        )
        .expect("cadence hold");
    assert_eq!(
        held_for_cadence.decision(),
        PhysicalAnimationLodDecisionV1::ReducedCadenceHeld
    );
    assert_eq!(
        held_for_cadence.publication_mode(),
        PhysicalAnimationLodPublicationModeV1::HeldPresentationPose
    );
    assert_eq!(held_for_cadence.pose_source_animation_tick(), Some(0));
    assert_eq!(
        held_for_cadence
            .pose()
            .expect("held pose")
            .skeleton_root_pose
            .translation_micrometres[0],
        previous_root.translation_micrometres[0] + 100
    );

    advance_without_motion(&mut owner, &mut physics);
    let reduced = owner
        .project_pose_lod(
            subject_id,
            &physics,
            Some(0),
            profile,
            request(
                PhysicalAnimationLodLevelV1::ReducedPose,
                PhysicalAnimationLodResourcesV1::FULL,
            ),
            Some(&held_for_cadence),
        )
        .expect("due reduced sample");
    assert_eq!(reduced.decision(), PhysicalAnimationLodDecisionV1::Sampled);
    assert_eq!(
        reduced.pose().expect("reduced pose").projection_mode,
        PhysicalAnimationProjectionModeV1::SampledWithoutFootIk
    );

    advance_without_motion(&mut owner, &mut physics);
    let requested_held = owner
        .project_pose_lod(
            subject_id,
            &physics,
            Some(0),
            profile,
            request(
                PhysicalAnimationLodLevelV1::HeldPresentationPose,
                PhysicalAnimationLodResourcesV1::FULL,
            ),
            Some(&reduced),
        )
        .expect("requested hold");
    assert_eq!(
        requested_held.decision(),
        PhysicalAnimationLodDecisionV1::RequestedHeld
    );

    let bind = owner
        .project_pose_lod(
            subject_id,
            &physics,
            Some(0),
            profile,
            request(
                PhysicalAnimationLodLevelV1::FullPose,
                PhysicalAnimationLodResourcesV1::CLIP_UNAVAILABLE,
            ),
            None,
        )
        .expect("bind fallback");
    assert_eq!(
        bind.decision(),
        PhysicalAnimationLodDecisionV1::ClipUnavailableBind
    );
    assert_eq!(
        bind.publication_mode(),
        PhysicalAnimationLodPublicationModeV1::BindPoseFallback
    );
    assert_eq!(
        bind.pose().expect("bind pose").projection_mode,
        PhysicalAnimationProjectionModeV1::BindPoseFallback
    );

    for level in [
        PhysicalAnimationLodLevelV1::IntentOnly,
        PhysicalAnimationLodLevelV1::CulledPresentation,
    ] {
        let no_pose = owner
            .project_pose_lod(
                subject_id,
                &physics,
                Some(0),
                profile,
                request(level, PhysicalAnimationLodResourcesV1::FULL),
                Some(&requested_held),
            )
            .expect("explicit no-pose level");
        assert_eq!(
            no_pose.publication_mode(),
            PhysicalAnimationLodPublicationModeV1::NoPose
        );
        assert!(no_pose.pose().is_none());
    }

    let resource_cull = owner
        .project_pose_lod(
            subject_id,
            &physics,
            Some(0),
            profile,
            request(
                PhysicalAnimationLodLevelV1::FullPose,
                PhysicalAnimationLodResourcesV1::NO_POSE,
            ),
            None,
        )
        .expect("resource cull");
    assert_eq!(
        resource_cull.decision(),
        PhysicalAnimationLodDecisionV1::ClipUnavailableCull
    );
    assert!(resource_cull.pose().is_none());
}

#[test]
fn stale_or_incompatible_held_input_uses_declared_fallback() {
    let fixture = fixture();
    let subject_id = fixture.bindings[0].subject_id;
    let mut physics = fixture.physics.clone();
    let mut owner = PhysicalAnimationOwnerV1::activate(
        fixture.profile,
        fixture.skeleton,
        fixture.idle,
        fixture.locomotion,
        fixture.bindings,
        &physics,
    )
    .expect("activate");
    let initial = owner
        .project_pose_lod(
            subject_id,
            &physics,
            Some(0),
            PhysicalAnimationLodProfileV1::REFERENCE_R5,
            PhysicalAnimationLodRequestV1::full_pose(),
            None,
        )
        .expect("initial pose");
    advance_without_motion(&mut owner, &mut physics);
    let mut foreign_physics = physics.clone();
    foreign_physics.world_id = next_contracts::ids::PhysicsWorldId::from_bytes([9; 16]);
    let incompatible = owner
        .project_pose_lod(
            subject_id,
            &foreign_physics,
            Some(0),
            PhysicalAnimationLodProfileV1::REFERENCE_R5,
            request(
                PhysicalAnimationLodLevelV1::FullPose,
                PhysicalAnimationLodResourcesV1::CLIP_UNAVAILABLE,
            ),
            Some(&initial),
        )
        .expect("foreign Physics projection falls back");
    assert_eq!(
        incompatible.decision(),
        PhysicalAnimationLodDecisionV1::ClipUnavailableBind
    );
    for _ in 1..5 {
        advance_without_motion(&mut owner, &mut physics);
    }
    let stale = owner
        .project_pose_lod(
            subject_id,
            &physics,
            Some(0),
            PhysicalAnimationLodProfileV1::REFERENCE_R5,
            request(
                PhysicalAnimationLodLevelV1::FullPose,
                PhysicalAnimationLodResourcesV1::CLIP_UNAVAILABLE,
            ),
            Some(&initial),
        )
        .expect("stale pose falls back");
    assert_eq!(
        stale.decision(),
        PhysicalAnimationLodDecisionV1::ClipUnavailableBind
    );
}
