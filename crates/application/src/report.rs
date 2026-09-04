use serde::{Deserialize, Serialize};

use next_contracts::ids::ContentHash;
use next_contracts::session::CompositionRootV1;

use crate::{ApplicationCloseOutcomeV2, ApplicationRunOutcomeV1};

pub const OPERATIONAL_REPORT_SCHEMA_VERSION: u32 = 1;

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct PresentationReportV1 {
    pub target: String,
    pub snapshot_hash: String,
    pub object_count: u64,
}

/// Plan `continuum-water/28` (ADR-106): the presentation fluid lane of a
/// run — presence, the probe's fallback reason and bounded statistics.
/// Strings and integers only; the vendor stays in the game root.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct PresentationFluidReportV1 {
    pub lane: String,
    pub active: bool,
    pub fallback_reason: Option<String>,
    pub frames: u64,
    pub peak_particles: u64,
    pub emitted: u64,
    pub absorbed: u64,
    pub last_particles: u64,
    pub cost_mean_us: u64,
    pub cost_max_us: u64,
    pub analysis_mean_us: u64,
    pub inside_colliders_max: u64,
    pub spray_fraction_max_permille: u32,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct RunReportV1 {
    pub schema_version: u32,
    pub status: String,
    pub composition_root: String,
    pub session_id: String,
    pub close_receipt_hash: String,
    pub close_result: String,
    pub final_save_generation_hash: Option<String>,
    pub project_composition_lock_hash: String,
    pub ticks: u64,
    pub events: u64,
    pub rpg_events: u64,
    pub authoritative_revision: u64,
    pub authoritative_state_root: String,
    pub command_archive_root: String,
    pub command_identity_index_root: String,
    pub command_ledger_hash: String,
    pub interactive_host_object_count: u64,
    pub presentation: Option<PresentationReportV1>,
    /// Plan 28: absent when no presentation fluid lane was requested.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub presentation_fluid: Option<PresentationFluidReportV1>,
}

impl RunReportV1 {
    pub fn new(
        root: CompositionRootV1,
        run: &ApplicationRunOutcomeV1,
        close: &ApplicationCloseOutcomeV2,
        interactive_host_object_count: u64,
    ) -> Option<Self> {
        let ApplicationCloseOutcomeV2::Closed {
            receipt_hash,
            save_generation_hash,
        } = close;
        let presentation =
            run.presentation_snapshot
                .as_ref()
                .map(|snapshot| PresentationReportV1 {
                    target: "Interactive".to_owned(),
                    snapshot_hash: snapshot.canonical_hash.to_hex(),
                    object_count: u64::try_from(snapshot.scene_records().count())
                        .unwrap_or(u64::MAX),
                });
        Some(Self {
            schema_version: OPERATIONAL_REPORT_SCHEMA_VERSION,
            status: "PASS".to_owned(),
            composition_root: composition_root_token(root).to_owned(),
            session_id: run.session_id.to_hex(),
            close_receipt_hash: receipt_hash.to_hex(),
            close_result: "Saved".to_owned(),
            final_save_generation_hash: Some(save_generation_hash.to_hex()),
            project_composition_lock_hash: run.project_composition_lock_hash.to_hex(),
            ticks: run.ticks,
            events: run.events,
            rpg_events: run.rpg_events,
            authoritative_revision: run.authoritative_revision,
            authoritative_state_root: run.authoritative_state_root.to_hex(),
            command_archive_root: run.command_archive_root.to_hex(),
            command_identity_index_root: run.command_identity_index_root.to_hex(),
            command_ledger_hash: run.command_ledger_hash.to_hex(),
            interactive_host_object_count,
            presentation,
            presentation_fluid: None,
        })
    }

    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum DiagnosticContextV1 {
    Message { message: String },
    Argument { argument: String },
    ExpectedActualHash { expected: String, actual: String },
}

impl DiagnosticContextV1 {
    #[must_use]
    pub fn message(value: impl AsRef<str>) -> Self {
        Self::Message {
            message: bounded(value.as_ref(), 512),
        }
    }

    #[must_use]
    pub fn argument(value: impl AsRef<str>) -> Self {
        Self::Argument {
            argument: bounded(value.as_ref(), 128),
        }
    }

    #[must_use]
    pub fn expected_actual_hash(expected: ContentHash, actual: ContentHash) -> Self {
        Self::ExpectedActualHash {
            expected: expected.to_hex(),
            actual: actual.to_hex(),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct DiagnosticReportV1 {
    pub schema_version: u32,
    pub status: String,
    pub code: String,
    pub context: DiagnosticContextV1,
}

impl DiagnosticReportV1 {
    #[must_use]
    pub fn new(code: impl AsRef<str>, context: DiagnosticContextV1) -> Self {
        let code = code.as_ref();
        let code = if code.is_empty()
            || code.len() > 96
            || !code
                .bytes()
                .all(|byte| byte.is_ascii_uppercase() || byte.is_ascii_digit() || byte == b'_')
        {
            "DIAGNOSTIC_CODE_INVALID".to_owned()
        } else {
            code.to_owned()
        };
        Self {
            schema_version: OPERATIONAL_REPORT_SCHEMA_VERSION,
            status: "ERROR".to_owned(),
            code,
            context,
        }
    }

    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }
}

const fn composition_root_token(root: CompositionRootV1) -> &'static str {
    match root {
        CompositionRootV1::Game => "Game",
        CompositionRootV1::Headless => "Headless",
        CompositionRootV1::Tools => "Tools",
        CompositionRootV1::CaptureWorker => "CaptureWorker",
    }
}

fn bounded(value: &str, maximum_chars: usize) -> String {
    value.chars().take(maximum_chars).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn diagnostic_report_is_typed_bounded_and_round_trips() {
        let report = DiagnosticReportV1::new(
            "CLI_ARGUMENT_INVALID",
            DiagnosticContextV1::message("x".repeat(900)),
        );
        let json = report.to_json().expect("JSON");
        let decoded: DiagnosticReportV1 = serde_json::from_str(&json).expect("decode");
        assert_eq!(decoded, report);
        let DiagnosticContextV1::Message { message } = decoded.context else {
            panic!("message context");
        };
        assert_eq!(message.len(), 512);
    }
}

#[cfg(test)]
mod presentation_fluid_tests {
    use super::*;

    fn report() -> RunReportV1 {
        RunReportV1 {
            schema_version: 1,
            status: "PASS".to_owned(),
            composition_root: "Game".to_owned(),
            session_id: "00".to_owned(),
            close_receipt_hash: "00".to_owned(),
            close_result: "Saved".to_owned(),
            final_save_generation_hash: None,
            project_composition_lock_hash: "00".to_owned(),
            ticks: 1,
            events: 0,
            rpg_events: 0,
            authoritative_revision: 1,
            authoritative_state_root: "00".to_owned(),
            command_archive_root: "00".to_owned(),
            command_identity_index_root: "00".to_owned(),
            command_ledger_hash: "00".to_owned(),
            interactive_host_object_count: 0,
            presentation: None,
            presentation_fluid: None,
        }
    }

    #[test]
    fn presentation_fluid_round_trips_and_is_absent_when_not_requested() {
        let plain = report();
        let json = plain.to_json().expect("serialises");
        assert!(!json.contains("presentation_fluid"));
        let back: RunReportV1 = serde_json::from_str(&json).expect("a report without the field");
        assert_eq!(back, plain);
        let mut with_lane = report();
        with_lane.presentation_fluid = Some(PresentationFluidReportV1 {
            lane: "physx-pbd".to_owned(),
            active: false,
            fallback_reason: Some("PhysX GPU library not found".to_owned()),
            frames: 0,
            peak_particles: 0,
            emitted: 0,
            absorbed: 0,
            last_particles: 0,
            cost_mean_us: 0,
            cost_max_us: 0,
            analysis_mean_us: 0,
            inside_colliders_max: 0,
            spray_fraction_max_permille: 0,
        });
        let json = with_lane.to_json().expect("serialises");
        assert!(json.contains("\"presentation_fluid\":{\"lane\":\"physx-pbd\",\"active\":false"));
        let back: RunReportV1 = serde_json::from_str(&json).expect("deserialises");
        assert_eq!(back, with_lane);
    }
}
