use std::collections::BTreeSet;

use xtask::docs_check::{
    sha256_hex, validate_gameplay_budget_matrix, validate_governance_documents,
    validate_index_entry_line, validate_normative_graph_documents, validate_supersession_document,
    validate_traceability_text,
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
