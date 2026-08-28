use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use super::super::super::EntrySpec;
use super::super::super::profiles::FrozenProfile;

pub(super) const SCHEMA: &str =
    "nextengine.experimental-realimpact-shape-spatial-preregistration.manifest.v1";
pub(super) const STUDY_ID: &str = "physical-sound-realimpact-shape-conditioned-vertical";
pub(super) const DEVELOPMENT_REVISION: &str = "v1-development-preregistration";
pub(super) const DEVELOPMENT_MANIFEST_SHA256: &str =
    "4e58eded790a2d00a667fd659827085f452e5a34094af62b7422da792a420b3d";
pub(super) const DEVELOPMENT_REPORT_SHA256: &str =
    "3d18358b50442ee9b171bbacc154f7713e3240efa6f4d50f61f2598b7ac4962f";
pub(super) const CALIBRATION_REVISION: &str = "v1-calibration";
pub(super) const CALIBRATION_MANIFEST_SHA256: &str =
    "2f8ea9b3de11fe5353fcddac2370d1889a9841f9881d83adc63992dbefb1e578";
pub(super) const DEVELOPMENT_OBJECTS: [&str; 10] = [
    "49_PlasticBowl",
    "67_IronPlate",
    "31_WoodSlab",
    "83_WoodVase",
    "81_WoodPad",
    "34_WoodMug",
    "48_PlasticBowl",
    "90_MetalLadle",
    "9_BowlCeramic",
    "68_WoodBoard",
];
pub(super) const CALIBRATION_OBJECTS: [&str; 2] = ["10_bowl", "36_SmallMeasuringCup"];
pub(super) const HOLDOUT_OBJECTS: [&str; 2] = ["65_PitcherCeramic", "63_SmallPlanterCeramic"];
pub(super) const SIGMA_GRID: [f64; 6] = [0.18, 0.26, 0.36, 0.52, 0.74, 1.05];
pub(super) const CONTROL_SIGMA: f64 = 0.52;
pub(super) const MODEL_RIDGE: f64 = 0.25;

#[derive(Deserialize)]
pub(super) struct Manifest {
    pub(super) schema: String,
    pub(super) study_id: String,
    pub(super) revision: String,
    pub(super) phase: String,
    pub(super) objective: String,
    pub(super) official_roster: serde_json::Value,
    pub(super) roles: Roles,
    pub(super) archive_profiles: Vec<ArchiveProfile>,
    pub(super) access_state_at_freeze: String,
    pub(super) compressed_audio_prefix_bytes: u64,
    pub(super) condition: Condition,
    pub(super) mode_extractor: ModeExtractor,
    pub(super) listener_split: ListenerSplit,
    pub(super) control: Control,
    pub(super) shape_model: ShapeModel,
    pub(super) condition_gate: ConditionGate,
    pub(super) calibration_rule: ComparisonRule,
    pub(super) holdout_rule: ComparisonRule,
    pub(super) allowed_claims: Vec<String>,
    pub(super) prohibited_claims: Vec<String>,
    #[serde(default)]
    pub(super) development_report: Option<FileRef>,
    #[serde(default)]
    pub(super) calibration_report: Option<FileRef>,
    #[serde(default)]
    pub(super) source_reports: Vec<FileRef>,
    #[serde(default)]
    pub(super) development_blocks: Vec<BlockRef>,
}

#[derive(Deserialize)]
pub(super) struct Roles {
    pub(super) development: Vec<String>,
    pub(super) calibration: Vec<String>,
    pub(super) holdout: Vec<String>,
}

#[derive(Deserialize)]
pub(super) struct FileRef {
    pub(super) path: PathBuf,
    pub(super) sha256: String,
}

#[derive(Deserialize)]
pub(super) struct BlockRef {
    pub(super) object_id: String,
    pub(super) path: PathBuf,
    pub(super) sha256: String,
    pub(super) sample_count: usize,
}

#[derive(Clone, Deserialize)]
pub(super) struct ArchiveProfile {
    pub(super) dataset_object_id: String,
    pub(super) role: String,
    pub(super) selection_sha256: String,
    pub(super) archive_url: String,
    pub(super) archive_bytes: u64,
    pub(super) archive_etag: String,
    pub(super) archive_last_modified_http: String,
    pub(super) central_offset: u64,
    pub(super) central_bytes: usize,
    pub(super) central_sha256: String,
    pub(super) entry_count: usize,
    pub(super) audio_entry: ArchiveEntry,
    pub(super) metadata_entries: Vec<ArchiveEntry>,
    pub(super) expected_impact_vertex_id: usize,
    pub(super) expected_impact_position_m: Vec<f64>,
    pub(super) expected_listener_position_m: Vec<f64>,
    pub(super) mesh_vertex_count: usize,
    pub(super) mesh_descriptor: MeshDescriptor,
}

#[derive(Clone, Deserialize)]
pub(super) struct ArchiveEntry {
    pub(super) name: String,
    pub(super) flags: u16,
    pub(super) method: u16,
    pub(super) crc32: String,
    pub(super) compressed_bytes: u64,
    pub(super) uncompressed_bytes: u64,
    pub(super) local_offset: u64,
    pub(super) data_offset: u64,
    #[serde(default)]
    pub(super) raw_sha256: String,
    #[serde(default)]
    pub(super) sample_count: usize,
}

#[derive(Clone, Deserialize, Serialize)]
pub(super) struct MeshDescriptor {
    pub(super) bbox_min_m: Vec<f64>,
    pub(super) bbox_max_m: Vec<f64>,
    pub(super) bbox_extents_m: Vec<f64>,
    pub(super) bbox_diagonal_m: f64,
    pub(super) minor_to_major_extent_ratio: f64,
    pub(super) middle_to_major_extent_ratio: f64,
    pub(super) impact_bbox_unit: Vec<f64>,
    pub(super) impact_radius_over_diagonal: f64,
}

#[derive(Deserialize)]
pub(super) struct Condition {
    pub(super) id: String,
    pub(super) row_start: usize,
    pub(super) row_count: usize,
    pub(super) azimuth_degrees: u32,
    pub(super) distance_offset_millimetres: u32,
}

#[derive(Deserialize)]
pub(super) struct ModeExtractor {
    pub(super) id: String,
    pub(super) sample_rate_hz: u32,
    pub(super) reference_listener: usize,
    pub(super) persistent_definition: String,
}

#[derive(Deserialize)]
pub(super) struct ListenerSplit {
    pub(super) anchor_listeners: Vec<usize>,
    pub(super) held_listeners: Vec<usize>,
    pub(super) normalization_listener: usize,
}

#[derive(Deserialize)]
pub(super) struct Control {
    pub(super) id: String,
    pub(super) sigma_metres: f64,
    pub(super) ridge: f64,
}

#[derive(Deserialize)]
pub(super) struct ShapeModel {
    pub(super) id: String,
    pub(super) development_target: String,
    pub(super) sigma_grid_metres: Vec<f64>,
    pub(super) features: Vec<String>,
    #[serde(default)]
    pub(super) speed_of_sound_metres_per_second: Option<f64>,
    pub(super) standardization: String,
    pub(super) regression: String,
    pub(super) prediction: String,
    pub(super) forbidden_inputs: Vec<String>,
}

#[derive(Clone, Deserialize, Serialize)]
pub(super) struct ConditionGate {
    pub(super) minimum_component_count: usize,
    pub(super) maximum_median_abs_error_db: f64,
    pub(super) maximum_p90_abs_error_db: f64,
    pub(super) maximum_persistent_median_abs_error_db: f64,
    pub(super) minimum_improved_component_fraction_vs_constant: f64,
    pub(super) maximum_median_error_ratio_to_constant: f64,
}

#[derive(Clone, Deserialize, Serialize)]
pub(super) struct ComparisonRule {
    pub(super) require_every_shape_candidate_condition_gate: bool,
    pub(super) maximum_median_object_candidate_to_control_median_error_ratio: f64,
    pub(super) maximum_each_object_candidate_to_control_median_error_ratio: f64,
    pub(super) maximum_each_object_p90_regression_db: f64,
    pub(super) minimum_aggregate_improved_component_fraction_vs_control: f64,
    #[serde(default)]
    pub(super) on_failure: Option<String>,
    #[serde(default)]
    pub(super) require_every_holdout_object: Option<bool>,
}

impl Manifest {
    pub(super) fn validate_development(&self) -> Result<(), String> {
        let development = self
            .roles
            .development
            .iter()
            .map(String::as_str)
            .collect::<Vec<_>>();
        let calibration = self
            .roles
            .calibration
            .iter()
            .map(String::as_str)
            .collect::<Vec<_>>();
        let holdout = self
            .roles
            .holdout
            .iter()
            .map(String::as_str)
            .collect::<Vec<_>>();
        if self.schema != SCHEMA
            || self.study_id != STUDY_ID
            || self.revision != DEVELOPMENT_REVISION
            || self.phase != "development"
            || self
                .official_roster
                .get("repository_commit")
                .and_then(serde_json::Value::as_str)
                != Some("fca2bd6cbb7e9f96ac61328d2a0d51594bf01987")
            || self.access_state_at_freeze
                != "central directories, local headers and non-audio metadata opened; no selected deconvolved audio payload byte opened"
            || development != DEVELOPMENT_OBJECTS
            || calibration != CALIBRATION_OBJECTS
            || holdout != HOLDOUT_OBJECTS
            || self.archive_profiles.len() != 14
            || self.compressed_audio_prefix_bytes != 16 * 1024 * 1024
            || self.condition.id != "angle000-distance000"
            || self.condition.row_start != 0
            || self.condition.row_count != 15
            || self.condition.azimuth_degrees != 0
            || self.condition.distance_offset_millimetres != 0
            || self.mode_extractor.id != "injective-modal-16-fft65536-v2"
            || self.mode_extractor.sample_rate_hz != 48_000
            || self.mode_extractor.reference_listener != 7
            || self.mode_extractor.persistent_definition != "matched_tail_frequency_hz is present"
            || self.listener_split.anchor_listeners != [0, 2, 4, 6, 7, 8, 10, 12, 14]
            || self.listener_split.held_listeners != [1, 3, 5, 9, 11, 13]
            || self.listener_split.normalization_listener != 7
            || self.control.id != "coordinate-only-rbf-sigma052-ridge001-v1"
            || self.control.sigma_metres != CONTROL_SIGMA
            || self.control.ridge != 0.001
            || self.shape_model.id != "mesh-bbox-conditioned-object-rbf-bandwidth-v1"
            || self.shape_model.development_target
                != "per-object log sigma minimizing frozen spatial calibration_loss over the declared sigma grid; ties choose lower sigma"
            || self.shape_model.sigma_grid_metres != SIGMA_GRID
            || self.shape_model.speed_of_sound_metres_per_second.is_some()
            || self.shape_model.features
                != [
                    "intercept",
                    "ln_bbox_diagonal_m",
                    "ln_minor_to_major_extent_ratio",
                    "ln_middle_to_major_extent_ratio",
                    "impact_radius_over_diagonal",
                ]
            || self.shape_model.standardization
                != "development-object mean and population standard deviation; intercept unchanged; reject zero-variance features"
            || self.shape_model.regression
                != "ridge least squares on log sigma; lambda 0.25 on non-intercept coefficients; every development object has equal weight"
            || self.shape_model.prediction != "exp(linear output), clamped to [0.18, 1.05] metres"
            || self.shape_model.forbidden_inputs
                != [
                    "object id",
                    "object name tokens",
                    "material label",
                    "calibration audio",
                    "holdout audio",
                    "held-listener samples at inference",
                ]
            || self.development_report.is_some()
            || self.calibration_report.is_some()
            || !self.source_reports.is_empty()
            || !self.development_blocks.is_empty()
        {
            return Err(
                "shape-spatial development manifest does not match the frozen protocol".to_owned(),
            );
        }
        for (index, profile) in self.archive_profiles.iter().enumerate() {
            let (expected_role, expected_id) = if index < DEVELOPMENT_OBJECTS.len() {
                ("development", DEVELOPMENT_OBJECTS[index])
            } else if index < DEVELOPMENT_OBJECTS.len() + CALIBRATION_OBJECTS.len() {
                (
                    "calibration",
                    CALIBRATION_OBJECTS[index - DEVELOPMENT_OBJECTS.len()],
                )
            } else {
                (
                    "holdout",
                    HOLDOUT_OBJECTS[index - DEVELOPMENT_OBJECTS.len() - CALIBRATION_OBJECTS.len()],
                )
            };
            if profile.role != expected_role || profile.dataset_object_id != expected_id {
                return Err("shape-spatial archive profile order or role changed".to_owned());
            }
            profile.validate()?;
        }
        Ok(())
    }

    pub(super) fn profiles_for<'a>(
        &'a self,
        role: &'a str,
    ) -> impl Iterator<Item = &'a ArchiveProfile> {
        self.archive_profiles
            .iter()
            .filter(move |profile| profile.role == role)
    }

    pub(super) fn validate_calibration(&self) -> Result<(), String> {
        let development = self
            .development_report
            .as_ref()
            .ok_or_else(|| "shape-spatial calibration has no development report".to_owned())?;
        if self.schema != SCHEMA
            || self.study_id != STUDY_ID
            || self.revision != CALIBRATION_REVISION
            || self.phase != "calibration"
            || self.access_state_at_freeze
                != "development audio opened under the frozen report; calibration and holdout deconvolved audio payloads remain unopened"
            || development.sha256 != DEVELOPMENT_REPORT_SHA256
            || development.path.as_path() != Path::new("development-a/report.json")
            || self.calibration_report.is_some()
            || self.compressed_audio_prefix_bytes != 16 * 1024 * 1024
            || self.control.sigma_metres != CONTROL_SIGMA
            || self.shape_model.sigma_grid_metres != SIGMA_GRID
        {
            return Err(
                "shape-spatial calibration manifest does not match the frozen protocol".to_owned(),
            );
        }
        self.validate_roles_and_profiles()
    }

    fn validate_roles_and_profiles(&self) -> Result<(), String> {
        if self.roles.development != DEVELOPMENT_OBJECTS
            || self.roles.calibration != CALIBRATION_OBJECTS
            || self.roles.holdout != HOLDOUT_OBJECTS
            || self.archive_profiles.len() != 14
        {
            return Err("shape-spatial roles changed".to_owned());
        }
        for (index, profile) in self.archive_profiles.iter().enumerate() {
            let (expected_role, expected_id) = if index < DEVELOPMENT_OBJECTS.len() {
                ("development", DEVELOPMENT_OBJECTS[index])
            } else if index < DEVELOPMENT_OBJECTS.len() + CALIBRATION_OBJECTS.len() {
                (
                    "calibration",
                    CALIBRATION_OBJECTS[index - DEVELOPMENT_OBJECTS.len()],
                )
            } else {
                (
                    "holdout",
                    HOLDOUT_OBJECTS[index - DEVELOPMENT_OBJECTS.len() - CALIBRATION_OBJECTS.len()],
                )
            };
            if profile.role != expected_role || profile.dataset_object_id != expected_id {
                return Err("shape-spatial archive profile order or role changed".to_owned());
            }
            profile.validate()?;
        }
        Ok(())
    }
}

impl ArchiveProfile {
    pub(super) fn validate(&self) -> Result<(), String> {
        let finite_positive = [
            self.mesh_descriptor.bbox_diagonal_m,
            self.mesh_descriptor.minor_to_major_extent_ratio,
            self.mesh_descriptor.middle_to_major_extent_ratio,
        ]
        .into_iter()
        .all(|value| value.is_finite() && value > 0.0);
        if !self
            .archive_url
            .starts_with("https://downloads.cs.stanford.edu/viscam/RealImpact/")
            || self.selection_sha256.len() != 64
            || self.archive_bytes == 0
            || self.central_bytes == 0
            || self.entry_count != 12
            || self.metadata_entries.len() != 7
            || self.audio_entry.flags != 0
            || self.audio_entry.method != 8
            || self.audio_entry.sample_count < 131_072
            || self.expected_impact_position_m.len() != 3
            || self.expected_listener_position_m.len() != 3
            || self.mesh_descriptor.bbox_extents_m.len() != 3
            || self.mesh_descriptor.impact_bbox_unit.len() != 3
            || !finite_positive
        {
            return Err(format!(
                "shape-spatial archive profile is invalid: {}",
                self.dataset_object_id
            ));
        }
        if self
            .metadata_entries
            .iter()
            .any(|entry| entry.flags != 0 || entry.method != 8 || entry.raw_sha256.len() != 64)
        {
            return Err(format!(
                "shape-spatial metadata entry is invalid: {}",
                self.dataset_object_id
            ));
        }
        Ok(())
    }

    pub(super) fn frozen(&self, prefix_bytes: u64) -> Result<&'static FrozenProfile, String> {
        let mut entries = self
            .metadata_entries
            .iter()
            .map(freeze_entry)
            .collect::<Result<Vec<_>, String>>()?;
        entries.push(freeze_entry(&self.audio_entry)?);
        let entries = Box::leak(entries.into_boxed_slice());
        let impact: [f64; 3] = self
            .expected_impact_position_m
            .clone()
            .try_into()
            .map_err(|_| "impact position must have three coordinates".to_owned())?;
        let listener: [f64; 3] = self
            .expected_listener_position_m
            .clone()
            .try_into()
            .map_err(|_| "listener position must have three coordinates".to_owned())?;
        let profile = FrozenProfile {
            id: leak(format!("shape-spatial-{}-v1", self.dataset_object_id)),
            archive_url: leak(self.archive_url.clone()),
            archive_bytes: self.archive_bytes,
            archive_etag: leak(self.archive_etag.clone()),
            archive_last_modified_http: leak(self.archive_last_modified_http.clone()),
            archive_last_modified_iso: "",
            central_offset: self.central_offset,
            central_bytes: self.central_bytes,
            central_sha256: leak(self.central_sha256.clone()),
            entry_count: self.entry_count,
            audio_prefix_bytes: usize::try_from(prefix_bytes)
                .map_err(|_| "audio prefix is too large".to_owned())?,
            audio_prefix_sha256: "",
            entries,
            dataset_object_id: leak(self.dataset_object_id.clone()),
            material_family: "unspecified",
            audio_entry_name: leak(self.audio_entry.name.clone()),
            audio_crc32: leak(self.audio_entry.crc32.clone()),
            audio_sample_count: self.audio_entry.sample_count,
            audio_row_sha256: "",
            expected_impact_vertex_id: self.expected_impact_vertex_id,
            expected_impact_position: impact,
            expected_listener_position: listener,
            expected_mesh_vertex_count: self.mesh_vertex_count,
            retrieved_at: "2026-08-28",
            inventory_id: "ps2-realimpact-shape-spatial-v1",
            inventory_entry_id: leak(format!("{}-base-listener-block", self.dataset_object_id)),
            domain_id: "realimpact-shape-spatial-base-line",
            object_family_id: "realimpact-shape-conditioned-object",
            object_id: leak(format!("realimpact-{}", self.dataset_object_id)),
            geometry_revision: leak(format!("mesh-{}", &self.central_sha256[..12])),
            impact_position_id: leak(format!("mesh-vertex-{}", self.expected_impact_vertex_id)),
            row_file_name: "shape-spatial-base-block.f32le",
            audition_file_name: "shape-spatial-not-published.wav",
        };
        Ok(Box::leak(Box::new(profile)))
    }
}

fn freeze_entry(entry: &ArchiveEntry) -> Result<EntrySpec, String> {
    let crc32 = u32::from_str_radix(&entry.crc32, 16)
        .map_err(|_| format!("invalid CRC32 for {}", entry.name))?;
    Ok(EntrySpec::new(
        leak(entry.name.clone()),
        entry.local_offset,
        entry.data_offset,
        entry.compressed_bytes,
        entry.uncompressed_bytes,
        crc32,
        leak(entry.raw_sha256.clone()),
    ))
}

fn leak(value: String) -> &'static str {
    Box::leak(value.into_boxed_str())
}
