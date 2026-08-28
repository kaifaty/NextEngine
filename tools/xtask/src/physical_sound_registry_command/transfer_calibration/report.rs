use serde::Serialize;

use super::{ExpectedRow, TransferAnalysis};

#[derive(Serialize)]
pub(super) struct ArtifactAccessReport {
    profile_row_id: &'static str,
    partition: &'static str,
    inventory_report_sha256: &'static str,
    transfer_sha256: &'static str,
    inventory_opened: bool,
    transfer_opened: bool,
}

impl ArtifactAccessReport {
    pub(super) const fn opened(row: ExpectedRow) -> Self {
        Self {
            profile_row_id: row.profile_row_id,
            partition: row.partition.as_str(),
            inventory_report_sha256: row.inventory_report_sha256,
            transfer_sha256: row.audio_sha256,
            inventory_opened: true,
            transfer_opened: true,
        }
    }

    pub(super) const fn sealed(row: ExpectedRow) -> Self {
        Self {
            profile_row_id: row.profile_row_id,
            partition: row.partition.as_str(),
            inventory_report_sha256: row.inventory_report_sha256,
            transfer_sha256: row.audio_sha256,
            inventory_opened: false,
            transfer_opened: false,
        }
    }
}

#[derive(Serialize)]
pub(super) struct CalibrationReport {
    pub(super) schema: &'static str,
    pub(super) status: &'static str,
    pub(super) decision: &'static str,
    pub(super) claim: &'static str,
    pub(super) study_id: &'static str,
    pub(super) revision: &'static str,
    pub(super) source_profile_id: &'static str,
    pub(super) manifest_sha256: String,
    pub(super) source_feasibility_report_sha256: String,
    pub(super) calibration_profile_sha256: String,
    pub(super) objective: &'static str,
    pub(super) absolute_amplitude_used: bool,
    pub(super) selection_policy: &'static str,
    pub(super) candidate_selection: Vec<CandidateSelectionReport>,
    pub(super) selected_profile_id: &'static str,
    pub(super) selected_rows: Vec<RowAnalysisReport>,
    pub(super) holdout_evaluation: RowAnalysisReport,
    pub(super) holdout_gate: HoldoutGateReport,
    pub(super) spatial_participation: SpatialParticipationReport,
    pub(super) artifact_access: Vec<ArtifactAccessReport>,
    pub(super) unopened_reserved_profile_row_id: Option<&'static str>,
    pub(super) allowed_capabilities: Vec<&'static str>,
    pub(super) prohibited_claims: Vec<&'static str>,
    pub(super) next_action: &'static str,
}

#[derive(Serialize)]
pub(super) struct CandidateSelectionReport {
    pub(super) profile_id: &'static str,
    pub(super) dev_mean_calibration_loss: f64,
    pub(super) calibration_loss: f64,
    pub(super) dev_sanity_pass: bool,
}

#[derive(Serialize)]
pub(super) struct RowAnalysisReport {
    pub(super) profile_row_id: &'static str,
    pub(super) partition: &'static str,
    pub(super) object_id: &'static str,
    pub(super) impact_position_id: &'static str,
    pub(super) listener_condition_id: &'static str,
    pub(super) transfer_sha256: &'static str,
    pub(super) analysis: TransferAnalysis,
}

#[derive(Serialize)]
pub(super) struct HoldoutGateReport {
    pub(super) passed: bool,
    pub(super) checks: Vec<GateCheck>,
}

#[derive(Serialize)]
pub(super) struct GateCheck {
    metric: &'static str,
    observed: f64,
    relation: &'static str,
    threshold: f64,
    pub(super) passed: bool,
}

impl GateCheck {
    pub(super) fn minimum_usize(metric: &'static str, observed: usize, threshold: usize) -> Self {
        Self::minimum_f64(metric, observed as f64, threshold as f64)
    }

    pub(super) fn minimum_f64(metric: &'static str, observed: f64, threshold: f64) -> Self {
        Self {
            metric,
            observed,
            relation: ">=",
            threshold,
            passed: observed >= threshold,
        }
    }

    pub(super) fn maximum_f64(metric: &'static str, observed: f64, threshold: f64) -> Self {
        Self {
            metric,
            observed,
            relation: "<=",
            threshold,
            passed: observed <= threshold,
        }
    }
}

#[derive(Serialize)]
pub(super) struct SpatialParticipationReport {
    pub(super) decision: &'static str,
    pub(super) observed_rows_per_object: usize,
    pub(super) required_rows_per_object: usize,
    pub(super) claim: &'static str,
}
