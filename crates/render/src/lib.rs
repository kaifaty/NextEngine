#![forbid(unsafe_code)]

use std::error::Error;
use std::fmt::{Display, Formatter};

use next_contracts::canonical::sha256;
use next_contracts::ids::{ContentHash, content_hash_from_bytes};
use next_contracts::presentation::{
    PresentationSnapshotV2, QuantizedPresentationTransformV1, ScenePresentationFlagsV1,
};
use next_contracts::project::AssetRevisionRefV1;
use next_contracts::render_content::{
    MaterialTextureSlotV1, MeshPrimitiveTopologyV1, RenderContentCatalogV1,
    RenderContentContractError, b0_shader_interface_manifest_sha256,
};

pub const B0_MAX_INDEXED_DRAWS_PER_FRAME: u32 = 65_536;

const B0_FRAME_PLAN_HASH_DOMAIN: &str = "nextengine.render-frame-plan.b0.v1";
const B0_FRAME_PLAN_HASH_HEADER_BYTES: usize = 88;
const B0_FRAME_PLAN_HASH_DRAW_BYTES: usize = 185;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RenderTargetV1 {
    pub extent: [u32; 2],
    pub target_revision: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct B0IndexedDrawV1 {
    pub scene_record_hash: ContentHash,
    pub mesh_revision: AssetRevisionRefV1,
    pub material_revision: AssetRevisionRefV1,
    pub texture_revision: AssetRevisionRefV1,
    pub first_index: u32,
    pub index_count: u32,
    pub transform: QuantizedPresentationTransformV1,
    pub base_color_rgba_unorm16: [u16; 4],
    pub fallback_material: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct B0FramePlanV1 {
    pub snapshot_hash: ContentHash,
    pub catalog_hash: ContentHash,
    pub target: RenderTargetV1,
    pub visible_object_count: u32,
    pub indexed_draw_count: u32,
    pub fallback_material_draw_count: u32,
    pub draws: Vec<B0IndexedDrawV1>,
    pub frame_plan_hash: ContentHash,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RenderFrameReportV1 {
    pub snapshot_hash: ContentHash,
    pub rendered_object_count: u32,
    pub indexed_draw_count: u32,
    pub fallback_material_draw_count: u32,
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

#[derive(Clone, Debug)]
pub struct ReferenceB0Renderer {
    catalog: RenderContentCatalogV1,
    device_available: bool,
}

impl ReferenceB0Renderer {
    pub fn new(catalog: RenderContentCatalogV1) -> Result<Self, RenderDeviceError> {
        validate_shader_interface(&catalog)?;
        Ok(Self {
            catalog,
            device_available: true,
        })
    }

    #[must_use]
    pub const fn catalog(&self) -> &RenderContentCatalogV1 {
        &self.catalog
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
        let plan = build_b0_frame_plan(snapshot, &self.catalog, target)?;
        Ok(RenderFrameReportV1 {
            snapshot_hash: plan.snapshot_hash,
            rendered_object_count: plan.visible_object_count,
            indexed_draw_count: plan.indexed_draw_count,
            fallback_material_draw_count: plan.fallback_material_draw_count,
            target_revision: target.target_revision,
            frame_plan_hash: plan.frame_plan_hash,
        })
    }

    fn invalidate_device(&mut self) {
        self.device_available = false;
    }

    fn recover_device(&mut self) {
        self.device_available = true;
    }
}

pub fn build_b0_frame_plan(
    snapshot: &PresentationSnapshotV2,
    catalog: &RenderContentCatalogV1,
    target: RenderTargetV1,
) -> Result<B0FramePlanV1, RenderDeviceError> {
    if target.extent[0] == 0 || target.extent[1] == 0 {
        return Err(RenderDeviceError::InvalidTarget);
    }
    snapshot.validate()?;
    validate_shader_interface(catalog)?;

    let fallback_revision = catalog.profile().fallback_material();
    let fallback_material = catalog
        .material(fallback_revision)
        .ok_or(RenderDeviceError::FallbackMaterialMissing)?;
    let indexed_draw_count = preflight_b0_indexed_draw_count(snapshot, catalog)?;
    let draw_capacity =
        usize::try_from(indexed_draw_count).map_err(|_| RenderDeviceError::CountOverflow)?;
    let mut draws = Vec::new();
    draws
        .try_reserve_exact(draw_capacity)
        .map_err(|_| RenderDeviceError::FramePlanAllocationFailed)?;
    let mut visible_object_count = 0_u32;
    let mut fallback_material_draw_count = 0_u32;

    for record in snapshot.scene_records().filter(|record| record.visible) {
        if record.feature_flags != ScenePresentationFlagsV1::NONE {
            return Err(RenderDeviceError::UnsupportedSceneFeature);
        }
        let mesh = catalog
            .mesh(record.mesh_revision)
            .ok_or(RenderDeviceError::MeshRevisionMissing)?;
        if mesh.bounds() != record.local_bounds {
            return Err(RenderDeviceError::PresentationBoundsMismatch);
        }
        let (material_revision, material, used_fallback) = catalog
            .material(record.material_revision)
            .map_or((fallback_revision, fallback_material, true), |material| {
                (record.material_revision, material, false)
            });
        let texture_binding = material
            .texture_bindings()
            .first()
            .filter(|binding| {
                binding.slot() == MaterialTextureSlotV1::BaseColor && binding.uv_set() == 0
            })
            .ok_or(RenderDeviceError::MaterialBindingInvalid)?;
        let texture_revision = texture_binding.texture();
        catalog
            .texture(texture_revision)
            .ok_or(RenderDeviceError::TextureRevisionMissing)?;

        visible_object_count = visible_object_count
            .checked_add(1)
            .ok_or(RenderDeviceError::CountOverflow)?;
        for primitive in mesh.primitives() {
            if primitive.topology() != MeshPrimitiveTopologyV1::Triangles {
                return Err(RenderDeviceError::UnsupportedTopology);
            }
            if primitive.material_slot() != 0 {
                return Err(RenderDeviceError::UnsupportedMaterialSlot);
            }
            if used_fallback {
                fallback_material_draw_count = fallback_material_draw_count
                    .checked_add(1)
                    .ok_or(RenderDeviceError::CountOverflow)?;
            }
            draws.push(B0IndexedDrawV1 {
                scene_record_hash: record.canonical_hash,
                mesh_revision: record.mesh_revision,
                material_revision,
                texture_revision,
                first_index: primitive.first_index(),
                index_count: primitive.index_count(),
                transform: record.current_transform,
                base_color_rgba_unorm16: material.base_color_rgba_unorm16(),
                fallback_material: used_fallback,
            });
        }
    }

    debug_assert_eq!(draws.len(), draw_capacity);
    let frame_plan_hash = frame_plan_hash(
        snapshot.canonical_hash,
        catalog.catalog_sha256(),
        target,
        visible_object_count,
        fallback_material_draw_count,
        &draws,
    )?;
    Ok(B0FramePlanV1 {
        snapshot_hash: snapshot.canonical_hash,
        catalog_hash: catalog.catalog_sha256(),
        target,
        visible_object_count,
        indexed_draw_count,
        fallback_material_draw_count,
        draws,
        frame_plan_hash,
    })
}

fn preflight_b0_indexed_draw_count(
    snapshot: &PresentationSnapshotV2,
    catalog: &RenderContentCatalogV1,
) -> Result<u32, RenderDeviceError> {
    let mut draw_count = 0_u64;
    for record in snapshot.scene_records().filter(|record| record.visible) {
        if record.feature_flags != ScenePresentationFlagsV1::NONE {
            return Err(RenderDeviceError::UnsupportedSceneFeature);
        }
        let mesh = catalog
            .mesh(record.mesh_revision)
            .ok_or(RenderDeviceError::MeshRevisionMissing)?;
        let primitive_count =
            u64::try_from(mesh.primitives().len()).map_err(|_| RenderDeviceError::CountOverflow)?;
        draw_count = draw_count
            .checked_add(primitive_count)
            .ok_or(RenderDeviceError::CountOverflow)?;
    }
    validate_b0_indexed_draw_budget(draw_count)
}

fn validate_b0_indexed_draw_budget(requested: u64) -> Result<u32, RenderDeviceError> {
    if requested > u64::from(B0_MAX_INDEXED_DRAWS_PER_FRAME) {
        return Err(RenderDeviceError::DrawBudgetExceeded {
            requested,
            limit: B0_MAX_INDEXED_DRAWS_PER_FRAME,
        });
    }
    u32::try_from(requested).map_err(|_| RenderDeviceError::CountOverflow)
}

fn validate_shader_interface(catalog: &RenderContentCatalogV1) -> Result<(), RenderDeviceError> {
    if catalog.profile().shader_interface_manifest_sha256() != b0_shader_interface_manifest_sha256()
    {
        Err(RenderDeviceError::ShaderInterfaceMismatch)
    } else {
        Ok(())
    }
}

fn frame_plan_hash(
    snapshot_hash: ContentHash,
    catalog_hash: ContentHash,
    target: RenderTargetV1,
    visible_object_count: u32,
    fallback_material_draw_count: u32,
    draws: &[B0IndexedDrawV1],
) -> Result<ContentHash, RenderDeviceError> {
    let draw_count = u64::try_from(draws.len()).map_err(|_| RenderDeviceError::CountOverflow)?;
    let _ = validate_b0_indexed_draw_budget(draw_count)?;
    let draw_bytes = B0_FRAME_PLAN_HASH_DRAW_BYTES
        .checked_mul(draws.len())
        .ok_or(RenderDeviceError::CountOverflow)?;
    let body_len = B0_FRAME_PLAN_HASH_HEADER_BYTES
        .checked_add(draw_bytes)
        .ok_or(RenderDeviceError::CountOverflow)?;
    let body_len_u64 = u64::try_from(body_len).map_err(|_| RenderDeviceError::CountOverflow)?;
    let preimage_len = B0_FRAME_PLAN_HASH_DOMAIN
        .len()
        .checked_add(1)
        .and_then(|length| length.checked_add(std::mem::size_of::<u64>()))
        .and_then(|length| length.checked_add(body_len))
        .ok_or(RenderDeviceError::CountOverflow)?;
    let mut preimage = Vec::new();
    preimage
        .try_reserve_exact(preimage_len)
        .map_err(|_| RenderDeviceError::FramePlanAllocationFailed)?;
    preimage.extend_from_slice(B0_FRAME_PLAN_HASH_DOMAIN.as_bytes());
    preimage.push(0);
    preimage.extend_from_slice(&body_len_u64.to_le_bytes());
    preimage.extend_from_slice(snapshot_hash.as_bytes());
    preimage.extend_from_slice(catalog_hash.as_bytes());
    preimage.extend_from_slice(&target.extent[0].to_le_bytes());
    preimage.extend_from_slice(&target.extent[1].to_le_bytes());
    preimage.extend_from_slice(&target.target_revision.to_le_bytes());
    preimage.extend_from_slice(&visible_object_count.to_le_bytes());
    preimage.extend_from_slice(&fallback_material_draw_count.to_le_bytes());
    for draw in draws {
        preimage.extend_from_slice(draw.scene_record_hash.as_bytes());
        extend_revision(&mut preimage, draw.mesh_revision);
        extend_revision(&mut preimage, draw.material_revision);
        extend_revision(&mut preimage, draw.texture_revision);
        preimage.extend_from_slice(&draw.first_index.to_le_bytes());
        preimage.extend_from_slice(&draw.index_count.to_le_bytes());
        preimage.push(u8::from(draw.fallback_material));
    }
    debug_assert_eq!(preimage.len(), preimage_len);
    Ok(content_hash_from_bytes(sha256(&preimage)))
}

fn extend_revision(bytes: &mut Vec<u8>, revision: AssetRevisionRefV1) {
    bytes.extend_from_slice(revision.asset_id.as_bytes());
    bytes.extend_from_slice(revision.record_sha256.as_bytes());
}

#[derive(Debug)]
#[non_exhaustive]
pub enum RenderDeviceError {
    Presentation(next_contracts::presentation::PresentationContractError),
    RenderContent(RenderContentContractError),
    InvalidTarget,
    DeviceLost,
    ShaderInterfaceMismatch,
    MeshRevisionMissing,
    TextureRevisionMissing,
    FallbackMaterialMissing,
    MaterialBindingInvalid,
    PresentationBoundsMismatch,
    UnsupportedSceneFeature,
    UnsupportedTopology,
    UnsupportedMaterialSlot,
    DrawBudgetExceeded { requested: u64, limit: u32 },
    FramePlanAllocationFailed,
    CountOverflow,
}

impl Display for RenderDeviceError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Presentation(error) => write!(formatter, "{error}"),
            Self::RenderContent(error) => write!(formatter, "{error}"),
            Self::InvalidTarget => formatter.write_str("render target invalid"),
            Self::DeviceLost => formatter.write_str("render device lost"),
            Self::ShaderInterfaceMismatch => {
                formatter.write_str("B0 shader interface manifest mismatch")
            }
            Self::MeshRevisionMissing => formatter.write_str("exact mesh revision missing"),
            Self::TextureRevisionMissing => formatter.write_str("exact texture revision missing"),
            Self::FallbackMaterialMissing => {
                formatter.write_str("declared fallback material missing")
            }
            Self::MaterialBindingInvalid => {
                formatter.write_str("B0 material base-color binding invalid")
            }
            Self::PresentationBoundsMismatch => {
                formatter.write_str("presentation bounds do not match exact mesh revision")
            }
            Self::UnsupportedSceneFeature => {
                formatter.write_str("scene feature is unsupported by B0")
            }
            Self::UnsupportedTopology => formatter.write_str("mesh topology is unsupported by B0"),
            Self::UnsupportedMaterialSlot => {
                formatter.write_str("mesh material slot is unsupported by B0")
            }
            Self::DrawBudgetExceeded { requested, limit } => {
                write!(
                    formatter,
                    "B0 indexed draw count {requested} exceeds per-frame limit {limit}"
                )
            }
            Self::FramePlanAllocationFailed => {
                formatter.write_str("B0 frame plan allocation failed")
            }
            Self::CountOverflow => formatter.write_str("render object count overflow"),
        }
    }
}

impl Error for RenderDeviceError {}

impl From<next_contracts::presentation::PresentationContractError> for RenderDeviceError {
    fn from(error: next_contracts::presentation::PresentationContractError) -> Self {
        Self::Presentation(error)
    }
}

impl From<RenderContentContractError> for RenderDeviceError {
    fn from(error: RenderContentContractError) -> Self {
        Self::RenderContent(error)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use next_contracts::ids::{AssetId, PersistentId};
    use next_contracts::presentation::{
        PresentationObjectKeyV1, PresentationRoleV1, ScenePresentationRecordV2,
    };
    use next_contracts::project::domain_hash;

    #[test]
    fn exact_catalog_builds_indexed_plan_and_survives_device_loss() {
        let catalog = catalog();
        let snapshot = snapshot(&catalog, false);
        let mut renderer = ReferenceB0Renderer::new(catalog).expect("renderer");
        let target = RenderTargetV1 {
            extent: [960, 540],
            target_revision: 1,
        };
        let first = renderer.render(&snapshot, target).expect("frame");
        assert_eq!(first.rendered_object_count, 5);
        assert_eq!(first.indexed_draw_count, 5);
        assert_eq!(first.fallback_material_draw_count, 0);
        renderer.invalidate_device();
        assert!(matches!(
            renderer.render(&snapshot, target),
            Err(RenderDeviceError::DeviceLost)
        ));
        renderer.recover_device();
        assert_eq!(renderer.render(&snapshot, target).expect("frame"), first);
    }

    #[test]
    fn missing_material_uses_the_exact_declared_fallback() {
        let catalog = catalog();
        let snapshot = snapshot(&catalog, true);
        let plan = build_b0_frame_plan(
            &snapshot,
            &catalog,
            RenderTargetV1 {
                extent: [960, 540],
                target_revision: 1,
            },
        )
        .expect("fallback frame");
        assert_eq!(plan.visible_object_count, 5);
        assert_eq!(plan.fallback_material_draw_count, 1);
        assert_eq!(
            plan.draws[0].material_revision,
            catalog.profile().fallback_material()
        );
        assert!(plan.draws[0].fallback_material);
    }

    #[test]
    fn indexed_draw_budget_accepts_limit_and_rejects_next_draw() {
        assert_eq!(
            validate_b0_indexed_draw_budget(u64::from(B0_MAX_INDEXED_DRAWS_PER_FRAME))
                .expect("limit is admitted"),
            B0_MAX_INDEXED_DRAWS_PER_FRAME
        );
        assert!(matches!(
            validate_b0_indexed_draw_budget(u64::from(B0_MAX_INDEXED_DRAWS_PER_FRAME) + 1),
            Err(RenderDeviceError::DrawBudgetExceeded {
                requested,
                limit: B0_MAX_INDEXED_DRAWS_PER_FRAME
            }) if requested == u64::from(B0_MAX_INDEXED_DRAWS_PER_FRAME) + 1
        ));
    }

    fn catalog() -> RenderContentCatalogV1 {
        next_project::cook_project_v1(next_reference_game::project_source_v2().expect("source"))
            .expect("cook")
            .render_content_catalog
    }

    fn snapshot(
        catalog: &RenderContentCatalogV1,
        missing_first_material: bool,
    ) -> PresentationSnapshotV2 {
        let epoch = domain_hash("test.epoch", b"epoch");
        let floor = &catalog.meshes()[0];
        let marker = &catalog.meshes()[1];
        let material = catalog
            .materials()
            .iter()
            .find(|material| {
                material.asset_revision().ok() != Some(catalog.profile().fallback_material())
            })
            .expect("base material")
            .asset_revision()
            .expect("material revision");
        let roles = [
            PresentationRoleV1::Environment,
            PresentationRoleV1::PlayerAvatar,
            PresentationRoleV1::InteractiveObject,
            PresentationRoleV1::Item,
            PresentationRoleV1::Character,
        ];
        let records = roles
            .into_iter()
            .enumerate()
            .map(|(index, role)| {
                let id = u8::try_from(index + 1).expect("small fixture");
                let mesh = if index == 0 { floor } else { marker };
                ScenePresentationRecordV2::new(
                    u16::try_from(index).expect("small fixture"),
                    PresentationObjectKeyV1 {
                        snapshot_epoch: epoch,
                        persistent_id: PersistentId::from_bytes([id; 16]),
                        presentation_role: role,
                        incarnation: 0,
                    },
                    mesh.asset_revision().expect("mesh revision"),
                    if missing_first_material && index == 0 {
                        AssetRevisionRefV1 {
                            asset_id: AssetId::from_bytes([0xee; 16]),
                            record_sha256: domain_hash("test.missing.material", b"missing"),
                        }
                    } else {
                        material
                    },
                    0,
                    mesh.bounds(),
                    ScenePresentationFlagsV1::NONE,
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
