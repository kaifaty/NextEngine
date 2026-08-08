use next_contracts::ids::{AssetId, ContentHash, PersistentId};
use next_contracts::presentation::CameraInterpolationPolicyV1;
use next_contracts::project::AssetRevisionRefV1;

use super::*;

#[test]
fn default_quantized_transform_encodes_identity_model() {
    assert_eq!(
        model_matrix(QuantizedPresentationTransformV1::default()),
        [
            1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0,
        ]
    );
}

#[test]
fn material_factor_uses_the_full_unorm16_range() {
    let bytes = draw_push_constant_bytes(
        QuantizedPresentationTransformV1::default(),
        [0, u16::MAX, 0, u16::MAX],
    );
    assert_eq!(&bytes[64..68], &0.0_f32.to_le_bytes());
    assert_eq!(&bytes[68..72], &1.0_f32.to_le_bytes());
    assert_eq!(&bytes[72..76], &0.0_f32.to_le_bytes());
    assert_eq!(&bytes[76..80], &1.0_f32.to_le_bytes());
}

#[test]
fn vulkan_y_projection_and_positive_viewport_preserve_ccw_front_faces() {
    assert!(B0_FRONT_FACE == vk::FrontFace::COUNTER_CLOCKWISE);
}

#[test]
fn b0_depth_uses_conventional_zero_to_one_less_or_equal_testing() {
    let state = b0_depth_stencil_state();
    assert!(state.depth_test_enable == vk::TRUE);
    assert!(state.depth_write_enable == vk::TRUE);
    assert!(state.depth_compare_op == vk::CompareOp::LESS_OR_EQUAL);
    assert!(state.depth_bounds_test_enable == vk::FALSE);
    assert!(state.stencil_test_enable == vk::FALSE);
}

#[test]
fn typed_camera_builds_vulkan_rh_zo_view_projection() {
    let camera = camera_frame(CameraViewportV1::full(0), [0, 0, 3_000_000], [0, 0, 0]);
    let state = frame_raster_state(
        Some(&camera),
        vk::Extent2D {
            width: 800,
            height: 800,
        },
    )
    .expect("camera raster state");
    let matrix = matrix_from_bytes(state.view_projection_bytes);

    let near = transform_homogeneous(matrix, [0.0, 0.0, 2.0, 1.0]);
    let far = transform_homogeneous(matrix, [0.0, 0.0, -7.0, 1.0]);
    let focus = transform_homogeneous(matrix, [0.0, 0.0, 0.0, 1.0]);
    let above_focus = transform_homogeneous(matrix, [0.0, 1.0, 0.0, 1.0]);

    assert_approx(near[2] / near[3], 0.0);
    assert_approx(far[2] / far[3], 1.0);
    assert_approx(focus[0] / focus[3], 0.0);
    assert_approx(focus[1] / focus[3], 0.0);
    assert!(
        above_focus[1] / above_focus[3] < 0.0,
        "projection Y must place world up toward Vulkan's top-left framebuffer origin"
    );
}

#[test]
fn typed_viewport_controls_dynamic_viewport_and_scissor() {
    let camera = camera_frame(
        CameraViewportV1::new(7, [2_570, 1_275], [25_700, 12_750]).expect("typed viewport"),
        [0, 0, 3_000_000],
        [0, 0, 0],
    );
    let state = frame_raster_state(
        Some(&camera),
        vk::Extent2D {
            width: 255,
            height: 257,
        },
    )
    .expect("camera raster state");

    assert_approx(state.viewport.x, 10.0);
    assert_approx(state.viewport.y, 5.0);
    assert_approx(state.viewport.width, 100.0);
    assert_approx(state.viewport.height, 50.0);
    assert!(state.scissor.offset == vk::Offset2D { x: 10, y: 5 });
    assert!(
        state.scissor.extent
            == vk::Extent2D {
                width: 100,
                height: 50
            }
    );
}

#[test]
fn absent_camera_uses_deterministic_full_target_fallback() {
    let extent = vk::Extent2D {
        width: 960,
        height: 540,
    };
    let first = frame_raster_state(None, extent).expect("fallback");
    let second = frame_raster_state(None, extent).expect("fallback");
    let matrix = matrix_from_bytes(first.view_projection_bytes);

    assert_eq!(first.view_projection_bytes, second.view_projection_bytes);
    assert_approx(first.viewport.x, 0.0);
    assert_approx(first.viewport.y, 0.0);
    assert_approx(first.viewport.width, 960.0);
    assert_approx(first.viewport.height, 540.0);
    assert!(first.scissor.offset == vk::Offset2D { x: 0, y: 0 });
    assert!(first.scissor.extent == extent);
    assert!(matrix[5] < 0.0, "fallback must use the same Vulkan Y flip");
}

#[test]
fn coincident_camera_eye_and_focus_fail_closed() {
    let camera = camera_frame(
        CameraViewportV1::full(0),
        [0, 1_000_000, 0],
        [0, 1_000_000, 0],
    );
    assert!(matches!(
        frame_raster_state(
            Some(&camera),
            vk::Extent2D {
                width: 800,
                height: 600
            }
        ),
        Err(B0GpuContentError::InvalidFramePlan(
            "camera eye and focus do not define a view direction"
        ))
    ));
}

#[test]
fn camera_parallel_to_world_up_fails_closed() {
    let camera = camera_frame(CameraViewportV1::full(0), [0, 0, 0], [0, 1_000_000, 0]);
    assert!(matches!(
        frame_raster_state(
            Some(&camera),
            vk::Extent2D {
                width: 800,
                height: 600
            }
        ),
        Err(B0GpuContentError::InvalidFramePlan(
            "camera view direction is parallel to world up"
        ))
    ));
}

#[test]
fn malformed_typed_camera_fields_fail_before_gpu_use() {
    let mut camera = camera_frame(CameraViewportV1::full(0), [0, 0, 3_000_000], [0, 0, 0]);
    camera.projection_profile.vertical_fov_millidegrees = 0;
    assert!(matches!(
        frame_raster_state(
            Some(&camera),
            vk::Extent2D {
                width: 800,
                height: 600
            }
        ),
        Err(B0GpuContentError::InvalidFramePlan(
            "camera projection profile is invalid"
        ))
    ));

    camera.projection_profile.vertical_fov_millidegrees = 90_000;
    camera.viewport = CameraViewportV1 {
        viewport_id: 0,
        origin_unorm16: [u16::MAX, 0],
        extent_unorm16: [1, u16::MAX],
    };
    assert!(matches!(
        frame_raster_state(
            Some(&camera),
            vk::Extent2D {
                width: 800,
                height: 600
            }
        ),
        Err(B0GpuContentError::InvalidFramePlan(
            "camera viewport is invalid"
        ))
    ));
}

#[test]
fn non_finite_matrix_component_fails_closed() {
    let mut matrix = [0.0; 16];
    matrix[0] = f64::INFINITY;
    assert!(matches!(
        f64_matrix_to_f32(matrix),
        Err(B0GpuContentError::InvalidFramePlan(
            "camera matrix contains a non-finite component"
        ))
    ));
}

#[test]
fn shadow_projection_snaps_camera_motion_to_2048_texels() {
    let first = shadow_view_projection_matrix([0.0, 2.0, 0.0]).expect("shadow matrix");
    let sub_texel = shadow_view_projection_matrix([0.001, 2.0, 0.0]).expect("shadow matrix");
    assert_eq!(first[12], sub_texel[12]);
    assert_eq!(first[13], sub_texel[13]);
    assert!(first.into_iter().all(f32::is_finite));
}

fn camera_frame(
    viewport: CameraViewportV1,
    eye_micrometres: [i64; 3],
    focus_micrometres: [i64; 3],
) -> B0CameraFrameV1 {
    let result = CameraResultSampleV1 {
        pose: QuantizedPresentationTransformV1 {
            translation_micrometres: eye_micrometres,
            ..QuantizedPresentationTransformV1::default()
        },
        focus_point_micrometres: focus_micrometres,
    };
    B0CameraFrameV1 {
        camera_record_hash: ContentHash::from_bytes([1; 32]),
        camera_id: PersistentId::from_bytes([2; 16]),
        camera_role: CameraRoleV1::PrimaryThirdPerson,
        viewport,
        projection_profile: CameraProjectionProfileV1::new(90_000, 1_000_000, 10_000_000)
            .expect("projection"),
        exposure_profile_revision: AssetRevisionRefV1 {
            asset_id: AssetId::from_bytes([3; 16]),
            record_sha256: ContentHash::from_bytes([4; 32]),
        },
        previous_result_sample: result,
        current_result_sample: result,
        cut: false,
        interpolation_policy: CameraInterpolationPolicyV1::LinearPose,
    }
}

fn matrix_from_bytes(bytes: [u8; FRAME_UNIFORM_SIZE as usize]) -> [f32; 16] {
    let mut matrix = [0.0; 16];
    for (destination, source) in matrix.iter_mut().zip(bytes.chunks_exact(4)) {
        *destination = f32::from_le_bytes(source.try_into().expect("one matrix component"));
    }
    matrix
}

fn transform_homogeneous(matrix: [f32; 16], value: [f32; 4]) -> [f32; 4] {
    let mut transformed = [0.0; 4];
    for row in 0..4 {
        transformed[row] = (0..4)
            .map(|column| matrix[column * 4 + row] * value[column])
            .sum();
    }
    transformed
}

fn assert_approx(actual: f32, expected: f32) {
    assert!(
        (actual - expected).abs() <= 1.0e-5,
        "expected {expected}, got {actual}"
    );
}
