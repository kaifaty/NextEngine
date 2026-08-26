//! Camera validation and initial/recovery presentation publication for the
//! live reference driver. Split from `live.rs` to keep the driver boundary
//! readable and within the repository source-file limit.

use super::*;

impl ReferenceGameDriverV2 {
    pub(super) fn validate_recovered_camera(
        &self,
        persisted_snapshot: &PresentationSnapshotV3,
    ) -> Result<(), ReferenceGameError> {
        let persisted = {
            let mut cameras = persisted_snapshot.camera_records();
            let camera = cameras.next();
            camera
                .and_then(|camera| cameras.next().is_none().then_some(camera))
                .ok_or(ReferenceGameError::RecoveryInvalid)?
                .clone()
        };
        let rebuilt = self.camera_binding()?;
        if persisted.camera_id != PersistentId::from_bytes(CAMERA_ID_BYTES)
            || persisted.camera_role != CameraRoleV1::PrimaryThirdPerson
            || persisted.camera_id != rebuilt.camera_id
            || persisted.viewport != rebuilt.viewport
            || persisted.projection_profile != rebuilt.projection_profile
            || persisted.intent_sample != rebuilt.intent_sample
            || persisted.current_result_sample != rebuilt.current_result_sample
            || persisted.exposure_profile_revision != rebuilt.exposure_profile_revision
        {
            return Err(ReferenceGameError::RecoveryInvalid);
        }
        Ok(())
    }

    pub(super) fn publish_presentation(
        &mut self,
    ) -> Result<&PresentationSnapshotV3, ReferenceGameError> {
        let camera = self.camera_binding()?;
        let ui_records = crate::ui::live_semantic_ui_records(
            self.presentation_extractor.snapshot_epoch(),
            &self.fixture,
            &self.runtime.rpg_snapshot(),
            self.ui_screen,
            self.dialogue,
            None,
            self.current_audio_subtitle(self.runtime.next_tick()),
        )?;
        let skinning_records = fixture_character_skinning_records(
            &self.fixture,
            &self.physical_animation,
            self.runtime.physics_snapshot(),
            self.presentation_extractor.snapshot_epoch(),
        )?;
        self.presentation_extractor
            .extract_with_character_skinning(
                self.runtime.next_tick(),
                self.fixture
                    .activated_project
                    .project_lock
                    .project_lock_sha256,
                self.fixture
                    .activated_project
                    .content_manifest
                    .content_manifest_sha256,
                self.runtime.physics_snapshot(),
                &self.presentation_bindings,
                &[camera],
                ui_records,
                skinning_records,
            )?;
        self.publish_audio(&[], self.runtime.next_tick())?;
        self.camera_cut = false;
        self.presentation_extractor
            .accepted_snapshot()
            .ok_or(ReferenceGameError::PresentationSnapshotMissing)
    }

    pub(super) fn camera_binding(&self) -> Result<CameraPresentationBindingV1, ReferenceGameError> {
        self.camera_binding_for(
            self.runtime.physics_snapshot(),
            self.camera_yaw_millidegrees,
            self.camera_pitch_millidegrees,
            self.camera_cut,
        )
    }

    pub(super) fn camera_binding_for(
        &self,
        physics_snapshot: &PhysicsCanonicalSnapshotV2,
        camera_yaw_millidegrees: i32,
        camera_pitch_millidegrees: i32,
        camera_cut: bool,
    ) -> Result<CameraPresentationBindingV1, ReferenceGameError> {
        let player = self.fixture.physics_body_id;
        let player = physics_snapshot
            .sorted_body_states
            .get(&player)
            .ok_or(ReferenceGameError::BodyMissing)?;
        let mut focus = player.pose.translation_micrometres;
        focus[1] = focus[1]
            .checked_add(CAMERA_FOCUS_HEIGHT_MICROMETRES)
            .ok_or(ReferenceGameError::CountOverflow)?;
        let offset =
            camera_orbit_offset_micrometres(camera_yaw_millidegrees, camera_pitch_millidegrees)?;
        let translation = [
            focus[0]
                .checked_add(offset[0])
                .ok_or(ReferenceGameError::CountOverflow)?,
            focus[1]
                .checked_add(offset[1])
                .ok_or(ReferenceGameError::CountOverflow)?,
            focus[2]
                .checked_add(offset[2])
                .ok_or(ReferenceGameError::CountOverflow)?,
        ];
        Ok(CameraPresentationBindingV1::primary_third_person(
            PersistentId::from_bytes(CAMERA_ID_BYTES),
            0,
            CameraProjectionProfileV1::new(60_000, 100_000, 100_000_000)?,
            ThirdPersonCameraIntentSampleV1 {
                focus_subject_id: Some(self.fixture.body_id),
                focus_point_micrometres: focus,
                orbit_yaw_millidegrees: camera_yaw_millidegrees,
                orbit_pitch_millidegrees: camera_pitch_millidegrees,
                distance_micrometres: CAMERA_DISTANCE_MICROMETRES,
                shoulder_offset_micrometres: [CAMERA_SHOULDER_MICROMETRES, 0, 0],
            },
            CameraResultSampleV1 {
                pose: QuantizedPresentationTransformV1 {
                    translation_micrometres: translation,
                    ..QuantizedPresentationTransformV1::default()
                },
                focus_point_micrometres: focus,
            },
            self.fixture
                .activated_project
                .render_content_catalog
                .profile_revision(),
            camera_cut,
        ))
    }
}
