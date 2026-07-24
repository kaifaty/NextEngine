use std::collections::BTreeSet;

use xtask::docs_check::{
    sha256_hex, validate_deterministic_substrate_documents,
    validate_foundation_packet_17_documents, validate_gameplay_budget_matrix,
    validate_governance_documents, validate_human_review_decision_v2_documents,
    validate_human_review_decision_v2_text, validate_index_entry_line,
    validate_normative_graph_documents, validate_p0_packet_18_documents,
    validate_packet_16_traceability_allocations, validate_packet_17_normative_graph_documents,
    validate_packet_17_traceability_allocations, validate_requirement_gate_documents,
    validate_requirement_gate_documents_with_evidence, validate_review_candidate_version,
    validate_supersession_document, validate_supersession_documents, validate_technology_registry,
    validate_traceability_allocations, validate_traceability_text,
};

const VALID_A: &str = include_str!("fixtures/valid-a.md");
const VALID_B: &str = include_str!("fixtures/valid-b.md");
const CYCLE_A: &str = include_str!("fixtures/cycle-a.md");
const CYCLE_B: &str = include_str!("fixtures/cycle-b.md");
const EXTERNAL: &str = include_str!("fixtures/external-normative.md");
const INDEX_MISMATCH: &str = include_str!("fixtures/index-mismatch.md");
const BROKEN_SUPERSESSION: &str = include_str!("fixtures/broken-supersession.md");
const COMPOSITE_OWNER: &str = include_str!("fixtures/composite-owner.md");
const UNKNOWN_GATE: &str = include_str!("fixtures/unknown-gate.md");
const BAD_ANNEX: &[u8] = include_bytes!("fixtures/bad-annex-hash.txt");
const MISSING_GOVERNANCE: &str = include_str!("fixtures/missing-governance-status.md");
const BUDGET_MISMATCH: &str = include_str!("fixtures/budget-mismatch.md");
const ACCEPTED_DEPENDS_PROPOSED: &str = include_str!("fixtures/accepted-depends-proposed.md");
const PROPOSED_TARGET: &str = include_str!("fixtures/proposed-target.md");
const DETERMINISTIC_SPEC: &str = include_str!(
    "../../../docs/architecture/21-deterministic-runtime-primitives-command-ledger-and-causal-identity.md"
);
const DETERMINISTIC_ADR: &str = include_str!(
    "../../../docs/architecture/adr/022-deterministic-command-identity-ledger-and-causal-identity.md"
);
const SCRIPTING_SPEC: &str =
    include_str!("../../../docs/architecture/07-rpg-scripting-and-plugins.md");
const EXTENSION_ADR: &str = include_str!(
    "../../../docs/architecture/adr/014-deterministic-extensions-and-package-trust.md"
);
const REVIEW_V2_ADR: &str = include_str!(
    "../../../docs/architecture/adr/023-human-review-decision-v2-and-offline-attestation.md"
);
const PROJECT_SPEC: &str = include_str!(
    "../../../docs/architecture/17-project-composition-configuration-and-application-lifecycle.md"
);
const PROJECT_ADR: &str = include_str!(
    "../../../docs/architecture/adr/018-authoritative-project-composition-and-configuration.md"
);
const PLAYER_SPEC: &str = include_str!(
    "../../../docs/architecture/18-player-interaction-ui-camera-localization-and-accessibility.md"
);
const PLAYER_ADR: &str = include_str!(
    "../../../docs/architecture/adr/019-canonical-player-actions-and-presentation-authority.md"
);
const RPG_SPEC: &str =
    include_str!("../../../docs/architecture/19-rpg-domain-and-narrative-state.md");
const RPG_ADR: &str = include_str!(
    "../../../docs/architecture/adr/020-rpg-domain-authority-and-extension-boundary.md"
);
const WORLD_SPEC: &str =
    include_str!("../../../docs/architecture/20-world-simulation-and-population-lifecycle.md");
const WORLD_ADR: &str = include_str!(
    "../../../docs/architecture/adr/021-deterministic-population-residency-and-time-advance.md"
);
const SCHEMA_REGISTRY_SPEC: &str =
    include_str!("../../../docs/architecture/22-schema-registry-compatibility-and-migration.md");
const RESOURCE_ADMISSION_SPEC: &str = include_str!(
    "../../../docs/architecture/23-jobs-memory-resource-residency-and-io-backpressure.md"
);
const CONTENT_CATALOG_SPEC: &str = include_str!(
    "../../../docs/architecture/24-content-catalog-bundle-and-neutral-asset-schemas.md"
);
const WORLD_PARTITION_SPEC: &str = include_str!(
    "../../../docs/architecture/25-world-partition-streaming-admission-and-persistent-spatial-objects.md"
);
const PHYSICS_WORLD_SPEC: &str = include_str!(
    "../../../docs/architecture/26-physics-world-collision-constraints-queries-and-canonical-snapshots.md"
);
const MOTOR_INFERENCE_SPEC: &str = include_str!(
    "../../../docs/architecture/27-motor-observation-action-and-deterministic-inference.md"
);
const ANIMATION_SPEC: &str =
    include_str!("../../../docs/architecture/28-skeletal-animation-retargeting-and-ik.md");
const PLATFORM_SESSION_SPEC: &str =
    include_str!("../../../docs/architecture/29-platform-host-and-application-session.md");
const PRESENTATION_SPEC: &str =
    include_str!("../../../docs/architecture/30-presentation-extraction-and-render-content.md");
const SCHEMA_AUTHORITY_ADR: &str =
    include_str!("../../../docs/architecture/adr/025-schema-content-and-migration-authority.md");
const WORK_ADMISSION_ADR: &str = include_str!(
    "../../../docs/architecture/adr/026-deterministic-work-resource-and-streaming-admission.md"
);
const PHYSICAL_LAYERING_ADR: &str =
    include_str!("../../../docs/architecture/adr/027-physics-motor-and-animation-layering.md");
const PRESENTATION_AUTHORITY_ADR: &str = include_str!(
    "../../../docs/architecture/adr/028-platform-session-and-presentation-authority.md"
);

fn traceability_with_rows(requirement_rows: &[String], reservation_rows: &[String]) -> String {
    let requirement_rows = requirement_rows
        .iter()
        .map(|row| {
            let cells = row
                .split('|')
                .map(str::trim)
                .filter(|cell| !cell.is_empty())
                .collect::<Vec<_>>();
            assert_eq!(cells.len(), 6, "test traceability row shape");
            format!(
                "| {} | {} | {} | SPEC-00 | {} | {} | {} |",
                cells[0], cells[1], cells[2], cells[3], cells[4], cells[5]
            )
        })
        .collect::<Vec<_>>();
    format!(
        "# Trace\n\n\
         | Requirement | Requirement text | Primary owner | RFC / ADR | Gate | VS / profile closure | Evidence |\n\
         |---|---|---|---|---|---|---|\n\
         {}\n\n\
         | Reserved ID | Allocation status | Owning document | Reason |\n\
         |---|---|---|---|\n\
         {}\n",
        requirement_rows.join("\n"),
        reservation_rows.join("\n")
    )
}

fn reservations(first: usize, last: usize) -> Vec<String> {
    (first..=last)
        .map(|number| {
            format!("| REQ-{number:03} | Reserved | SPEC-TEST | Permanent test allocation |")
        })
        .collect()
}

fn packet_16_traceability() -> String {
    let requirements = (1..=78)
        .chain(103..=111)
        .map(|number| {
            format!(
                "| REQ-{number:03} | requirement | Runtime | SPEC-00 | BASE-01 | VS-12 | report |"
            )
        })
        .collect::<Vec<_>>()
        .join("\n");
    let failures = (1..=24)
        .chain(39..=43)
        .map(|number| {
            format!(
                "| FAIL-{number:03} | failure | Runtime | SPEC-00 | fail closed | BASE-01 | report | VS-12 |"
            )
        })
        .collect::<Vec<_>>()
        .join("\n");
    let mut reservations = Vec::new();
    let mut add = |prefix: &str, first: usize, last: usize, document: &str| {
        reservations.extend((first..=last).map(|number| {
            format!("| {prefix}{number:03} | Reserved | {document} | Permanent packet allocation |")
        }));
    };
    add("REQ-", 79, 86, "SPEC-16");
    add("REQ-", 87, 90, "SPEC-17");
    add("REQ-", 91, 94, "SPEC-18");
    add("REQ-", 95, 98, "SPEC-19");
    add("REQ-", 99, 102, "SPEC-20");
    for (offset, document_number) in (22..=30).enumerate() {
        let first = 112 + offset * 4;
        add(
            "REQ-",
            first,
            first + 3,
            &format!("SPEC-{document_number:02}"),
        );
    }
    add("FAIL-", 25, 30, "SPEC-16");
    add("FAIL-", 31, 32, "SPEC-17");
    add("FAIL-", 33, 34, "SPEC-18");
    add("FAIL-", 35, 36, "SPEC-19");
    add("FAIL-", 37, 38, "SPEC-20");
    for (offset, document_number) in (22..=30).enumerate() {
        let first = 44 + offset * 2;
        add(
            "FAIL-",
            first,
            first + 1,
            &format!("SPEC-{document_number:02}"),
        );
    }
    format!(
        "| Requirement | Requirement text | Primary owner | RFC / ADR | Gate | VS / profile closure | Evidence |\n\
         |---|---|---|---|---|---|---|\n\
         {requirements}\n\n\
         | Failure requirement | Failure / trigger | Primary owner | RFC / ADR | Нормативный путь | Gate | Evidence | VS / profile closure |\n\
         |---|---|---|---|---|---|---|---|\n\
         {failures}\n\n\
         | Reserved ID | Allocation status | Owning document | Reason |\n\
         |---|---|---|---|\n\
         {}\n",
        reservations.join("\n")
    )
}

fn packet_17_traceability() -> String {
    let requirements = (1..=78)
        .chain(87..=111)
        .map(|number| {
            let document = match number {
                87..=90 => "SPEC-17",
                91..=94 => "SPEC-18",
                95..=98 => "SPEC-19",
                99..=102 => "SPEC-20",
                103..=110 => "SPEC-21",
                _ => "SPEC-00",
            };
            format!(
                "| REQ-{number:03} | requirement | Runtime | {document} | BASE-01 | VS-12 | report |"
            )
        })
        .collect::<Vec<_>>()
        .join("\n");
    let failures = (1..=24)
        .chain(31..=43)
        .map(|number| {
            let document = match number {
                31..=32 => "SPEC-17",
                33..=34 => "SPEC-18",
                35..=36 => "SPEC-19",
                37..=38 => "SPEC-20",
                39..=42 => "SPEC-21",
                _ => "SPEC-00",
            };
            format!(
                "| FAIL-{number:03} | failure | Runtime | {document} | fail closed | BASE-01 | report | VS-12 |"
            )
        })
        .collect::<Vec<_>>()
        .join("\n");
    let mut reservations = Vec::new();
    let mut add = |prefix: &str, first: usize, last: usize, document: &str| {
        reservations.extend((first..=last).map(|number| {
            format!("| {prefix}{number:03} | Reserved | {document} | Permanent packet allocation |")
        }));
    };
    add("REQ-", 79, 86, "SPEC-16");
    for (offset, document_number) in (22..=30).enumerate() {
        let first = 112 + offset * 4;
        add(
            "REQ-",
            first,
            first + 3,
            &format!("SPEC-{document_number:02}"),
        );
    }
    add("FAIL-", 25, 30, "SPEC-16");
    for (offset, document_number) in (22..=30).enumerate() {
        let first = 44 + offset * 2;
        add(
            "FAIL-",
            first,
            first + 1,
            &format!("SPEC-{document_number:02}"),
        );
    }
    format!(
        "| Requirement | Requirement text | Primary owner | RFC / ADR | Gate | VS / profile closure | Evidence |\n\
         |---|---|---|---|---|---|---|\n\
         {requirements}\n\n\
         | Failure requirement | Failure / trigger | Primary owner | RFC / ADR | Нормативный путь | Gate | Evidence | VS / profile closure |\n\
         |---|---|---|---|---|---|---|---|\n\
         {failures}\n\n\
         | Reserved ID | Allocation status | Owning document | Reason |\n\
         |---|---|---|---|\n\
         {}\n",
        reservations.join("\n")
    )
}

fn foundation_documents() -> [(&'static str, &'static str); 8] {
    [
        ("SPEC-17", PROJECT_SPEC),
        ("ADR-018", PROJECT_ADR),
        ("SPEC-18", PLAYER_SPEC),
        ("ADR-019", PLAYER_ADR),
        ("SPEC-19", RPG_SPEC),
        ("ADR-020", RPG_ADR),
        ("SPEC-20", WORLD_SPEC),
        ("ADR-021", WORLD_ADR),
    ]
}

fn validate_foundation_mutation(
    document_id: &str,
    original: &str,
    replacement: &str,
) -> Result<(), String> {
    let mut documents = foundation_documents()
        .into_iter()
        .map(|(id, body)| (id, body.to_owned()))
        .collect::<Vec<_>>();
    let body = documents
        .iter_mut()
        .find_map(|(id, body)| (*id == document_id).then_some(body))
        .expect("foundation mutation document must exist");
    let mutated = body.replacen(original, replacement, 1);
    assert_ne!(*body, mutated, "foundation mutation target must exist");
    *body = mutated;
    let borrowed = documents
        .iter()
        .map(|(id, body)| (*id, body.as_str()))
        .collect::<Vec<_>>();
    validate_foundation_packet_17_documents(&borrowed)
}

fn packet_18_documents() -> [(&'static str, &'static str); 13] {
    [
        ("SPEC-22", SCHEMA_REGISTRY_SPEC),
        ("SPEC-23", RESOURCE_ADMISSION_SPEC),
        ("SPEC-24", CONTENT_CATALOG_SPEC),
        ("SPEC-25", WORLD_PARTITION_SPEC),
        ("SPEC-26", PHYSICS_WORLD_SPEC),
        ("SPEC-27", MOTOR_INFERENCE_SPEC),
        ("SPEC-28", ANIMATION_SPEC),
        ("SPEC-29", PLATFORM_SESSION_SPEC),
        ("SPEC-30", PRESENTATION_SPEC),
        ("ADR-025", SCHEMA_AUTHORITY_ADR),
        ("ADR-026", WORK_ADMISSION_ADR),
        ("ADR-027", PHYSICAL_LAYERING_ADR),
        ("ADR-028", PRESENTATION_AUTHORITY_ADR),
    ]
}

fn validate_packet_18_mutation(
    document_id: &str,
    original: &str,
    replacement: &str,
) -> Result<(), String> {
    let mut documents = packet_18_documents()
        .into_iter()
        .map(|(id, body)| (id, body.to_owned()))
        .collect::<Vec<_>>();
    let body = documents
        .iter_mut()
        .find_map(|(id, body)| (*id == document_id).then_some(body))
        .expect("packet-1.8 mutation document must exist");
    let mutated = body.replace(original, replacement);
    assert_ne!(*body, mutated, "packet-1.8 mutation target must exist");
    *body = mutated;
    let borrowed = documents
        .iter()
        .map(|(id, body)| (*id, body.as_str()))
        .collect::<Vec<_>>();
    validate_p0_packet_18_documents(&borrowed)
}

fn architecture_document(id: &str, status: &str, owner: &str, body: &str) -> String {
    format!(
        "# {id}\n\n\
         | Field | Value |\n\
         |---|---|\n\
         | ID | {id} |\n\
         | Статус | {status} |\n\
         | Версия | 1.0 |\n\
         | Владелец | {owner} |\n\
         | Нормативные зависимости | отсутствуют |\n\
         | Заменяет | отсутствует |\n\n\
         {body}\n"
    )
}

fn architecture_document_with_dependencies(id: &str, status: &str, dependencies: &str) -> String {
    architecture_document(id, status, "Architecture", "").replace(
        "| Нормативные зависимости | отсутствуют |",
        &format!("| Нормативные зависимости | {dependencies} |"),
    )
}

fn supersession_document(id: &str, status: &str, supersedes: &str, superseded_by: &str) -> String {
    format!(
        "# {id}\n\n\
         | Field | Value |\n\
         |---|---|\n\
         | ID | {id} |\n\
         | Статус | {status} |\n\
         | Версия | 1.0 |\n\
         | Владелец | Architecture |\n\
         | Нормативные зависимости | отсутствуют |\n\
         | Заменяет | {supersedes} |\n\
         | Заменён | {superseded_by} |\n"
    )
}

fn gate_table(rows: &[&str]) -> String {
    format!(
        "| Gate ID | Owner | Threshold | Evidence | Fallback |\n\
         |---|---|---|---|---|\n\
         {}\n",
        rows.join("\n")
    )
}

fn vertical_with_perf() -> String {
    let rows = (1..=15)
        .map(|number| {
            let children = if number == 12 {
                "BASE-01, PERF-01"
            } else {
                "BASE-01"
            };
            format!(
                "| VS-{number:02} Test vertical | Release | all pass | review packet | block | {children} |"
            )
        })
        .collect::<Vec<_>>()
        .join("\n");
    format!(
        "| Gate | Owner | Threshold | Evidence | Fallback | Blocking child gates |\n\
         |---|---|---|---|---|---|\n\
         {rows}\n"
    )
}

fn valid_requirement_graph_fixture() -> (String, String, String) {
    let reserved = reservations(1, 110);
    let traceability = traceability_with_rows(
        &[
            "| REQ-111 | Integrated performance | Release | BASE-01, PERF-01 | VS-12 | report |"
                .to_owned(),
        ],
        &reserved,
    );
    let descriptor = architecture_document(
        "SPEC-00",
        "Accepted",
        "Verification",
        &gate_table(&[
            "| BASE-01 | Verification | exact | base report | block |",
            "| PERF-01 | Release | exact | performance report | block |",
        ]),
    );
    (traceability, descriptor, vertical_with_perf())
}

fn valid_review_v2_contract_body() -> String {
    REVIEW_V2_ADR.to_owned()
}

fn with_duplicate_mention_elsewhere(contract: String, original: &str) -> String {
    format!("{contract}\n\n## Non-normative duplicate mention\n\n~~~text\n{original}\n~~~\n")
}

fn review_contract_mutation_with_decoy(original: &str, replacement: &str) -> String {
    let contract = valid_review_v2_contract_body().replacen(original, replacement, 1);
    assert_ne!(
        contract,
        valid_review_v2_contract_body(),
        "test mutation target must exist in ADR-023"
    );
    with_duplicate_mention_elsewhere(contract, original)
}

#[test]
fn valid_normative_graph_passes() {
    validate_normative_graph_documents(&[("a.md", VALID_A), ("b.md", VALID_B)])
        .expect("valid graph should pass");
}

#[test]
fn dependency_cycle_is_rejected() {
    let error = validate_normative_graph_documents(&[("a.md", CYCLE_A), ("b.md", CYCLE_B)])
        .expect_err("cycle must fail");
    assert!(error.contains("DOCS_GRAPH_CYCLE"));
}

#[test]
fn external_normative_link_is_rejected() {
    let error = validate_normative_graph_documents(&[("external.md", EXTERNAL)])
        .expect_err("external normative dependency must fail");
    assert!(error.contains("DOCS_EXTERNAL_NORMATIVE"));
}

#[test]
fn accepted_document_cannot_depend_on_proposed_target() {
    let error = validate_normative_graph_documents(&[
        ("accepted-depends-proposed.md", ACCEPTED_DEPENDS_PROPOSED),
        ("proposed-target.md", PROPOSED_TARGET),
    ])
    .expect_err("Accepted dependency on Proposed target must fail");
    assert!(error.contains("DOCS_NON_ACCEPTED_NORMATIVE_TARGET"));
}

#[test]
fn packet_17_induced_graph_accepts_links_into_admitted_legacy_scc() {
    let legacy_01 =
        architecture_document_with_dependencies("SPEC-01", "Accepted", "[SPEC-12](12-legacy.md)");
    let legacy_12 =
        architecture_document_with_dependencies("SPEC-12", "Accepted", "[SPEC-01](01-legacy.md)");
    let adr_018 = architecture_document_with_dependencies(
        "ADR-018",
        "Accepted",
        "[SPEC-01](../01-legacy.md)",
    );
    let spec_17 =
        architecture_document_with_dependencies("SPEC-17", "Accepted", "[ADR-018](adr/018.md)");
    let adr_019 = architecture_document_with_dependencies(
        "ADR-019",
        "Accepted",
        "[SPEC-01](../01-legacy.md)",
    );
    let spec_18 = architecture_document_with_dependencies(
        "SPEC-18",
        "Accepted",
        "[SPEC-17](17.md), [ADR-019](adr/019.md)",
    );
    let adr_020 = architecture_document_with_dependencies(
        "ADR-020",
        "Accepted",
        "[SPEC-01](../01-legacy.md)",
    );
    let spec_19 = architecture_document_with_dependencies(
        "SPEC-19",
        "Accepted",
        "[SPEC-17](17.md), [ADR-020](adr/020.md)",
    );
    let adr_021 = architecture_document_with_dependencies(
        "ADR-021",
        "Accepted",
        "[SPEC-19](../19.md), [ADR-020](020.md)",
    );
    let spec_20 = architecture_document_with_dependencies(
        "SPEC-20",
        "Accepted",
        "[SPEC-17](17.md), [SPEC-19](19.md), [ADR-021](adr/021.md)",
    );
    validate_packet_17_normative_graph_documents(&[
        ("01-legacy.md", &legacy_01),
        ("12-legacy.md", &legacy_12),
        ("17.md", &spec_17),
        ("18.md", &spec_18),
        ("19.md", &spec_19),
        ("20.md", &spec_20),
        ("adr/018.md", &adr_018),
        ("adr/019.md", &adr_019),
        ("adr/020.md", &adr_020),
        ("adr/021.md", &adr_021),
    ])
    .expect("packet-1.7 induced DAG may attach to the already-admitted legacy SCC");
}

#[test]
fn packet_17_induced_graph_rejects_internal_cycle() {
    let adr_018 =
        architecture_document_with_dependencies("ADR-018", "Accepted", "[SPEC-17](../17.md)");
    let spec_17 =
        architecture_document_with_dependencies("SPEC-17", "Accepted", "[ADR-018](adr/018.md)");
    let independent = |id: &str| architecture_document(id, "Accepted", "Architecture", "");
    let error = validate_packet_17_normative_graph_documents(&[
        ("17.md", &spec_17),
        ("18.md", &independent("SPEC-18")),
        ("19.md", &independent("SPEC-19")),
        ("20.md", &independent("SPEC-20")),
        ("adr/018.md", &adr_018),
        ("adr/019.md", &independent("ADR-019")),
        ("adr/020.md", &independent("ADR-020")),
        ("adr/021.md", &independent("ADR-021")),
    ])
    .expect_err("a W2-to-W2 dependency cycle must fail packet admission");
    assert!(
        error.contains("DOCS_PACKET_17_GRAPH_CYCLE"),
        "unexpected W2 cycle diagnostic: {error}"
    );
}

#[test]
fn packet_17_cycle_boundary_requires_all_documents_accepted() {
    let documents = [
        ("SPEC-17", "Accepted"),
        ("SPEC-18", "Accepted"),
        ("SPEC-19", "Proposed"),
        ("SPEC-20", "Accepted"),
        ("ADR-018", "Accepted"),
        ("ADR-019", "Accepted"),
        ("ADR-020", "Accepted"),
        ("ADR-021", "Accepted"),
    ]
    .map(|(id, status)| (id, architecture_document(id, status, "Architecture", "")));
    let borrowed = documents
        .iter()
        .map(|(id, body)| (format!("{id}.md"), body.as_str()))
        .collect::<Vec<_>>();
    let borrowed = borrowed
        .iter()
        .map(|(path, body)| (path.as_str(), *body))
        .collect::<Vec<_>>();
    let error = validate_packet_17_normative_graph_documents(&borrowed)
        .expect_err("all eight packet-1.7 documents must be Accepted");
    assert!(
        error.contains("DOCS_PACKET_17_ACCEPTED_DOCUMENT_STATUS_INVALID"),
        "unexpected W2 status diagnostic: {error}"
    );
}

#[test]
fn index_metadata_mismatch_is_rejected() {
    let error =
        validate_index_entry_line(INDEX_MISMATCH, VALID_A).expect_err("index mismatch must fail");
    assert!(error.contains("DOCS_INDEX_ID_MISMATCH"));
}

#[test]
fn broken_supersession_backlink_is_rejected() {
    let error = validate_supersession_document(BROKEN_SUPERSESSION)
        .expect_err("missing backlink must fail");
    assert!(error.contains("DOCS_SUPERSESSION_BACKLINK_MISSING"));
}

#[test]
fn supersession_chain_may_cross_superseded_intermediate() {
    let adr_007 =
        supersession_document("ADR-007", "Superseded", "отсутствует", "[ADR-012](012.md)");
    let adr_012 = supersession_document(
        "ADR-012",
        "Superseded",
        "[ADR-007](007.md)",
        "[ADR-022](022.md)",
    );
    let adr_022 = supersession_document("ADR-022", "Accepted", "[ADR-012](012.md)", "не заменён");

    validate_supersession_documents(&[
        ("adr/007.md", &adr_007),
        ("adr/012.md", &adr_012),
        ("adr/022.md", &adr_022),
    ])
    .expect("historical chain must terminate at its Accepted replacement");
}

#[test]
fn supersession_cycle_is_rejected() {
    let adr_a = supersession_document("ADR-A", "Superseded", "[ADR-B](b.md)", "[ADR-B](b.md)");
    let adr_b = supersession_document("ADR-B", "Superseded", "[ADR-A](a.md)", "[ADR-A](a.md)");
    let error = validate_supersession_documents(&[("adr/a.md", &adr_a), ("adr/b.md", &adr_b)])
        .expect_err("a supersession cycle can never terminate in Accepted");
    assert!(error.contains("DOCS_SUPERSESSION_CYCLE"));
}

#[test]
fn supersession_chain_requires_exact_forward_link() {
    let adr_a = supersession_document("ADR-A", "Superseded", "отсутствует", "[ADR-B](b.md)");
    let adr_b = supersession_document("ADR-B", "Accepted", "отсутствует", "не заменён");
    let error = validate_supersession_documents(&[("adr/a.md", &adr_a), ("adr/b.md", &adr_b)])
        .expect_err("replacement must point back to the exact superseded ADR");
    assert!(error.contains("DOCS_SUPERSESSION_FORWARDLINK_MISSING"));
}

#[test]
fn supersession_chain_rejects_proposed_or_rejected_terminal() {
    for terminal_status in ["Proposed", "Rejected"] {
        let adr_a = supersession_document("ADR-A", "Superseded", "отсутствует", "[ADR-B](b.md)");
        let adr_b = supersession_document("ADR-B", terminal_status, "[ADR-A](a.md)", "не заменён");
        let error = validate_supersession_documents(&[("adr/a.md", &adr_a), ("adr/b.md", &adr_b)])
            .expect_err("supersession chain must terminate in Accepted");
        assert!(
            error.contains("DOCS_SUPERSESSION_REPLACEMENT_NOT_ACCEPTED"),
            "unexpected error for {terminal_status}: {error}"
        );
    }
}

#[test]
fn composite_primary_owner_is_rejected() {
    let gates = BTreeSet::from(["DOCS-01".to_owned()]);
    let error = validate_traceability_text(COMPOSITE_OWNER, &gates)
        .expect_err("composite primary owner must fail");
    assert!(error.contains("DOCS_COMPOSITE_PRIMARY_OWNER"));
}

#[test]
fn unknown_gate_is_rejected() {
    let gates = BTreeSet::from(["DOCS-01".to_owned()]);
    let error =
        validate_traceability_text(UNKNOWN_GATE, &gates).expect_err("unknown gate must fail");
    assert!(error.contains("DOCS_UNKNOWN_GATE"));
}

#[test]
fn bad_annex_hash_is_detectable() {
    assert_ne!(
        sha256_hex(BAD_ANNEX),
        "90533ed15c4c1a5ef41a24f26f4d17cf8c59f467e07619316d3c9744f4d2d79b"
    );
    assert_eq!(
        sha256_hex(b"abc"),
        "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
    );
}

#[test]
fn missing_governance_status_is_rejected() {
    let security =
        "security_disclosure_status: BootstrapOutOfBand\npublic_contact_status: AwaitingCapability";
    let conduct =
        "Contributor Covenant version 2.1\nconduct_enforcement_status: BootstrapOutOfBand";
    let error = validate_governance_documents(MISSING_GOVERNANCE, security, conduct)
        .expect_err("missing governance marker must fail");
    assert!(error.contains("DOCS_GOVERNANCE_STATUS_MISSING"));
}

#[test]
fn gameplay_budget_sum_mismatch_is_rejected() {
    let error =
        validate_gameplay_budget_matrix(BUDGET_MISMATCH).expect_err("budget mismatch must fail");
    assert!(
        error.contains("DOCS_BUDGET_DEFAULT_MISMATCH")
            || error.contains("DOCS_BUDGET_SUM_MISMATCH")
    );
}

#[test]
fn review_transition_registry_supports_immediate_candidates_through_1_10() {
    validate_review_candidate_version("1.5.1", "1.6")
        .expect("packet 1.6 must immediately follow 1.5.1");
    validate_review_candidate_version("1.6", "1.7")
        .expect("candidate tree 1.6 must advertise review candidate 1.7");
    validate_review_candidate_version("1.9", "1.10")
        .expect("transition registry must extend through packet 1.10");
}

#[test]
fn review_candidate_must_be_immediate_successor() {
    let error = validate_review_candidate_version("1.6", "1.8")
        .expect_err("review candidate must not skip packet 1.7");
    assert!(error.contains("DOCS_REVIEW_CANDIDATE_NOT_IMMEDIATE"));
}

#[test]
fn technology_registry_requires_stable_unique_ids() {
    let registry = "| Technology ID | Technology | Статус | Gate / fallback |\n\
                    |---|---|---|---|\n\
                    | TECH-001 | Rust | Accepted | BASE-01 |\n\
                    | TECH-002 | Adapter | Proposed | CAND-P1 |\n";
    assert_eq!(
        validate_technology_registry(registry).expect("stable technology registry should pass"),
        (2, 1)
    );

    let duplicate = registry.replace("TECH-002", "TECH-001");
    let error = validate_technology_registry(&duplicate)
        .expect_err("duplicate stable technology ID must fail");
    assert!(error.contains("DOCS_TECHNOLOGY_ID_DUPLICATE"));
}

#[test]
fn accepted_projection_with_reserved_gaps_passes() {
    let traceability = traceability_with_rows(
        &[
            "| REQ-001 | first | Runtime | BASE-01 | VS-01 | report |".to_owned(),
            "| REQ-003 | third | Runtime | BASE-03 | VS-01 | report |".to_owned(),
        ],
        &["| REQ-002 | Reserved | SPEC-TEST | Permanent allocation |".to_owned()],
    );
    let counts = validate_traceability_allocations(&traceability)
        .expect("Accepted projection may have a permanently reserved gap");
    assert_eq!(counts, (2, 0));
}

#[test]
fn duplicate_or_reused_reservation_is_rejected() {
    let traceability = traceability_with_rows(
        &["| REQ-001 | first | Runtime | BASE-01 | VS-01 | report |".to_owned()],
        &["| REQ-001 | Reserved | SPEC-TEST | Reused allocation |".to_owned()],
    );
    let error = validate_traceability_allocations(&traceability)
        .expect_err("Accepted and reserved allocations must be globally unique");
    assert!(error.contains("DOCS_ID_ALLOCATION_REUSED"));
}

#[test]
fn conditional_reserved_id_is_rejected() {
    let traceability = traceability_with_rows(
        &["| REQ-001 | first | Runtime | BASE-01 | VS-01 | report |".to_owned()],
        &["| REQ-002 | Conditional | SPEC-TEST | If accepted |".to_owned()],
    );
    let error = validate_traceability_allocations(&traceability)
        .expect_err("conditional allocation must not reserve a reusable ID");
    assert!(error.contains("DOCS_RESERVED_ID_STATUS_INVALID"));
}

#[test]
fn missing_reserved_id_breaks_global_contiguity() {
    let traceability = traceability_with_rows(
        &[
            "| REQ-001 | first | Runtime | BASE-01 | VS-01 | report |".to_owned(),
            "| REQ-003 | third | Runtime | BASE-03 | VS-01 | report |".to_owned(),
        ],
        &[],
    );
    let error = validate_traceability_allocations(&traceability)
        .expect_err("global allocation union must not contain a gap");
    assert!(error.contains("DOCS_NON_SEQUENTIAL_ID"));
}

#[test]
fn packet_16_requires_complete_reserved_high_water() {
    let traceability = packet_16_traceability();
    validate_packet_16_traceability_allocations(&traceability)
        .expect("packet 1.6 must preserve the exact Accepted and reserved allocation contract");

    let truncated = traceability.replace(
        "| FAIL-061 | Reserved | SPEC-30 | Permanent packet allocation |\n",
        "",
    );
    let error = validate_packet_16_traceability_allocations(&truncated)
        .expect_err("truncated packet allocation ledger must fail");
    assert!(
        error.contains("DOCS_NON_SEQUENTIAL_ID")
            || error.contains("DOCS_PACKET_16_RESERVED_SET_MISMATCH")
    );

    let wrong_owner = traceability.replace(
        "| REQ-087 | Reserved | SPEC-17 |",
        "| REQ-087 | Reserved | SPEC-18 |",
    );
    let error = validate_packet_16_traceability_allocations(&wrong_owner)
        .expect_err("reserved ranges have exact owning documents");
    assert!(error.contains("DOCS_PACKET_16_RESERVED_OWNER_MISMATCH"));

    let wrong_accepted_projection =
        traceability.replace("| REQ-111 | requirement |", "| REQ-112 | requirement |");
    let error = validate_packet_16_traceability_allocations(&wrong_accepted_projection)
        .expect_err("packet 1.6 Accepted IDs must match the exact projection");
    assert!(
        error.contains("DOCS_ID_ALLOCATION_REUSED")
            || error.contains("DOCS_PACKET_16_ACCEPTED_REQUIREMENT_SET_MISMATCH")
    );
}

#[test]
fn packet_17_requires_exact_accepted_and_reserved_projection() {
    let traceability = packet_17_traceability();
    assert_eq!(
        validate_packet_17_traceability_allocations(&traceability)
            .expect("packet 1.7 must admit exactly the fixed foundation projection"),
        (103, 37)
    );

    let incorrectly_reserved = traceability.replace(
        "| REQ-087 | requirement | Runtime | SPEC-17 | BASE-01 | VS-12 | report |",
        "| REQ-087 | Reserved | SPEC-17 | Permanent packet allocation |",
    );
    let error = validate_packet_17_traceability_allocations(&incorrectly_reserved)
        .expect_err("an Accepted packet-1.7 ID cannot remain reserved");
    assert!(
        error.contains("DOCS_PACKET_17_ACCEPTED_REQUIREMENT_SET_MISMATCH")
            || error.contains("DOCS_ID_ALLOCATION_REUSED")
            || error.contains("DOCS_TRACEABILITY_ROW_INVALID"),
        "unexpected packet-1.7 projection error: {error}"
    );

    let reused_future = traceability.replace(
        "| REQ-112 | Reserved | SPEC-22 | Permanent packet allocation |",
        "| REQ-112 | requirement | Runtime | SPEC-22 | BASE-01 | VS-12 | report |",
    );
    let error = validate_packet_17_traceability_allocations(&reused_future)
        .expect_err("future packet allocation must remain reserved");
    assert!(
        error.contains("DOCS_PACKET_17_ACCEPTED_REQUIREMENT_SET_MISMATCH")
            || error.contains("DOCS_ID_ALLOCATION_REUSED")
            || error.contains("DOCS_RESERVED_ID_ROW_INVALID"),
        "unexpected future-allocation error: {error}"
    );
}

#[test]
fn packet_17_requires_exact_foundation_owner_document_edges() {
    let traceability = packet_17_traceability();
    for (id, expected, replacement) in [
        ("REQ-087", "SPEC-17", "SPEC-18"),
        ("REQ-091", "SPEC-18", "SPEC-19"),
        ("REQ-095", "SPEC-19", "SPEC-20"),
        ("REQ-099", "SPEC-20", "SPEC-17"),
        ("REQ-103", "SPEC-21", "SPEC-17"),
        ("FAIL-031", "SPEC-17", "SPEC-18"),
        ("FAIL-033", "SPEC-18", "SPEC-19"),
        ("FAIL-035", "SPEC-19", "SPEC-20"),
        ("FAIL-037", "SPEC-20", "SPEC-17"),
        ("FAIL-039", "SPEC-21", "SPEC-17"),
    ] {
        let row_prefix = format!("| {id} |");
        let mutated = traceability
            .lines()
            .map(|line| {
                if line.starts_with(&row_prefix) {
                    line.replacen(
                        &format!("| Runtime | {expected} |"),
                        &format!("| Runtime | {replacement} |"),
                        1,
                    )
                } else {
                    line.to_owned()
                }
            })
            .collect::<Vec<_>>()
            .join("\n");
        let error = validate_packet_17_traceability_allocations(&mutated)
            .expect_err("foundation range must retain its exact owning document edge");
        assert!(
            error.contains("DOCS_PACKET_17_ACCEPTED_OWNER_MISMATCH"),
            "unexpected owner error for {id}: {error}"
        );
    }
}

#[test]
fn id_shorthand_slash_is_rejected() {
    let traceability = traceability_with_rows(
        &["| REQ-001 | first | Runtime | BASE-01/02 | VS-01 | report |".to_owned()],
        &[],
    );
    let error = validate_traceability_allocations(&traceability)
        .expect_err("slash shorthand must be rejected before graph admission");
    assert!(error.contains("DOCS_GATE_ID_SHORTHAND"));
}

#[test]
fn id_shorthand_ellipsis_is_rejected() {
    let traceability = traceability_with_rows(
        &["| REQ-001 | first | Runtime | BASE-01…BASE-03 | VS-01 | report |".to_owned()],
        &[],
    );
    let error = validate_traceability_allocations(&traceability)
        .expect_err("ellipsis shorthand must be rejected before graph admission");
    assert!(error.contains("DOCS_GATE_ID_SHORTHAND"));
}

#[test]
fn omitted_gate_prefix_is_rejected() {
    let traceability = traceability_with_rows(
        &["| REQ-001 | first | Runtime | BASE-01, 02 | VS-01 | report |".to_owned()],
        &[],
    );
    let error = validate_traceability_allocations(&traceability)
        .expect_err("each gate list element must repeat its complete prefix");
    assert!(error.contains("DOCS_GATE_ID_INVALID"));
}

#[test]
fn trace_01_is_a_valid_gate_id() {
    let traceability = traceability_with_rows(
        &["| REQ-001 | first | Runtime | TRACE-01 | VS-01 | report |".to_owned()],
        &[],
    );
    validate_traceability_allocations(&traceability)
        .expect("two-digit TRACE-01 is an exact gate identifier");
}

#[test]
fn trace_001_document_id_is_not_a_gate_id() {
    let traceability = traceability_with_rows(
        &["| REQ-001 | first | Runtime | TRACE-001 | VS-01 | report |".to_owned()],
        &[],
    );
    let error = validate_traceability_allocations(&traceability)
        .expect_err("three-digit TRACE-001 is a document ID, not a gate");
    assert!(error.contains("DOCS_GATE_ID_INVALID"));
}

#[test]
fn evidence_01_is_a_valid_gate_but_evidence_001_is_not() {
    let accepted = traceability_with_rows(
        &["| REQ-001 | first | Verification | EVIDENCE-01 | VS-01 | report |".to_owned()],
        &[],
    );
    validate_traceability_allocations(&accepted)
        .expect("two-digit EVIDENCE-01 is an exact gate identifier");

    let document_id = accepted.replace("EVIDENCE-01", "EVIDENCE-001");
    let error = validate_traceability_allocations(&document_id)
        .expect_err("three-digit EVIDENCE-001 is a document ID, not a gate");
    assert!(error.contains("DOCS_GATE_ID_INVALID"));
}

#[test]
fn valid_requirement_gate_reverse_closure_passes() {
    let (traceability, descriptors, vertical) = valid_requirement_graph_fixture();
    validate_requirement_gate_documents(
        &traceability,
        &[("spec-test.md", &descriptors)],
        &vertical,
    )
    .expect("complete Accepted owner/gate/evidence/profile closure must pass");
}

#[test]
fn unknown_gate_is_rejected_by_requirement_graph() {
    let (traceability, descriptors, vertical) = valid_requirement_graph_fixture();
    let traceability = traceability.replace("BASE-01, PERF-01", "UNKNOWN-01, PERF-01");
    let error = validate_requirement_gate_documents(
        &traceability,
        &[("spec-test.md", &descriptors)],
        &vertical,
    )
    .expect_err("unknown graph edge must fail");
    assert!(error.contains("DOCS_UNKNOWN_GATE"));
}

#[test]
fn trace_owner_document_must_resolve_to_accepted_architecture() {
    let (traceability, descriptors, vertical) = valid_requirement_graph_fixture();
    let proposed = architecture_document("SPEC-01", "Proposed", "Research", "No descriptors.");
    for (mutated, documents, expected) in [
        (
            traceability.replace("SPEC-00", "SPEC-99"),
            vec![("spec-test.md", descriptors.as_str())],
            "DOCS_TRACEABILITY_DOCUMENT_UNKNOWN",
        ),
        (
            traceability.replace("SPEC-00", "SPEC-01"),
            vec![
                ("spec-test.md", descriptors.as_str()),
                ("spec-proposed.md", proposed.as_str()),
            ],
            "DOCS_TRACEABILITY_DOCUMENT_STATUS_INCOMPATIBLE",
        ),
    ] {
        let error = validate_requirement_gate_documents(&mutated, &documents, &vertical)
            .expect_err("trace RFC / ADR owner must resolve to an Accepted document");
        assert!(error.contains(expected), "unexpected error: {error}");
    }
}

#[test]
fn accepted_document_requirement_definition_must_biject_to_trace() {
    let (traceability, descriptors, vertical) = valid_requirement_graph_fixture();
    let definition_table = "\n| ID | Requirement | Primary owner | Blocking gates |\n\
                            |---|---|---|---|\n\
                            | REQ-111 | Integrated performance | Release | BASE-01, PERF-01 |\n";
    let owning = format!("{descriptors}{definition_table}");
    validate_requirement_gate_documents(&traceability, &[("spec-test.md", &owning)], &vertical)
        .expect("matching owning definition and TRACE edge must pass");

    for (mutated, expected) in [
        (
            owning.replace("| Release | BASE-01", "| Runtime | BASE-01"),
            "DOCS_REQUIREMENT_DEFINITION_OWNER_MISMATCH",
        ),
        (
            owning.replace("BASE-01, PERF-01 |", "BASE-01 |"),
            "DOCS_REQUIREMENT_DEFINITION_GATE_MISMATCH",
        ),
    ] {
        let error = validate_requirement_gate_documents(
            &traceability,
            &[("spec-test.md", &mutated)],
            &vertical,
        )
        .expect_err("owning definition drift must fail");
        assert!(error.contains(expected), "unexpected error: {error}");
    }
}

#[test]
fn orphan_accepted_gate_is_rejected() {
    let (traceability, descriptors, vertical) = valid_requirement_graph_fixture();
    let descriptors = descriptors.replace(
        "| PERF-01 | Release | exact | performance report | block |",
        "| PERF-01 | Release | exact | performance report | block |\n\
         | ORPHAN-01 | Verification | exact | orphan report | block |",
    );
    let error = validate_requirement_gate_documents(
        &traceability,
        &[("spec-test.md", &descriptors)],
        &vertical,
    )
    .expect_err("Accepted descriptor without reverse coverage must fail");
    assert!(error.contains("DOCS_ORPHAN_GATE"));
}

#[test]
fn conflicting_duplicate_accepted_gate_descriptor_is_rejected() {
    let (traceability, descriptors, vertical) = valid_requirement_graph_fixture();
    let duplicate = architecture_document(
        "SPEC-01",
        "Accepted",
        "Verification",
        &gate_table(&["| BASE-01 | Verification | different threshold | base report | block |"]),
    );
    let error = validate_requirement_gate_documents(
        &traceability,
        &[
            ("spec-test.md", &descriptors),
            ("spec-duplicate.md", &duplicate),
        ],
        &vertical,
    )
    .expect_err("conflicting full Accepted descriptors must fail");
    assert!(error.contains("DOCS_GATE_DESCRIPTOR_CONFLICT"));
}

#[test]
fn canonically_identical_duplicate_gate_descriptor_is_compatible() {
    let (traceability, descriptors, vertical) = valid_requirement_graph_fixture();
    let duplicate = architecture_document(
        "SPEC-01",
        "Accepted",
        "Verification",
        &gate_table(&["| BASE-01 | Verification | exact | base report | block |"]),
    );
    validate_requirement_gate_documents(
        &traceability,
        &[
            ("spec-test.md", &descriptors),
            ("spec-duplicate.md", &duplicate),
        ],
        &vertical,
    )
    .expect("byte-equivalent canonical descriptors may be repeated");
}

#[test]
fn proposed_only_gate_is_rejected_for_accepted_row() {
    let (traceability, descriptors, vertical) = valid_requirement_graph_fixture();
    let traceability = traceability.replace("BASE-01, PERF-01", "BASE-01, PROPOSED-01, PERF-01");
    let proposed = architecture_document(
        "SPEC-PROPOSED",
        "Proposed",
        "Research",
        &gate_table(&["| PROPOSED-01 | Research | exact | proposal report | fallback |"]),
    );
    let error = validate_requirement_gate_documents(
        &traceability,
        &[
            ("spec-test.md", &descriptors),
            ("spec-proposed.md", &proposed),
        ],
        &vertical,
    )
    .expect_err("unclassified Proposed-only gate must not enter Accepted closure");
    assert!(error.contains("DOCS_PROPOSED_ONLY_GATE"));
}

#[test]
fn rejected_gate_definition_is_status_incompatible() {
    let (traceability, descriptors, vertical) = valid_requirement_graph_fixture();
    let traceability = traceability.replace("BASE-01, PERF-01", "BASE-01, REJECTED-01, PERF-01");
    let rejected = architecture_document(
        "SPEC-REJECTED",
        "Rejected",
        "Research",
        &gate_table(&["| REJECTED-01 | Research | exact | rejection report | no fallback |"]),
    );
    let error = validate_requirement_gate_documents(
        &traceability,
        &[
            ("spec-test.md", &descriptors),
            ("spec-rejected.md", &rejected),
        ],
        &vertical,
    )
    .expect_err("Rejected gate definition must not satisfy Accepted closure");
    assert!(error.contains("DOCS_GATE_DEFINITION_STATUS_INCOMPATIBLE"));
}

#[test]
fn candidate_only_gate_cannot_satisfy_baseline_row() {
    let (_, descriptors, vertical) = valid_requirement_graph_fixture();
    let rows = vec![
        "| REQ-111 | Integrated performance | Release | BASE-01, PERF-01 | VS-12 | report |"
            .to_owned(),
        "| REQ-112 | Candidate-only baseline | Runtime | CAND-P1 | VS-12 | report |".to_owned(),
    ];
    let traceability = traceability_with_rows(&rows, &reservations(1, 110));
    let proposed = architecture_document(
        "SPEC-PROPOSED",
        "Proposed",
        "Research",
        &gate_table(&["| CAND-P1 | Research | exact | candidate report | fallback |"]),
    );
    let candidate_registry = architecture_document(
        "ADR-024",
        "Accepted",
        "Verification",
        "| Gate ID | Classification | Subject | Rationale |\n\
         |---|---|---|---|\n\
         | CAND-P1 | CandidateOnly | TECH-001 | Optional implementation |",
    );
    let error = validate_requirement_gate_documents(
        &traceability,
        &[
            ("spec-test.md", &descriptors),
            ("spec-proposed.md", &proposed),
            ("adr-024.md", &candidate_registry),
        ],
        &vertical,
    )
    .expect_err("CandidateOnly must not be the only baseline gate");
    assert!(error.contains("DOCS_CANDIDATE_ONLY_GATE_CANNOT_CLOSE"));
}

#[test]
fn candidate_only_requires_exact_technology_subject() {
    let (traceability, descriptors, vertical) = valid_requirement_graph_fixture();
    let proposed = architecture_document(
        "SPEC-PROPOSED",
        "Proposed",
        "Research",
        &gate_table(&["| CAND-P1 | Research | exact | candidate report | fallback |"]),
    );
    let candidate_registry = architecture_document(
        "ADR-024",
        "Accepted",
        "Verification",
        "| Gate ID | Classification | Subject | Rationale |\n\
         |---|---|---|---|\n\
         | CAND-P1 | CandidateOnly | optional adapter | Optional implementation |",
    );
    let error = validate_requirement_gate_documents(
        &traceability,
        &[
            ("spec-test.md", &descriptors),
            ("spec-proposed.md", &proposed),
            ("adr-024.md", &candidate_registry),
        ],
        &vertical,
    )
    .expect_err("CandidateOnly subject must start with a stable TECH-NNN ID");
    assert!(error.contains("DOCS_CANDIDATE_ONLY_SUBJECT_INVALID"));
}

#[test]
fn candidate_only_requires_matching_proposed_technology_row() {
    let (traceability, descriptors, vertical) = valid_requirement_graph_fixture();
    let proposed = architecture_document(
        "SPEC-PROPOSED",
        "Proposed",
        "Research",
        &gate_table(&["| CAND-P1 | Research | exact | candidate report | fallback |"]),
    );
    let candidate_registry = architecture_document(
        "ADR-024",
        "Accepted",
        "Verification",
        "| Gate ID | Classification | Subject | Rationale |\n\
         |---|---|---|---|\n\
         | CAND-P1 | CandidateOnly | TECH-001 | Optional implementation |",
    );
    let evidence = "| Technology ID | Technology | Статус | Gate / fallback |\n\
                    |---|---|---|---|\n\
                    | TECH-001 | Optional adapter | Proposed | CAND-P1; fallback: baseline |\n";
    validate_requirement_gate_documents_with_evidence(
        &traceability,
        &[
            ("spec-test.md", &descriptors),
            ("spec-proposed.md", &proposed),
            ("adr-024.md", &candidate_registry),
        ],
        &vertical,
        evidence,
    )
    .expect("CandidateOnly subject must resolve to its Proposed technology gate mapping");

    for (mutated, expected) in [
        (
            evidence.replace("TECH-001", "TECH-002"),
            "DOCS_CANDIDATE_ONLY_TECHNOLOGY_MISSING",
        ),
        (
            evidence.replace("Proposed", "Accepted"),
            "DOCS_CANDIDATE_ONLY_TECHNOLOGY_STATUS_INVALID",
        ),
        (
            evidence.replace("CAND-P1", "OTHER-P1"),
            "DOCS_CANDIDATE_ONLY_TECHNOLOGY_GATE_MISMATCH",
        ),
    ] {
        let error = validate_requirement_gate_documents_with_evidence(
            &traceability,
            &[
                ("spec-test.md", &descriptors),
                ("spec-proposed.md", &proposed),
                ("adr-024.md", &candidate_registry),
            ],
            &vertical,
            &mutated,
        )
        .expect_err("CandidateOnly technology registry drift must fail");
        let missing_subject_rejected_earlier = expected == "DOCS_CANDIDATE_ONLY_TECHNOLOGY_MISSING"
            && error.contains("DOCS_NON_SEQUENTIAL_ID");
        assert!(
            error.contains(expected) || missing_subject_rejected_earlier,
            "unexpected error: {error}"
        );
    }
}

#[test]
fn missing_gate_fallback_is_rejected() {
    let (traceability, descriptors, vertical) = valid_requirement_graph_fixture();
    let descriptors = descriptors.replace(
        "| BASE-01 | Verification | exact | base report | block |",
        "| BASE-01 | Verification | exact | base report | — |",
    );
    let error = validate_requirement_gate_documents(
        &traceability,
        &[("spec-test.md", &descriptors)],
        &vertical,
    )
    .expect_err("every Accepted gate needs a complete fallback");
    assert!(error.contains("DOCS_GATE_DESCRIPTOR_INCOMPLETE"));
}

#[test]
fn missing_owner_evidence_or_profile_closure_is_rejected() {
    let (traceability, descriptors, vertical) = valid_requirement_graph_fixture();
    for (needle, expected) in [
        ("| Release | BASE-01", "DOCS_TRACEABILITY_OWNER_MISSING"),
        ("| report |", "DOCS_TRACEABILITY_EVIDENCE_MISSING"),
        ("| VS-12 |", "DOCS_GATE_LIST_EMPTY"),
    ] {
        let mutated = match expected {
            "DOCS_TRACEABILITY_OWNER_MISSING" => {
                traceability.replace("| Release | SPEC-00 | BASE-01", "| — | SPEC-00 | BASE-01")
            }
            "DOCS_TRACEABILITY_EVIDENCE_MISSING" => traceability.replace("| report |", "| — |"),
            _ => traceability
                .lines()
                .map(|line| {
                    if line.starts_with("| REQ-111 ") {
                        line.replace("VS-12", "unknown-profile")
                    } else {
                        line.to_owned()
                    }
                })
                .collect::<Vec<_>>()
                .join("\n"),
        };
        let error = validate_requirement_gate_documents(
            &mutated,
            &[("spec-test.md", &descriptors)],
            &vertical,
        )
        .expect_err("missing closure field must fail");
        let closure_error = expected == "DOCS_GATE_LIST_EMPTY"
            && error.contains("DOCS_TRACEABILITY_CLOSURE_UNKNOWN");
        assert!(
            error.contains(expected) || closure_error,
            "{needle} mutation returned unexpected error: {error}"
        );
    }
}

#[test]
fn perf_01_must_map_from_req_111() {
    let (traceability, descriptors, vertical) = valid_requirement_graph_fixture();
    let traceability = traceability.replace("BASE-01, PERF-01", "BASE-01");
    let error = validate_requirement_gate_documents(
        &traceability,
        &[("spec-test.md", &descriptors)],
        &vertical,
    )
    .expect_err("REQ-111 must directly map PERF-01");
    assert!(error.contains("DOCS_PERF_REQUIREMENT_MAPPING_MISSING"));
}

#[test]
fn perf_01_must_be_blocking_vs_12_child() {
    let (traceability, descriptors, vertical) = valid_requirement_graph_fixture();
    let vertical = vertical.replace("BASE-01, PERF-01", "BASE-01");
    let error = validate_requirement_gate_documents(
        &traceability,
        &[("spec-test.md", &descriptors)],
        &vertical,
    )
    .expect_err("VS-12 must explicitly list PERF-01 as blocking child");
    assert!(error.contains("DOCS_PERF_BLOCKING_CHILD_MISSING"));
}

#[test]
fn every_vertical_child_must_be_exact_known_and_reverse_mapped() {
    let (traceability, descriptors, vertical) = valid_requirement_graph_fixture();
    for (mutated, expected) in [
        (
            vertical.replacen("BASE-01", "UNKNOWN-01", 1),
            "DOCS_VERTICAL_CHILD_UNKNOWN",
        ),
        (
            vertical.replacen("BASE-01", "BASE-01/BASE-02", 1),
            "DOCS_GATE_ID_SHORTHAND",
        ),
        (
            vertical.replacen("BASE-01", "BASE-01, BASE-01", 1),
            "DOCS_GATE_ID_DUPLICATE",
        ),
    ] {
        let error = validate_requirement_gate_documents(
            &traceability,
            &[("spec-test.md", &descriptors)],
            &mutated,
        )
        .expect_err("invalid SPEC-12 child catalog entry must fail");
        assert!(error.contains(expected), "unexpected error: {error}");
    }
}

#[test]
fn review_v2_exact_contract_passes() {
    validate_human_review_decision_v2_text(REVIEW_V2_ADR)
        .expect("ADR-023 must carry the exact closed V2 review contract");
}

#[test]
fn review_v2_payload_closed_field_sets_reject_tamper_despite_decoys() {
    for (original, replacement) in [
        (
            "evidence_bundle_sha256\nbaseline =",
            "evidence_digest\nbaseline =",
        ),
        (
            "  changeset_sha256\n  revision_kind",
            "  changeset_digest\n  revision_kind",
        ),
        (
            "  { kind = \"None\" }\n  | { kind = \"EvidenceBaseline\", sha256 }",
            "  { kind = \"None\", sha256 }\n  | { kind = \"EvidenceBaseline\", sha256 }",
        ),
        (
            "  class = \"vertical-v1-neutral\"\n  sha256",
            "  fixture_class = \"vertical-v1-neutral\"\n  sha256",
        ),
        (
            "review_category\ndecision =",
            "review_category\nreviewer_display_name\ndecision =",
        ),
        (
            "requirement_graph_sha256\ngate_descriptor_set_sha256",
            "requirement_graph_digest\ngate_descriptor_set_sha256",
        ),
        (
            "decision = \"Approve\" | \"Reject\" | \"NeedsChanges\"",
            "decision = \"Approve\" | \"Reject\" | \"NeedsChanges\" | \"Escalate\"",
        ),
    ] {
        let contract = review_contract_mutation_with_decoy(original, replacement);
        let error = validate_human_review_decision_v2_text(&contract)
            .expect_err("payload top-level and nested field sets must remain exact and closed");
        assert!(
            error.contains("DOCS_REVIEW_V2_PAYLOAD_SCHEMA_DRIFT"),
            "unexpected payload error for {original:?}: {error}"
        );
    }

    let closure_rule = "- object, `subject`, `baseline` и `fixture` являются closed schemas: unknown, duplicate, missing, `null` и implicit-default fields rejected;";
    let contract = review_contract_mutation_with_decoy(
        closure_rule,
        "- object and nested objects are generally closed;",
    );
    let error = validate_human_review_decision_v2_text(&contract)
        .expect_err("payload closure semantics must remain explicit");
    assert!(error.contains("DOCS_REVIEW_V2_PAYLOAD_CLOSURE_DRIFT"));
}

#[test]
fn review_v2_envelope_fields_and_literals_reject_tamper_despite_decoys() {
    for (original, replacement) in [
        (
            "schema = \"nextengine.attestation-envelope.v2\"\nalgorithm =",
            "schema = \"nextengine.attestation-envelope.v3\"\nalgorithm =",
        ),
        (
            "algorithm = \"ed25519\"\ndomain =",
            "algorithm = \"ed448\"\ndomain =",
        ),
        (
            "domain = \"nextengine.human-review-decision.v2\"\nproject_id",
            "domain = \"nextengine.human-review.v2\"\nproject_id",
        ),
        (
            "domain = \"nextengine.human-review-decision.v2\"\nproject_id",
            "domain = \"nextengine.human-review-decision.v2\"\ndomain = \"nextengine.human-review.v2\"\nproject_id",
        ),
        ("project_id\nkey_id", "project_id\nreviewer_key_id"),
        (
            "payload_type = \"HumanReviewDecisionV2\"\npayload_hash",
            "payload_type = \"HumanReviewDecision\"\npayload_hash",
        ),
        (
            "revocation_snapshot_hash\nsignature",
            "revocation_snapshot_hash\ncertificate_chain\nsignature",
        ),
    ] {
        let contract = review_contract_mutation_with_decoy(original, replacement);
        let error = validate_human_review_decision_v2_text(&contract)
            .expect_err("envelope fields and literal values must remain exact");
        assert!(
            error.contains("DOCS_REVIEW_V2_ENVELOPE_OR_PREIMAGE_DRIFT"),
            "unexpected envelope error for {original:?}: {error}"
        );
    }
}

#[test]
fn review_v2_signature_preimage_membership_and_order_reject_tamper_despite_decoys() {
    for (original, replacement) in [
        (
            "\"nextengine.human-attestation.v2\\0\"\n|| u32_le(schema.len)",
            "\"nextengine.human-attestation.v3\\0\"\n|| u32_le(schema.len)",
        ),
        ("|| u32_le(domain.len) || domain_ascii", "|| domain_ascii"),
        (
            "|| u32_le(project_id.len) || project_id_nfc_utf8\n|| u32_le(key_id.len) || key_id_ascii",
            "|| u32_le(key_id.len) || key_id_ascii\n|| u32_le(project_id.len) || project_id_nfc_utf8",
        ),
        (
            "|| payload_hash[32]\n|| trust_policy_hash[32]",
            "|| payload_hash[32]",
        ),
        (
            "|| trust_policy_hash[32]\n|| trust_manifest_hash[32]",
            "|| trust_manifest_hash[32]\n|| trust_policy_hash[32]",
        ),
        (
            "|| trust_manifest_hash[32]\n|| revocation_snapshot_hash[32]",
            "|| trust_manifest_hash[32]\n|| revocation_snapshot_hash_hex",
        ),
        (
            "|| u32_le(payload_type.len) || payload_type_ascii\n|| payload_hash[32]",
            "|| payload_hash[32]\n|| u32_le(payload_type.len) || payload_type_ascii",
        ),
    ] {
        let contract = review_contract_mutation_with_decoy(original, replacement);
        let error = validate_human_review_decision_v2_text(&contract)
            .expect_err("signature preimage members and order must remain exact");
        assert!(
            error.contains("DOCS_REVIEW_V2_ENVELOPE_OR_PREIMAGE_DRIFT"),
            "unexpected preimage error for {original:?}: {error}"
        );
    }
}

#[test]
fn review_v2_enum_and_admission_matrix_reject_tamper_despite_decoys() {
    for (original, replacement, expected) in [
        (
            "decision = \"Approve\" | \"Reject\" | \"NeedsChanges\"",
            "decision = \"Approved\" | \"Rejected\" | \"NeedsChanges\"",
            "DOCS_REVIEW_V2_PAYLOAD_SCHEMA_DRIFT",
        ),
        (
            "4. only `VerifiedDecision::Approve` with every required automatic gate `PASS` and complete valid evidence returns `Admit`;",
            "4. every verified decision returns `Admit`;",
            "DOCS_REVIEW_V2_ADMISSION_MATRIX_DRIFT",
        ),
        (
            "5. `VerifiedDecision::Reject` and `VerifiedDecision::NeedsChanges` are signed, cryptographically valid, machine-readable non-admitting feedback;",
            "5. `VerifiedDecision::Reject` and `VerifiedDecision::NeedsChanges` are signed admission tokens;",
            "DOCS_REVIEW_V2_ADMISSION_MATRIX_DRIFT",
        ),
        (
            "6. `Approve` with any automatic result other than `PASS` is rejected as `AUTO_GATE_NOT_PASS`; human decision cannot waive failure or missing capability.",
            "6. `Approve` may waive an automatic failure.",
            "DOCS_REVIEW_V2_ADMISSION_MATRIX_DRIFT",
        ),
        (
            "Success of signature verification alone MUST NOT emit gate `PASS`, mutate project state, merge, promote a baseline/model/package or publish a release.",
            "Success of signature verification alone MAY emit gate `PASS` and mutate project state.",
            "DOCS_REVIEW_V2_SIGNATURE_AUTHORITY_DRIFT",
        ),
    ] {
        let contract = review_contract_mutation_with_decoy(original, replacement);
        let error = validate_human_review_decision_v2_text(&contract)
            .expect_err("only Approve plus automatic PASS may admit or mutate");
        assert!(
            error.contains(expected),
            "unexpected decision/admission error for {original:?}: {error}"
        );
    }
}

#[test]
fn review_v1_historical_only_denial_rejects_tamper_despite_decoys() {
    for (original, replacement) in [
        (
            "Successful legacy verification returns `HistoricalVerified`, never `VerifiedDecision`, `PASS`, `Admit`, merge, promotion or baseline acceptance.",
            "Successful legacy verification returns `VerifiedDecision::Approve` and `Admit`.",
        ),
        (
            "Current admission policy MUST set `review_contract_major = 2`.",
            "Current admission policy MAY set `review_contract_major = 1`.",
        ),
        (
            "Supplying V1 to an admission path fails closed as `REVIEW_SCHEMA_HISTORICAL_ONLY`.",
            "Supplying V1 to an admission path is accepted.",
        ),
    ] {
        let contract = review_contract_mutation_with_decoy(original, replacement);
        let error = validate_human_review_decision_v2_text(&contract)
            .expect_err("V1 must stay read-only and impossible to admit");
        assert!(
            error.contains("DOCS_REVIEW_V1_ADMISSION_DENIAL_DRIFT"),
            "unexpected V1 error for {original:?}: {error}"
        );
    }
}

#[test]
fn current_review_contract_rejects_v1_operational_drift() {
    let adr = architecture_document(
        "ADR-023",
        "Accepted",
        "Security",
        &valid_review_v2_contract_body(),
    );
    let stale = architecture_document(
        "SPEC-15",
        "Accepted",
        "Verification",
        "Current admission uses HumanReviewDecisionV1.",
    );
    let error = validate_human_review_decision_v2_documents(&[
        ("adr-023.md", &adr),
        ("spec-15.md", &stale),
    ])
    .expect_err("V1 must not remain on an operational admission path");
    assert!(error.contains("DOCS_REVIEW_CONTRACT_VERSION_DRIFT"));
}

#[test]
fn historical_v1_reference_is_allowed() {
    let adr = architecture_document(
        "ADR-023",
        "Accepted",
        "Security",
        &valid_review_v2_contract_body(),
    );
    let history = architecture_document(
        "SPEC-15",
        "Accepted",
        "Verification",
        "HumanReviewDecisionV1 is historical-audit read-only.",
    );
    validate_human_review_decision_v2_documents(&[("adr-023.md", &adr), ("spec-15.md", &history)])
        .expect("explicit historical-only V1 reference is valid");
}

#[test]
fn historical_word_cannot_mask_v1_admission_authority() {
    let adr = architecture_document(
        "ADR-023",
        "Accepted",
        "Security",
        &valid_review_v2_contract_body(),
    );
    let stale = architecture_document(
        "SPEC-15",
        "Accepted",
        "Verification",
        "HumanReviewDecisionV1 is historical-audit read-only.\nIt may authorize admission.",
    );
    let error = validate_human_review_decision_v2_documents(&[
        ("adr-023.md", &adr),
        ("spec-15.md", &stale),
    ])
    .expect_err("a historical marker must not mask V1 admission authority");
    assert!(error.contains("DOCS_REVIEW_CONTRACT_VERSION_DRIFT"));
}

#[test]
fn deterministic_substrate_and_script_breaker_contracts_pass() {
    validate_deterministic_substrate_documents(
        DETERMINISTIC_SPEC,
        DETERMINISTIC_ADR,
        SCRIPTING_SPEC,
        EXTENSION_ADR,
    )
    .expect("Accepted deterministic substrate and circuit breaker contracts must be complete");
}

#[test]
fn command_body_self_field_tamper_is_rejected() {
    let tampered = DETERMINISTIC_SPEC.replace(
        "`command_id` либо `claimed_command_id`;",
        "`command_id` may be embedded in the body;",
    );
    let error = validate_deterministic_substrate_documents(
        &tampered,
        DETERMINISTIC_ADR,
        SCRIPTING_SPEC,
        EXTENSION_ADR,
    )
    .expect_err("command body must explicitly exclude both ID fields");
    assert!(error.contains("DOCS_DETERMINISTIC_COMMAND_BODY_SELF_FIELD"));
}

#[test]
fn command_id_preimage_tamper_is_rejected() {
    let tampered = DETERMINISTIC_SPEC.replace("u64_le(body_bytes.len)", "u32_le(body_bytes.len)");
    let error = validate_deterministic_substrate_documents(
        &tampered,
        DETERMINISTIC_ADR,
        SCRIPTING_SPEC,
        EXTENSION_ADR,
    )
    .expect_err("command ID preimage must retain exact u64 length framing");
    assert!(error.contains("DOCS_DETERMINISTIC_COMMAND_ID_PREIMAGE_DRIFT"));
}

#[test]
fn command_id_claim_is_recomputed_before_reservation_or_mutation() {
    for tampered in [
        DETERMINISTIC_SPEC.replace(
            "Validator MUST повторно построить `body_bytes`",
            "Validator MAY trust caller-owned `body_bytes`",
        ),
        DETERMINISTIC_SPEC.replace("`COMMAND_ID_MISMATCH`", "`COMMAND_ID_WARNING`"),
        DETERMINISTIC_SPEC.replace(
            "до ledger reservation и без gameplay mutation",
            "после ledger reservation и gameplay mutation",
        ),
    ] {
        let error = validate_deterministic_substrate_documents(
            &tampered,
            DETERMINISTIC_ADR,
            SCRIPTING_SPEC,
            EXTENSION_ADR,
        )
        .expect_err("claim mismatch must fail before reservation or gameplay mutation");
        assert!(
            error.contains("DOCS_DETERMINISTIC_COMMAND_ID_VALIDATION_DRIFT"),
            "unexpected command validation error: {error}"
        );
    }
}

#[test]
fn causal_identity_domains_and_fail_closed_provenance_are_pinned() {
    for tampered in [
        DETERMINISTIC_SPEC.replace("WorldIdentityManifestV1 {", "WorldIdentityManifestV2 {"),
        DETERMINISTIC_SPEC.replace(
            "\"nextengine.command-stream.v1\\0\"",
            "\"nextengine.command-stream.unversioned\"",
        ),
        DETERMINISTIC_SPEC.replace(
            "\"nextengine.event-id.v1\\0\"",
            "\"nextengine.event-id.v2\\0\"",
        ),
        DETERMINISTIC_SPEC.replace(
            "\"nextengine.persistent-id.v2\\0\"",
            "\"nextengine.persistent-id.random\\0\"",
        ),
        DETERMINISTIC_SPEC.replace(
            "Collision не получает salt/random fallback",
            "Collision получает random fallback",
        ),
        DETERMINISTIC_SPEC.replace(
            "для durable record — causal full command body hash",
            "для durable record — truncated command ID",
        ),
    ] {
        let error = validate_deterministic_substrate_documents(
            &tampered,
            DETERMINISTIC_ADR,
            SCRIPTING_SPEC,
            EXTENSION_ADR,
        )
        .expect_err("causal identity domain/provenance drift must fail closed");
        assert!(
            error.contains("DOCS_DETERMINISTIC_CAUSAL_IDENTITY_DRIFT"),
            "unexpected causal identity error: {error}"
        );
    }
}

#[test]
fn current_next_cutoff_assignments_and_queue_generation_are_pinned() {
    for tampered in [
        DETERMINISTIC_SPEC.replace(
            "wall-clock timestamp не решает сторону cutoff",
            "wall-clock timestamp решает сторону cutoff",
        ),
        DETERMINISTIC_SPEC.replace("`queue_generation = 0`", "`queue_generation = 1`"),
        DETERMINISTIC_SPEC.replace("ровно один checked\nincrement", "два unchecked\nincrement"),
        DETERMINISTIC_SPEC.replace(
            "`INGRESS_QUEUE_GENERATION_EXHAUSTED`",
            "`INGRESS_QUEUE_GENERATION_WRAPPED`",
        ),
        DETERMINISTIC_SPEC.replace(
            "Async result публикуется в тот же current/next cutoff",
            "Async result публикуется по worker completion order",
        ),
    ] {
        let error = validate_deterministic_substrate_documents(
            &tampered,
            DETERMINISTIC_ADR,
            SCRIPTING_SPEC,
            EXTENSION_ADR,
        )
        .expect_err("current/next cutoff and queue generation must remain exact");
        assert!(
            error.contains("DOCS_DETERMINISTIC_INGRESS_CUTOFF_DRIFT")
                || error.contains("DOCS_DETERMINISTIC_QUEUE_GENERATION_DRIFT"),
            "unexpected ingress error: {error}"
        );
    }
}

#[test]
fn ledger_policy_and_finalized_retry_tamper_are_rejected() {
    for tampered in [
        DETERMINISTIC_SPEC.replace("StrictHighWatermarkGapsFinal", "ReusableHighWatermarkGaps"),
        DETERMINISTIC_SPEC.replace(
            "Если sequence уже закрыта high-watermark и отсутствует в retained state, вернуть `COMMAND_SEQUENCE_FINALIZED`",
            "Если sequence закрыта high-watermark, разрешить её reuse",
        ),
        DETERMINISTIC_SPEC.replace(
            "retry после eviction возвращает только `COMMAND_SEQUENCE_FINALIZED`",
            "retry после eviction выполняется повторно",
        ),
    ] {
        let error = validate_deterministic_substrate_documents(
            &tampered,
            DETERMINISTIC_ADR,
            SCRIPTING_SPEC,
            EXTENSION_ADR,
        )
        .expect_err("ledger gaps and evicted retries must remain final");
        assert!(
            error.contains("DOCS_DETERMINISTIC_LEDGER"),
            "unexpected ledger error: {error}"
        );
    }
}

#[test]
fn ledger_archive_index_restart_and_collision_no_mutation_are_pinned() {
    for (tampered, expected) in [
        (
            DETERMINISTIC_SPEC.replace(
                "Union всех `identity_index.body.bindings.values().occurrences.body_hash` MUST точно совпадать с key set body archive",
                "Identity index and body archive MAY drift",
            ),
            "DOCS_DETERMINISTIC_LEDGER_ARCHIVE_CLOSURE_DRIFT",
        ),
        (
            DETERMINISTIC_SPEC.replace(
                "  4 occurrence_count: u64,\n}\n\nCommandIdentityIndexV1 {",
                "  4 occurrence_count: u64,\n  5 index_root: Hash256,\n}\n\nCommandIdentityIndexV1 {",
            ),
            "DOCS_DETERMINISTIC_LEDGER_IDENTITY_INDEX_SELF_REFERENCE",
        ),
        (
            DETERMINISTIC_SPEC.replace(
                "exact retry retained command возвращает receipt без republish events",
                "exact retry retained command republishes events",
            ),
            "DOCS_DETERMINISTIC_LEDGER_RESTART_DRIFT",
        ),
        (
            DETERMINISTIC_SPEC.replace(
                "`CommandLedgerV2`, включая body archive/index",
                "`CommandLedgerV2`, без body archive/index",
            ),
            "DOCS_DETERMINISTIC_LEDGER_RESTART_DRIFT",
        ),
        (
            DETERMINISTIC_SPEC.replace(
                "no partial gameplay mutation",
                "partial gameplay mutation is allowed",
            ),
            "DOCS_DETERMINISTIC_COMMAND_COLLISION_MUTATION_DRIFT",
        ),
    ] {
        let error = validate_deterministic_substrate_documents(
            &tampered,
            DETERMINISTIC_ADR,
            SCRIPTING_SPEC,
            EXTENSION_ADR,
        )
        .expect_err("ledger closure/restart/collision mutation drift must fail");
        assert!(
            error.contains(expected),
            "unexpected ledger error for {expected}: {error}"
        );
    }
}

#[test]
fn breaker_early_tick_and_1799_1800_boundaries_are_pinned() {
    for (current_tick, expected_window_start) in
        [(0_u64, 0_u64), (1, 0), (1_798, 0), (1_799, 0), (1_800, 1)]
    {
        assert_eq!(
            current_tick.saturating_sub(1_799),
            expected_window_start,
            "unexpected inclusive window start at tick {current_tick}"
        );
    }

    for tampered in [
        SCRIPTING_SPEC.replace(
            "inclusive sliding window `1_800` gameplay ticks",
            "inclusive sliding window `1_799` gameplay ticks",
        ),
        SCRIPTING_SPEC.replace(
            "current_tick.saturating_sub(1_799)",
            "current_tick - 1_799",
        ),
        SCRIPTING_SPEC.replace(
            "ticks `0…1_798` lower bound равен `0`",
            "ticks `1…1_798` lower bound равен `1`",
        ),
        SCRIPTING_SPEC.replace(
            "начиная с tick `1_799` inclusive window всегда содержит не более `1_800` gameplay ticks",
            "начиная с tick `1_800` inclusive window всегда содержит не более `1_799` gameplay ticks",
        ),
    ] {
        let error = validate_deterministic_substrate_documents(
            DETERMINISTIC_SPEC,
            DETERMINISTIC_ADR,
            &tampered,
            EXTENSION_ADR,
        )
        .expect_err("inclusive breaker boundary must not drift");
        assert!(error.contains("DOCS_SCRIPT_BREAKER_CONTRACT_DRIFT"));
    }

    let tampered_adr =
        EXTENSION_ADR.replace("current_tick.saturating_sub(1_799)", "current_tick - 1_799");
    let error = validate_deterministic_substrate_documents(
        DETERMINISTIC_SPEC,
        DETERMINISTIC_ADR,
        SCRIPTING_SPEC,
        &tampered_adr,
    )
    .expect_err("ADR-014 must retain the same saturating early-tick boundary");
    assert!(error.contains("DOCS_SCRIPT_BREAKER_CONTRACT_DRIFT"));
}

#[test]
fn watchdog_strike_or_pass_tamper_is_rejected() {
    for tampered in [
        SCRIPTING_SPEC.replace(
            "NonConforming(EXTENSION_WALL_WATCHDOG)",
            "AuthoritativeStrike(EXTENSION_WALL_WATCHDOG)",
        ),
        SCRIPTING_SPEC.replace("не может дать PASS", "может дать PASS"),
    ] {
        let error = validate_deterministic_substrate_documents(
            DETERMINISTIC_SPEC,
            DETERMINISTIC_ADR,
            &tampered,
            EXTENSION_ADR,
        )
        .expect_err("wall watchdog must remain NonConforming without strike or PASS");
        assert!(error.contains("DOCS_SCRIPT_WATCHDOG_CONTRACT_DRIFT"));
    }

    for tampered_adr in [
        EXTENSION_ADR.replace(
            "не создавать обычный gameplay strike",
            "создать обычный gameplay strike",
        ),
        EXTENSION_ADR.replace("запретить PASS", "разрешить PASS"),
    ] {
        let error = validate_deterministic_substrate_documents(
            DETERMINISTIC_SPEC,
            DETERMINISTIC_ADR,
            SCRIPTING_SPEC,
            &tampered_adr,
        )
        .expect_err("ADR watchdog contract must never create a strike or PASS");
        assert!(error.contains("DOCS_SCRIPT_WATCHDOG_CONTRACT_DRIFT"));
    }
}

#[test]
fn rng_schedule_and_numeric_contract_tamper_is_rejected() {
    for (tampered, expected) in [
        (
            DETERMINISTIC_SPEC
                .replace("nextengine.chacha12-ietf.v1", "nextengine.chacha20-ietf.v1"),
            "DOCS_DETERMINISTIC_RNG_CONTRACT_DRIFT",
        ),
        (
            DETERMINISTIC_SPEC.replace(
                "lexicographically smallest canonical `SystemId`",
                "first registered `SystemId`",
            ),
            "DOCS_DETERMINISTIC_SCHEDULE_CONTRACT_DRIFT",
        ),
        (
            DETERMINISTIC_SPEC.replace(
                "checked signed `i128` numerator/denominator",
                "native float numerator/denominator",
            ),
            "DOCS_DETERMINISTIC_NUMERIC_CONTRACT_DRIFT",
        ),
    ] {
        let error = validate_deterministic_substrate_documents(
            &tampered,
            DETERMINISTIC_ADR,
            SCRIPTING_SPEC,
            EXTENSION_ADR,
        )
        .expect_err("deterministic substrate contract drift must fail");
        assert!(
            error.contains(expected),
            "unexpected substrate error: {error}"
        );
    }
}

#[test]
fn chacha12_round_order_endianness_exhaustion_and_vectors_are_pinned() {
    for (tampered, expected) in [
        (
            DETERMINISTIC_SPEC.replace(
                "a = a +% b; d = rotl(d xor a, 16)",
                "a = a +% b; d = rotl(d xor a, 15)",
            ),
            "DOCS_DETERMINISTIC_RNG_CONTRACT_DRIFT",
        ),
        (
            DETERMINISTIC_SPEC.replace(
                "QR(0,5,10,15); QR(1,6,11,12); QR(2,7,8,13); QR(3,4,9,14)",
                "QR(3,4,9,14); QR(2,7,8,13); QR(1,6,11,12); QR(0,5,10,15)",
            ),
            "DOCS_DETERMINISTIC_RNG_CONTRACT_DRIFT",
        ),
        (
            DETERMINISTIC_SPEC.replace(
                "ChaCha12 выполняет ровно 6 double rounds",
                "ChaCha12 выполняет ровно 10 double rounds",
            ),
            "DOCS_DETERMINISTIC_RNG_CONTRACT_DRIFT",
        ),
        (
            DETERMINISTIC_SPEC.replace(
                "64 bytes как 16 little-endian words",
                "64 bytes в native-endian order",
            ),
            "DOCS_DETERMINISTIC_RNG_CONTRACT_DRIFT",
        ),
        (
            DETERMINISTIC_SPEC.replace(
                "Следующий draw возвращает `RNG_STREAM_EXHAUSTED` без state mutation",
                "Следующий draw wraps counter and mutates state",
            ),
            "DOCS_DETERMINISTIC_RNG_CONTRACT_DRIFT",
        ),
        (
            DETERMINISTIC_SPEC.replace(
                "checked-in neutral golden vectors",
                "implementation-local smoke examples",
            ),
            "DOCS_DETERMINISTIC_RNG_GOLDEN_VECTOR_MISSING",
        ),
    ] {
        let error = validate_deterministic_substrate_documents(
            &tampered,
            DETERMINISTIC_ADR,
            SCRIPTING_SPEC,
            EXTENSION_ADR,
        )
        .expect_err("ChaCha12 exact profile and golden-vector anchor must not drift");
        assert!(
            error.contains(expected),
            "unexpected RNG error for {expected}: {error}"
        );
    }
}

#[test]
fn accepted_document_cannot_depend_on_superseded_adr() {
    let accepted = architecture_document("SPEC-X", "Accepted", "Runtime", "").replace(
        "| Нормативные зависимости | отсутствуют |",
        "| Нормативные зависимости | [ADR-OLD](adr-old.md) |",
    );
    let superseded = architecture_document("ADR-OLD", "Superseded", "Runtime", "");
    let error = validate_normative_graph_documents(&[
        ("spec-x.md", &accepted),
        ("adr-old.md", &superseded),
    ])
    .expect_err("Accepted document must depend on the Accepted replacement, not a Superseded ADR");
    assert!(error.contains("DOCS_NON_ACCEPTED_NORMATIVE_TARGET"));
}

#[test]
fn packet_17_foundation_semantic_contracts_pass() {
    validate_foundation_packet_17_documents(&foundation_documents())
        .expect("all four packet-1.7 foundation contracts must be decision-complete");
}

#[test]
fn project_resolution_and_lock_semantic_drift_is_rejected() {
    for (document, original, replacement) in [
        (
            "SPEC-17",
            "SemVer 2.0.0 precedence descending",
            "source order ascending",
        ),
        (
            "SPEC-17",
            "exact full prerelease SemVer в closed `prerelease_admissions`",
            "implicit prerelease admission",
        ),
        (
            "SPEC-17",
            "lexicographically smallest 32-byte `record_sha256`",
            "first downloaded record",
        ),
        (
            "SPEC-17",
            "equal invalid inputs MUST produce byte-identical report and diagnostic",
            "invalid inputs may produce local diagnostics",
        ),
        (
            "SPEC-17",
            "Runtime roots MUST NOT читать registry, network/cache state, выбирать package version или повторно разрешать floating range",
            "Runtime roots may consult a live registry",
        ),
        (
            "SPEC-17",
            "schema registry and runtime-determinism profile hashes",
            "runtime-local schema state",
        ),
    ] {
        let error = validate_foundation_mutation(document, original, replacement)
            .expect_err("project resolver/lock semantic drift must fail");
        assert!(
            error.contains("DOCS_PACKET_17_PROJECT_CONTRACT_DRIFT"),
            "unexpected project diagnostic: {error}"
        );
    }
}

#[test]
fn player_action_cutoff_and_presentation_authority_drift_is_rejected() {
    for (original, replacement) in [
        (
            "same `PlayerActionFrame` schema through the same gateway",
            "separate interactive and headless frame schemas",
        ),
        (
            "enqueue before the barrier belongs to current tick, while enqueue linearized in or after the barrier belongs to next tick",
            "wall time selects the target tick",
        ),
        (
            "The frame is production input, not committed gameplay",
            "The frame is committed gameplay authority",
        ),
        (
            "the data flow never reverses",
            "camera results may write targeting authority",
        ),
    ] {
        let error = validate_foundation_mutation("SPEC-18", original, replacement)
            .expect_err("player action/presentation authority drift must fail");
        assert!(
            error.contains("DOCS_PACKET_17_PLAYER_CONTRACT_DRIFT"),
            "unexpected player diagnostic: {error}"
        );
    }
}

#[test]
fn rpg_owner_operation_plan_order_and_migration_drift_is_rejected() {
    for (original, replacement) in [
        ("`RpgAggregateEnvelopeV1`", "`BackendAggregateRecord`"),
        (
            "contiguous `operation_slot: u32`",
            "arrival-ordered operation slot",
        ),
        (
            "schema = \"nextengine.rpg-transaction-plan.v1\"",
            "schema = \"implementation-local-plan\"",
        ),
        ("primary_persistent_id_bytes", "worker_completion_ordinal"),
        (
            "N-2 --migration[N-2,N-1]--> N-1 --migration[N-1,N]--> N",
            "N-2 --best-effort--> N",
        ),
        (
            "leaves original bytes/hash and prior published generation unchanged",
            "updates the original generation in place",
        ),
    ] {
        let error = validate_foundation_mutation("SPEC-19", original, replacement)
            .expect_err("RPG owner/transaction/migration semantic drift must fail");
        assert!(
            error.contains("DOCS_PACKET_17_RPG_CONTRACT_DRIFT"),
            "unexpected RPG diagnostic: {error}"
        );
    }
}

#[test]
fn world_calendar_migration_time_and_abstract_outcome_drift_is_rejected() {
    for (original, replacement) in [
        (
            "stored only in the World Services save segment",
            "mirrored in RPG and World Services segments",
        ),
        (
            "`world-calendar-rpg-to-world-services-v1`",
            "`calendar-best-effort-copy`",
        ),
        (
            "Stepped and bulk time use the same boundary evaluator, operation order and canonical plan partition",
            "bulk time uses a separate fast mutation path",
        ),
        (
            "emit one revision-bound upgrade proposal and leave the activity outcome/cursor uncommitted",
            "fabricate an abstract success and advance the cursor",
        ),
        (
            "next_retry_world_tick = current_world_tick + min(retry_interval, maximum_deferral_tick - current_world_tick)",
            "next_retry_world_tick = wall_clock_now",
        ),
    ] {
        let error = validate_foundation_mutation("SPEC-20", original, replacement)
            .expect_err("world calendar/time/abstract semantic drift must fail");
        assert!(
            error.contains("DOCS_PACKET_17_WORLD_CONTRACT_DRIFT"),
            "unexpected world diagnostic: {error}"
        );
    }
}

#[test]
fn packet_18_p0_semantic_contracts_pass() {
    validate_p0_packet_18_documents(&packet_18_documents())
        .expect("all thirteen packet-1.8 P0 contracts must be decision-complete");
}

#[test]
fn packet_18_non_accepted_document_is_rejected() {
    let error =
        validate_packet_18_mutation("SPEC-22", "| Статус | Accepted |", "| Статус | Proposed |")
            .expect_err("every packet-1.8 document must remain Accepted");
    assert!(
        error.contains("DOCS_PACKET_18_P0_DOCUMENT_STATUS_INVALID"),
        "unexpected packet status diagnostic: {error}"
    );
}

#[test]
fn packet_18_schema_immutable_field_and_migration_drift_is_rejected() {
    for (document, original, replacement) in [
        ("ADR-025", "`field_id`", "`mutable_field_name`"),
        ("SPEC-22", "`MigrationPlanV1`", "`InPlaceMigrationPlan`"),
        ("SPEC-22", "copy-on-write", "in-place rewrite"),
    ] {
        let error = validate_packet_18_mutation(document, original, replacement)
            .expect_err("schema immutable-field and migration contracts must not drift");
        assert!(
            error.contains("DOCS_PACKET_18_P0_CONTRACT_DRIFT") && error.contains(original),
            "unexpected schema/migration diagnostic: {error}"
        );
    }
}

#[test]
fn packet_18_resource_and_backpressure_contract_loss_is_rejected() {
    for (original, replacement) in [
        (
            "`ComputeResourceLeaseV1`",
            "`UnboundedComputeResourceLease`",
        ),
        (
            "`ArchiveDecompressionLimitsV1`",
            "`UnboundedArchiveDecompression`",
        ),
        ("IO-BACKPRESSURE-P1", "IO-BEST-EFFORT"),
    ] {
        let error = validate_packet_18_mutation("SPEC-23", original, replacement)
            .expect_err("resource admission and I/O backpressure contracts must not disappear");
        assert!(
            error.contains("DOCS_PACKET_18_P0_CONTRACT_DRIFT") && error.contains(original),
            "unexpected resource/backpressure diagnostic: {error}"
        );
    }
}

#[test]
fn packet_18_content_variant_fallback_contract_loss_is_rejected() {
    for (original, replacement) in [
        ("`TargetCapabilitySetV1`", "`AmbientHardwareProbe`"),
        ("`VariantFallbackPlanV1`", "`BestEffortVariantSearch`"),
        (
            "`fallback_target_profile_id`",
            "`implicit_fallback_profile`",
        ),
        (
            "`CONTENT_VARIANT_CAPABILITY_UNSATISFIED`",
            "`CONTENT_VARIANT_GUESSED`",
        ),
    ] {
        let error = validate_packet_18_mutation("SPEC-24", original, replacement)
            .expect_err("content target-profile fallback must remain deterministic and closed");
        assert!(
            error.contains("DOCS_PACKET_18_P0_CONTRACT_DRIFT") && error.contains(original),
            "unexpected content-variant diagnostic: {error}"
        );
    }
}

#[test]
fn packet_18_world_placement_seed_and_state_root_loss_is_rejected() {
    for (original, replacement) in [
        (
            "`initial_placement_catalog_hash`",
            "`mutable_placement_catalog_hash`",
        ),
        ("`WorldPlacementStateV1`", "`BackendPlacementRows`"),
        (
            "`SpatialPlacementStateRootV1`",
            "`UnverifiedPlacementCacheRoot`",
        ),
        (
            "`WORLD_PLACEMENT_ROOT_MISMATCH`",
            "`WORLD_PLACEMENT_BEST_EFFORT`",
        ),
    ] {
        let error = validate_packet_18_mutation("SPEC-25", original, replacement)
            .expect_err("immutable placement seed and mutable state root must remain separate");
        assert!(
            error.contains("DOCS_PACKET_18_P0_CONTRACT_DRIFT") && error.contains(original),
            "unexpected world-placement diagnostic: {error}"
        );
    }
}

#[test]
fn packet_18_physics_exact_classification_and_snapshot_loss_is_rejected() {
    for (original, replacement) in [
        ("`ExactCanonical`", "`BackendExact`"),
        ("`QuantizedExact`", "`BackendQuantized`"),
        ("`ToleranceDiagnosticOnly`", "`ToleranceAuthoritative`"),
        ("`PhysicsCanonicalSnapshotV1`", "`BackendPhysicsSnapshot`"),
    ] {
        let error = validate_packet_18_mutation("SPEC-26", original, replacement)
            .expect_err("physics comparison classes and canonical snapshots must remain explicit");
        assert!(
            error.contains("DOCS_PACKET_18_P0_CONTRACT_DRIFT") && error.contains(original),
            "unexpected physics diagnostic: {error}"
        );
    }
}

#[test]
fn packet_18_motor_canonical_batching_and_safety_loss_is_rejected() {
    for (original, replacement) in [
        (
            "(motor_tick, PersistentId, PolicyId)",
            "(worker_completion_time, RuntimeEntityId)",
        ),
        ("`MotorSafetyEnvelopeV1`", "`UncheckedMotorAction`"),
        ("`authoritative_state_hash`", "`recurrent_vector_hash`"),
        ("`PolicyStateCommitV1`", "`PartialPolicyStateWrite`"),
        ("`MOTOR_STATE_HASH_MISMATCH`", "`MOTOR_STATE_ACCEPT_TAMPER`"),
    ] {
        let error = validate_packet_18_mutation("SPEC-27", original, replacement)
            .expect_err("motor canonical batching and safety validation must remain explicit");
        assert!(
            error.contains("DOCS_PACKET_18_P0_CONTRACT_DRIFT") && error.contains(original),
            "unexpected motor diagnostic: {error}"
        );
    }
}

#[test]
fn packet_18_animation_root_motion_and_ik_loss_is_rejected() {
    for (original, replacement) in [
        ("`RootMotionIntentV1`", "`DirectRootTransformMutation`"),
        ("`PhysicalIkConstraintV1`", "`PresentationOnlyPhysicalIk`"),
        ("`PresentationIkRequestV1`", "`AuthoritativePresentationIk`"),
    ] {
        let error = validate_packet_18_mutation("SPEC-28", original, replacement)
            .expect_err("root-motion and physical/presentation IK contracts must remain explicit");
        assert!(
            error.contains("DOCS_PACKET_18_P0_CONTRACT_DRIFT") && error.contains(original),
            "unexpected animation diagnostic: {error}"
        );
    }
}

#[test]
fn packet_18_session_exactly_once_and_state_machine_loss_is_rejected() {
    for (original, replacement) in [
        ("CloseSessionReceiptV1", "BestEffortCloseReceipt"),
        ("SESSION-RECOVERY-P1", "SESSION-BEST-EFFORT"),
        ("CompositionStaged", "CompositionPending"),
        ("Finalizing", "ClosingEventually"),
        (
            "`CloseSessionOperationJournalV1`",
            "`BestEffortCloseProgress`",
        ),
        ("`canonical_close_request_hash`", "`save_subrequest_hash`"),
        (
            "`FinalSaveReservationBodyV1`",
            "`UnversionedSaveReservation`",
        ),
        (
            "nextengine.final-save-reservation.v1",
            "backend.final-save-reservation",
        ),
        (
            "nextengine.session-final-save-ledger.v1",
            "backend.session-final-save-ledger",
        ),
        ("`attempt_count: u16 = 0`", "`attempt_count = unknown`"),
        ("`k = attempt_count + 1`", "`k = wall_clock_retry + 1`"),
        ("`attempt_count = k`", "`attempt_count = backend_attempts`"),
        ("`FinalSaveReceiptV1`", "`OpaqueSaveReceipt`"),
        ("`RecoverySessionLinkV1`", "`UnboundNewSession`"),
        (
            "`prior_application_session_manifest_hash`",
            "`launcher_session_hint`",
        ),
        (
            "nextengine.application-session-state.v1",
            "backend.application-session-state",
        ),
        ("`CloseSessionResultV1`", "`UndefinedCloseResult`"),
        ("`RetryPending`", "`RetryByWallTimer`"),
        ("`FinalSaveRequiredFailed`", "`ClosedWithoutFinalSaveState`"),
    ] {
        let error = validate_packet_18_mutation("SPEC-29", original, replacement).expect_err(
            "session exactly-once close and state-machine contracts must remain explicit",
        );
        assert!(
            error.contains("DOCS_PACKET_18_P0_CONTRACT_DRIFT") && error.contains(original),
            "unexpected session diagnostic: {error}"
        );
    }
}

#[test]
fn packet_18_presentation_snapshot_material_color_vfx_and_cache_loss_is_rejected() {
    for (original, replacement) in [
        ("`PresentationSnapshotV2`", "`BackendSceneSnapshot`"),
        ("`MaterialDefinitionV1`", "`VendorMaterialDefinition`"),
        ("`SdrColorProfileV1`", "`ImplicitDisplayColor`"),
        ("VfxCueV1", "ImmediateBackendVfx"),
        (
            "CanonicalPresentationBatchV1",
            "WorkerLocalPresentationBatch",
        ),
        ("`PresentationCueEnvelopeV1`", "`UnorderedCueFamilyBatch`"),
        ("`cue_kind_rank`", "`backend_kind_order`"),
        (
            "`PresentationConsumptionStateV1`",
            "`InvalidatableVfxCacheState`",
        ),
        (
            "`continuous_instance_id_or_none`",
            "`implicit_continuous_target`",
        ),
        ("`target_cue_id_or_none`", "`latest_cue_guess`"),
        ("`acknowledgment_key`", "`snapshot_epoch_dedupe_key`"),
        ("`pending_one_shots`", "`ephemeral_pending_one_shots`"),
        ("`pending_one_shot_root`", "`unhashed_pending_one_shots`"),
        ("`PrimaryEligible`", "`RealizedBeforeBackend`"),
        (
            "`PresentationRealizationOutcomeV1`",
            "`MutableBackendOutcome`",
        ),
        (
            "`PresentationConsumptionEvidenceV1`",
            "`UnboundPresentationEvidence`",
        ),
        ("`SubmittedNotObserved`", "`PendingObservation`"),
        ("`presentation_snapshot_hash`", "`snapshot_epoch_only`"),
        (
            "`presentation_consumption_state_hash`",
            "`vfx_cache_state_guess`",
        ),
        (
            "nextengine.vfx-cue-prefix-empty.v1",
            "backend.vfx-prefix-empty",
        ),
        (
            "`PresentationCacheManifestV1`",
            "`MutablePresentationCache`",
        ),
    ] {
        let error = validate_packet_18_mutation("SPEC-30", original, replacement).expect_err(
            "presentation snapshot, material, color, VFX, and cache contracts must remain explicit",
        );
        assert!(
            error.contains("DOCS_PACKET_18_P0_CONTRACT_DRIFT") && error.contains(original),
            "unexpected presentation diagnostic: {error}"
        );
    }
}
