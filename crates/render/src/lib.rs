#![forbid(unsafe_code)]

use std::error::Error;
use std::fmt::{Display, Formatter};

use next_contracts::canonical::sha256;
use next_contracts::ids::{ContentHash, PersistentId, content_hash_from_bytes};
use next_contracts::presentation::{
    CameraInterpolationPolicyV1, CameraProjectionProfileV1, CameraResultSampleV1, CameraRoleV1,
    CameraViewportV1, PresentationSnapshotV2, QuantizedPresentationTransformV1,
    ScenePresentationFlagsV1,
};
use next_contracts::project::AssetRevisionRefV1;
use next_contracts::render_content::{
    MaterialTextureSlotV1, MeshPrimitiveTopologyV1, RenderContentCatalogV1,
    RenderContentContractError, b0_shader_interface_manifest_sha256,
};

pub const B0_MAX_INDEXED_DRAWS_PER_FRAME: u32 = 65_536;

const B0_FRAME_PLAN_HASH_DOMAIN: &str = "nextengine.render-frame-plan.b0.v1";
const B0_FRAME_PLAN_HASH_BASE_HEADER_BYTES: usize = 89;
const B0_FRAME_PLAN_HASH_CAMERA_BYTES: usize = 257;
const B0_FRAME_PLAN_HASH_DRAW_BYTES: usize = 185;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RenderTargetV1 {
    pub extent: [u32; 2],
    pub target_revision: u64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct B0CameraFrameV1 {
    pub camera_record_hash: ContentHash,
    pub camera_id: PersistentId,
    pub camera_role: CameraRoleV1,
    pub viewport: CameraViewportV1,
    pub projection_profile: CameraProjectionProfileV1,
    pub exposure_profile_revision: AssetRevisionRefV1,
    pub previous_result_sample: CameraResultSampleV1,
    pub current_result_sample: CameraResultSampleV1,
    pub cut: bool,
    pub interpolation_policy: CameraInterpolationPolicyV1,
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
    pub camera: Option<B0CameraFrameV1>,
    pub visible_object_count: u32,
    pub indexed_draw_count: u32,
    pub fallback_material_draw_count: u32,
    pub draws: Vec<B0IndexedDrawV1>,
    pub frame_plan_hash: ContentHash,
}

mod planner;

pub use planner::{B0FramePlannerMetricsV1, B0FramePlannerV1};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RenderFrameReportV1 {
    pub snapshot_hash: ContentHash,
    pub rendered_object_count: u32,
    pub indexed_draw_count: u32,
    pub fallback_material_draw_count: u32,
    pub camera_record_hash: Option<ContentHash>,
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
    frame_planner: B0FramePlannerV1,
}

impl ReferenceB0Renderer {
    pub fn new(catalog: RenderContentCatalogV1) -> Result<Self, RenderDeviceError> {
        validate_shader_interface(&catalog)?;
        Ok(Self {
            catalog,
            device_available: true,
            frame_planner: B0FramePlannerV1::new(),
        })
    }

    #[must_use]
    pub const fn catalog(&self) -> &RenderContentCatalogV1 {
        &self.catalog
    }

    #[must_use]
    pub const fn frame_planner_metrics(&self) -> B0FramePlannerMetricsV1 {
        self.frame_planner.metrics()
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
        let plan = self
            .frame_planner
            .build_or_reuse(snapshot, &self.catalog, target)?;
        Ok(RenderFrameReportV1 {
            snapshot_hash: plan.snapshot_hash,
            rendered_object_count: plan.visible_object_count,
            indexed_draw_count: plan.indexed_draw_count,
            fallback_material_draw_count: plan.fallback_material_draw_count,
            camera_record_hash: plan.camera.map(|camera| camera.camera_record_hash),
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
    let mut draws = Vec::new();
    let mut hash_preimage = Vec::new();
    let parts =
        build_b0_frame_plan_parts(snapshot, catalog, target, &mut draws, &mut hash_preimage)?;
    Ok(parts.finish(draws))
}

struct B0FramePlanPartsV1 {
    snapshot_hash: ContentHash,
    catalog_hash: ContentHash,
    target: RenderTargetV1,
    camera: Option<B0CameraFrameV1>,
    visible_object_count: u32,
    indexed_draw_count: u32,
    fallback_material_draw_count: u32,
    frame_plan_hash: ContentHash,
}

impl B0FramePlanPartsV1 {
    fn finish(self, draws: Vec<B0IndexedDrawV1>) -> B0FramePlanV1 {
        B0FramePlanV1 {
            snapshot_hash: self.snapshot_hash,
            catalog_hash: self.catalog_hash,
            target: self.target,
            camera: self.camera,
            visible_object_count: self.visible_object_count,
            indexed_draw_count: self.indexed_draw_count,
            fallback_material_draw_count: self.fallback_material_draw_count,
            draws,
            frame_plan_hash: self.frame_plan_hash,
        }
    }
}

fn build_b0_frame_plan_parts(
    snapshot: &PresentationSnapshotV2,
    catalog: &RenderContentCatalogV1,
    target: RenderTargetV1,
    draws: &mut Vec<B0IndexedDrawV1>,
    hash_preimage: &mut Vec<u8>,
) -> Result<B0FramePlanPartsV1, RenderDeviceError> {
    if target.extent[0] == 0 || target.extent[1] == 0 {
        return Err(RenderDeviceError::InvalidTarget);
    }
    snapshot.validate()?;
    validate_shader_interface(catalog)?;
    let camera = select_b0_camera(snapshot, catalog.profile_revision())?;

    let fallback_revision = catalog.profile().fallback_material();
    let fallback_material = catalog
        .material(fallback_revision)
        .ok_or(RenderDeviceError::FallbackMaterialMissing)?;
    let indexed_draw_count = preflight_b0_indexed_draw_count(snapshot, catalog)?;
    let draw_capacity =
        usize::try_from(indexed_draw_count).map_err(|_| RenderDeviceError::CountOverflow)?;
    draws.clear();
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
        B0FramePlanHashInputV1 {
            snapshot_hash: snapshot.canonical_hash,
            catalog_hash: catalog.catalog_sha256(),
            target,
            camera: camera.as_ref(),
            visible_object_count,
            fallback_material_draw_count,
            draws,
        },
        hash_preimage,
    )?;
    Ok(B0FramePlanPartsV1 {
        snapshot_hash: snapshot.canonical_hash,
        catalog_hash: catalog.catalog_sha256(),
        target,
        camera,
        visible_object_count,
        indexed_draw_count,
        fallback_material_draw_count,
        frame_plan_hash,
    })
}

fn select_b0_camera(
    snapshot: &PresentationSnapshotV2,
    expected_exposure_profile_revision: AssetRevisionRefV1,
) -> Result<Option<B0CameraFrameV1>, RenderDeviceError> {
    let mut cameras = snapshot.camera_records();
    let Some(camera) = cameras.next() else {
        return Ok(None);
    };
    if cameras.next().is_some() || camera.camera_role != CameraRoleV1::PrimaryThirdPerson {
        return Err(RenderDeviceError::UnsupportedCameraConfiguration);
    }
    if camera.exposure_profile_revision != expected_exposure_profile_revision {
        return Err(RenderDeviceError::CameraExposureProfileMismatch);
    }
    Ok(Some(B0CameraFrameV1 {
        camera_record_hash: camera.canonical_hash,
        camera_id: camera.camera_id,
        camera_role: camera.camera_role,
        viewport: camera.viewport,
        projection_profile: camera.projection_profile,
        exposure_profile_revision: camera.exposure_profile_revision,
        previous_result_sample: camera.previous_result_sample,
        current_result_sample: camera.current_result_sample,
        cut: camera.cut,
        interpolation_policy: camera.interpolation_policy,
    }))
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

struct B0FramePlanHashInputV1<'a> {
    snapshot_hash: ContentHash,
    catalog_hash: ContentHash,
    target: RenderTargetV1,
    camera: Option<&'a B0CameraFrameV1>,
    visible_object_count: u32,
    fallback_material_draw_count: u32,
    draws: &'a [B0IndexedDrawV1],
}

fn frame_plan_hash(
    input: B0FramePlanHashInputV1<'_>,
    preimage: &mut Vec<u8>,
) -> Result<ContentHash, RenderDeviceError> {
    let draw_count =
        u64::try_from(input.draws.len()).map_err(|_| RenderDeviceError::CountOverflow)?;
    let _ = validate_b0_indexed_draw_budget(draw_count)?;
    let draw_bytes = B0_FRAME_PLAN_HASH_DRAW_BYTES
        .checked_mul(input.draws.len())
        .ok_or(RenderDeviceError::CountOverflow)?;
    let camera_bytes = if input.camera.is_some() {
        B0_FRAME_PLAN_HASH_CAMERA_BYTES
    } else {
        0
    };
    let body_len = B0_FRAME_PLAN_HASH_BASE_HEADER_BYTES
        .checked_add(camera_bytes)
        .ok_or(RenderDeviceError::CountOverflow)?
        .checked_add(draw_bytes)
        .ok_or(RenderDeviceError::CountOverflow)?;
    let body_len_u64 = u64::try_from(body_len).map_err(|_| RenderDeviceError::CountOverflow)?;
    let preimage_len = B0_FRAME_PLAN_HASH_DOMAIN
        .len()
        .checked_add(1)
        .and_then(|length| length.checked_add(std::mem::size_of::<u64>()))
        .and_then(|length| length.checked_add(body_len))
        .ok_or(RenderDeviceError::CountOverflow)?;
    preimage.clear();
    preimage
        .try_reserve_exact(preimage_len)
        .map_err(|_| RenderDeviceError::FramePlanAllocationFailed)?;
    preimage.extend_from_slice(B0_FRAME_PLAN_HASH_DOMAIN.as_bytes());
    preimage.push(0);
    preimage.extend_from_slice(&body_len_u64.to_le_bytes());
    preimage.extend_from_slice(input.snapshot_hash.as_bytes());
    preimage.extend_from_slice(input.catalog_hash.as_bytes());
    preimage.extend_from_slice(&input.target.extent[0].to_le_bytes());
    preimage.extend_from_slice(&input.target.extent[1].to_le_bytes());
    preimage.extend_from_slice(&input.target.target_revision.to_le_bytes());
    preimage.extend_from_slice(&input.visible_object_count.to_le_bytes());
    preimage.extend_from_slice(&input.fallback_material_draw_count.to_le_bytes());
    match input.camera {
        Some(camera) => {
            preimage.push(1);
            extend_camera_frame(preimage, camera);
        }
        None => preimage.push(0),
    }
    for draw in input.draws {
        preimage.extend_from_slice(draw.scene_record_hash.as_bytes());
        extend_revision(preimage, draw.mesh_revision);
        extend_revision(preimage, draw.material_revision);
        extend_revision(preimage, draw.texture_revision);
        preimage.extend_from_slice(&draw.first_index.to_le_bytes());
        preimage.extend_from_slice(&draw.index_count.to_le_bytes());
        preimage.push(u8::from(draw.fallback_material));
    }
    debug_assert_eq!(preimage.len(), preimage_len);
    Ok(content_hash_from_bytes(sha256(preimage)))
}

fn extend_camera_frame(bytes: &mut Vec<u8>, camera: &B0CameraFrameV1) {
    bytes.extend_from_slice(camera.camera_record_hash.as_bytes());
    bytes.extend_from_slice(camera.camera_id.as_bytes());
    bytes.push(camera.camera_role as u8);
    bytes.extend_from_slice(&camera.viewport.viewport_id.to_le_bytes());
    for value in camera.viewport.origin_unorm16 {
        bytes.extend_from_slice(&value.to_le_bytes());
    }
    for value in camera.viewport.extent_unorm16 {
        bytes.extend_from_slice(&value.to_le_bytes());
    }
    bytes.extend_from_slice(
        &camera
            .projection_profile
            .vertical_fov_millidegrees
            .to_le_bytes(),
    );
    bytes.extend_from_slice(
        &camera
            .projection_profile
            .near_plane_micrometres
            .to_le_bytes(),
    );
    bytes.extend_from_slice(
        &camera
            .projection_profile
            .far_plane_micrometres
            .to_le_bytes(),
    );
    extend_revision(bytes, camera.exposure_profile_revision);
    extend_camera_result(bytes, camera.previous_result_sample);
    extend_camera_result(bytes, camera.current_result_sample);
    bytes.push(u8::from(camera.cut));
    bytes.push(camera.interpolation_policy as u8);
}

fn extend_camera_result(bytes: &mut Vec<u8>, result: CameraResultSampleV1) {
    for value in result.pose.translation_micrometres {
        bytes.extend_from_slice(&value.to_le_bytes());
    }
    for value in result.pose.orientation_q30 {
        bytes.extend_from_slice(&value.to_le_bytes());
    }
    for value in result.focus_point_micrometres {
        bytes.extend_from_slice(&value.to_le_bytes());
    }
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
    UnsupportedCameraConfiguration,
    CameraExposureProfileMismatch,
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
            Self::UnsupportedCameraConfiguration => {
                formatter.write_str("B0 supports one primary third-person camera")
            }
            Self::CameraExposureProfileMismatch => {
                formatter.write_str("camera exposure profile does not match the exact B0 profile")
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
        CameraInterpolationPolicyV1, CameraPresentationRecordV2, CameraProjectionProfileV1,
        CameraResultSampleV1, CameraRoleV1, CameraViewportV1, PresentationObjectKeyV1,
        PresentationRoleV1, ScenePresentationRecordV2, ThirdPersonCameraIntentSampleV1,
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
        assert_eq!(
            renderer.frame_planner_metrics(),
            B0FramePlannerMetricsV1 {
                cache_hits: 1,
                cache_misses: 1,
                build_failures: 0,
                explicit_invalidations: 0,
            }
        );
    }

    #[test]
    fn reusable_planner_hits_for_exact_inputs_and_matches_uncached_builder() {
        let catalog = catalog();
        let snapshot = snapshot(&catalog, false);
        let target = RenderTargetV1 {
            extent: [960, 540],
            target_revision: 1,
        };
        let expected = build_b0_frame_plan(&snapshot, &catalog, target).expect("uncached plan");
        let equivalent_catalog = catalog.clone();
        let mut planner = B0FramePlannerV1::new();

        let first = planner
            .build_or_reuse(&snapshot, &catalog, target)
            .expect("cache miss")
            .clone();
        let second = planner
            .build_or_reuse(&snapshot, &equivalent_catalog, target)
            .expect("cache hit");

        assert_eq!(first, expected);
        assert_eq!(second, &expected);
        assert_eq!(planner.cached_plan(), Some(&expected));
        assert_eq!(
            planner.metrics(),
            B0FramePlannerMetricsV1 {
                cache_hits: 1,
                cache_misses: 1,
                build_failures: 0,
                explicit_invalidations: 0,
            }
        );
    }

    #[test]
    fn reusable_planner_invalidates_on_snapshot_and_target_change() {
        let catalog = catalog();
        let first_snapshot = snapshot(&catalog, false);
        let next_snapshot = snapshot_at(&catalog, false, 1, 1);
        let first_target = RenderTargetV1 {
            extent: [960, 540],
            target_revision: 1,
        };
        let next_target = RenderTargetV1 {
            extent: [1280, 720],
            target_revision: 2,
        };
        let mut planner = B0FramePlannerV1::new();

        let first = planner
            .build_or_reuse(&first_snapshot, &catalog, first_target)
            .expect("first snapshot")
            .clone();
        let next = planner
            .build_or_reuse(&next_snapshot, &catalog, first_target)
            .expect("next snapshot")
            .clone();
        let resized = planner
            .build_or_reuse(&next_snapshot, &catalog, next_target)
            .expect("resized target")
            .clone();
        let repeated = planner
            .build_or_reuse(&next_snapshot, &catalog, next_target)
            .expect("repeated resized target");

        assert_ne!(first.snapshot_hash, next.snapshot_hash);
        assert_ne!(next.frame_plan_hash, resized.frame_plan_hash);
        assert_eq!(repeated, &resized);
        assert_eq!(planner.metrics().cache_misses, 3);
        assert_eq!(planner.metrics().cache_hits, 1);
        assert_eq!(planner.metrics().build_failures, 0);
    }

    #[test]
    fn failed_rebuild_preserves_the_last_exact_cached_plan() {
        let catalog = catalog();
        let snapshot = snapshot(&catalog, false);
        let target = RenderTargetV1 {
            extent: [960, 540],
            target_revision: 1,
        };
        let mut planner = B0FramePlannerV1::new();
        let baseline = planner
            .build_or_reuse(&snapshot, &catalog, target)
            .expect("baseline")
            .clone();
        let mut malformed = snapshot.clone();
        malformed.simulation_tick = malformed.simulation_tick.saturating_add(1);

        assert!(matches!(
            planner.build_or_reuse(&malformed, &catalog, target),
            Err(RenderDeviceError::Presentation(
                next_contracts::presentation::PresentationContractError::HashMismatch
            ))
        ));
        assert_eq!(planner.cached_plan(), Some(&baseline));
        assert_eq!(
            planner
                .build_or_reuse(&snapshot, &catalog, target)
                .expect("prior exact input remains cached"),
            &baseline
        );
        assert_eq!(
            planner.metrics(),
            B0FramePlannerMetricsV1 {
                cache_hits: 1,
                cache_misses: 2,
                build_failures: 1,
                explicit_invalidations: 0,
            }
        );
    }

    #[test]
    fn explicit_invalidation_drops_only_the_cached_value_and_reuses_the_planner() {
        let catalog = catalog();
        let snapshot = snapshot(&catalog, false);
        let target = RenderTargetV1 {
            extent: [960, 540],
            target_revision: 1,
        };
        let mut planner = B0FramePlannerV1::new();
        let expected = planner
            .build_or_reuse(&snapshot, &catalog, target)
            .expect("initial plan")
            .clone();

        planner.invalidate();
        planner.invalidate();
        assert!(planner.cached_plan().is_none());
        assert_eq!(planner.metrics().explicit_invalidations, 1);
        assert_eq!(
            planner
                .build_or_reuse(&snapshot, &catalog, target)
                .expect("rebuilt plan"),
            &expected
        );
        assert_eq!(planner.metrics().cache_misses, 2);
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
    fn typed_third_person_camera_is_bound_into_the_frame_plan_hash() {
        let catalog = catalog();
        let without_camera = snapshot(&catalog, false);
        let camera_record = CameraPresentationRecordV2::new(
            without_camera.snapshot_epoch,
            PersistentId::from_bytes([0xc0; 16]),
            CameraRoleV1::PrimaryThirdPerson,
            CameraViewportV1::full(0),
            CameraProjectionProfileV1::new(60_000, 100_000, 100_000_000).expect("projection"),
            ThirdPersonCameraIntentSampleV1 {
                focus_subject_id: Some(PersistentId::from_bytes([2; 16])),
                focus_point_micrometres: [0, 1_000_000, 0],
                orbit_yaw_millidegrees: 0,
                orbit_pitch_millidegrees: -15_000,
                distance_micrometres: 3_000_000,
                shoulder_offset_micrometres: [350_000, 0, 0],
            },
            camera_result([0, 2_000_000, 3_000_000]),
            camera_result([250_000, 2_000_000, 3_000_000]),
            catalog.profile_revision(),
            false,
            CameraInterpolationPolicyV1::LinearPose,
        )
        .expect("camera");
        let with_camera = PresentationSnapshotV2::new_with_camera_records(
            without_camera.snapshot_epoch,
            without_camera.snapshot_sequence,
            without_camera.simulation_tick,
            without_camera.project_composition_lock_hash,
            without_camera.content_manifest_hash,
            without_camera.presentation_profile_hash,
            without_camera.scene_records().cloned().collect(),
            vec![camera_record.clone()],
            8,
            8,
            without_camera.environment_batch,
        )
        .expect("camera snapshot");
        let target = RenderTargetV1 {
            extent: [960, 540],
            target_revision: 1,
        };
        let camera_plan = build_b0_frame_plan(&with_camera, &catalog, target).expect("camera plan");
        let plain_plan =
            build_b0_frame_plan(&without_camera, &catalog, target).expect("plain plan");
        assert_eq!(
            camera_plan.camera.map(|camera| camera.camera_record_hash),
            Some(camera_record.canonical_hash)
        );
        assert_eq!(
            camera_plan
                .camera
                .map(|camera| camera.exposure_profile_revision),
            Some(catalog.profile_revision())
        );
        assert_ne!(camera_plan.frame_plan_hash, plain_plan.frame_plan_hash);

        let wrong_profile_camera = CameraPresentationRecordV2::new(
            camera_record.snapshot_epoch,
            camera_record.camera_id,
            camera_record.camera_role,
            camera_record.viewport,
            camera_record.projection_profile,
            camera_record.intent_sample,
            camera_record.previous_result_sample,
            camera_record.current_result_sample,
            catalog.materials()[0]
                .asset_revision()
                .expect("non-profile revision"),
            camera_record.cut,
            camera_record.interpolation_policy,
        )
        .expect("well-formed camera with unavailable profile");
        let wrong_profile_snapshot = PresentationSnapshotV2::new_with_camera_records(
            without_camera.snapshot_epoch,
            without_camera.snapshot_sequence,
            without_camera.simulation_tick,
            without_camera.project_composition_lock_hash,
            without_camera.content_manifest_hash,
            without_camera.presentation_profile_hash,
            without_camera.scene_records().cloned().collect(),
            vec![wrong_profile_camera],
            8,
            8,
            without_camera.environment_batch,
        )
        .expect("camera snapshot");
        assert!(matches!(
            build_b0_frame_plan(&wrong_profile_snapshot, &catalog, target),
            Err(RenderDeviceError::CameraExposureProfileMismatch)
        ));
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

    fn camera_result(translation_micrometres: [i64; 3]) -> CameraResultSampleV1 {
        CameraResultSampleV1 {
            pose: QuantizedPresentationTransformV1 {
                translation_micrometres,
                ..QuantizedPresentationTransformV1::default()
            },
            focus_point_micrometres: [0, 1_000_000, 0],
        }
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
        snapshot_at(catalog, missing_first_material, 0, 0)
    }

    fn snapshot_at(
        catalog: &RenderContentCatalogV1,
        missing_first_material: bool,
        snapshot_sequence: u64,
        simulation_tick: u64,
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
            snapshot_sequence,
            simulation_tick,
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
