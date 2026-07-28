#![forbid(unsafe_code)]

use std::error::Error;
use std::fmt::{Display, Formatter};

use next_contracts::{
    AssetId, ContentHash, PersistentId, PhysicsBodyIdV1, PhysicsCanonicalSnapshotV2,
    PresentationContractError, PresentationObjectKeyV1, PresentationPrimitiveV1,
    PresentationRoleV1, PresentationSnapshotV2, QuantizedPresentationTransformV1,
    ScenePresentationRecordV2, domain_hash,
};

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct PresentationBindingV1 {
    pub persistent_id: PersistentId,
    pub presentation_role: PresentationRoleV1,
    pub incarnation: u32,
    pub presentation_layer: u16,
    pub asset_id: AssetId,
    pub instance_ordinal: u32,
    pub primitive: PresentationPrimitiveV1,
    pub physics_body_id: Option<PhysicsBodyIdV1>,
    pub fallback_transform: QuantizedPresentationTransformV1,
    pub visible: bool,
}

#[derive(Clone, Debug)]
pub struct PresentationExtractorV1 {
    snapshot_epoch: ContentHash,
    next_snapshot_sequence: u64,
    presentation_profile_hash: ContentHash,
    max_scene_records_per_batch: usize,
    accepted_snapshot: Option<PresentationSnapshotV2>,
}

impl PresentationExtractorV1 {
    pub fn new(
        project_composition_lock_hash: ContentHash,
        presentation_profile_hash: ContentHash,
        max_scene_records_per_batch: usize,
    ) -> Result<Self, PresentationExtractionError> {
        if max_scene_records_per_batch == 0 {
            return Err(PresentationExtractionError::InvalidProfile);
        }
        Ok(Self {
            snapshot_epoch: domain_hash(
                "nextengine.presentation-snapshot-epoch.v1",
                project_composition_lock_hash.as_bytes(),
            ),
            next_snapshot_sequence: 0,
            presentation_profile_hash,
            max_scene_records_per_batch,
            accepted_snapshot: None,
        })
    }

    pub fn extract(
        &mut self,
        simulation_tick: u64,
        project_composition_lock_hash: ContentHash,
        content_manifest_hash: ContentHash,
        physics: &PhysicsCanonicalSnapshotV2,
        bindings: &[PresentationBindingV1],
    ) -> Result<&PresentationSnapshotV2, PresentationExtractionError> {
        let mut canonical_bindings = bindings.to_vec();
        canonical_bindings.sort();
        if canonical_bindings.windows(2).any(|pair| pair[0] == pair[1]) {
            return Err(PresentationExtractionError::DuplicateBinding);
        }
        let records = canonical_bindings
            .iter()
            .map(|binding| {
                let current = binding
                    .physics_body_id
                    .and_then(|body_id| physics.sorted_body_states.get(&body_id))
                    .map_or(binding.fallback_transform, |body| {
                        QuantizedPresentationTransformV1 {
                            translation_micrometres: body.pose.translation_micrometres,
                            orientation_q30: body.pose.rotation_q1_30,
                        }
                    });
                let previous = self
                    .accepted_snapshot
                    .as_ref()
                    .and_then(|snapshot| {
                        snapshot.scene_records().find(|record| {
                            record.object_key.persistent_id == binding.persistent_id
                                && record.object_key.presentation_role == binding.presentation_role
                                && record.object_key.incarnation == binding.incarnation
                        })
                    })
                    .map_or(current, |record| record.current_transform);
                ScenePresentationRecordV2::new(
                    binding.presentation_layer,
                    PresentationObjectKeyV1 {
                        snapshot_epoch: self.snapshot_epoch,
                        persistent_id: binding.persistent_id,
                        presentation_role: binding.presentation_role,
                        incarnation: binding.incarnation,
                    },
                    binding.asset_id,
                    binding.instance_ordinal,
                    binding.primitive,
                    previous,
                    current,
                    binding.visible,
                )
            })
            .collect();
        let candidate = PresentationSnapshotV2::new(
            self.snapshot_epoch,
            self.next_snapshot_sequence,
            simulation_tick,
            project_composition_lock_hash,
            content_manifest_hash,
            self.presentation_profile_hash,
            records,
            self.max_scene_records_per_batch,
            domain_hash("nextengine.presentation.environment.empty.v1", &[]),
        )?;
        candidate.validate()?;
        self.next_snapshot_sequence = self
            .next_snapshot_sequence
            .checked_add(1)
            .ok_or(PresentationExtractionError::SequenceOverflow)?;
        self.accepted_snapshot = Some(candidate);
        self.accepted_snapshot
            .as_ref()
            .ok_or(PresentationExtractionError::PublicationFailed)
    }

    #[must_use]
    pub fn accepted_snapshot(&self) -> Option<&PresentationSnapshotV2> {
        self.accepted_snapshot.as_ref()
    }
}

#[derive(Debug)]
#[non_exhaustive]
pub enum PresentationExtractionError {
    Contract(PresentationContractError),
    InvalidProfile,
    DuplicateBinding,
    SequenceOverflow,
    PublicationFailed,
}

impl Display for PresentationExtractionError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Contract(error) => write!(formatter, "{error}"),
            Self::InvalidProfile => formatter.write_str("presentation extraction profile invalid"),
            Self::DuplicateBinding => {
                formatter.write_str("presentation extraction binding duplicated")
            }
            Self::SequenceOverflow => {
                formatter.write_str("presentation snapshot sequence overflow")
            }
            Self::PublicationFailed => {
                formatter.write_str("presentation snapshot publication failed")
            }
        }
    }
}

impl Error for PresentationExtractionError {}

impl From<PresentationContractError> for PresentationExtractionError {
    fn from(error: PresentationContractError) -> Self {
        Self::Contract(error)
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use super::*;
    use next_contracts::{
        AuthoritativeNumericProfileV1, PhysicsCoordinateProfileV1, PhysicsLimitsProfileV1,
        PhysicsQuantizationProfileV1, PhysicsSolverSemanticsProfileV1,
        PhysicsWorldCatalogProfilesV1, PhysicsWorldCatalogV1, PhysicsWorldId, TickRateProfileV1,
    };

    #[test]
    fn binding_order_does_not_change_snapshot_and_failed_candidate_is_not_published() {
        let lock = domain_hash("test.lock", b"lock");
        let content = domain_hash("test.content", b"content");
        let mut first =
            PresentationExtractorV1::new(lock, domain_hash("test.profile", b"profile"), 2)
                .expect("extractor");
        let mut second = first.clone();
        let physics = empty_physics();
        let mut bindings = vec![binding(2), binding(1)];
        let forward = first
            .extract(0, lock, content, &physics, &bindings)
            .expect("snapshot")
            .clone();
        bindings.reverse();
        let reverse = second
            .extract(0, lock, content, &physics, &bindings)
            .expect("snapshot")
            .clone();
        assert_eq!(forward, reverse);

        let prior = first.accepted_snapshot().expect("published").clone();
        assert!(
            first
                .extract(1, lock, content, &physics, &[binding(1), binding(1)],)
                .is_err()
        );
        assert_eq!(first.accepted_snapshot(), Some(&prior));
    }

    fn binding(id: u8) -> PresentationBindingV1 {
        PresentationBindingV1 {
            persistent_id: PersistentId::from_bytes([id; 16]),
            presentation_role: PresentationRoleV1::Item,
            incarnation: 0,
            presentation_layer: 3,
            asset_id: AssetId::from_bytes([id; 16]),
            instance_ordinal: 0,
            primitive: PresentationPrimitiveV1::Item,
            physics_body_id: None,
            fallback_transform: QuantizedPresentationTransformV1::default(),
            visible: true,
        }
    }

    fn empty_physics() -> PhysicsCanonicalSnapshotV2 {
        let tick_rate = TickRateProfileV1::at_30_hz();
        let quantization =
            PhysicsQuantizationProfileV1::capsule_reference_v1().expect("quantization");
        let numeric =
            AuthoritativeNumericProfileV1::capsule_reference_v1(&quantization).expect("numeric");
        let catalog = PhysicsWorldCatalogV1::new(
            PhysicsWorldId::from_bytes([9; 16]),
            PhysicsWorldCatalogProfilesV1 {
                coordinate: PhysicsCoordinateProfileV1::reference_v1().expect("coordinate"),
                limits: PhysicsLimitsProfileV1::reference_v1().expect("limits"),
                solver: PhysicsSolverSemanticsProfileV1::grounded_capsule_v1().expect("solver"),
                tick_rate_hash: tick_rate.profile_hash().expect("tick-rate hash"),
                authoritative_numeric_hash: numeric.profile_hash().expect("numeric hash"),
                quantization_hash: quantization.profile_hash().expect("quantization hash"),
            },
            BTreeMap::new(),
            BTreeMap::new(),
            BTreeMap::new(),
        )
        .expect("catalog");
        PhysicsCanonicalSnapshotV2::genesis(&catalog, &tick_rate, &numeric, &quantization)
            .expect("snapshot")
    }
}
