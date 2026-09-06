use serde::{Deserialize, Serialize};

use super::super::{ArtifactReport, FileRef};

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(super) struct NeuralDataPlaneManifest {
    pub(super) schema: String,
    pub(super) projection_id: String,
    pub(super) revision: String,
    pub(super) task_scope: TaskScope,
    pub(super) split_policy: SplitPolicy,
    pub(super) lineage_reports: Vec<LineageReport>,
    pub(super) rows: Vec<NeuralRow>,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub(super) enum TaskScope {
    ExactObjectFewShotImpactListenerField,
}

impl TaskScope {
    pub(super) const fn as_str(self) -> &'static str {
        match self {
            Self::ExactObjectFewShotImpactListenerField => {
                "exact_object_few_shot_impact_listener_field"
            }
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(super) struct SplitPolicy {
    pub(super) object_groups_disjoint_across_roles: bool,
    pub(super) source_groups_disjoint_across_roles: bool,
    pub(super) recording_parents_disjoint_across_roles: bool,
    pub(super) condition_groups_disjoint_across_roles: bool,
    pub(super) mutation_parents_disjoint_across_roles: bool,
    pub(super) identical_audio_disjoint_across_roles: bool,
    pub(super) method_holdout_sealed_before_candidate_freeze: bool,
    pub(super) admission_shadow_sealed_until_validator_release: bool,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(super) struct LineageReport {
    pub(super) id: String,
    pub(super) expected_schema: String,
    pub(super) expected_claim: String,
    pub(super) artifact: FileRef,
}

#[derive(Deserialize)]
pub(super) struct LineageReportProbe {
    pub(super) schema: String,
    pub(super) status: String,
    pub(super) claim: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(super) struct NeuralRow {
    pub(super) row_id: String,
    pub(super) split_role: SplitRole,
    pub(super) sample_role: SampleRole,
    pub(super) corpus_role: CorpusRole,
    #[serde(default)]
    pub(super) evidence_lane: Option<EvidenceLane>,
    pub(super) audio_semantics: AudioSemantics,
    pub(super) source_group_id: String,
    pub(super) family_group_id: String,
    pub(super) object_group_id: String,
    pub(super) recording_parent_id: String,
    pub(super) condition_group_id: String,
    #[serde(default)]
    pub(super) mutation_parent_id: Option<String>,
    pub(super) lineage_report_ids: Vec<String>,
    pub(super) audio: FileRef,
    pub(super) audio_provenance: FileRef,
    #[serde(default)]
    pub(super) axes: AxisClaims,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "snake_case")]
pub(super) enum EvidenceLane {
    SyntheticTeacher,
    ExactRealTransfer,
    IdentifiedRealRecording,
}

impl EvidenceLane {
    pub(super) const fn as_str(self) -> &'static str {
        match self {
            Self::SyntheticTeacher => "synthetic_teacher",
            Self::ExactRealTransfer => "exact_real_transfer",
            Self::IdentifiedRealRecording => "identified_real_recording",
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(super) enum AudioSemantics {
    RecordedImpactWaveform,
    ForceDeconvolvedTransferResponse,
    SyntheticModalRender,
}

impl AudioSemantics {
    pub(super) const fn as_str(self) -> &'static str {
        match self {
            Self::RecordedImpactWaveform => "recorded_impact_waveform",
            Self::ForceDeconvolvedTransferResponse => "force_deconvolved_transfer_response",
            Self::SyntheticModalRender => "synthetic_modal_render",
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "snake_case")]
pub(super) enum SplitRole {
    Train,
    Development,
    Calibration,
    MethodHoldout,
    AdmissionShadow,
}

impl SplitRole {
    pub(super) const fn as_str(self) -> &'static str {
        match self {
            Self::Train => "train",
            Self::Development => "development",
            Self::Calibration => "calibration",
            Self::MethodHoldout => "method_holdout",
            Self::AdmissionShadow => "admission_shadow",
        }
    }

    pub(super) const fn is_sealed(self) -> bool {
        matches!(self, Self::MethodHoldout | Self::AdmissionShadow)
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(super) enum SampleRole {
    Context,
    Query,
}

impl SampleRole {
    pub(super) const fn as_str(self) -> &'static str {
        match self {
            Self::Context => "context",
            Self::Query => "query",
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(super) enum CorpusRole {
    Target,
    RejectParent,
}

impl CorpusRole {
    pub(super) const fn as_str(self) -> &'static str {
        match self {
            Self::Target => "target",
            Self::RejectParent => "reject_parent",
        }
    }
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(super) struct AxisClaims {
    #[serde(default)]
    pub(super) material: Option<LabelClaim>,
    #[serde(default)]
    pub(super) geometry: Option<GeometryClaim>,
    #[serde(default)]
    pub(super) support: Option<LabelClaim>,
    #[serde(default)]
    pub(super) impact: Option<ImpactClaim>,
    #[serde(default)]
    pub(super) listener: Option<ListenerClaim>,
    #[serde(default)]
    pub(super) excitation: Option<ExcitationClaim>,
    #[serde(default)]
    pub(super) teacher_target: Option<TeacherTargetClaim>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(super) struct LabelClaim {
    pub(super) value_id: String,
    pub(super) evidence: FileRef,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(super) struct GeometryClaim {
    pub(super) geometry_id: String,
    pub(super) feature_artifact: FileRef,
    pub(super) evidence: FileRef,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(super) struct ImpactClaim {
    pub(super) coordinate_profile: String,
    pub(super) point_metres: [f64; 3],
    #[serde(default)]
    pub(super) outward_normal: Option<[f64; 3]>,
    pub(super) evidence: FileRef,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(super) struct ListenerClaim {
    pub(super) coordinate_profile: String,
    pub(super) point_metres: [f64; 3],
    pub(super) evidence: FileRef,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(super) struct ExcitationClaim {
    #[serde(default)]
    pub(super) impulse_newton_seconds: Option<f64>,
    #[serde(default)]
    pub(super) energy_joules: Option<f64>,
    #[serde(default)]
    pub(super) force_profile: Option<FileRef>,
    pub(super) evidence: FileRef,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(super) struct TeacherTargetClaim {
    pub(super) representation_id: String,
    pub(super) mode_count: usize,
    pub(super) modal_parameters: FileRef,
    pub(super) contact_gain_field: FileRef,
    pub(super) evidence: FileRef,
}

#[derive(Clone, Debug, Serialize)]
pub(super) struct VerifiedArtifact {
    pub(super) sha256: String,
    pub(super) byte_count: usize,
}

impl From<ArtifactReport> for VerifiedArtifact {
    fn from(report: ArtifactReport) -> Self {
        Self {
            sha256: report.sha256,
            byte_count: report.byte_count,
        }
    }
}

#[derive(Clone, Debug, Serialize)]
pub(super) struct ProjectedLineageReport {
    pub(super) id: String,
    pub(super) schema: String,
    pub(super) claim: String,
    pub(super) artifact: VerifiedArtifact,
}

#[derive(Clone, Debug, Serialize)]
pub(super) struct ProjectedRow {
    pub(super) row_id: String,
    pub(super) split_role: &'static str,
    pub(super) sample_role: &'static str,
    pub(super) corpus_role: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) evidence_lane: Option<&'static str>,
    pub(super) audio_semantics: &'static str,
    pub(super) source_group_id: String,
    pub(super) family_group_id: String,
    pub(super) object_group_id: String,
    pub(super) recording_parent_id: String,
    pub(super) condition_group_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) mutation_parent_id: Option<String>,
    pub(super) lineage_report_ids: Vec<String>,
    pub(super) audio: VerifiedArtifact,
    pub(super) audio_provenance: VerifiedArtifact,
    pub(super) axes: ProjectedAxes,
}

#[derive(Clone, Debug, Default, Serialize)]
pub(super) struct ProjectedAxes {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) material: Option<ProjectedLabelClaim>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) geometry: Option<ProjectedGeometryClaim>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) support: Option<ProjectedLabelClaim>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) impact: Option<ProjectedImpactClaim>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) listener: Option<ProjectedListenerClaim>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) excitation: Option<ProjectedExcitationClaim>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) teacher_target: Option<ProjectedTeacherTarget>,
}

impl ProjectedAxes {
    pub(super) const fn complete_for_modal_field(&self) -> bool {
        self.material.is_some()
            && self.geometry.is_some()
            && self.support.is_some()
            && self.impact.is_some()
            && self.listener.is_some()
            && self.excitation.is_some()
    }
}

#[derive(Clone, Debug, Serialize)]
pub(super) struct ProjectedLabelClaim {
    pub(super) value_id: String,
    pub(super) evidence: VerifiedArtifact,
}

#[derive(Clone, Debug, Serialize)]
pub(super) struct ProjectedGeometryClaim {
    pub(super) geometry_id: String,
    pub(super) feature_artifact: VerifiedArtifact,
    pub(super) evidence: VerifiedArtifact,
}

#[derive(Clone, Debug, Serialize)]
pub(super) struct ProjectedImpactClaim {
    pub(super) coordinate_profile: String,
    pub(super) point_metres: [f64; 3],
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) outward_normal: Option<[f64; 3]>,
    pub(super) evidence: VerifiedArtifact,
}

#[derive(Clone, Debug, Serialize)]
pub(super) struct ProjectedListenerClaim {
    pub(super) coordinate_profile: String,
    pub(super) point_metres: [f64; 3],
    pub(super) evidence: VerifiedArtifact,
}

#[derive(Clone, Debug, Serialize)]
pub(super) struct ProjectedExcitationClaim {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) impulse_newton_seconds: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) energy_joules: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) force_profile: Option<VerifiedArtifact>,
    pub(super) evidence: VerifiedArtifact,
}

#[derive(Clone, Debug, Serialize)]
pub(super) struct ProjectedTeacherTarget {
    pub(super) representation_id: String,
    pub(super) mode_count: usize,
    pub(super) modal_parameters: VerifiedArtifact,
    pub(super) contact_gain_field: VerifiedArtifact,
    pub(super) evidence: VerifiedArtifact,
}
