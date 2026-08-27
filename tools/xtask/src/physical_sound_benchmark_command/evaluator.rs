use std::collections::{BTreeMap, BTreeSet};

use serde::Serialize;

use super::EntryAudioReport;
use super::manifest::{CorpusEntry, DistanceMetric, EntryOrigin, Partition};

#[derive(Clone, Debug)]
pub(super) struct ResolvedEntry {
    pub(super) manifest: CorpusEntry,
    pub(super) audio: EntryAudioReport,
    pub(super) features: BTreeMap<String, Vec<f64>>,
}

#[derive(Clone, Debug)]
pub(super) struct FeatureProfile {
    pub(super) id: String,
    pub(super) distance: DistanceMetric,
    pub(super) dimensions: usize,
}

#[derive(Clone, Debug, Serialize)]
pub(super) struct TaskReport {
    pub(super) id: &'static str,
    pub(super) feature_set_id: String,
    pub(super) status: &'static str,
    pub(super) target_count: usize,
    pub(super) evaluated_count: usize,
    pub(super) skipped_missing_gallery_count: usize,
    pub(super) correct_count: usize,
    pub(super) accuracy: Option<f64>,
    pub(super) macro_accuracy: Option<f64>,
    pub(super) leakage_guard: &'static str,
    pub(super) confusion: Vec<ConfusionCount>,
    pub(super) origin_groups: Vec<GroupMetric>,
    pub(super) predictions: Vec<PredictionReport>,
}

#[derive(Clone, Debug, Serialize)]
pub(super) struct ConfusionCount {
    pub(super) expected: String,
    pub(super) predicted: String,
    pub(super) count: usize,
}

#[derive(Clone, Debug, Serialize)]
pub(super) struct GroupMetric {
    pub(super) group_id: String,
    pub(super) evaluated_count: usize,
    pub(super) correct_count: usize,
    pub(super) accuracy: f64,
}

#[derive(Clone, Debug, Serialize)]
pub(super) struct PredictionReport {
    pub(super) entry_id: String,
    pub(super) expected: String,
    pub(super) predicted: String,
    pub(super) correct: bool,
    pub(super) origin_group: String,
    pub(super) nearest_entry_id: String,
    pub(super) nearest_distance: f64,
    pub(super) second_nearest_distance: Option<f64>,
    pub(super) distance_margin: Option<f64>,
    pub(super) expected_label_nearest_entry_id: String,
    pub(super) expected_label_nearest_distance: f64,
    pub(super) nearest_competing_label: Option<String>,
    pub(super) nearest_competing_entry_id: Option<String>,
    pub(super) nearest_competing_distance: Option<f64>,
    /// Positive means the nearest expected-label anchor is closer than every
    /// competing label; unlike `distance_margin`, same-label neighbors cannot
    /// inflate this material/object separation diagnostic.
    pub(super) expected_label_margin: Option<f64>,
}

#[derive(Clone, Copy)]
enum TaskKind {
    DevelopmentLeaveObjectOutMaterial,
    DevelopmentLeaveFamilyOutMaterial,
    DevelopmentLeavePositionOutObject,
    DevelopmentLeaveListenerOutObject,
    DevelopmentLeaveForceOutObject,
    PartitionMaterial(Partition),
}

impl TaskKind {
    const ALL: [Self; 8] = [
        Self::DevelopmentLeaveObjectOutMaterial,
        Self::DevelopmentLeaveFamilyOutMaterial,
        Self::DevelopmentLeavePositionOutObject,
        Self::DevelopmentLeaveListenerOutObject,
        Self::DevelopmentLeaveForceOutObject,
        Self::PartitionMaterial(Partition::Calibration),
        Self::PartitionMaterial(Partition::Holdout),
        Self::PartitionMaterial(Partition::Shadow),
    ];

    const fn id(self) -> &'static str {
        match self {
            Self::DevelopmentLeaveObjectOutMaterial => "development_leave_object_out_material",
            Self::DevelopmentLeaveFamilyOutMaterial => "development_leave_family_out_material",
            Self::DevelopmentLeavePositionOutObject => "development_leave_position_out_object",
            Self::DevelopmentLeaveListenerOutObject => "development_leave_listener_out_object",
            Self::DevelopmentLeaveForceOutObject => "development_leave_force_out_object",
            Self::PartitionMaterial(Partition::Calibration) => {
                "calibration_material_from_real_development"
            }
            Self::PartitionMaterial(Partition::Holdout) => "holdout_material_from_real_development",
            Self::PartitionMaterial(Partition::Shadow) => "shadow_material_from_real_development",
            Self::PartitionMaterial(Partition::Development) => {
                "development_material_from_real_development"
            }
        }
    }

    const fn leakage_guard(self) -> &'static str {
        match self {
            Self::DevelopmentLeaveObjectOutMaterial => {
                "target object_id excluded from the real development gallery"
            }
            Self::DevelopmentLeaveFamilyOutMaterial => {
                "target object_family_id excluded from the real development gallery"
            }
            Self::DevelopmentLeavePositionOutObject => {
                "target object_id/impact_position_id group excluded from the real development gallery"
            }
            Self::DevelopmentLeaveListenerOutObject => {
                "target object_id/listener_position_id group excluded from the real development gallery"
            }
            Self::DevelopmentLeaveForceOutObject => {
                "target object_id/force_band group excluded from the real development gallery"
            }
            Self::PartitionMaterial(_) => {
                "gallery is real development only; calibration/holdout/shadow objects, generators and mutations never train"
            }
        }
    }

    fn is_target(self, entry: &CorpusEntry) -> bool {
        match self {
            Self::DevelopmentLeaveObjectOutMaterial
            | Self::DevelopmentLeaveFamilyOutMaterial
            | Self::DevelopmentLeavePositionOutObject
            | Self::DevelopmentLeaveListenerOutObject
            | Self::DevelopmentLeaveForceOutObject => {
                entry.partition == Partition::Development
                    && matches!(&entry.origin, EntryOrigin::Real)
            }
            Self::PartitionMaterial(partition) => entry.partition == partition,
        }
    }

    fn is_gallery(self, target: &CorpusEntry, candidate: &CorpusEntry) -> bool {
        if candidate.partition != Partition::Development
            || !matches!(&candidate.origin, EntryOrigin::Real)
        {
            return false;
        }
        match self {
            Self::DevelopmentLeaveObjectOutMaterial => candidate.object_id != target.object_id,
            Self::DevelopmentLeaveFamilyOutMaterial => {
                candidate.object_family_id != target.object_family_id
            }
            Self::DevelopmentLeavePositionOutObject => {
                candidate.object_id != target.object_id
                    || candidate.impact_position_id != target.impact_position_id
            }
            Self::DevelopmentLeaveListenerOutObject => {
                candidate.object_id != target.object_id
                    || candidate.listener_position_id != target.listener_position_id
            }
            Self::DevelopmentLeaveForceOutObject => {
                candidate.object_id != target.object_id || candidate.force_band != target.force_band
            }
            Self::PartitionMaterial(_) => true,
        }
    }

    fn expected_label(self, entry: &CorpusEntry) -> &str {
        match self {
            Self::DevelopmentLeavePositionOutObject
            | Self::DevelopmentLeaveListenerOutObject
            | Self::DevelopmentLeaveForceOutObject => &entry.object_id,
            Self::DevelopmentLeaveObjectOutMaterial
            | Self::DevelopmentLeaveFamilyOutMaterial
            | Self::PartitionMaterial(_) => &entry.material,
        }
    }

    fn predicted_label(self, entry: &CorpusEntry) -> &str {
        self.expected_label(entry)
    }
}

pub(super) fn evaluate_tasks(
    entries: &[ResolvedEntry],
    feature_profiles: &[FeatureProfile],
) -> Vec<TaskReport> {
    let mut reports = Vec::with_capacity(TaskKind::ALL.len() * feature_profiles.len());
    for profile in feature_profiles {
        for task in TaskKind::ALL {
            reports.push(evaluate_task(entries, profile, task));
        }
    }
    reports
}

fn evaluate_task(
    entries: &[ResolvedEntry],
    profile: &FeatureProfile,
    task: TaskKind,
) -> TaskReport {
    let targets = entries
        .iter()
        .filter(|entry| task.is_target(&entry.manifest))
        .collect::<Vec<_>>();
    let mut evaluated_count = 0_usize;
    let mut correct_count = 0_usize;
    let mut skipped_missing_gallery_count = 0_usize;
    let mut confusion = BTreeMap::<(String, String), usize>::new();
    let mut labels = BTreeMap::<String, (usize, usize)>::new();
    let mut groups = BTreeMap::<String, (usize, usize)>::new();
    let mut predictions = Vec::new();

    for target in &targets {
        let expected = task.expected_label(&target.manifest);
        let gallery = entries
            .iter()
            .filter(|candidate| task.is_gallery(&target.manifest, &candidate.manifest))
            .collect::<Vec<_>>();
        if !gallery
            .iter()
            .any(|entry| task.predicted_label(&entry.manifest) == expected)
        {
            skipped_missing_gallery_count += 1;
            continue;
        }
        let Some(nearest) = nearest_match(target, &gallery, profile, task, expected) else {
            skipped_missing_gallery_count += 1;
            continue;
        };
        let predicted = task.predicted_label(&nearest.entry.manifest);
        let correct = predicted == expected;
        evaluated_count += 1;
        correct_count += usize::from(correct);
        *confusion
            .entry((expected.to_owned(), predicted.to_owned()))
            .or_default() += 1;
        let label = labels.entry(expected.to_owned()).or_default();
        label.0 += 1;
        label.1 += usize::from(correct);
        let group = groups.entry(target.manifest.origin.group_id()).or_default();
        group.0 += 1;
        group.1 += usize::from(correct);
        predictions.push(PredictionReport {
            entry_id: target.manifest.id.clone(),
            expected: expected.to_owned(),
            predicted: predicted.to_owned(),
            correct,
            origin_group: target.manifest.origin.group_id(),
            nearest_entry_id: nearest.entry.manifest.id.clone(),
            nearest_distance: nearest.distance,
            second_nearest_distance: nearest.second_distance,
            distance_margin: nearest
                .second_distance
                .map(|second| second - nearest.distance),
            expected_label_nearest_entry_id: nearest.expected_entry.manifest.id.clone(),
            expected_label_nearest_distance: nearest.expected_distance,
            nearest_competing_label: nearest
                .competing_entry
                .map(|entry| task.predicted_label(&entry.manifest).to_owned()),
            nearest_competing_entry_id: nearest
                .competing_entry
                .map(|entry| entry.manifest.id.clone()),
            nearest_competing_distance: nearest.competing_distance,
            expected_label_margin: nearest
                .competing_distance
                .map(|distance| distance - nearest.expected_distance),
        });
    }

    let accuracy = ratio(correct_count, evaluated_count);
    let macro_accuracy = (!labels.is_empty()).then(|| {
        labels
            .values()
            .map(|(count, correct)| *correct as f64 / *count as f64)
            .sum::<f64>()
            / labels.len() as f64
    });
    TaskReport {
        id: task.id(),
        feature_set_id: profile.id.clone(),
        status: if evaluated_count == 0 {
            "Unavailable"
        } else {
            "Measured"
        },
        target_count: targets.len(),
        evaluated_count,
        skipped_missing_gallery_count,
        correct_count,
        accuracy,
        macro_accuracy,
        leakage_guard: task.leakage_guard(),
        confusion: confusion
            .into_iter()
            .map(|((expected, predicted), count)| ConfusionCount {
                expected,
                predicted,
                count,
            })
            .collect(),
        origin_groups: groups
            .into_iter()
            .map(|(group_id, (count, correct))| GroupMetric {
                group_id,
                evaluated_count: count,
                correct_count: correct,
                accuracy: correct as f64 / count as f64,
            })
            .collect(),
        predictions,
    }
}

struct NearestMatch<'a> {
    entry: &'a ResolvedEntry,
    distance: f64,
    second_distance: Option<f64>,
    expected_entry: &'a ResolvedEntry,
    expected_distance: f64,
    competing_entry: Option<&'a ResolvedEntry>,
    competing_distance: Option<f64>,
}

fn nearest_match<'a>(
    target: &ResolvedEntry,
    gallery: &[&'a ResolvedEntry],
    profile: &FeatureProfile,
    task: TaskKind,
    expected: &str,
) -> Option<NearestMatch<'a>> {
    let target_features = target.features.get(&profile.id)?;
    let mut ranked = gallery
        .iter()
        .copied()
        .filter_map(|entry| {
            let distance = feature_distance(
                target_features,
                entry
                    .features
                    .get(&profile.id)
                    .expect("validated feature set"),
                profile.distance,
            );
            distance.is_finite().then_some((entry, distance))
        })
        .collect::<Vec<_>>();
    ranked.sort_by(|(left, left_distance), (right, right_distance)| {
        left_distance
            .total_cmp(right_distance)
            .then_with(|| left.manifest.id.cmp(&right.manifest.id))
    });
    let (entry, distance) = ranked.first().copied()?;
    let (expected_entry, expected_distance) = ranked
        .iter()
        .copied()
        .find(|(entry, _)| task.predicted_label(&entry.manifest) == expected)?;
    let competing = ranked
        .iter()
        .copied()
        .find(|(entry, _)| task.predicted_label(&entry.manifest) != expected);
    Some(NearestMatch {
        entry,
        distance,
        second_distance: ranked.get(1).map(|(_, distance)| *distance),
        expected_entry,
        expected_distance,
        competing_entry: competing.map(|(entry, _)| entry),
        competing_distance: competing.map(|(_, distance)| distance),
    })
}

fn feature_distance(left: &[f64], right: &[f64], metric: DistanceMetric) -> f64 {
    debug_assert_eq!(left.len(), right.len());
    match metric {
        DistanceMetric::Euclidean => (left
            .iter()
            .zip(right)
            .map(|(left, right)| (left - right).powi(2))
            .sum::<f64>()
            / left.len() as f64)
            .sqrt(),
        DistanceMetric::Cosine => {
            let dot = left
                .iter()
                .zip(right)
                .map(|(left, right)| left * right)
                .sum::<f64>();
            let left_norm = left.iter().map(|value| value * value).sum::<f64>().sqrt();
            let right_norm = right.iter().map(|value| value * value).sum::<f64>().sqrt();
            if left_norm <= 1.0e-20 || right_norm <= 1.0e-20 {
                f64::INFINITY
            } else {
                1.0 - (dot / (left_norm * right_norm)).clamp(-1.0, 1.0)
            }
        }
    }
}

fn ratio(numerator: usize, denominator: usize) -> Option<f64> {
    (denominator > 0).then_some(numerator as f64 / denominator as f64)
}

pub(super) fn grouped_identity_counts(entries: &[ResolvedEntry]) -> GroupedIdentityCounts {
    GroupedIdentityCounts {
        object_count: entries
            .iter()
            .map(|entry| entry.manifest.object_id.as_str())
            .collect::<BTreeSet<_>>()
            .len(),
        object_family_count: entries
            .iter()
            .map(|entry| entry.manifest.object_family_id.as_str())
            .collect::<BTreeSet<_>>()
            .len(),
        material_count: entries
            .iter()
            .map(|entry| entry.manifest.material.as_str())
            .collect::<BTreeSet<_>>()
            .len(),
        generator_out_group_count: entries
            .iter()
            .filter_map(|entry| match &entry.manifest.origin {
                EntryOrigin::Generated {
                    generator_revision, ..
                } => Some(generator_revision),
                EntryOrigin::Real | EntryOrigin::Mutation { .. } => None,
            })
            .collect::<BTreeSet<_>>()
            .len(),
        mutation_out_group_count: entries
            .iter()
            .filter_map(|entry| match &entry.manifest.origin {
                EntryOrigin::Mutation {
                    mutation_family, ..
                } => Some(mutation_family),
                EntryOrigin::Real | EntryOrigin::Generated { .. } => None,
            })
            .collect::<BTreeSet<_>>()
            .len(),
    }
}

#[derive(Clone, Debug, Serialize)]
pub(super) struct GroupedIdentityCounts {
    pub(super) object_count: usize,
    pub(super) object_family_count: usize,
    pub(super) material_count: usize,
    pub(super) generator_out_group_count: usize,
    pub(super) mutation_out_group_count: usize,
}
