use std::collections::BTreeMap;

use serde::Serialize;

use super::TEMPORAL_FEATURE_SET_ID;
use super::evaluator::ResolvedEntry;
use super::manifest::{EntryOrigin, MutationExpectedValidatorOutcome, Partition};

const WILSON_95_Z: f64 = 1.959_963_984_540_054;

#[derive(Clone, Debug, Serialize)]
pub(super) struct TemporalSelectiveRiskReport {
    status: &'static str,
    authority: &'static str,
    score_definition: &'static str,
    normalization: &'static str,
    threshold_selection: &'static str,
    provisional_threshold: Option<f64>,
    calibration_curve: Vec<CalibrationCurvePoint>,
    partitions: Vec<PartitionRiskReport>,
    scores: Vec<QualityScoreReport>,
}

#[derive(Clone, Debug, Serialize)]
struct CalibrationCurvePoint {
    threshold: f64,
    accepted_real_object_count: usize,
    real_object_count: usize,
    false_pass_object_count: usize,
    reject_object_count: usize,
    balanced_accuracy: f64,
}

#[derive(Clone, Debug, Serialize)]
struct PartitionRiskReport {
    partition: &'static str,
    real_object_count: usize,
    accepted_real_object_count: usize,
    real_object_coverage: Option<f64>,
    reject_object_count: usize,
    false_pass_object_count: usize,
    observed_grouped_false_pass_risk: Option<f64>,
    wilson_upper_95_grouped_false_pass_risk: Option<f64>,
    mutation_entry_count: usize,
    false_pass_mutation_entry_count: usize,
    mutation_families: Vec<MutationFamilyRiskReport>,
}

#[derive(Clone, Debug, Serialize)]
struct MutationFamilyRiskReport {
    mutation_family: String,
    entry_count: usize,
    false_pass_entry_count: usize,
    observed_false_pass_risk: f64,
}

#[derive(Clone, Debug, Serialize)]
struct QualityScoreReport {
    entry_id: String,
    partition: &'static str,
    material: String,
    origin_group: String,
    expected_validator_outcome: &'static str,
    nearest_real_development_entry_id: String,
    normalized_temporal_distance: f64,
}

#[derive(Clone, Copy, Eq, PartialEq)]
enum ExpectedOutcome {
    Accept,
    Reject,
    Unspecified,
}

impl ExpectedOutcome {
    const fn as_str(self) -> &'static str {
        match self {
            Self::Accept => "accept",
            Self::Reject => "reject",
            Self::Unspecified => "unspecified",
        }
    }
}

struct ScoredEntry<'a> {
    entry: &'a ResolvedEntry,
    expected: ExpectedOutcome,
    nearest_anchor_id: &'a str,
    score: f64,
}

pub(super) fn evaluate_temporal_selective_risk(
    entries: &[ResolvedEntry],
) -> TemporalSelectiveRiskReport {
    let development = entries
        .iter()
        .filter(|entry| {
            entry.manifest.partition == Partition::Development
                && matches!(&entry.manifest.origin, EntryOrigin::Real)
        })
        .collect::<Vec<_>>();
    let Some((means, standard_deviations)) = normalization(&development) else {
        return unavailable_report();
    };
    let scores = entries
        .iter()
        .filter(|entry| entry.manifest.partition != Partition::Development)
        .filter_map(|entry| score_entry(entry, &development, &means, &standard_deviations))
        .collect::<Vec<_>>();
    let calibration = grouped_scores(&scores, Partition::Calibration);
    if calibration.real.is_empty() || calibration.reject.is_empty() {
        return TemporalSelectiveRiskReport {
            status: "UnavailableNoControlledCalibrationMutations",
            authority: "diagnostic only; cannot emit Pass or promote a formula family",
            score_definition: score_definition(),
            normalization: normalization_definition(),
            threshold_selection: threshold_selection_definition(),
            provisional_threshold: None,
            calibration_curve: Vec::new(),
            partitions: Vec::new(),
            scores: score_reports(&scores),
        };
    }
    let curve = calibration_curve(&calibration);
    let threshold = curve
        .iter()
        .max_by(|left, right| {
            left.balanced_accuracy
                .total_cmp(&right.balanced_accuracy)
                .then_with(|| right.threshold.total_cmp(&left.threshold))
        })
        .map(|point| point.threshold)
        .expect("non-empty calibration curve");
    TemporalSelectiveRiskReport {
        status: "MeasuredControlledMutationsOnly",
        authority: "diagnostic only; cannot emit Pass or promote a formula family",
        score_definition: score_definition(),
        normalization: normalization_definition(),
        threshold_selection: threshold_selection_definition(),
        provisional_threshold: Some(threshold),
        calibration_curve: curve,
        partitions: [
            Partition::Calibration,
            Partition::Holdout,
            Partition::Shadow,
        ]
        .into_iter()
        .map(|partition| partition_risk(&scores, partition, threshold))
        .collect(),
        scores: score_reports(&scores),
    }
}

fn unavailable_report() -> TemporalSelectiveRiskReport {
    TemporalSelectiveRiskReport {
        status: "UnavailableNoRealDevelopmentNormalization",
        authority: "diagnostic only; cannot emit Pass or promote a formula family",
        score_definition: score_definition(),
        normalization: normalization_definition(),
        threshold_selection: threshold_selection_definition(),
        provisional_threshold: None,
        calibration_curve: Vec::new(),
        partitions: Vec::new(),
        scores: Vec::new(),
    }
}

const fn score_definition() -> &'static str {
    "RMS standardized Euclidean distance to the nearest real development anchor of the declared material"
}

const fn normalization_definition() -> &'static str {
    "per-dimension mean and population standard deviation from real development entries only; standard deviation floor 1e-9"
}

const fn threshold_selection_definition() -> &'static str {
    "maximum grouped balanced accuracy on calibration real objects versus expected-reject mutation parent groups; lower threshold wins ties"
}

fn normalization(development: &[&ResolvedEntry]) -> Option<(Vec<f64>, Vec<f64>)> {
    let dimensions = development
        .first()?
        .features
        .get(TEMPORAL_FEATURE_SET_ID)?
        .len();
    let mut means = vec![0.0; dimensions];
    for entry in development {
        let values = entry.features.get(TEMPORAL_FEATURE_SET_ID)?;
        for (mean, value) in means.iter_mut().zip(values) {
            *mean += value;
        }
    }
    for mean in &mut means {
        *mean /= development.len() as f64;
    }
    let mut deviations = vec![0.0; dimensions];
    for entry in development {
        let values = entry.features.get(TEMPORAL_FEATURE_SET_ID)?;
        for ((deviation, value), mean) in deviations.iter_mut().zip(values).zip(&means) {
            *deviation += (value - mean).powi(2);
        }
    }
    for deviation in &mut deviations {
        *deviation = (*deviation / development.len() as f64).sqrt().max(1.0e-9);
    }
    Some((means, deviations))
}

fn score_entry<'a>(
    target: &'a ResolvedEntry,
    development: &[&'a ResolvedEntry],
    means: &[f64],
    deviations: &[f64],
) -> Option<ScoredEntry<'a>> {
    let target_features = target.features.get(TEMPORAL_FEATURE_SET_ID)?;
    let (nearest, score) = development
        .iter()
        .copied()
        .filter(|entry| entry.manifest.material == target.manifest.material)
        .filter_map(|entry| {
            let values = entry.features.get(TEMPORAL_FEATURE_SET_ID)?;
            let distance = standardized_distance(target_features, values, means, deviations);
            distance.is_finite().then_some((entry, distance))
        })
        .min_by(|(left, left_score), (right, right_score)| {
            left_score
                .total_cmp(right_score)
                .then_with(|| left.manifest.id.cmp(&right.manifest.id))
        })?;
    Some(ScoredEntry {
        entry: target,
        expected: expected_outcome(&target.manifest.origin),
        nearest_anchor_id: &nearest.manifest.id,
        score,
    })
}

fn expected_outcome(origin: &EntryOrigin) -> ExpectedOutcome {
    match origin {
        EntryOrigin::Real => ExpectedOutcome::Accept,
        EntryOrigin::Mutation {
            expected_validator_outcome: MutationExpectedValidatorOutcome::Reject,
            ..
        } => ExpectedOutcome::Reject,
        EntryOrigin::Generated { .. } | EntryOrigin::Mutation { .. } => {
            ExpectedOutcome::Unspecified
        }
    }
}

fn standardized_distance(left: &[f64], right: &[f64], means: &[f64], deviations: &[f64]) -> f64 {
    debug_assert_eq!(left.len(), right.len());
    debug_assert_eq!(left.len(), means.len());
    let squared = left
        .iter()
        .zip(right)
        .zip(means)
        .zip(deviations)
        .map(|(((left, right), _mean), deviation)| ((left - right) / deviation).powi(2))
        .sum::<f64>();
    (squared / left.len() as f64).sqrt()
}

struct GroupedScores {
    real: Vec<f64>,
    reject: Vec<f64>,
}

fn grouped_scores(scores: &[ScoredEntry<'_>], partition: Partition) -> GroupedScores {
    let real = scores
        .iter()
        .filter(|score| {
            score.entry.manifest.partition == partition && score.expected == ExpectedOutcome::Accept
        })
        .map(|score| score.score)
        .collect();
    let mut reject_by_parent = BTreeMap::<&str, f64>::new();
    for score in scores.iter().filter(|score| {
        score.entry.manifest.partition == partition && score.expected == ExpectedOutcome::Reject
    }) {
        let EntryOrigin::Mutation {
            parent_entry_id, ..
        } = &score.entry.manifest.origin
        else {
            unreachable!("reject expectation is mutation-only")
        };
        reject_by_parent
            .entry(parent_entry_id)
            .and_modify(|minimum| *minimum = minimum.min(score.score))
            .or_insert(score.score);
    }
    GroupedScores {
        real,
        reject: reject_by_parent.into_values().collect(),
    }
}

fn calibration_curve(scores: &GroupedScores) -> Vec<CalibrationCurvePoint> {
    let mut thresholds = scores
        .real
        .iter()
        .chain(&scores.reject)
        .copied()
        .collect::<Vec<_>>();
    thresholds.sort_by(f64::total_cmp);
    thresholds.dedup_by(|left, right| left.total_cmp(right).is_eq());
    let below_minimum = thresholds[0] - thresholds[0].abs().mul_add(1.0e-12, 1.0e-12);
    thresholds.insert(0, below_minimum);
    thresholds
        .into_iter()
        .map(|threshold| {
            let accepted_real = scores
                .real
                .iter()
                .filter(|score| **score <= threshold)
                .count();
            let false_pass = scores
                .reject
                .iter()
                .filter(|score| **score <= threshold)
                .count();
            let true_positive_rate = accepted_real as f64 / scores.real.len() as f64;
            let true_negative_rate =
                (scores.reject.len() - false_pass) as f64 / scores.reject.len() as f64;
            CalibrationCurvePoint {
                threshold,
                accepted_real_object_count: accepted_real,
                real_object_count: scores.real.len(),
                false_pass_object_count: false_pass,
                reject_object_count: scores.reject.len(),
                balanced_accuracy: (true_positive_rate + true_negative_rate) * 0.5,
            }
        })
        .collect()
}

fn partition_risk(
    scores: &[ScoredEntry<'_>],
    partition: Partition,
    threshold: f64,
) -> PartitionRiskReport {
    let grouped = grouped_scores(scores, partition);
    let accepted_real = grouped
        .real
        .iter()
        .filter(|score| **score <= threshold)
        .count();
    let false_pass_objects = grouped
        .reject
        .iter()
        .filter(|score| **score <= threshold)
        .count();
    let mutations = scores
        .iter()
        .filter(|score| {
            score.entry.manifest.partition == partition && score.expected == ExpectedOutcome::Reject
        })
        .collect::<Vec<_>>();
    let false_pass_mutations = mutations
        .iter()
        .filter(|score| score.score <= threshold)
        .count();
    let mut families = BTreeMap::<&str, (usize, usize)>::new();
    for mutation in &mutations {
        let EntryOrigin::Mutation {
            mutation_family, ..
        } = &mutation.entry.manifest.origin
        else {
            unreachable!("reject expectation is mutation-only")
        };
        let counts = families.entry(mutation_family).or_default();
        counts.0 += 1;
        counts.1 += usize::from(mutation.score <= threshold);
    }
    PartitionRiskReport {
        partition: partition.as_str(),
        real_object_count: grouped.real.len(),
        accepted_real_object_count: accepted_real,
        real_object_coverage: ratio(accepted_real, grouped.real.len()),
        reject_object_count: grouped.reject.len(),
        false_pass_object_count: false_pass_objects,
        observed_grouped_false_pass_risk: ratio(false_pass_objects, grouped.reject.len()),
        wilson_upper_95_grouped_false_pass_risk: wilson_upper_95(
            false_pass_objects,
            grouped.reject.len(),
        ),
        mutation_entry_count: mutations.len(),
        false_pass_mutation_entry_count: false_pass_mutations,
        mutation_families: families
            .into_iter()
            .map(|(family, (count, false_pass))| MutationFamilyRiskReport {
                mutation_family: family.to_owned(),
                entry_count: count,
                false_pass_entry_count: false_pass,
                observed_false_pass_risk: false_pass as f64 / count as f64,
            })
            .collect(),
    }
}

fn score_reports(scores: &[ScoredEntry<'_>]) -> Vec<QualityScoreReport> {
    scores
        .iter()
        .map(|score| QualityScoreReport {
            entry_id: score.entry.manifest.id.clone(),
            partition: score.entry.manifest.partition.as_str(),
            material: score.entry.manifest.material.clone(),
            origin_group: score.entry.manifest.origin.group_id(),
            expected_validator_outcome: score.expected.as_str(),
            nearest_real_development_entry_id: score.nearest_anchor_id.to_owned(),
            normalized_temporal_distance: score.score,
        })
        .collect()
}

fn ratio(numerator: usize, denominator: usize) -> Option<f64> {
    (denominator > 0).then_some(numerator as f64 / denominator as f64)
}

fn wilson_upper_95(successes: usize, trials: usize) -> Option<f64> {
    if trials == 0 {
        return None;
    }
    let count = trials as f64;
    let observed = successes as f64 / count;
    let z_squared = WILSON_95_Z * WILSON_95_Z;
    let center = observed + z_squared / (2.0 * count);
    let radius =
        WILSON_95_Z * ((observed * (1.0 - observed) + z_squared / (4.0 * count)) / count).sqrt();
    Some(((center + radius) / (1.0 + z_squared / count)).min(1.0))
}

#[cfg(test)]
mod tests {
    use super::wilson_upper_95;

    #[test]
    fn wilson_bound_is_conservative_for_small_zero_failure_samples() {
        let upper = wilson_upper_95(0, 3).expect("bound");
        assert!(upper > 0.5 && upper < 0.6);
        assert_eq!(wilson_upper_95(0, 0), None);
    }
}
