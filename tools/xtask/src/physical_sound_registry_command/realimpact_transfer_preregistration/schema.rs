use std::path::PathBuf;

use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Manifest {
    pub(super) schema: String,
    pub(super) study_id: String,
    pub(super) revision: String,
    pub(super) phase: String,
    pub(super) objective: String,
    pub(super) access_state_at_freeze: String,
    pub(super) prerequisites: Vec<Prerequisite>,
    pub(super) roles: Roles,
    pub(super) objects: Vec<ObjectIdentity>,
    pub(super) opening_protocol: OpeningProtocol,
    pub(super) geometry_preflight: GeometryPreflight,
    pub(super) signal_extractor: SignalExtractor,
    pub(super) listener_split: ListenerSplit,
    pub(super) candidate: Candidate,
    pub(super) controls: Vec<Control>,
    pub(super) mode_admission: ModeAdmission,
    pub(super) condition_gate: ConditionGate,
    pub(super) comparison_gate: ComparisonGate,
    pub(super) fallback: Fallback,
    pub(super) allowed_claims: Vec<String>,
    pub(super) prohibited_claims: Vec<String>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Prerequisite {
    pub(super) id: String,
    pub(super) path: PathBuf,
    pub(super) sha256: String,
    pub(super) expected_schema: String,
    #[serde(default)]
    pub(super) expected_status: Option<String>,
    #[serde(default)]
    pub(super) expected_decision: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Roles {
    pub(super) calibration: String,
    pub(super) holdout: String,
    pub(super) selection_rule: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct ObjectIdentity {
    pub(super) object_id: String,
    pub(super) role: String,
    pub(super) selection_sha256: String,
    pub(super) archive_url: String,
    pub(super) archive_bytes: u64,
    pub(super) archive_etag: String,
    pub(super) archive_last_modified_http: String,
    pub(super) central_sha256: String,
    pub(super) mesh_entry_name: String,
    pub(super) mesh_raw_sha256: String,
    pub(super) mesh_vertex_count: usize,
    pub(super) audio_entry_name: String,
    pub(super) audio_crc32: String,
    pub(super) audio_data_offset: u64,
    pub(super) audio_compressed_bytes: u64,
    pub(super) audio_uncompressed_bytes: u64,
    pub(super) audio_sample_count: usize,
    pub(super) required_compressed_prefix_bytes: u64,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct OpeningProtocol {
    pub(super) preregistration_network_access_allowed: bool,
    pub(super) geometry_preflight_before_audio: bool,
    pub(super) calibration_payload_requires_preregistration_report: bool,
    pub(super) holdout_payload_requires_calibration_pass: bool,
    pub(super) holdout_open_count: usize,
    pub(super) on_calibration_failure: String,
    pub(super) on_holdout_failure: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct GeometryPreflight {
    pub(super) parser: String,
    pub(super) simplifier: String,
    pub(super) spectral_face_target: usize,
    pub(super) bem_face_target: usize,
    pub(super) field_transfer: String,
    pub(super) topology_gate: String,
    pub(super) repeat_requirement: String,
    pub(super) on_failure: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct SignalExtractor {
    pub(super) id: String,
    pub(super) sample_rate_hz: u32,
    pub(super) impact_ordinal: usize,
    pub(super) impact_row_count: usize,
    pub(super) normalization_selector: String,
    pub(super) persistent_definition: String,
    pub(super) floor_db: f64,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct ListenerSplit {
    pub(super) anchor_angles_degrees: Vec<u32>,
    pub(super) anchor_distance_offsets_millimetres: Vec<u32>,
    pub(super) anchor_microphone_ids: Vec<usize>,
    pub(super) anchor_count: usize,
    pub(super) held_rule: String,
    pub(super) held_count: usize,
    pub(super) strata: Vec<String>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Candidate {
    pub(super) id: String,
    pub(super) surface_operator: String,
    pub(super) shell_proxy: String,
    pub(super) eigensolver: String,
    pub(super) calibration_scale: String,
    pub(super) holdout_mapping: String,
    pub(super) surface_velocity: String,
    pub(super) bempp: String,
    pub(super) cooker: String,
    pub(super) audio_inputs: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Control {
    pub(super) id: String,
    pub(super) definition: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct ModeAdmission {
    pub(super) minimum_admitted_modes: usize,
    pub(super) maximum_relative_eigen_residual: f64,
    pub(super) maximum_gmres_residual: f64,
    pub(super) minimum_reference_magnitude_to_mode_peak: f64,
    pub(super) calibration_maximum_median_frequency_error_octaves: f64,
    pub(super) calibration_maximum_p90_frequency_error_octaves: f64,
    pub(super) holdout_maximum_median_frequency_error_octaves: f64,
    pub(super) holdout_maximum_p90_frequency_error_octaves: f64,
    pub(super) per_mode_failure: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct ConditionGate {
    pub(super) require_every_held_stratum: bool,
    pub(super) maximum_median_abs_error_db: f64,
    pub(super) maximum_p90_abs_error_db: f64,
    pub(super) maximum_persistent_median_abs_error_db: f64,
    pub(super) minimum_improved_component_fraction_vs_constant: f64,
    pub(super) maximum_median_error_ratio_to_constant: f64,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct ComparisonGate {
    pub(super) calibration_maximum_candidate_to_rbf_median_error_ratio: f64,
    pub(super) calibration_maximum_p90_regression_db: f64,
    pub(super) calibration_minimum_improved_component_fraction_vs_rbf: f64,
    pub(super) holdout_maximum_candidate_to_rbf_median_error_ratio: f64,
    pub(super) holdout_maximum_p90_regression_db: f64,
    pub(super) holdout_minimum_improved_component_fraction_vs_rbf: f64,
    pub(super) require_byte_identical_repeat_reports: bool,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Fallback {
    pub(super) mode: String,
    pub(super) object: String,
    pub(super) no_silent_generalization: bool,
}
