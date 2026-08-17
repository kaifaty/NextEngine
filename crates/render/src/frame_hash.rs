use next_contracts::canonical::sha256;
use next_contracts::ids::{ContentHash, content_hash_from_bytes};
use next_contracts::presentation::CameraResultSampleV1;
use next_contracts::project::AssetRevisionRefV1;

use super::{
    B0CameraFrameV1, B0IndexedDrawV1, B0SkinnedVertexStreamV1, RenderDeviceError, RenderTargetV1,
    validate_b0_indexed_draw_budget,
};

const B0_FRAME_PLAN_HASH_DOMAIN: &str = "nextengine.render-frame-plan.b0.v1";
const B0_FRAME_PLAN_HASH_BASE_HEADER_BYTES: usize = 89;
const B0_FRAME_PLAN_HASH_CAMERA_BYTES: usize = 257;
const B0_FRAME_PLAN_HASH_DRAW_BYTES: usize = 223;

pub(super) struct B0FramePlanHashInputV1<'a> {
    pub(super) snapshot_hash: ContentHash,
    pub(super) catalog_hash: ContentHash,
    pub(super) target: RenderTargetV1,
    pub(super) camera: Option<&'a B0CameraFrameV1>,
    pub(super) visible_object_count: u32,
    pub(super) fallback_material_draw_count: u32,
    pub(super) draws: &'a [B0IndexedDrawV1],
    pub(super) skinned_vertex_streams: &'a [B0SkinnedVertexStreamV1],
}

pub(super) fn frame_plan_hash(
    input: B0FramePlanHashInputV1<'_>,
    preimage: &mut Vec<u8>,
) -> Result<ContentHash, RenderDeviceError> {
    let draw_count =
        u64::try_from(input.draws.len()).map_err(|_| RenderDeviceError::CountOverflow)?;
    let _ = validate_b0_indexed_draw_budget(draw_count)?;
    for draw in input.draws {
        match (
            draw.skinning_vertex_stream_index,
            draw.skinning_vertex_stream_hash,
        ) {
            (Some(index), Some(expected_hash)) => {
                let stream = input
                    .skinned_vertex_streams
                    .get(usize::try_from(index).map_err(|_| RenderDeviceError::CountOverflow)?)
                    .ok_or(RenderDeviceError::SkinningBindingInvalid)?;
                if stream.vertex_stream_hash != expected_hash {
                    return Err(RenderDeviceError::SkinningBindingInvalid);
                }
            }
            (None, None) => {}
            _ => return Err(RenderDeviceError::SkinningBindingInvalid),
        }
    }
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
        preimage.push(u8::from(draw.casts_shadow));
        preimage.extend_from_slice(
            &draw
                .skinning_vertex_stream_index
                .unwrap_or(u32::MAX)
                .to_le_bytes(),
        );
        preimage.extend_from_slice(
            draw.skinning_vertex_stream_hash
                .unwrap_or_default()
                .as_bytes(),
        );
        preimage.push(u8::from(draw.base_skinning_fallback));
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
