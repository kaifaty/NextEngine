use std::collections::BTreeMap;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use super::model::{AngularFamily, BasisColumn, ComplexValue};

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Manifest {
    pub(super) schema: String,
    pub(super) study_id: String,
    pub(super) protocol_revision: String,
    pub(super) source_manifest: ArtifactReference,
    pub(super) source_report: ArtifactReference,
    pub(super) source_contract: SourceContract,
    pub(super) split: Split,
    pub(super) candidates: Vec<Candidate>,
    pub(super) shared_model: SharedModel,
    pub(super) admission_gates: AdmissionGates,
    pub(super) allowed_claims: Vec<String>,
    pub(super) prohibited_claims: Vec<String>,
    pub(super) data_policy: DataPolicy,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct ArtifactReference {
    pub(super) path: PathBuf,
    pub(super) sha256: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct SourceContract {
    pub(super) schema: String,
    pub(super) study_id: String,
    pub(super) protocol_revision: String,
    pub(super) manifest_sha256: String,
    pub(super) decision: String,
    pub(super) fine_mesh_refinement_level: usize,
    pub(super) fine_panel_count: usize,
    pub(super) fine_condition_count: usize,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Split {
    pub(super) fit_listener_radius_reference_multiplier: f64,
    pub(super) fit_direction_indices: Vec<usize>,
    pub(super) held_near_direction_indices: Vec<usize>,
    pub(super) held_far_listener_radius_reference_multipliers: Vec<f64>,
    pub(super) held_far_direction_indices: Vec<usize>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Candidate {
    pub(super) id: String,
    pub(super) angular_family: AngularFamily,
    pub(super) maximum_degree: usize,
    pub(super) ridge: f64,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct SharedModel {
    pub(super) speed_of_sound_metres_per_second: f64,
    pub(super) radial_basis: String,
    pub(super) angular_basis: String,
    pub(super) coefficient_fit: String,
    pub(super) active_field_minimum_peak_ratio: f64,
    pub(super) field_classification_absolute_tolerance: f64,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(super) struct AdmissionGates {
    pub(super) require_every_source_gate_passed: bool,
    pub(super) require_axisymmetric_control_rejected: bool,
    pub(super) selection_rule: String,
    pub(super) held_max_peak_normalized_complex_error: f64,
    pub(super) held_max_active_relative_complex_error: f64,
    pub(super) held_max_active_absolute_magnitude_error_db: f64,
    pub(super) held_max_active_absolute_phase_error_degrees: f64,
    pub(super) held_min_directional_complex_correlation: f64,
    pub(super) repeat_report_bytes_identical: bool,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(super) struct DataPolicy {
    pub(super) generated_synthetic_fixture_only: bool,
    pub(super) fresh_realimpact_payload_access_allowed: bool,
    pub(super) network_training_allowed: bool,
    pub(super) runtime_or_quality_admission_credit_allowed: bool,
}

#[derive(Debug, Deserialize)]
pub(super) struct SourceManifest {
    pub(super) schema: String,
    pub(super) study_id: String,
    pub(super) protocol_revision: String,
    pub(super) fixture: SourceFixture,
}

#[derive(Debug, Deserialize)]
pub(super) struct SourceFixture {
    pub(super) reference_length_metres: f64,
    pub(super) speed_of_sound_metres_per_second: f64,
    #[serde(default)]
    pub(super) id: String,
    #[serde(default)]
    pub(super) semiaxes_metres: Vec<f64>,
    #[serde(default)]
    pub(super) surface_mode: String,
    #[serde(default)]
    pub(super) surface_profile_projection: String,
    #[serde(default)]
    pub(super) wave_number_reference_length_values: Vec<f64>,
    pub(super) listener_radius_reference_multipliers: Vec<f64>,
    #[serde(default)]
    pub(super) listener_directions: Vec<[f64; 3]>,
}

#[derive(Debug, Deserialize)]
pub(super) struct SourceReport {
    pub(super) schema: String,
    pub(super) study_id: String,
    pub(super) protocol_revision: String,
    pub(super) manifest_sha256: String,
    pub(super) decision: String,
    pub(super) gate: BTreeMap<String, bool>,
    #[serde(default)]
    pub(super) common_wave_number_reference_length: Option<f64>,
    #[serde(default)]
    pub(super) listener_directions: Vec<[f64; 3]>,
    #[serde(default)]
    pub(super) fine: Option<SourceLevel>,
    #[serde(default)]
    pub(super) bem_fine: Option<SourceLevel>,
}

#[derive(Debug, Deserialize)]
pub(super) struct SourceLevel {
    #[serde(default)]
    pub(super) mesh_refinement_level: Option<usize>,
    #[serde(default)]
    pub(super) angular_refinement_level: Option<usize>,
    pub(super) panel_count: usize,
    pub(super) condition_count: usize,
    pub(super) conditions: Vec<SourceCondition>,
}

#[derive(Debug, Deserialize)]
pub(super) struct SourceCondition {
    pub(super) wave_number_reference_length: f64,
    pub(super) frequency_hz: f64,
    pub(super) listener_radius_reference_multiplier: f64,
    pub(super) direction_index: usize,
    #[serde(default)]
    pub(super) surface_profile_direction_value: Option<f64>,
    pub(super) computed: ComplexValue,
}

impl SourceCondition {
    pub(super) fn is_finite(&self) -> bool {
        self.wave_number_reference_length.is_finite()
            && self.frequency_hz.is_finite()
            && self.listener_radius_reference_multiplier.is_finite()
            && self
                .surface_profile_direction_value
                .is_none_or(f64::is_finite)
            && self.computed.is_finite()
    }
}

#[derive(Debug, Serialize)]
pub(super) struct Report<'a> {
    pub(super) schema: &'static str,
    pub(super) status: &'static str,
    pub(super) decision: &'static str,
    pub(super) claim: &'static str,
    pub(super) study_id: &'a str,
    pub(super) protocol_revision: &'a str,
    pub(super) manifest_sha256: &'a str,
    pub(super) source_manifest_sha256: &'a str,
    pub(super) source_report_sha256: &'a str,
    pub(super) split: &'a Split,
    pub(super) candidates: Vec<CandidateReport>,
    pub(super) selected_candidate_id: Option<String>,
    pub(super) thresholds: &'a AdmissionGates,
    pub(super) gate: OverallGate,
    pub(super) allowed_claims: &'a [String],
    pub(super) prohibited_claims: &'a [String],
    pub(super) data_policy: &'a DataPolicy,
    pub(super) next_action: &'static str,
}

#[derive(Debug, Serialize)]
pub(super) struct CandidateReport {
    pub(super) id: String,
    pub(super) angular_family: AngularFamily,
    pub(super) maximum_degree: usize,
    pub(super) frequencies: Vec<FrequencyReport>,
    pub(super) aggregate: Aggregate,
    pub(super) gate: CandidateGate,
    pub(super) held_conditions: Vec<HeldCondition>,
}

#[derive(Debug, Serialize)]
pub(super) struct FrequencyReport {
    pub(super) wave_number_reference_length: f64,
    pub(super) frequency_hz: f64,
    pub(super) fit_condition_count: usize,
    pub(super) columns: Vec<BasisColumn>,
    pub(super) coefficients: Vec<ComplexValue>,
    pub(super) coefficient_total_energy: f64,
    pub(super) coefficient_energy_fraction_by_degree: Vec<f64>,
    pub(super) fit_max_peak_normalized_complex_error: f64,
}

#[derive(Debug, Serialize)]
pub(super) struct HeldCondition {
    pub(super) wave_number_reference_length: f64,
    pub(super) frequency_hz: f64,
    pub(super) listener_radius_reference_multiplier: f64,
    pub(super) direction_index: usize,
    pub(super) surface_profile_direction_value: Option<f64>,
    pub(super) active_field: bool,
    pub(super) target: ComplexValue,
    pub(super) predicted: ComplexValue,
    pub(super) group_peak_magnitude: f64,
    pub(super) peak_normalized_complex_error: f64,
    pub(super) active_relative_complex_error: Option<f64>,
    pub(super) active_absolute_magnitude_error_db: Option<f64>,
    pub(super) active_absolute_phase_error_degrees: Option<f64>,
}

#[derive(Debug, Serialize)]
pub(super) struct Aggregate {
    pub(super) frequency_count: usize,
    pub(super) held_condition_count: usize,
    pub(super) held_active_condition_count: usize,
    pub(super) held_median_peak_normalized_complex_error: f64,
    pub(super) held_max_peak_normalized_complex_error: f64,
    pub(super) held_max_active_relative_complex_error: f64,
    pub(super) held_max_active_absolute_magnitude_error_db: f64,
    pub(super) held_max_active_absolute_phase_error_degrees: f64,
    pub(super) held_min_directional_complex_correlation: f64,
    pub(super) directional_correlations: Vec<DirectionalCorrelation>,
}

#[derive(Debug, Serialize)]
pub(super) struct DirectionalCorrelation {
    pub(super) wave_number_reference_length: f64,
    pub(super) listener_radius_reference_multiplier: f64,
    pub(super) direction_count: usize,
    pub(super) complex_correlation: f64,
}

#[derive(Debug, Serialize)]
pub(super) struct CandidateGate {
    pub(super) held_peak_normalized_error_passed: bool,
    pub(super) held_active_relative_error_passed: bool,
    pub(super) held_active_magnitude_error_passed: bool,
    pub(super) held_active_phase_error_passed: bool,
    pub(super) held_directional_correlation_passed: bool,
}

impl CandidateGate {
    pub(super) fn all_passed(&self) -> bool {
        self.held_peak_normalized_error_passed
            && self.held_active_relative_error_passed
            && self.held_active_magnitude_error_passed
            && self.held_active_phase_error_passed
            && self.held_directional_correlation_passed
    }
}

#[derive(Debug, Serialize)]
pub(super) struct OverallGate {
    pub(super) every_source_gate_passed: bool,
    pub(super) axisymmetric_control_rejected: bool,
    pub(super) full_angular_candidate_selected: bool,
}

impl OverallGate {
    pub(super) fn all_passed(&self) -> bool {
        self.every_source_gate_passed
            && self.axisymmetric_control_rejected
            && self.full_angular_candidate_selected
    }
}
