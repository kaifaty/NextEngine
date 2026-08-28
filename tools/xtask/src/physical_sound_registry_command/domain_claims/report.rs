use std::collections::{BTreeMap, BTreeSet};

use serde::Serialize;

use super::{
    CorpusPlanReport, CorpusRole, DOMAIN_ID, E2Entry, E3ObjectGroup, FALLBACK_DECISION,
    IdentifiedReport, MATRIX_ID, MATRIX_REVISION, PARTITIONS, PROFILE_ID, PlanDomain, REPORT_CLAIM,
    REPORT_SCHEMA, ValidatedLink,
};

#[derive(Serialize)]
pub(super) struct DomainClaimsReport {
    schema: &'static str,
    status: &'static str,
    decision: &'static str,
    fallback_decision: &'static str,
    claim: &'static str,
    matrix_id: &'static str,
    revision: &'static str,
    profile_id: &'static str,
    domain_id: &'static str,
    manifest_sha256: String,
    corpus_plan_report_sha256: String,
    partitioned_identified_report_sha256: String,
    split_verification_report_sha256: String,
    e2_inventory_report_count: usize,
    e2_entry_count: usize,
    e3_object_count: usize,
    e3_recording_count: usize,
    cross_tier_object_link_count: usize,
    unlinked_e2_entry_ids: Vec<String>,
    object_links: Vec<ObjectLinkReport>,
    partition_coverage: Vec<PartitionCoverageReport>,
    claims: Vec<ClaimReport>,
    blockers: Vec<String>,
}

#[derive(Serialize)]
struct ObjectLinkReport {
    link_id: String,
    e2_entry_id: String,
    e3_object_group_id: String,
    partition: String,
    e3_recording_count: usize,
    material_composition_aligned: bool,
    geometry_aligned: bool,
    support_aligned: bool,
    excitation_aligned: bool,
    impact_condition_aligned: bool,
    listener_condition_aligned: bool,
    matched_condition_identity: bool,
    exact_domain_eligible: bool,
}

#[derive(Serialize)]
struct PartitionCoverageReport {
    partition: &'static str,
    e3_target_objects: usize,
    e3_target_recordings: usize,
    identity_linked_objects: usize,
    exact_domain_eligible_objects: usize,
    exact_domain_closed: bool,
}

#[derive(Serialize)]
struct ClaimReport {
    claim_id: &'static str,
    required_for_domain: bool,
    status: &'static str,
    evidence_tiers: Vec<&'static str>,
    expected: Vec<String>,
    observed: Vec<String>,
    blocker: Option<&'static str>,
}

pub(super) struct ReportInputs<'a> {
    pub(super) hashes: ReportHashes,
    pub(super) plan: &'a CorpusPlanReport,
    pub(super) identified: &'a IdentifiedReport,
    pub(super) e2_entries: &'a BTreeMap<String, E2Entry>,
    pub(super) e3_groups: &'a BTreeMap<String, E3ObjectGroup>,
}

pub(super) struct ReportHashes {
    pub(super) manifest: String,
    pub(super) plan: String,
    pub(super) identified: String,
    pub(super) split: String,
}

pub(super) fn build_report(
    inputs: ReportInputs<'_>,
    links: Vec<ValidatedLink>,
) -> DomainClaimsReport {
    let domain = &inputs.plan.domains[0];
    let linked_e2 = links
        .iter()
        .map(|link| link.e2_entry_id.as_str())
        .collect::<BTreeSet<_>>();
    let link_reports = links
        .iter()
        .map(|link| ObjectLinkReport {
            link_id: link.link_id.clone(),
            e2_entry_id: link.e2_entry_id.clone(),
            e3_object_group_id: link.e3_object_group_id.clone(),
            partition: link.partition.clone(),
            e3_recording_count: link.e3_recording_count,
            material_composition_aligned: link.material_composition_aligned,
            geometry_aligned: link.geometry_aligned,
            support_aligned: link.support_aligned,
            excitation_aligned: link.excitation_aligned,
            impact_condition_aligned: link.impact_condition_aligned,
            listener_condition_aligned: link.listener_condition_aligned,
            matched_condition_identity: link.matched_condition_identity,
            exact_domain_eligible: link.exact_domain_eligible(),
        })
        .collect::<Vec<_>>();
    let partition_coverage = PARTITIONS
        .into_iter()
        .map(|partition| {
            let target_groups = inputs
                .e3_groups
                .values()
                .filter(|group| {
                    group.partition == partition && group.corpus_role == CorpusRole::Target
                })
                .collect::<Vec<_>>();
            let linked = links
                .iter()
                .filter(|link| link.partition == partition)
                .count();
            let eligible = links
                .iter()
                .filter(|link| link.partition == partition && link.exact_domain_eligible())
                .count();
            PartitionCoverageReport {
                partition,
                e3_target_objects: target_groups.len(),
                e3_target_recordings: target_groups
                    .iter()
                    .map(|group| group.recording_ids.len())
                    .sum(),
                identity_linked_objects: linked,
                exact_domain_eligible_objects: eligible,
                exact_domain_closed: eligible > 0,
            }
        })
        .collect::<Vec<_>>();
    let claims = build_claims(domain, inputs.e2_entries, &links, &partition_coverage);
    let blockers = claims
        .iter()
        .filter(|claim| claim.required_for_domain && claim.status == "Unsupported")
        .map(|claim| {
            claim
                .blocker
                .expect("unsupported required claim has blocker")
                .to_owned()
        })
        .collect::<Vec<_>>();
    let decision = if blockers.is_empty() {
        "DomainEvidenceComplete"
    } else {
        "DomainEvidenceIncomplete"
    };
    DomainClaimsReport {
        schema: REPORT_SCHEMA,
        status: "Validated",
        decision,
        fallback_decision: FALLBACK_DECISION,
        claim: REPORT_CLAIM,
        matrix_id: MATRIX_ID,
        revision: MATRIX_REVISION,
        profile_id: PROFILE_ID,
        domain_id: DOMAIN_ID,
        manifest_sha256: inputs.hashes.manifest,
        corpus_plan_report_sha256: inputs.hashes.plan,
        partitioned_identified_report_sha256: inputs.hashes.identified,
        split_verification_report_sha256: inputs.hashes.split,
        e2_inventory_report_count: inputs
            .e2_entries
            .values()
            .map(|entry| entry.report_sha256.as_str())
            .collect::<BTreeSet<_>>()
            .len(),
        e2_entry_count: inputs.e2_entries.len(),
        e3_object_count: inputs.e3_groups.len(),
        e3_recording_count: inputs.identified.recording_count,
        cross_tier_object_link_count: links.len(),
        unlinked_e2_entry_ids: inputs
            .e2_entries
            .keys()
            .filter(|entry_id| !linked_e2.contains(entry_id.as_str()))
            .cloned()
            .collect(),
        object_links: link_reports,
        partition_coverage,
        claims,
        blockers,
    }
}

fn build_claims(
    domain: &PlanDomain,
    e2_entries: &BTreeMap<String, E2Entry>,
    links: &[ValidatedLink],
    partition_coverage: &[PartitionCoverageReport],
) -> Vec<ClaimReport> {
    let link_ids = links
        .iter()
        .map(|link| link.link_id.clone())
        .collect::<Vec<_>>();
    let linked_e2_ids = links
        .iter()
        .map(|link| link.e2_entry_id.clone())
        .collect::<Vec<_>>();
    let linked_e3_groups = links
        .iter()
        .map(|link| link.e3_object_group_id.clone())
        .collect::<Vec<_>>();
    let repeat_observations = links
        .iter()
        .map(|link| format!("{}:{}", link.e3_object_group_id, link.e3_recording_count))
        .collect::<Vec<_>>();
    let all_links_repeat_complete = !links.is_empty()
        && links
            .iter()
            .all(|link| link.e3_recording_count >= domain.minimum_repeats_per_condition);
    let force_entries = linked_capability_entries(e2_entries, links, "force_deconvolved_transfer");
    let geometry_entries = capability_entries(e2_entries, "geometry");
    let impact_entries = capability_entries(e2_entries, "impact_position");
    let listener_entries = capability_entries(e2_entries, "listener_position");
    let material_aligned = aligned_link_ids(links, |link| link.material_composition_aligned);
    let geometry_aligned = aligned_link_ids(links, |link| link.geometry_aligned);
    let support_aligned = aligned_link_ids(links, |link| link.support_aligned);
    let excitation_aligned = aligned_link_ids(links, |link| link.excitation_aligned);
    let impact_aligned = aligned_link_ids(links, |link| link.impact_condition_aligned);
    let listener_aligned = aligned_link_ids(links, |link| link.listener_condition_aligned);
    let matched = aligned_link_ids(links, |link| link.matched_condition_identity);
    let partition_observed = partition_coverage
        .iter()
        .map(|coverage| {
            format!(
                "{}:{}",
                coverage.partition, coverage.exact_domain_eligible_objects
            )
        })
        .collect::<Vec<_>>();
    let all_partitions_closed = partition_coverage
        .iter()
        .all(|coverage| coverage.exact_domain_closed);
    vec![
        claim(
            "cross_tier_object_identity",
            true,
            !links.is_empty(),
            vec!["E2TransferResponse", "E3IdentifiedRecording"],
            vec!["at-least-one-frozen-exact-object-link".to_owned()],
            link_ids,
            "cross_tier_object_identity_unavailable",
        ),
        claim(
            "real_recording_identity",
            true,
            !links.is_empty(),
            vec!["E2TransferResponse", "E3IdentifiedRecording"],
            linked_e2_ids
                .iter()
                .map(|id| format!("e2:{id}"))
                .chain(linked_e3_groups.iter().map(|id| format!("e3:{id}")))
                .collect(),
            linked_e2_ids
                .iter()
                .map(|id| format!("e2:{id}"))
                .chain(linked_e3_groups.iter().map(|id| format!("e3:{id}")))
                .collect(),
            "linked_real_recording_identity_unavailable",
        ),
        claim(
            "e3_repeat_recording_identity",
            true,
            all_links_repeat_complete,
            vec!["E3IdentifiedRecording"],
            vec![format!(
                "minimum-repeats-per-linked-object:{}",
                domain.minimum_repeats_per_condition
            )],
            repeat_observations,
            "e3_repeat_recording_identity_incomplete",
        ),
        claim(
            "force_deconvolved_transfer",
            true,
            !force_entries.is_empty() && !links.is_empty(),
            vec!["E2TransferResponse"],
            vec!["force-deconvolved-transfer".to_owned()],
            force_entries,
            "force_deconvolved_transfer_unavailable",
        ),
        claim(
            "geometry_observation",
            false,
            !geometry_entries.is_empty(),
            vec!["E2TransferResponse"],
            vec!["geometry-observation".to_owned()],
            geometry_entries,
            "geometry_observation_unavailable",
        ),
        claim(
            "impact_position_observation",
            false,
            !impact_entries.is_empty(),
            vec!["E2TransferResponse"],
            vec!["impact-position-observation".to_owned()],
            impact_entries,
            "impact_position_observation_unavailable",
        ),
        claim(
            "listener_position_observation",
            false,
            !listener_entries.is_empty(),
            vec!["E2TransferResponse"],
            vec!["listener-position-observation".to_owned()],
            listener_entries,
            "listener_position_observation_unavailable",
        ),
        claim(
            "material_composition_revision_alignment",
            true,
            !material_aligned.is_empty(),
            vec!["E2TransferResponse"],
            vec![domain.material_family.clone()],
            observed_values(e2_entries, |entry| entry.material_family.as_str()),
            "material_composition_revision_unavailable",
        ),
        claim(
            "geometry_revision_alignment",
            true,
            !geometry_aligned.is_empty(),
            vec!["E2TransferResponse"],
            vec![domain.geometry_revision.clone()],
            observed_values(e2_entries, |entry| entry.geometry_revision.as_str()),
            "geometry_revision_not_domain_aligned",
        ),
        claim(
            "support_fixture_revision_alignment",
            true,
            !support_aligned.is_empty(),
            vec!["E2TransferResponse"],
            vec![domain.support_condition.clone()],
            observed_values(e2_entries, |entry| entry.support_condition.as_str()),
            "support_fixture_revision_unavailable",
        ),
        claim(
            "absolute_excitation_profile_alignment",
            true,
            !excitation_aligned.is_empty(),
            vec!["E2TransferResponse"],
            vec![domain.excitation_method.clone()],
            observed_values(e2_entries, |entry| entry.excitation_method.as_str()),
            "absolute_excitation_profile_unavailable",
        ),
        claim(
            "impact_condition_alignment",
            true,
            !impact_aligned.is_empty(),
            vec!["E2TransferResponse"],
            domain.impact_position_ids.clone(),
            observed_values(e2_entries, |entry| entry.impact_position_id.as_str()),
            "impact_condition_not_domain_aligned",
        ),
        claim(
            "listener_condition_alignment",
            true,
            !listener_aligned.is_empty(),
            vec!["E2TransferResponse"],
            domain.listener_condition_ids.clone(),
            observed_values(e2_entries, |entry| entry.listener_condition_id.as_str()),
            "listener_condition_not_domain_aligned",
        ),
        claim(
            "matched_cross_tier_condition_identity",
            true,
            !matched.is_empty(),
            vec!["E2TransferResponse", "E3IdentifiedRecording"],
            vec!["same-object-same-condition-repeat-lineage".to_owned()],
            matched,
            "matched_cross_tier_condition_identity_unavailable",
        ),
        claim(
            "four_partition_exact_domain_coverage",
            true,
            all_partitions_closed,
            vec!["E2TransferResponse", "E3IdentifiedRecording"],
            PARTITIONS
                .into_iter()
                .map(|partition| format!("{partition}:at-least-one"))
                .collect(),
            partition_observed,
            "four_partition_exact_domain_coverage_incomplete",
        ),
    ]
}

fn claim(
    claim_id: &'static str,
    required_for_domain: bool,
    supported: bool,
    evidence_tiers: Vec<&'static str>,
    expected: Vec<String>,
    observed: Vec<String>,
    blocker: &'static str,
) -> ClaimReport {
    ClaimReport {
        claim_id,
        required_for_domain,
        status: if supported {
            "Supported"
        } else {
            "Unsupported"
        },
        evidence_tiers,
        expected,
        observed,
        blocker: (!supported).then_some(blocker),
    }
}

fn capability_entries(entries: &BTreeMap<String, E2Entry>, capability: &str) -> Vec<String> {
    entries
        .values()
        .filter(|entry| entry.validated_capabilities.contains(capability))
        .map(|entry| entry.id.clone())
        .collect()
}

fn linked_capability_entries(
    entries: &BTreeMap<String, E2Entry>,
    links: &[ValidatedLink],
    capability: &str,
) -> Vec<String> {
    links
        .iter()
        .filter_map(|link| entries.get(&link.e2_entry_id))
        .filter(|entry| entry.validated_capabilities.contains(capability))
        .map(|entry| entry.id.clone())
        .collect()
}

fn aligned_link_ids(
    links: &[ValidatedLink],
    predicate: impl Fn(&ValidatedLink) -> bool,
) -> Vec<String> {
    links
        .iter()
        .filter(|link| predicate(link))
        .map(|link| link.link_id.clone())
        .collect()
}

fn observed_values(
    entries: &BTreeMap<String, E2Entry>,
    value: impl Fn(&E2Entry) -> &str,
) -> Vec<String> {
    entries
        .values()
        .map(value)
        .map(str::to_owned)
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}
