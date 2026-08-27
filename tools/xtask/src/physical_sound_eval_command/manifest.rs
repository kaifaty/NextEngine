use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

use super::{MANIFEST_SCHEMA, MAX_ENTRIES, POSITION_MAXIMUM_NEIGHBOR_DISTANCE_MICROMETRES};

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(super) struct QualityManifest {
    pub(super) schema: String,
    pub(super) split: String,
    #[serde(default)]
    pub(super) validator: Option<ValidatorDeclaration>,
    #[serde(default)]
    pub(super) relations: Vec<RelationSpec>,
    pub(super) entries: Vec<ManifestEntry>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(super) struct ManifestEntry {
    pub(super) id: String,
    pub(super) object_id: String,
    pub(super) material: String,
    pub(super) impact_position: String,
    pub(super) force_band: String,
    #[serde(default)]
    pub(super) expected_signal: ExpectedSignal,
    #[serde(default)]
    pub(super) control: Option<ImpactControl>,
    pub(super) candidate: AudioFileRef,
    pub(super) reference: Option<AudioFileRef>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub(super) struct ImpactControl {
    pub(super) impact_position_micrometres: [i64; 3],
    pub(super) impulse_micronewton_seconds: u64,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(super) struct AudioFileRef {
    pub(super) path: String,
    pub(super) sha256: String,
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(super) enum ExpectedSignal {
    #[default]
    Impact,
    Silence,
}

impl ExpectedSignal {
    pub(super) const fn as_str(self) -> &'static str {
        match self {
            Self::Impact => "impact",
            Self::Silence => "silence",
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(super) struct ValidatorDeclaration {
    pub(super) source_family: String,
    pub(super) generator_revision: String,
    pub(super) generator_sha256: String,
    pub(super) deterministic_probe_seed: u64,
    pub(super) fallback: AudioFileRef,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields, tag = "kind", rename_all = "snake_case")]
pub(super) enum RelationSpec {
    ExactWavRepeat {
        id: String,
        left: String,
        right: String,
    },
    ForceResponse {
        id: String,
        ordered_entries: Vec<String>,
    },
    PositionContinuity {
        id: String,
        ordered_entries: Vec<String>,
    },
}

impl RelationSpec {
    pub(super) fn id(&self) -> &str {
        match self {
            Self::ExactWavRepeat { id, .. }
            | Self::ForceResponse { id, .. }
            | Self::PositionContinuity { id, .. } => id,
        }
    }

    pub(super) const fn kind(&self) -> &'static str {
        match self {
            Self::ExactWavRepeat { .. } => "exact_wav_repeat",
            Self::ForceResponse { .. } => "force_response",
            Self::PositionContinuity { .. } => "position_continuity",
        }
    }

    pub(super) fn entry_ids(&self) -> Vec<&str> {
        match self {
            Self::ExactWavRepeat { left, right, .. } => vec![left, right],
            Self::ForceResponse {
                ordered_entries, ..
            }
            | Self::PositionContinuity {
                ordered_entries, ..
            } => ordered_entries.iter().map(String::as_str).collect(),
        }
    }
}

pub(super) fn validate_manifest(manifest: &QualityManifest) -> Result<(), String> {
    if manifest.schema != MANIFEST_SCHEMA {
        return Err(format!(
            "unsupported physical sound validator manifest schema: {}",
            manifest.schema
        ));
    }
    validate_label(&manifest.split, "split")?;
    if manifest.entries.is_empty() || manifest.entries.len() > MAX_ENTRIES {
        return Err(format!(
            "manifest entry count must be 1..={MAX_ENTRIES}, got {}",
            manifest.entries.len()
        ));
    }
    let mut previous = None;
    for entry in &manifest.entries {
        validate_label(&entry.id, "entry id")?;
        validate_label(&entry.object_id, "object_id")?;
        validate_label(&entry.material, "material")?;
        validate_label(&entry.impact_position, "impact_position")?;
        validate_label(&entry.force_band, "force_band")?;
        validate_control(entry)?;
        if previous.is_some_and(|value: &String| value >= &entry.id) {
            return Err(format!(
                "manifest entries must be strictly sorted by id; offending id {}",
                entry.id
            ));
        }
        previous = Some(&entry.id);
        validate_audio_ref(&entry.candidate, "candidate")?;
        if let Some(reference) = &entry.reference {
            validate_audio_ref(reference, "reference")?;
        }
    }
    if let Some(validator) = &manifest.validator {
        validate_label(&validator.source_family, "validator source_family")?;
        validate_label(
            &validator.generator_revision,
            "validator generator_revision",
        )?;
        validate_sha256(&validator.generator_sha256, "validator generator_sha256")?;
        validate_audio_ref(&validator.fallback, "validator fallback")?;
    }
    let mut previous_relation: Option<&str> = None;
    for relation in &manifest.relations {
        validate_label(relation.id(), "relation id")?;
        if previous_relation.is_some_and(|value| value >= relation.id()) {
            return Err(format!(
                "manifest relations must be strictly sorted by id; offending id {}",
                relation.id()
            ));
        }
        previous_relation = Some(relation.id());
        validate_relation(manifest, relation)?;
    }
    Ok(())
}

fn validate_relation(manifest: &QualityManifest, relation: &RelationSpec) -> Result<(), String> {
    let entry_ids = relation.entry_ids();
    if entry_ids.len() < 2 || entry_ids.len() > 64 {
        return Err(format!(
            "relation {} must contain 2..=64 entries",
            relation.id()
        ));
    }
    let mut unique = BTreeSet::new();
    let mut entries = Vec::with_capacity(entry_ids.len());
    for id in entry_ids {
        validate_label(id, "relation entry id")?;
        if !unique.insert(id) {
            return Err(format!(
                "relation {} contains duplicate entry {id}",
                relation.id()
            ));
        }
        let entry = manifest
            .entries
            .binary_search_by(|entry| entry.id.as_str().cmp(id))
            .map(|index| &manifest.entries[index])
            .map_err(|_| format!("relation {} references missing entry {id}", relation.id()))?;
        entries.push(entry);
    }
    let first = entries[0];
    if entries
        .iter()
        .any(|entry| entry.object_id != first.object_id || entry.material != first.material)
    {
        return Err(format!(
            "relation {} must stay within one object_id/material pair",
            relation.id()
        ));
    }
    match relation {
        RelationSpec::ExactWavRepeat { .. } => {
            if entries.iter().any(|entry| {
                entry.impact_position != first.impact_position
                    || entry.force_band != first.force_band
                    || entry.expected_signal != first.expected_signal
            }) {
                return Err(format!(
                    "exact repeat relation {} must preserve position, force and expected signal",
                    relation.id()
                ));
            }
            require_equal_controls(relation, &entries)?;
        }
        RelationSpec::ForceResponse { .. } => {
            if entries.iter().any(|entry| {
                entry.expected_signal != ExpectedSignal::Impact
                    || entry.impact_position != first.impact_position
            }) || entries
                .iter()
                .map(|entry| entry.force_band.as_str())
                .collect::<BTreeSet<_>>()
                .len()
                != entries.len()
            {
                return Err(format!(
                    "force relation {} must use distinct force bands at one impact position",
                    relation.id()
                ));
            }
            let controls = relation_controls(relation, &entries)?;
            if controls.windows(2).any(|pair| {
                pair[0].impact_position_micrometres != pair[1].impact_position_micrometres
                    || pair[0].impulse_micronewton_seconds >= pair[1].impulse_micronewton_seconds
            }) {
                return Err(format!(
                    "force relation {} must declare one position and strictly increasing impulse",
                    relation.id()
                ));
            }
        }
        RelationSpec::PositionContinuity { .. } => {
            if entries.iter().any(|entry| {
                entry.expected_signal != ExpectedSignal::Impact
                    || entry.force_band != first.force_band
            }) || entries
                .iter()
                .map(|entry| entry.impact_position.as_str())
                .collect::<BTreeSet<_>>()
                .len()
                != entries.len()
            {
                return Err(format!(
                    "position relation {} must use distinct positions at one force band",
                    relation.id()
                ));
            }
            let controls = relation_controls(relation, &entries)?;
            if controls.windows(2).any(|pair| {
                pair[0].impulse_micronewton_seconds != pair[1].impulse_micronewton_seconds
                    || neighbor_distance_micrometres(
                        pair[0].impact_position_micrometres,
                        pair[1].impact_position_micrometres,
                    )
                    .is_none_or(|distance| {
                        distance == 0 || distance > POSITION_MAXIMUM_NEIGHBOR_DISTANCE_MICROMETRES
                    })
            }) {
                return Err(format!(
                    "position relation {} must declare one impulse and distinct neighboring positions within {} micrometres",
                    relation.id(),
                    POSITION_MAXIMUM_NEIGHBOR_DISTANCE_MICROMETRES
                ));
            }
        }
    }
    Ok(())
}

fn validate_control(entry: &ManifestEntry) -> Result<(), String> {
    const MAX_IMPULSE_MICRONEWTON_SECONDS: u64 = 1_000_000_000_000_000_000;
    let Some(control) = &entry.control else {
        return Ok(());
    };
    if control
        .impact_position_micrometres
        .iter()
        .any(|value| value.unsigned_abs() > 8_388_608_000_000)
    {
        return Err(format!(
            "entry {} impact position exceeds the canonical physics range",
            entry.id
        ));
    }
    if control.impulse_micronewton_seconds > MAX_IMPULSE_MICRONEWTON_SECONDS {
        return Err(format!(
            "entry {} impulse exceeds the canonical physics range",
            entry.id
        ));
    }
    match entry.expected_signal {
        ExpectedSignal::Impact if control.impulse_micronewton_seconds == 0 => Err(format!(
            "impact entry {} must declare a positive impulse",
            entry.id
        )),
        ExpectedSignal::Silence if control.impulse_micronewton_seconds != 0 => Err(format!(
            "silence entry {} must declare zero impulse",
            entry.id
        )),
        ExpectedSignal::Impact | ExpectedSignal::Silence => Ok(()),
    }
}

fn relation_controls<'a>(
    relation: &RelationSpec,
    entries: &[&'a ManifestEntry],
) -> Result<Vec<&'a ImpactControl>, String> {
    entries
        .iter()
        .map(|entry| {
            entry.control.as_ref().ok_or_else(|| {
                format!(
                    "relation {} entry {} is missing physical control values",
                    relation.id(),
                    entry.id
                )
            })
        })
        .collect()
}

fn require_equal_controls(
    relation: &RelationSpec,
    entries: &[&ManifestEntry],
) -> Result<(), String> {
    let controls = relation_controls(relation, entries)?;
    if controls
        .iter()
        .skip(1)
        .any(|control| *control != controls[0])
    {
        return Err(format!(
            "exact repeat relation {} must preserve physical control values",
            relation.id()
        ));
    }
    Ok(())
}

fn neighbor_distance_micrometres(left: [i64; 3], right: [i64; 3]) -> Option<u64> {
    let squared = left
        .into_iter()
        .zip(right)
        .try_fold(0_u128, |sum, (left, right)| {
            let difference = i128::from(left).checked_sub(i128::from(right))?;
            let squared = difference.checked_mul(difference)?;
            sum.checked_add(u128::try_from(squared).ok()?)
        })?;
    Some(integer_sqrt(squared))
}

fn integer_sqrt(value: u128) -> u64 {
    if value == 0 {
        return 0;
    }
    let mut low = 1_u128;
    let mut high = value.min(u128::from(u64::MAX));
    while low < high {
        let middle = low + (high - low).div_ceil(2);
        if middle <= value / middle {
            low = middle;
        } else {
            high = middle - 1;
        }
    }
    low as u64
}

fn validate_label(value: &str, field: &str) -> Result<(), String> {
    if value.is_empty()
        || value.len() > 96
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_' | b'-'))
    {
        return Err(format!(
            "{field} must be 1..=96 ASCII [A-Za-z0-9._-] characters: {value:?}"
        ));
    }
    Ok(())
}

fn validate_audio_ref(value: &AudioFileRef, role: &str) -> Result<(), String> {
    if value.path.is_empty() || value.path.len() > 4_096 {
        return Err(format!("{role} path length is invalid"));
    }
    validate_sha256(&value.sha256, &format!("{role} sha256"))
}

fn validate_sha256(value: &str, field: &str) -> Result<(), String> {
    if value.len() != 64
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        return Err(format!("{field} must be 64 lowercase hex digits"));
    }
    Ok(())
}
