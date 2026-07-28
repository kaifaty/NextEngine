#![forbid(unsafe_code)]

use std::collections::BTreeSet;
use std::error::Error;
use std::fmt::{Display, Formatter};

use next_contracts::{ContentHash, PresentationPrimitiveV1, PresentationSnapshotV2, domain_hash};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RenderTargetV1 {
    pub extent: [u32; 2],
    pub target_revision: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RenderFrameReportV1 {
    pub snapshot_hash: ContentHash,
    pub rendered_object_count: u32,
    pub target_revision: u64,
    pub frame_plan_hash: ContentHash,
}

pub trait RenderDevice {
    fn render(
        &mut self,
        snapshot: &PresentationSnapshotV2,
        target: RenderTargetV1,
    ) -> Result<RenderFrameReportV1, RenderDeviceError>;

    fn invalidate_device(&mut self);

    fn recover_device(&mut self);
}

#[derive(Clone, Debug, Default)]
pub struct ReferenceB0Renderer {
    device_available: bool,
}

impl ReferenceB0Renderer {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            device_available: true,
        }
    }
}

impl RenderDevice for ReferenceB0Renderer {
    fn render(
        &mut self,
        snapshot: &PresentationSnapshotV2,
        target: RenderTargetV1,
    ) -> Result<RenderFrameReportV1, RenderDeviceError> {
        if !self.device_available {
            return Err(RenderDeviceError::DeviceLost);
        }
        if target.extent[0] == 0 || target.extent[1] == 0 {
            return Err(RenderDeviceError::InvalidTarget);
        }
        snapshot.validate()?;
        let visible: Vec<_> = snapshot
            .scene_records()
            .filter(|record| record.visible)
            .collect();
        let primitives: BTreeSet<_> = visible.iter().map(|record| record.primitive).collect();
        let required = [
            PresentationPrimitiveV1::Floor,
            PresentationPrimitiveV1::Capsule,
            PresentationPrimitiveV1::Switch,
            PresentationPrimitiveV1::Item,
            PresentationPrimitiveV1::Character,
        ];
        if required
            .iter()
            .any(|primitive| !primitives.contains(primitive))
        {
            return Err(RenderDeviceError::RequiredScenePrimitiveMissing);
        }
        let mut preimage = Vec::new();
        preimage.extend_from_slice(snapshot.canonical_hash.as_bytes());
        preimage.extend_from_slice(&target.extent[0].to_le_bytes());
        preimage.extend_from_slice(&target.extent[1].to_le_bytes());
        preimage.extend_from_slice(&target.target_revision.to_le_bytes());
        for record in visible {
            preimage.extend_from_slice(record.canonical_hash.as_bytes());
        }
        Ok(RenderFrameReportV1 {
            snapshot_hash: snapshot.canonical_hash,
            rendered_object_count: u32::try_from(
                snapshot
                    .scene_records()
                    .filter(|record| record.visible)
                    .count(),
            )
            .map_err(|_| RenderDeviceError::CountOverflow)?,
            target_revision: target.target_revision,
            frame_plan_hash: domain_hash("nextengine.render-frame-plan.v1", &preimage),
        })
    }

    fn invalidate_device(&mut self) {
        self.device_available = false;
    }

    fn recover_device(&mut self) {
        self.device_available = true;
    }
}

#[derive(Debug)]
#[non_exhaustive]
pub enum RenderDeviceError {
    Presentation(next_contracts::PresentationContractError),
    InvalidTarget,
    DeviceLost,
    RequiredScenePrimitiveMissing,
    CountOverflow,
}

impl Display for RenderDeviceError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Presentation(error) => write!(formatter, "{error}"),
            Self::InvalidTarget => formatter.write_str("render target invalid"),
            Self::DeviceLost => formatter.write_str("render device lost"),
            Self::RequiredScenePrimitiveMissing => {
                formatter.write_str("required B0 scene primitive missing")
            }
            Self::CountOverflow => formatter.write_str("render object count overflow"),
        }
    }
}

impl Error for RenderDeviceError {}

impl From<next_contracts::PresentationContractError> for RenderDeviceError {
    fn from(error: next_contracts::PresentationContractError) -> Self {
        Self::Presentation(error)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use next_contracts::{
        AssetId, PersistentId, PresentationObjectKeyV1, PresentationRoleV1,
        QuantizedPresentationTransformV1, ScenePresentationRecordV2,
    };

    #[test]
    fn device_loss_never_changes_snapshot_and_recovery_reuses_exact_input() {
        let snapshot = snapshot();
        let mut renderer = ReferenceB0Renderer::new();
        let target = RenderTargetV1 {
            extent: [960, 540],
            target_revision: 1,
        };
        let first = renderer.render(&snapshot, target).expect("frame");
        renderer.invalidate_device();
        assert!(matches!(
            renderer.render(&snapshot, target),
            Err(RenderDeviceError::DeviceLost)
        ));
        renderer.recover_device();
        assert_eq!(renderer.render(&snapshot, target).expect("frame"), first);
    }

    fn snapshot() -> PresentationSnapshotV2 {
        let epoch = domain_hash("test.epoch", b"epoch");
        let primitives = [
            (
                PresentationRoleV1::Environment,
                PresentationPrimitiveV1::Floor,
            ),
            (
                PresentationRoleV1::PlayerAvatar,
                PresentationPrimitiveV1::Capsule,
            ),
            (
                PresentationRoleV1::InteractiveObject,
                PresentationPrimitiveV1::Switch,
            ),
            (PresentationRoleV1::Item, PresentationPrimitiveV1::Item),
            (
                PresentationRoleV1::Character,
                PresentationPrimitiveV1::Character,
            ),
        ];
        let records = primitives
            .into_iter()
            .enumerate()
            .map(|(index, (role, primitive))| {
                let id = u8::try_from(index + 1).expect("small fixture");
                ScenePresentationRecordV2::new(
                    u16::try_from(index).expect("small fixture"),
                    PresentationObjectKeyV1 {
                        snapshot_epoch: epoch,
                        persistent_id: PersistentId::from_bytes([id; 16]),
                        presentation_role: role,
                        incarnation: 0,
                    },
                    AssetId::from_bytes([id; 16]),
                    0,
                    primitive,
                    QuantizedPresentationTransformV1::default(),
                    QuantizedPresentationTransformV1::default(),
                    true,
                )
            })
            .collect();
        PresentationSnapshotV2::new(
            epoch,
            0,
            0,
            domain_hash("test.lock", b"lock"),
            domain_hash("test.content", b"content"),
            domain_hash("test.profile", b"profile"),
            records,
            8,
            domain_hash("test.environment", b"environment"),
        )
        .expect("snapshot")
    }
}
