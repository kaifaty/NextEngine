use std::path::Path;

use super::FEM_MANIFEST_SCHEMA;
use super::model::AngularFamily;
use super::schema::{Manifest, SourceManifest, SourceReport};

pub(super) fn validate_manifest(manifest: &Manifest) -> Result<(), String> {
    let expected_directions = (0..56).collect::<Vec<_>>();
    let expected_candidates = [
        (
            "axisymmetric-mzero-outgoing-order4-ridge1e12-v1",
            AngularFamily::AxisymmetricMZero,
            4,
        ),
        (
            "full-real-spherical-angular-outgoing-order2-ridge1e12-v1",
            AngularFamily::FullRealSphericalHarmonics,
            2,
        ),
        (
            "full-real-spherical-angular-outgoing-order4-ridge1e12-v1",
            AngularFamily::FullRealSphericalHarmonics,
            4,
        ),
        (
            "full-real-spherical-angular-outgoing-order6-ridge1e12-v1",
            AngularFamily::FullRealSphericalHarmonics,
            6,
        ),
    ];
    let candidates_match = manifest.candidates.len() == expected_candidates.len()
        && manifest
            .candidates
            .iter()
            .zip(expected_candidates)
            .all(|(actual, expected)| {
                actual.id == expected.0
                    && actual.angular_family == expected.1
                    && actual.maximum_degree == expected.2
                    && actual.ridge == 1.0e-12
            });
    if manifest.schema != FEM_MANIFEST_SCHEMA
        || manifest.study_id != "physical-sound-full-near-shell-cooker-elastic-fem-mode-control"
        || manifest.protocol_revision != "outgoing-spherical-full-near-shell-fem-mode-candidates-v1"
        || manifest.source_manifest.path != Path::new("refined-run-a/manifest.json")
        || manifest.source_manifest.sha256
            != "1adfe3d68ee5c9d806fe2161311a322cb6df2dd6161acbe340369fc9ee5aabf7"
        || manifest.source_report.path != Path::new("refined-run-a/report.json")
        || manifest.source_report.sha256
            != "a71515fa9edf71663986f1a0c8e37627fff499da5eed5b84aff41a69b23da667"
        || manifest.source_contract.schema
            != "nextengine.experimental-physical-sound-bempp-fem-mode.report.v1"
        || manifest.source_contract.study_id
            != "physical-sound-elastic-fem-mode-to-bempp-triaxial-refined-control"
        || manifest.source_contract.protocol_revision
            != "core-clamped-linear-tetrahedral-fem-to-bempp-refined-v1"
        || manifest.source_contract.manifest_sha256
            != "1adfe3d68ee5c9d806fe2161311a322cb6df2dd6161acbe340369fc9ee5aabf7"
        || manifest.source_contract.decision != "ElasticFemEigenmodeToBemppCouplingSupported"
        || manifest.source_contract.fine_mesh_refinement_level != 4
        || manifest.source_contract.fine_panel_count != 2048
        || manifest.source_contract.fine_condition_count != 168
        || manifest.split.fit_listener_radius_reference_multiplier != 2.0
        || manifest.split.fit_direction_indices != expected_directions
        || !manifest.split.held_near_direction_indices.is_empty()
        || manifest
            .split
            .held_far_listener_radius_reference_multipliers
            != [4.0, 10.0]
        || manifest.split.held_far_direction_indices != (0..56).collect::<Vec<_>>()
        || !candidates_match
        || manifest.shared_model.speed_of_sound_metres_per_second != 343.0
        || manifest.shared_model.radial_basis != "spherical-hankel-first-kind"
        || manifest.shared_model.angular_basis != "associated-legendre-times-real-sine-cosine"
        || manifest.shared_model.coefficient_fit
            != "complex-column-scaled-ridge-normal-equations-partial-pivot"
        || manifest.shared_model.active_field_minimum_peak_ratio != 0.25
        || manifest
            .shared_model
            .field_classification_absolute_tolerance
            != 1.0e-12
        || !manifest.admission_gates.require_every_source_gate_passed
        || !manifest
            .admission_gates
            .require_axisymmetric_control_rejected
        || manifest.admission_gates.selection_rule
            != "smallest-full-angular-maximum-degree-passing-all-held-gates"
        || !manifest.admission_gates.repeat_report_bytes_identical
        || manifest.allowed_claims
            != [
                "synthetic_elastic_fem_mode_full_near_shell_cooker_sufficiency",
                "near_to_far_full_angular_outgoing_multipole_transfer",
            ]
        || manifest.prohibited_claims.len() != 5
        || !manifest.data_policy.generated_synthetic_fixture_only
        || manifest.data_policy.fresh_realimpact_payload_access_allowed
        || manifest.data_policy.network_training_allowed
        || manifest
            .data_policy
            .runtime_or_quality_admission_credit_allowed
    {
        return Err("FEM-mode cooker manifest does not match the frozen protocol".to_owned());
    }
    Ok(())
}

pub(super) fn validate_source(
    manifest: &Manifest,
    source_manifest: &SourceManifest,
    source_report: &SourceReport,
) -> Result<(), String> {
    let fixture = &source_manifest.fixture;
    let contract = &manifest.source_contract;
    let fine = source_report
        .bem_fine
        .as_ref()
        .ok_or_else(|| "FEM/Bempp source has no fine BEM level".to_owned())?;
    let wave_number = source_report
        .common_wave_number_reference_length
        .ok_or_else(|| "FEM/Bempp source has no common wave number".to_owned())?;
    if source_manifest.schema != "nextengine.experimental-physical-sound-bempp-fem-mode.manifest.v1"
        || source_manifest.study_id != contract.study_id
        || source_manifest.protocol_revision != contract.protocol_revision
        || fixture.reference_length_metres != 0.1
        || fixture.speed_of_sound_metres_per_second != 343.0
        || fixture.listener_radius_reference_multipliers != [2.0, 4.0, 10.0]
        || !wave_number.is_finite()
        || wave_number <= 0.0
        || source_report.listener_directions.len() != 56
        || source_report.schema != contract.schema
        || source_report.study_id != contract.study_id
        || source_report.protocol_revision != contract.protocol_revision
        || source_report.manifest_sha256 != contract.manifest_sha256
        || source_report.decision != contract.decision
        || fine.angular_refinement_level != Some(contract.fine_mesh_refinement_level)
        || fine.panel_count != contract.fine_panel_count
        || fine.condition_count != contract.fine_condition_count
        || fine.conditions.len() != contract.fine_condition_count
        || source_report.gate.len() != 16
        || source_report.gate.values().any(|passed| !passed)
    {
        return Err("FEM/Bempp source does not match the frozen contract".to_owned());
    }
    for direction in &source_report.listener_directions {
        let norm = direction
            .iter()
            .map(|value| value * value)
            .sum::<f64>()
            .sqrt();
        if direction.iter().any(|value| !value.is_finite()) || (norm - 1.0).abs() > 1.0e-12 {
            return Err("FEM/Bempp listener direction is invalid".to_owned());
        }
    }
    for condition in &fine.conditions {
        if !condition.is_finite()
            || condition.wave_number_reference_length != wave_number
            || !fixture
                .listener_radius_reference_multipliers
                .contains(&condition.listener_radius_reference_multiplier)
            || condition.direction_index >= source_report.listener_directions.len()
        {
            return Err("FEM/Bempp source condition is invalid".to_owned());
        }
    }
    for radius in &fixture.listener_radius_reference_multipliers {
        for direction_index in 0..source_report.listener_directions.len() {
            let count = fine
                .conditions
                .iter()
                .filter(|condition| {
                    condition.wave_number_reference_length == wave_number
                        && condition.listener_radius_reference_multiplier == *radius
                        && condition.direction_index == direction_index
                })
                .count();
            if count != 1 {
                return Err("FEM/Bempp source condition grid is incomplete".to_owned());
            }
        }
    }
    Ok(())
}
