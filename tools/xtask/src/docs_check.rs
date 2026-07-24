use std::collections::{BTreeMap, BTreeSet};
use std::ffi::OsStr;
use std::fs;
use std::path::{Component, Path, PathBuf};

const ANNEX_PATH: &str = "research/physical-avatar-research-spec.md";
const ANNEX_ID: &str = "RESEARCH-001";
const ANNEX_SHA256: &str = "90533ed15c4c1a5ef41a24f26f4d17cf8c59f467e07619316d3c9744f4d2d79b";
const TEMPLATE_PATH: &str = "adr/000-template.md";
const LEGACY_PHYSICAL_RESEARCH_URL: &str = "https://github.com/kaifaty/OpenGothic/blob/c56e15f1fa68430eaa618dcc892edc00bff6209d/docs/physical-avatar-research-spec.md";
const ARCHITECTURE_REVIEW_ROOT: &str = "docs/reviews/architecture";
const ARCHITECTURE_REVIEW_ALGORITHM: &str = "sha256-path-nul-file-sha256-lf-v1";
const ARCHITECTURE_REVIEW_SCOPE: &str = "docs/architecture/**/*.md";
const ARCHITECTURE_APPROVED_ROOTS: &[(&str, &str)] = &[
    (
        "1.5",
        "ca4d9a0a8ccc4a7b062181f74a2a417022a13bca6c1bff9be138a0624abb14cd",
    ),
    (
        "1.5.1",
        "c0f76ba2eb594160a55608a5e029753144746f33d4f267d47c29c9f4e77ec443",
    ),
    (
        "1.6",
        "0826bd1f005676eec92e19ee70c1d0f176ac8f883fbfc2422027526e30927cd8",
    ),
    (
        "1.7",
        "a8c8f8820cafb135f6e9e513d20fcbe38fd175352f5808965e5ddd9c442855ba",
    ),
    (
        "1.8",
        "e04a4ef255599df1d3f8f6a96b9f9e0b6c0bf745a57ca63e68d5c0df5107ce5f",
    ),
];
const ARCHITECTURE_REVIEW_BASE_CHECKS: &[&str] = &[
    "cargo fmt --all -- --check",
    "cargo clippy --workspace --all-targets -- -D warnings",
    "cargo test --workspace",
    "cargo run -p xtask -- boundary-scan",
    "git diff --check",
];
#[derive(Clone, Copy, Debug)]
struct ArchitectureReviewTransition {
    from: &'static str,
    to: &'static str,
    file: &'static str,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ArchitectureReviewMode<'a> {
    AuthoritativeAdmission,
    CandidatePreflight { target: &'a str },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ReviewRecordMode {
    Historical,
    AuthoritativeAdmission,
    CandidatePreflight,
    Future,
}

#[derive(Debug)]
struct DocsCheckSummary {
    authoritative_version: String,
    candidate_version: String,
    architecture_file_count: usize,
    preflight_candidate_root: Option<String>,
}

const ARCHITECTURE_REVIEW_TRANSITIONS: &[ArchitectureReviewTransition] = &[
    ArchitectureReviewTransition {
        from: "1.4",
        to: "1.5",
        file: "packet-1.5.md",
    },
    ArchitectureReviewTransition {
        from: "1.5",
        to: "1.5.1",
        file: "packet-1.5.1.md",
    },
    ArchitectureReviewTransition {
        from: "1.5.1",
        to: "1.6",
        file: "packet-1.6.md",
    },
    ArchitectureReviewTransition {
        from: "1.6",
        to: "1.7",
        file: "packet-1.7.md",
    },
    ArchitectureReviewTransition {
        from: "1.7",
        to: "1.8",
        file: "packet-1.8.md",
    },
    ArchitectureReviewTransition {
        from: "1.8",
        to: "1.9",
        file: "packet-1.9.md",
    },
    ArchitectureReviewTransition {
        from: "1.9",
        to: "1.10",
        file: "packet-1.10.md",
    },
];

#[derive(Clone, Debug)]
struct Document {
    relative: String,
    body: String,
    id: String,
    status: String,
    version: String,
    owner: String,
    normative: String,
    supersedes: String,
    superseded_by: String,
}

pub fn docs_check(root: &Path) -> Result<(), String> {
    let summary = validate_docs(root, ArchitectureReviewMode::AuthoritativeAdmission)?;
    println!(
        "PASS docs-check: authoritative packet {}, candidate {}, {} indexed architecture documents",
        summary.authoritative_version, summary.candidate_version, summary.architecture_file_count
    );
    Ok(())
}

pub fn architecture_review_preflight(root: &Path, target: &str) -> Result<(), String> {
    let summary = validate_docs(root, ArchitectureReviewMode::CandidatePreflight { target })?;
    let candidate_root = summary
        .preflight_candidate_root
        .ok_or_else(|| format!("DOCS_REVIEW_PREFLIGHT_ROOT_MISSING: packet {target}"))?;
    println!(
        "PASS architecture-review-preflight: authoritative candidate packet {}, next candidate {}, root={candidate_root}",
        summary.authoritative_version, summary.candidate_version
    );
    Ok(())
}

fn validate_docs(
    root: &Path,
    review_mode: ArchitectureReviewMode<'_>,
) -> Result<DocsCheckSummary, String> {
    let docs_root = root.join("docs/architecture");
    let mut files = Vec::new();
    collect_files(&docs_root, Some("md"), &mut files)?;
    files.sort();

    let readme = read(&docs_root.join("README.md"))?;
    let summary = packet_summary(&readme)?;
    let authoritative_version = required_field(&readme, "Версия", "README.md")?;
    let candidate_version = required_field(&readme, "Review candidate", "README.md")?;
    validate_review_candidate_version(&authoritative_version, &candidate_version)?;
    if let ArchitectureReviewMode::CandidatePreflight { target } = review_mode
        && target != authoritative_version
    {
        return Err(format!(
            "DOCS_REVIEW_PREFLIGHT_TARGET_MISMATCH: candidate tree version {authoritative_version}, requested {target}"
        ));
    }
    let architecture_file_count = files
        .iter()
        .filter(|path| !path.ends_with(ANNEX_PATH))
        .count();
    require_count(
        "Markdown documents",
        architecture_file_count,
        summary_value(&summary, "Markdown documents")?,
    )?;

    let spec_count = files.iter().filter(|path| is_spec_file(path)).count();
    require_count(
        "subsystem SPEC files",
        spec_count,
        summary_value(&summary, "Subsystem SPEC files")?,
    )?;

    let decision_adr_count = files
        .iter()
        .filter(|path| {
            path.parent().is_some_and(|parent| parent.ends_with("adr"))
                && path.file_name() != Some(OsStr::new("000-template.md"))
        })
        .count();
    require_count(
        "decision ADR files",
        decision_adr_count,
        summary_value(&summary, "Decision ADR files")?,
    )?;
    validate_adr_sequence(&docs_root, &files)?;

    let mut documents = BTreeMap::new();
    let mut document_ids = BTreeSet::new();
    for file in &files {
        let relative = relative_path(&docs_root, file)?;
        let body = read(file)?;
        if relative != ANNEX_PATH && relative != TEMPLATE_PATH {
            reject_markers(file, &body)?;
            validate_relative_links(file, &body)?;
        }
        if relative == ANNEX_PATH || relative == TEMPLATE_PATH {
            continue;
        }
        let document = parse_document(relative.clone(), body)?;
        if !document_ids.insert(document.id.clone()) {
            return Err(format!("DOCS_DUPLICATE_ID: {}", document.id));
        }
        validate_status(&document)?;
        validate_supersession_document(&document.body)?;
        documents.insert(relative, document);
    }

    validate_readme_index(&docs_root, &readme, &files, &documents)?;
    validate_normative_graph(&docs_root, &documents)?;
    if matches!(authoritative_version.as_str(), "1.7" | "1.8") {
        validate_packet_17_accepted_cycle_contract(&documents)?;
    }
    if authoritative_version == "1.8" {
        validate_packet_18_accepted_cycle_contract(&documents)?;
    }
    validate_supersession_graph(&docs_root, &documents)?;
    validate_annex(root, &docs_root)?;
    validate_governance(root)?;

    let traceability = read(&docs_root.join("traceability.md"))?;
    let traceability_contract = parse_traceability_contract(&traceability)?;
    require_count(
        "requirements",
        traceability_contract.requirements.len(),
        summary_value(&summary, "Requirements")?,
    )?;
    require_count(
        "failure paths",
        traceability_contract.failures.len(),
        summary_value(&summary, "Failure paths")?,
    )?;
    validate_traceability_allocations_contract(&traceability_contract)?;
    validate_versioned_allocation_contract(&traceability_contract, &authoritative_version)?;

    let vertical = read(&docs_root.join("12-vertical-slice-conformance.md"))?;
    let vertical_ids = table_ids(&vertical, "VS-");
    require_count(
        "vertical gates",
        vertical_ids.len(),
        summary_value(&summary, "Vertical gates")?,
    )?;
    require_sequential(&vertical_ids, "VS-", 2)?;

    let evidence = read(&docs_root.join("evidence-register.md"))?;
    let technology_registry = parse_technology_registry(&evidence)?;
    validate_requirement_gate_contract(
        &traceability_contract,
        &documents,
        &vertical,
        Some(&technology_registry),
    )?;
    validate_human_review_decision_v2_contract(&documents)?;
    validate_deterministic_substrate_contract(&documents)?;
    if matches!(authoritative_version.as_str(), "1.7" | "1.8") {
        validate_foundation_packet_17_contract(&documents)?;
    }
    if authoritative_version == "1.8" {
        validate_data_substrate_packet_18_contract(&documents)?;
    }

    let budget_adr = read(&docs_root.join("adr/016-compositional-gameplay-budgets.md"))?;
    validate_gameplay_budget_matrix(&budget_adr)?;

    let technology_rows = technology_registry.len();
    let proposed_technology_rows = technology_registry
        .values()
        .filter(|entry| entry.status == "Proposed")
        .count();
    require_count(
        "technology rows",
        technology_rows,
        summary_value(&summary, "Technology rows")?,
    )?;
    require_count(
        "Proposed technology rows",
        proposed_technology_rows,
        summary_value(&summary, "Proposed technology rows")?,
    )?;

    let preflight_candidate_root =
        validate_architecture_review_records_with_mode(root, &authoritative_version, review_mode)?;
    Ok(DocsCheckSummary {
        authoritative_version,
        candidate_version,
        architecture_file_count,
        preflight_candidate_root,
    })
}

fn parse_document(relative: String, body: String) -> Result<Document, String> {
    let id = required_field(&body, "ID", &relative)?;
    let status = required_field(&body, "Статус", &relative)?;
    let version = required_field(&body, "Версия", &relative)?;
    let owner = required_field(&body, "Владелец", &relative)?;
    let normative = required_field(&body, "Нормативные зависимости", &relative)?;
    let supersedes = required_field(&body, "Заменяет", &relative)?;
    let superseded_by = table_field(&body, "Заменён").unwrap_or_else(|| "не заменён".to_owned());
    Ok(Document {
        relative,
        body,
        id,
        status,
        version,
        owner,
        normative,
        supersedes,
        superseded_by,
    })
}

fn required_field(body: &str, field: &str, relative: &str) -> Result<String, String> {
    table_field(body, field).ok_or_else(|| format!("DOCS_METADATA_MISSING: {relative}: {field}"))
}

fn table_field(body: &str, field: &str) -> Option<String> {
    let prefix = format!("| {field} | ");
    body.lines()
        .find_map(|line| line.strip_prefix(&prefix))
        .and_then(|value| value.strip_suffix(" |"))
        .map(str::trim)
        .map(str::to_owned)
}

fn validate_status(document: &Document) -> Result<(), String> {
    let allowed = ["Draft", "Proposed", "Accepted", "Rejected", "Superseded"];
    if !allowed.contains(&document.status.as_str()) {
        return Err(format!(
            "DOCS_STATUS_INVALID: {}: {}",
            document.relative, document.status
        ));
    }
    if document.version.is_empty()
        || document.owner.is_empty()
        || document.normative.is_empty()
        || document.supersedes.is_empty()
    {
        return Err(format!("DOCS_METADATA_EMPTY: {}", document.relative));
    }
    Ok(())
}

pub fn validate_supersession_document(body: &str) -> Result<(), String> {
    let status = table_field(body, "Статус").unwrap_or_default();
    if status == "Superseded" {
        let replacement = table_field(body, "Заменён").unwrap_or_default();
        if replacement.is_empty() || replacement == "не заменён" {
            return Err("DOCS_SUPERSESSION_BACKLINK_MISSING".to_owned());
        }
    }
    Ok(())
}

pub fn validate_supersession_documents(docs: &[(&str, &str)]) -> Result<(), String> {
    let mut documents = BTreeMap::new();
    for (relative, body) in docs {
        let relative = normalize_relative(relative);
        let document = parse_document(relative.clone(), (*body).to_owned())?;
        validate_status(&document)?;
        validate_supersession_document(&document.body)?;
        if documents.insert(relative.clone(), document).is_some() {
            return Err(format!("DOCS_DUPLICATE_PATH: {relative}"));
        }
    }
    validate_supersession_graph(Path::new("."), &documents)
}

fn validate_adr_sequence(docs_root: &Path, files: &[PathBuf]) -> Result<(), String> {
    let mut numbers = BTreeMap::new();
    for path in files.iter().filter(|path| {
        path.parent().is_some_and(|parent| parent.ends_with("adr"))
            && path.extension() == Some(OsStr::new("md"))
    }) {
        let name = path
            .file_name()
            .and_then(OsStr::to_str)
            .ok_or_else(|| format!("DOCS_ADR_FILENAME_INVALID: {}", path.display()))?;
        let prefix = name
            .get(..3)
            .ok_or_else(|| format!("DOCS_ADR_FILENAME_INVALID: {name}"))?;
        if name.as_bytes().get(3) != Some(&b'-')
            || !prefix.bytes().all(|byte| byte.is_ascii_digit())
        {
            return Err(format!("DOCS_ADR_FILENAME_INVALID: {name}"));
        }
        let number = prefix
            .parse::<usize>()
            .map_err(|error| format!("DOCS_ADR_FILENAME_INVALID: {name}: {error}"))?;
        if numbers.insert(number, name.to_owned()).is_some() {
            return Err(format!("DOCS_ADR_DUPLICATE_NUMBER: {prefix}"));
        }
    }
    for expected in 0..numbers.len() {
        if !numbers.contains_key(&expected) {
            return Err(format!("DOCS_ADR_SEQUENCE_GAP: ADR-{expected:03}"));
        }
    }
    let readme = read(&docs_root.join("README.md"))?;
    for (number, name) in numbers {
        let id = format!("ADR-{number:03}");
        let target = format!("adr/{name}");
        if !readme.lines().any(|line| {
            let cells = table_cells(line);
            cells.first() == Some(&id) && line.contains(&format!("]({target})"))
        }) {
            return Err(format!("DOCS_ADR_INDEX_MISSING: {id} -> {target}"));
        }
    }
    Ok(())
}

fn validate_readme_index(
    docs_root: &Path,
    readme: &str,
    files: &[PathBuf],
    documents: &BTreeMap<String, Document>,
) -> Result<(), String> {
    for file in files
        .iter()
        .filter(|file| file.file_name() != Some(OsStr::new("README.md")))
    {
        let relative = relative_path(docs_root, file)?;
        if !readme.contains(&format!("]({relative})")) {
            return Err(format!("DOCS_INDEX_ORPHAN: {relative}"));
        }
    }

    let mut indexed_ids = BTreeSet::new();
    let mut indexed_targets = BTreeSet::new();
    for line in readme.lines() {
        let cells = table_cells(line);
        if cells.len() < 3 || !is_index_id(&cells[0]) {
            continue;
        }
        if !indexed_ids.insert(cells[0].clone()) {
            return Err(format!("DOCS_INDEX_DUPLICATE_ID: {}", cells[0]));
        }
        let Some(target) = markdown_targets(&cells[1]).into_iter().next() else {
            return Err(format!("DOCS_INDEX_LINK_MISSING: {}", cells[0]));
        };
        if !indexed_targets.insert(target.clone()) {
            return Err(format!("DOCS_INDEX_DUPLICATE_TARGET: {target}"));
        }
        if is_frozen_annex_id(&cells[0]) {
            if target != ANNEX_PATH {
                return Err("DOCS_INDEX_RESEARCH_PATH_MISMATCH".to_owned());
            }
            continue;
        }
        if cells[0] == "ADR-000" {
            if target != TEMPLATE_PATH || cells[2] != "Accepted" {
                return Err("DOCS_INDEX_ADR_TEMPLATE_MISMATCH".to_owned());
            }
            continue;
        }
        let document = documents
            .get(&normalize_relative(&target))
            .ok_or_else(|| format!("DOCS_INDEX_TARGET_MISSING: {} -> {target}", cells[0]))?;
        validate_index_entry_line(line, &document.body)?;
    }
    let expected = files
        .iter()
        .filter(|file| !file.ends_with(ANNEX_PATH))
        .count();
    let indexed_architecture = indexed_ids
        .iter()
        .filter(|id| !is_frozen_annex_id(id))
        .count()
        + 1;
    require_count(
        "README indexed architecture documents",
        indexed_architecture,
        expected,
    )?;
    Ok(())
}

pub fn validate_index_entry_line(line: &str, target_body: &str) -> Result<(), String> {
    let cells = table_cells(line);
    if cells.len() < 3 {
        return Err("DOCS_INDEX_ROW_INVALID".to_owned());
    }
    let id = table_field(target_body, "ID").unwrap_or_default();
    let status = table_field(target_body, "Статус").unwrap_or_default();
    if cells[0] != id {
        return Err(format!("DOCS_INDEX_ID_MISMATCH: {} != {id}", cells[0]));
    }
    if cells[2] != status {
        return Err(format!(
            "DOCS_INDEX_STATUS_MISMATCH: {}: {} != {status}",
            cells[0], cells[2]
        ));
    }
    Ok(())
}

fn is_index_id(value: &str) -> bool {
    value.starts_with("SPEC-")
        || value.starts_with("ADR-")
        || has_numeric_suffix(value, "RESEARCH-")
        || matches!(value, "GLOSSARY-001" | "TRACE-001" | "EVIDENCE-001")
}

fn has_numeric_suffix(value: &str, prefix: &str) -> bool {
    value.strip_prefix(prefix).is_some_and(|suffix| {
        !suffix.is_empty() && suffix.bytes().all(|byte| byte.is_ascii_digit())
    })
}

fn is_frozen_annex_id(value: &str) -> bool {
    value == ANNEX_ID
}

fn validate_normative_graph(
    docs_root: &Path,
    documents: &BTreeMap<String, Document>,
) -> Result<(), String> {
    let pairs: Vec<_> = documents
        .values()
        .map(|document| (document.relative.as_str(), document.body.as_str()))
        .collect();
    validate_normative_graph_documents_at(docs_root, &pairs, true)
}

pub fn validate_normative_graph_documents(docs: &[(&str, &str)]) -> Result<(), String> {
    validate_normative_graph_documents_at(Path::new("."), docs, false)
}

pub fn validate_packet_17_normative_graph_documents(docs: &[(&str, &str)]) -> Result<(), String> {
    validate_normative_graph_documents_at(Path::new("."), docs, true)?;
    let parsed = docs
        .iter()
        .map(|(relative, body)| parse_document(normalize_relative(relative), (*body).to_owned()))
        .collect::<Result<Vec<_>, _>>()?;
    let documents = parsed
        .into_iter()
        .map(|document| (document.relative.clone(), document))
        .collect::<BTreeMap<_, _>>();
    validate_packet_17_accepted_cycle_contract(&documents)
}

fn validate_normative_graph_documents_at(
    docs_root: &Path,
    docs: &[(&str, &str)],
    allow_packet_14_legacy: bool,
) -> Result<(), String> {
    let known: BTreeMap<String, String> = docs
        .iter()
        .map(|(path, body)| {
            (
                normalize_relative(path),
                table_field(body, "Статус").unwrap_or_default(),
            )
        })
        .collect();
    let mut graph: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for (relative, body) in docs {
        let node = normalize_relative(relative);
        let normative = table_field(body, "Нормативные зависимости").unwrap_or_default();
        let external_targets: Vec<_> = markdown_targets(&normative)
            .into_iter()
            .filter(|target| target.starts_with("http://") || target.starts_with("https://"))
            .collect();
        let legacy_external_allowed = allow_packet_14_legacy
            && matches!(
                node.as_str(),
                "05-physics-animation-and-motor-control.md"
                    | "adr/004-physics-avatar-backend-boundary.md"
            )
            && external_targets.as_slice() == [LEGACY_PHYSICAL_RESEARCH_URL];
        if !external_targets.is_empty() && !legacy_external_allowed {
            return Err(format!("DOCS_EXTERNAL_NORMATIVE: {node}"));
        }
        let parent = Path::new(&node).parent().unwrap_or_else(|| Path::new(""));
        let mut edges = Vec::new();
        for target in markdown_targets(&normative) {
            let target = target.split('#').next().unwrap_or_default();
            if target.starts_with("http://") || target.starts_with("https://") {
                continue;
            }
            if target.is_empty() || !target.ends_with(".md") {
                continue;
            }
            let resolved = normalize_path(&docs_root.join(parent).join(target));
            let dependency = resolved
                .strip_prefix(normalize_path(docs_root))
                .unwrap_or(&resolved)
                .to_string_lossy()
                .trim_start_matches('/')
                .to_owned();
            let Some(dependency_status) = known.get(&dependency) else {
                return Err(format!(
                    "DOCS_NORMATIVE_TARGET_MISSING: {node} -> {dependency}"
                ));
            };
            if table_field(body, "Статус").as_deref() == Some("Accepted")
                && dependency_status != "Accepted"
            {
                return Err(format!(
                    "DOCS_NON_ACCEPTED_NORMATIVE_TARGET: {node} -> {dependency} ({dependency_status})"
                ));
            }
            edges.push(dependency);
        }
        graph.insert(node, edges);
    }

    if allow_packet_14_legacy {
        validate_w1_accepted_cycle_boundary(&graph, &known)?;
    }
    let cycle_graph = if allow_packet_14_legacy {
        graph
            .iter()
            .filter(|(node, _)| known.get(*node).is_some_and(|status| status != "Accepted"))
            .map(|(node, edges)| {
                let candidate_edges = edges
                    .iter()
                    .filter(|edge| known.get(*edge).is_some_and(|status| status != "Accepted"))
                    .cloned()
                    .collect();
                (node.clone(), candidate_edges)
            })
            .collect()
    } else {
        graph
    };
    let mut visiting = BTreeSet::new();
    let mut visited = BTreeSet::new();
    for node in cycle_graph.keys() {
        visit_graph(node, &cycle_graph, &mut visiting, &mut visited)?;
    }
    Ok(())
}

fn validate_packet_17_accepted_cycle_contract(
    documents: &BTreeMap<String, Document>,
) -> Result<(), String> {
    const PACKET_17_IDS: [&str; 8] = [
        "SPEC-17", "SPEC-18", "SPEC-19", "SPEC-20", "ADR-018", "ADR-019", "ADR-020", "ADR-021",
    ];
    validate_packet_accepted_cycle_contract(documents, &PACKET_17_IDS, "DOCS_PACKET_17")
}

fn validate_packet_18_accepted_cycle_contract(
    documents: &BTreeMap<String, Document>,
) -> Result<(), String> {
    const PACKET_18_IDS: [&str; 13] = [
        "SPEC-22", "SPEC-23", "SPEC-24", "SPEC-25", "SPEC-26", "SPEC-27", "SPEC-28", "SPEC-29",
        "SPEC-30", "ADR-025", "ADR-026", "ADR-027", "ADR-028",
    ];
    validate_packet_accepted_cycle_contract(documents, &PACKET_18_IDS, "DOCS_PACKET_18")
}

fn validate_packet_accepted_cycle_contract(
    documents: &BTreeMap<String, Document>,
    packet_ids: &[&str],
    code: &str,
) -> Result<(), String> {
    let by_id = documents
        .values()
        .map(|document| (document.id.as_str(), document))
        .collect::<BTreeMap<_, _>>();
    for id in packet_ids {
        let document = by_id
            .get(id)
            .ok_or_else(|| format!("{code}_ACCEPTED_DOCUMENT_MISSING: {id}"))?;
        if document.status != "Accepted" {
            return Err(format!(
                "{code}_ACCEPTED_DOCUMENT_STATUS_INVALID: {id}: {}",
                document.status
            ));
        }
    }

    let packet_ids = packet_ids.iter().copied().collect::<BTreeSet<_>>();
    let paths_to_ids = documents
        .values()
        .map(|document| (normalize_relative(&document.relative), document.id.as_str()))
        .collect::<BTreeMap<_, _>>();
    let mut graph = BTreeMap::new();
    for id in packet_ids.iter().copied() {
        let document = by_id[id];
        let parent = Path::new(&document.relative)
            .parent()
            .unwrap_or_else(|| Path::new(""));
        let mut edges = Vec::new();
        for target in markdown_targets(&document.normative) {
            let target = target.split('#').next().unwrap_or_default();
            if target.is_empty()
                || !target.ends_with(".md")
                || target.starts_with("http://")
                || target.starts_with("https://")
            {
                continue;
            }
            let resolved = normalize_path(&parent.join(target))
                .to_string_lossy()
                .trim_start_matches('/')
                .to_owned();
            if let Some(dependency_id) = paths_to_ids.get(&resolved)
                && packet_ids.contains(dependency_id)
            {
                edges.push((*dependency_id).to_owned());
            }
        }
        graph.insert(id.to_owned(), edges);
    }

    let mut visiting = BTreeSet::new();
    let mut visited = BTreeSet::new();
    for node in graph.keys() {
        if visit_graph(node, &graph, &mut visiting, &mut visited).is_err() {
            return Err(format!("{code}_GRAPH_CYCLE: {node}"));
        }
    }
    Ok(())
}

fn validate_w1_accepted_cycle_boundary(
    graph: &BTreeMap<String, Vec<String>>,
    statuses: &BTreeMap<String, String>,
) -> Result<(), String> {
    const W1_DOCUMENT_IDS: [&str; 4] = ["SPEC-21", "ADR-022", "ADR-023", "ADR-024"];
    let nodes_by_id = graph
        .keys()
        .filter_map(|node| {
            let file_name = Path::new(node).file_name()?.to_str()?;
            let body_id = if file_name.starts_with("21-") {
                Some("SPEC-21")
            } else if file_name.starts_with("022-") {
                Some("ADR-022")
            } else if file_name.starts_with("023-") {
                Some("ADR-023")
            } else if file_name.starts_with("024-") {
                Some("ADR-024")
            } else {
                None
            }?;
            Some((body_id, node.as_str()))
        })
        .collect::<BTreeMap<_, _>>();

    for document_id in W1_DOCUMENT_IDS {
        let Some(start) = nodes_by_id.get(document_id).copied() else {
            continue;
        };
        if statuses.get(start).map(String::as_str) != Some("Accepted") {
            continue;
        }
        let mut visited = BTreeSet::new();
        if path_reaches_start(start, start, graph, &mut visited, true) {
            return Err(format!(
                "DOCS_GRAPH_CYCLE: W1 Accepted dependency cycle includes {document_id} ({start})"
            ));
        }
    }
    Ok(())
}

fn path_reaches_start(
    start: &str,
    current: &str,
    graph: &BTreeMap<String, Vec<String>>,
    visited: &mut BTreeSet<String>,
    skip_origin_match: bool,
) -> bool {
    if !skip_origin_match && current == start {
        return true;
    }
    if !visited.insert(current.to_owned()) {
        return false;
    }
    graph.get(current).is_some_and(|edges| {
        edges
            .iter()
            .any(|edge| path_reaches_start(start, edge, graph, visited, false))
    })
}

fn visit_graph(
    node: &str,
    graph: &BTreeMap<String, Vec<String>>,
    visiting: &mut BTreeSet<String>,
    visited: &mut BTreeSet<String>,
) -> Result<(), String> {
    if visited.contains(node) || !graph.contains_key(node) {
        return Ok(());
    }
    if !visiting.insert(node.to_owned()) {
        return Err(format!("DOCS_GRAPH_CYCLE: {node}"));
    }
    if let Some(edges) = graph.get(node) {
        for edge in edges {
            visit_graph(edge, graph, visiting, visited)?;
        }
    }
    visiting.remove(node);
    visited.insert(node.to_owned());
    Ok(())
}

fn validate_supersession_graph(
    docs_root: &Path,
    documents: &BTreeMap<String, Document>,
) -> Result<(), String> {
    for document in documents.values() {
        let supersedes = resolve_metadata_link(docs_root, &document.relative, &document.supersedes);
        if let Some(target) = supersedes.as_ref() {
            let previous = documents.get(target).ok_or_else(|| {
                format!(
                    "DOCS_SUPERSESSION_TARGET_MISSING: {} -> {target}",
                    document.relative
                )
            })?;
            if previous.status != "Superseded" {
                return Err(format!(
                    "DOCS_SUPERSESSION_STATUS_MISMATCH: {} replaces {} ({})",
                    document.relative, previous.relative, previous.status
                ));
            }
            let backlink =
                resolve_metadata_link(docs_root, &previous.relative, &previous.superseded_by);
            if backlink.as_deref() != Some(document.relative.as_str()) {
                return Err(format!(
                    "DOCS_SUPERSESSION_BACKLINK_MISSING: {} -> {}",
                    previous.relative, document.relative
                ));
            }
        }

        if document.status == "Superseded" {
            let replacement =
                resolve_metadata_link(docs_root, &document.relative, &document.superseded_by)
                    .ok_or_else(|| {
                        format!("DOCS_SUPERSESSION_BACKLINK_MISSING: {}", document.relative)
                    })?;
            let replacement_document = documents.get(&replacement).ok_or_else(|| {
                format!(
                    "DOCS_SUPERSESSION_TARGET_MISSING: {} -> {replacement}",
                    document.relative
                )
            })?;
            if !matches!(
                replacement_document.status.as_str(),
                "Accepted" | "Superseded"
            ) {
                return Err(format!(
                    "DOCS_SUPERSESSION_REPLACEMENT_NOT_ACCEPTED: {} -> {replacement} ({})",
                    document.relative, replacement_document.status
                ));
            }
            let reverse = resolve_metadata_link(
                docs_root,
                &replacement_document.relative,
                &replacement_document.supersedes,
            );
            if reverse.as_deref() != Some(document.relative.as_str()) {
                return Err(format!(
                    "DOCS_SUPERSESSION_FORWARDLINK_MISSING: {} -> {replacement}",
                    document.relative
                ));
            }
        }
    }

    for document in documents
        .values()
        .filter(|document| document.status == "Superseded")
    {
        validate_supersession_chain(docs_root, documents, &document.relative)?;
    }
    Ok(())
}

fn validate_supersession_chain(
    docs_root: &Path,
    documents: &BTreeMap<String, Document>,
    start: &str,
) -> Result<(), String> {
    let mut visited = BTreeSet::new();
    let mut current = start.to_owned();
    loop {
        if !visited.insert(current.clone()) {
            return Err(format!("DOCS_SUPERSESSION_CYCLE: {start} -> {current}"));
        }
        let document = documents
            .get(&current)
            .ok_or_else(|| format!("DOCS_SUPERSESSION_TARGET_MISSING: {start} -> {current}"))?;
        match document.status.as_str() {
            "Accepted" => return Ok(()),
            "Superseded" => {
                current =
                    resolve_metadata_link(docs_root, &document.relative, &document.superseded_by)
                        .ok_or_else(|| {
                        format!("DOCS_SUPERSESSION_BACKLINK_MISSING: {}", document.relative)
                    })?;
            }
            status => {
                return Err(format!(
                    "DOCS_SUPERSESSION_REPLACEMENT_NOT_ACCEPTED: {start} -> {current} ({status})"
                ));
            }
        }
    }
}

fn resolve_metadata_link(docs_root: &Path, source: &str, value: &str) -> Option<String> {
    let target = markdown_targets(value).into_iter().find(|target| {
        !target.starts_with("http://")
            && !target.starts_with("https://")
            && target
                .split('#')
                .next()
                .is_some_and(|path| path.ends_with(".md"))
    })?;
    let target = target.split('#').next()?;
    let parent = Path::new(source).parent().unwrap_or_else(|| Path::new(""));
    let resolved = normalize_path(&docs_root.join(parent).join(target));
    resolved
        .strip_prefix(normalize_path(docs_root))
        .ok()
        .map(|path| path.to_string_lossy().replace('\\', "/"))
}

fn validate_annex(root: &Path, docs_root: &Path) -> Result<(), String> {
    let annex = fs::read(docs_root.join(ANNEX_PATH))
        .map_err(|error| format!("DOCS_ANNEX_MISSING: {error}"))?;
    let actual = sha256_hex(&annex);
    if actual != ANNEX_SHA256 {
        return Err(format!(
            "DOCS_ANNEX_HASH_MISMATCH: expected {ANNEX_SHA256}, got {actual}"
        ));
    }
    let provenance = read(&root.join("MIGRATION_PROVENANCE.md"))?;
    let notice = read(&root.join("THIRD_PARTY_NOTICES.md"))?;
    for required in [
        "c56e15f1fa68430eaa618dcc892edc00bff6209d",
        "ae922aa2a182f14108ae364714dc86072ecccbe8",
        ANNEX_SHA256,
        ANNEX_PATH,
    ] {
        if !provenance.contains(required) {
            return Err(format!("DOCS_ANNEX_PROVENANCE_MISSING: {required}"));
        }
    }
    if !notice.contains(ANNEX_PATH) || !notice.contains("MIT License") {
        return Err("DOCS_ANNEX_NOTICE_MISSING".to_owned());
    }
    Ok(())
}

fn validate_governance(root: &Path) -> Result<(), String> {
    let governance = read(&root.join("GOVERNANCE.md"))?;
    let security = read(&root.join("SECURITY.md"))?;
    let conduct = read(&root.join("CODE_OF_CONDUCT.md"))?;
    validate_governance_documents(&governance, &security, &conduct)
}

pub fn validate_governance_documents(
    governance: &str,
    security: &str,
    conduct: &str,
) -> Result<(), String> {
    for required in [
        "baseline.promote",
        "release_signing_status: AwaitingCapability",
        "public_contact_status: AwaitingCapability",
    ] {
        if !governance.contains(required) {
            return Err(format!("DOCS_GOVERNANCE_STATUS_MISSING: {required}"));
        }
    }
    if !security.contains("security_disclosure_status: BootstrapOutOfBand")
        || !security.contains("public_contact_status: AwaitingCapability")
    {
        return Err("DOCS_SECURITY_STATUS_MISSING".to_owned());
    }
    if !conduct.contains("Contributor Covenant")
        || !conduct.contains("version 2.1")
        || !conduct.contains("conduct_enforcement_status: BootstrapOutOfBand")
    {
        return Err("DOCS_CONDUCT_STATUS_MISSING".to_owned());
    }
    Ok(())
}

pub fn validate_architecture_review_records_at(
    root: &Path,
    authoritative_version: &str,
) -> Result<(), String> {
    validate_architecture_review_records_with_mode(
        root,
        authoritative_version,
        ArchitectureReviewMode::AuthoritativeAdmission,
    )?;
    Ok(())
}

pub fn validate_review_candidate_version(
    authoritative_version: &str,
    candidate_version: &str,
) -> Result<(), String> {
    validate_review_transition_registry()?;
    let transition = ARCHITECTURE_REVIEW_TRANSITIONS
        .iter()
        .find(|transition| transition.from == authoritative_version)
        .ok_or_else(|| format!("DOCS_REVIEW_VERSION_UNSUPPORTED: {authoritative_version}"))?;
    if transition.to != candidate_version {
        return Err(format!(
            "DOCS_REVIEW_CANDIDATE_NOT_IMMEDIATE: authoritative {authoritative_version}, expected {}, got {candidate_version}",
            transition.to
        ));
    }
    Ok(())
}

fn validate_review_transition_registry() -> Result<(), String> {
    let Some(first) = ARCHITECTURE_REVIEW_TRANSITIONS.first() else {
        return Err("DOCS_REVIEW_TRANSITION_REGISTRY_EMPTY".to_owned());
    };
    let mut versions = BTreeSet::from([first.from]);
    for (index, transition) in ARCHITECTURE_REVIEW_TRANSITIONS.iter().enumerate() {
        if index > 0 && ARCHITECTURE_REVIEW_TRANSITIONS[index - 1].to != transition.from {
            return Err(format!(
                "DOCS_REVIEW_TRANSITION_REGISTRY_GAP: {} -> {} followed by {} -> {}",
                ARCHITECTURE_REVIEW_TRANSITIONS[index - 1].from,
                ARCHITECTURE_REVIEW_TRANSITIONS[index - 1].to,
                transition.from,
                transition.to
            ));
        }
        if !versions.insert(transition.to) {
            return Err(format!(
                "DOCS_REVIEW_TRANSITION_REGISTRY_DUPLICATE: {}",
                transition.to
            ));
        }
        let expected_file = format!("packet-{}.md", transition.to);
        if transition.file != expected_file {
            return Err(format!(
                "DOCS_REVIEW_TRANSITION_FILE_MISMATCH: {} -> {}: expected {expected_file}, got {}",
                transition.from, transition.to, transition.file
            ));
        }
    }
    Ok(())
}

fn supported_review_versions() -> Result<Vec<&'static str>, String> {
    validate_review_transition_registry()?;
    let first = ARCHITECTURE_REVIEW_TRANSITIONS
        .first()
        .ok_or_else(|| "DOCS_REVIEW_TRANSITION_REGISTRY_EMPTY".to_owned())?;
    Ok(std::iter::once(first.from)
        .chain(
            ARCHITECTURE_REVIEW_TRANSITIONS
                .iter()
                .map(|transition| transition.to),
        )
        .collect())
}

fn validate_architecture_review_records_with_mode(
    root: &Path,
    authoritative_version: &str,
    mode: ArchitectureReviewMode<'_>,
) -> Result<Option<String>, String> {
    let approved_roots = ARCHITECTURE_APPROVED_ROOTS
        .iter()
        .map(|(version, root)| ((*version).to_owned(), (*root).to_owned()))
        .collect();
    validate_architecture_review_records_with_mode_and_anchors(
        root,
        authoritative_version,
        mode,
        &approved_roots,
    )
}

fn validate_architecture_review_records_with_mode_and_anchors(
    root: &Path,
    authoritative_version: &str,
    mode: ArchitectureReviewMode<'_>,
    approved_roots: &BTreeMap<String, String>,
) -> Result<Option<String>, String> {
    let supported_versions = supported_review_versions()?;
    let authoritative_index = supported_versions
        .iter()
        .position(|version| *version == authoritative_version)
        .ok_or_else(|| format!("DOCS_REVIEW_VERSION_UNSUPPORTED: {authoritative_version}"))?;

    let preflight_index = match mode {
        ArchitectureReviewMode::AuthoritativeAdmission => None,
        ArchitectureReviewMode::CandidatePreflight { target } => {
            if target != authoritative_version {
                return Err(format!(
                    "DOCS_REVIEW_PREFLIGHT_TARGET_MISMATCH: candidate tree version {authoritative_version}, requested {target}"
                ));
            }
            Some(
                ARCHITECTURE_REVIEW_TRANSITIONS
                    .iter()
                    .position(|transition| transition.to == target)
                    .ok_or_else(|| format!("DOCS_REVIEW_PREFLIGHT_TARGET_UNSUPPORTED: {target}"))?,
            )
        }
    };
    let mut preflight_candidate_root = None;

    for (index, transition) in ARCHITECTURE_REVIEW_TRANSITIONS.iter().enumerate() {
        let required_for_authoritative_packet = match preflight_index {
            Some(target_index) => index < target_index,
            None => index < authoritative_index,
        };
        let path = root.join(ARCHITECTURE_REVIEW_ROOT).join(transition.file);
        if !path.exists() {
            if required_for_authoritative_packet || preflight_index == Some(index) {
                return Err(format!(
                    "DOCS_REVIEW_RECORD_MISSING: {} -> {}: {}",
                    transition.from,
                    transition.to,
                    path.display()
                ));
            }
            continue;
        }

        let body = read(&path)?;
        let record_mode = if preflight_index == Some(index) {
            ReviewRecordMode::CandidatePreflight
        } else if required_for_authoritative_packet {
            if preflight_index.is_none() && transition.to == authoritative_version {
                ReviewRecordMode::AuthoritativeAdmission
            } else {
                ReviewRecordMode::Historical
            }
        } else {
            ReviewRecordMode::Future
        };
        let (status, candidate_root) = validate_architecture_review_record(
            root,
            *transition,
            &body,
            record_mode,
            approved_roots.get(transition.to).map(String::as_str),
        )?;
        if preflight_index == Some(index) {
            if status != "Pending" {
                return Err(format!(
                    "DOCS_REVIEW_PREFLIGHT_STATUS_INVALID: {} -> {}: expected Pending, got {status}",
                    transition.from, transition.to
                ));
            }
            preflight_candidate_root = Some(candidate_root);
            continue;
        }
        if required_for_authoritative_packet && status != "Approved" {
            return Err(format!(
                "DOCS_REVIEW_RECORD_NOT_APPROVED: {} -> {}: {status}",
                transition.from, transition.to
            ));
        }
        if !required_for_authoritative_packet && status == "Approved" {
            return Err(format!(
                "DOCS_REVIEW_TRANSITION_OUT_OF_SEQUENCE: authoritative {authoritative_version}, approved {} -> {}",
                transition.from, transition.to
            ));
        }
    }
    Ok(preflight_candidate_root)
}

fn validate_architecture_review_record(
    root: &Path,
    transition: ArchitectureReviewTransition,
    body: &str,
    mode: ReviewRecordMode,
    approved_root: Option<&str>,
) -> Result<(String, String), String> {
    let expected_id = format!("ARCH-REVIEW-{}", transition.to);
    require_review_field(body, "Record ID", &expected_id, transition)?;
    require_review_field(body, "From packet", transition.from, transition)?;
    require_review_field(body, "To packet", transition.to, transition)?;
    require_review_field(
        body,
        "Candidate root algorithm",
        ARCHITECTURE_REVIEW_ALGORITHM,
        transition,
    )?;
    require_review_field(
        body,
        "Candidate scope",
        ARCHITECTURE_REVIEW_SCOPE,
        transition,
    )?;

    let status = required_field(body, "Status", transition.file)?;
    if !matches!(status.as_str(), "Pending" | "Approved") {
        return Err(format!(
            "DOCS_REVIEW_STATUS_INVALID: {}: {status}",
            transition.file
        ));
    }
    let candidate_root = required_field(body, "Candidate root SHA-256", transition.file)?;

    let manifest_rows = review_section_rows(
        body,
        "## Candidate file manifest",
        &["Path", "SHA-256"],
        transition.file,
    )?;
    let manifest = validate_review_manifest(
        root,
        transition,
        &candidate_root,
        &manifest_rows,
        matches!(
            mode,
            ReviewRecordMode::AuthoritativeAdmission | ReviewRecordMode::CandidatePreflight
        ),
    )?;
    if mode == ReviewRecordMode::CandidatePreflight {
        if candidate_root == "absent" {
            return Err(format!(
                "DOCS_REVIEW_PREFLIGHT_CANDIDATE_ROOT_MISSING: {}",
                transition.file
            ));
        }
        if manifest.is_empty() {
            return Err(format!(
                "DOCS_REVIEW_PREFLIGHT_MANIFEST_EMPTY: {}",
                transition.file
            ));
        }
        if !manifest.contains_key("docs/architecture/README.md") {
            return Err(format!(
                "DOCS_REVIEW_PREFLIGHT_PACKET_INDEX_NOT_HASHED: {}",
                transition.file
            ));
        }
    }

    let check_rows = review_section_rows(
        body,
        "## Automatic checks",
        &["Check", "Result", "Evidence reference"],
        transition.file,
    )?;
    validate_review_checks(transition, &candidate_root, &status, &check_rows, mode)?;

    let capability_rows = review_section_rows(
        body,
        "## Bootstrap capability decisions",
        &["Capability", "Decision", "Reviewer", "Decision reference"],
        transition.file,
    )?;
    let decisions = validate_review_decisions(
        transition,
        &candidate_root,
        &status,
        &["architecture.promote"],
        &capability_rows,
        "CAPABILITY",
    )?;
    if mode == ReviewRecordMode::CandidatePreflight {
        let decision = decisions
            .get("architecture.promote")
            .map(|(decision, _, _)| decision.as_str())
            .unwrap_or_default();
        if decision != "Pending" {
            return Err(format!(
                "DOCS_REVIEW_PREFLIGHT_DECISION_INVALID: {}: expected Pending, got {decision}",
                transition.file
            ));
        }
    }

    if status == "Approved" {
        if candidate_root == "absent" || manifest.is_empty() {
            return Err(format!(
                "DOCS_REVIEW_APPROVED_WITHOUT_CANDIDATE: {}",
                transition.file
            ));
        }
        if !manifest.contains_key("docs/architecture/README.md") {
            return Err(format!(
                "DOCS_REVIEW_PACKET_INDEX_NOT_HASHED: {}",
                transition.file
            ));
        }
        if matches!(
            mode,
            ReviewRecordMode::Historical | ReviewRecordMode::AuthoritativeAdmission
        ) {
            let approved_root = approved_root.ok_or_else(|| {
                format!(
                    "DOCS_REVIEW_APPROVED_ROOT_UNANCHORED: {}: {}",
                    transition.file, candidate_root
                )
            })?;
            if candidate_root != approved_root {
                return Err(format!(
                    "DOCS_REVIEW_APPROVED_ROOT_MISMATCH: {}: expected {approved_root}, got {candidate_root}",
                    transition.file
                ));
            }
        }
    }

    Ok((status, candidate_root))
}

fn require_review_field(
    body: &str,
    field: &str,
    expected: &str,
    transition: ArchitectureReviewTransition,
) -> Result<(), String> {
    let actual = required_field(body, field, transition.file)?;
    if actual == expected {
        Ok(())
    } else {
        Err(format!(
            "DOCS_REVIEW_TRANSITION_MISMATCH: {}: {field}: expected {expected}, got {actual}",
            transition.file
        ))
    }
}

fn review_section_rows(
    body: &str,
    heading: &str,
    expected_header: &[&str],
    relative: &str,
) -> Result<Vec<Vec<String>>, String> {
    let mut lines = body.lines().skip_while(|line| line.trim() != heading);
    if lines.next().is_none() {
        return Err(format!(
            "DOCS_REVIEW_SECTION_MISSING: {relative}: {heading}"
        ));
    }
    let section: Vec<_> = lines
        .take_while(|line| !line.trim_start().starts_with("## "))
        .filter(|line| line.trim_start().starts_with('|'))
        .map(table_cells)
        .collect();
    if section.len() < 2
        || section[0]
            != expected_header
                .iter()
                .map(|header| (*header).to_owned())
                .collect::<Vec<_>>()
        || section[1].len() != expected_header.len()
        || !section[1]
            .iter()
            .all(|cell| !cell.is_empty() && cell.bytes().all(|byte| matches!(byte, b'-' | b':')))
    {
        return Err(format!("DOCS_REVIEW_TABLE_INVALID: {relative}: {heading}"));
    }
    Ok(section.into_iter().skip(2).collect())
}

fn validate_review_manifest(
    root: &Path,
    transition: ArchitectureReviewTransition,
    candidate_root: &str,
    rows: &[Vec<String>],
    compare_current_tree: bool,
) -> Result<BTreeMap<String, String>, String> {
    if candidate_root == "absent" {
        if rows == [vec!["none".to_owned(), "absent".to_owned()]] || rows.is_empty() {
            return Ok(BTreeMap::new());
        }
        return Err(format!(
            "DOCS_REVIEW_MANIFEST_WITHOUT_ROOT: {}",
            transition.file
        ));
    }
    if !is_lower_sha256(candidate_root) {
        return Err(format!(
            "DOCS_REVIEW_CANDIDATE_ROOT_INVALID: {}: {candidate_root}",
            transition.file
        ));
    }

    let mut manifest = BTreeMap::new();
    let mut previous_path: Option<&str> = None;
    for row in rows {
        if row.len() != 2 {
            return Err(format!(
                "DOCS_REVIEW_MANIFEST_ROW_INVALID: {}",
                transition.file
            ));
        }
        let path = &row[0];
        let expected_hash = &row[1];
        if previous_path.is_some_and(|previous| previous >= path.as_str()) {
            return Err(format!(
                "DOCS_REVIEW_MANIFEST_NOT_SORTED: {}: {path}",
                transition.file
            ));
        }
        previous_path = Some(path);
        if !is_safe_manifest_path(path)
            || !path.starts_with("docs/architecture/")
            || !path.ends_with(".md")
            || path.starts_with(&format!("{ARCHITECTURE_REVIEW_ROOT}/"))
        {
            return Err(format!(
                "DOCS_REVIEW_MANIFEST_PATH_INVALID: {}: {path}",
                transition.file
            ));
        }
        if !is_lower_sha256(expected_hash) {
            return Err(format!(
                "DOCS_REVIEW_FILE_HASH_INVALID: {}: {path}",
                transition.file
            ));
        }
        if manifest
            .insert(path.clone(), expected_hash.clone())
            .is_some()
        {
            return Err(format!(
                "DOCS_REVIEW_MANIFEST_DUPLICATE: {}: {path}",
                transition.file
            ));
        }
        if compare_current_tree {
            let bytes = fs::read(root.join(path)).map_err(|error| {
                format!(
                    "DOCS_REVIEW_CANDIDATE_FILE_MISSING: {}: {path}: {error}",
                    transition.file
                )
            })?;
            let actual_hash = sha256_hex(&bytes);
            if actual_hash != *expected_hash {
                return Err(format!(
                    "DOCS_REVIEW_FILE_HASH_MISMATCH: {}: {path}: expected {expected_hash}, got {actual_hash}",
                    transition.file
                ));
            }
        }
    }
    if manifest.is_empty() {
        return Err(format!("DOCS_REVIEW_MANIFEST_EMPTY: {}", transition.file));
    }
    if !manifest.contains_key("docs/architecture/README.md") {
        return Err(format!(
            "DOCS_REVIEW_PACKET_INDEX_NOT_HASHED: {}",
            transition.file
        ));
    }
    if compare_current_tree {
        let mut candidate_files = Vec::new();
        collect_files(
            &root.join("docs/architecture"),
            Some("md"),
            &mut candidate_files,
        )?;
        let expected_paths: BTreeSet<_> = candidate_files
            .iter()
            .map(|path| relative_path(root, path))
            .collect::<Result<_, _>>()?;
        let actual_paths: BTreeSet<_> = manifest.keys().cloned().collect();
        if actual_paths != expected_paths {
            let missing = expected_paths
                .difference(&actual_paths)
                .next()
                .map_or("none", String::as_str);
            let unexpected = actual_paths
                .difference(&expected_paths)
                .next()
                .map_or("none", String::as_str);
            return Err(format!(
                "DOCS_REVIEW_MANIFEST_SCOPE_MISMATCH: {}: missing {missing}, unexpected {unexpected}",
                transition.file
            ));
        }
    }
    let actual_root = architecture_candidate_root(&manifest);
    if actual_root != candidate_root {
        return Err(format!(
            "DOCS_REVIEW_CANDIDATE_ROOT_MISMATCH: {}: expected {candidate_root}, got {actual_root}",
            transition.file
        ));
    }
    Ok(manifest)
}

fn validate_review_checks(
    transition: ArchitectureReviewTransition,
    candidate_root: &str,
    record_status: &str,
    rows: &[Vec<String>],
    mode: ReviewRecordMode,
) -> Result<(), String> {
    let mut checks = BTreeMap::new();
    for row in rows {
        if row.len() != 3 {
            return Err(format!(
                "DOCS_REVIEW_CHECK_ROW_INVALID: {}",
                transition.file
            ));
        }
        let result = row[1].as_str();
        if !matches!(result, "Pending" | "PASS" | "FAIL" | "AwaitingCapability") {
            return Err(format!(
                "DOCS_REVIEW_CHECK_RESULT_INVALID: {}: {}: {result}",
                transition.file, row[0]
            ));
        }
        if result != "Pending" && is_absent(&row[2]) {
            return Err(format!(
                "DOCS_REVIEW_CHECK_EVIDENCE_MISSING: {}: {}",
                transition.file, row[0]
            ));
        }
        if checks
            .insert(row[0].clone(), (row[1].clone(), row[2].clone()))
            .is_some()
        {
            return Err(format!(
                "DOCS_REVIEW_CHECK_DUPLICATE: {}: {}",
                transition.file, row[0]
            ));
        }
    }
    let preflight_check = format!(
        "cargo run -p xtask -- architecture-review-preflight {}",
        transition.to
    );
    let required_checks: BTreeSet<String> = ARCHITECTURE_REVIEW_BASE_CHECKS
        .iter()
        .map(|check| (*check).to_owned())
        .chain(std::iter::once(preflight_check.clone()))
        .collect();
    let actual_checks: BTreeSet<String> = checks.keys().cloned().collect();
    if actual_checks != required_checks {
        let missing = required_checks
            .difference(&actual_checks)
            .next()
            .map_or("none", String::as_str);
        let unexpected = actual_checks
            .difference(&required_checks)
            .next()
            .map_or("none", String::as_str);
        return Err(format!(
            "DOCS_REVIEW_CHECK_SET_MISMATCH: {}: missing {missing}, unexpected {unexpected}",
            transition.file
        ));
    }
    for required in &required_checks {
        let Some((result, _)) = checks.get(required) else {
            return Err(format!(
                "DOCS_REVIEW_CHECK_MISSING: {}: {required}",
                transition.file
            ));
        };
        if record_status == "Approved" && result != "PASS" {
            return Err(format!(
                "DOCS_REVIEW_CHECK_NOT_PASS: {}: {required}: {result}",
                transition.file
            ));
        }
    }
    if mode == ReviewRecordMode::CandidatePreflight {
        for required in ARCHITECTURE_REVIEW_BASE_CHECKS {
            let result = &checks[*required].0;
            if result != "PASS" {
                return Err(format!(
                    "DOCS_REVIEW_PREFLIGHT_CHECK_NOT_PASS: {}: {required}: {result}",
                    transition.file
                ));
            }
        }
        let preflight_result = &checks[&preflight_check].0;
        if !matches!(preflight_result.as_str(), "Pending" | "PASS") {
            return Err(format!(
                "DOCS_REVIEW_PREFLIGHT_SELF_CHECK_INVALID: {}: {preflight_result}",
                transition.file
            ));
        }
    }
    if record_status == "Approved" && candidate_root == "absent" {
        return Err(format!("DOCS_REVIEW_CHECKS_UNBOUND: {}", transition.file));
    }
    Ok(())
}

fn validate_review_decisions(
    transition: ArchitectureReviewTransition,
    candidate_root: &str,
    record_status: &str,
    required: &[&str],
    rows: &[Vec<String>],
    kind: &str,
) -> Result<BTreeMap<String, (String, String, String)>, String> {
    let mut decisions = BTreeMap::new();
    for row in rows {
        if row.len() != 4 {
            return Err(format!(
                "DOCS_REVIEW_{kind}_ROW_INVALID: {}",
                transition.file
            ));
        }
        let decision = row[1].as_str();
        if !matches!(decision, "Pending" | "Approved" | "Rejected") {
            return Err(format!(
                "DOCS_REVIEW_{kind}_DECISION_INVALID: {}: {}: {decision}",
                transition.file, row[0]
            ));
        }
        if decision != "Pending"
            && (candidate_root == "absent" || is_absent(&row[2]) || is_absent(&row[3]))
        {
            return Err(format!(
                "DOCS_REVIEW_{kind}_DECISION_UNBOUND: {}: {}",
                transition.file, row[0]
            ));
        }
        if row[0] == "architecture.promote"
            && decision != "Pending"
            // Packet 1.5 predates exact-root decision references and remains
            // immutable historical evidence. Every later transition is bound
            // mechanically to its candidate root.
            && transition.to != "1.5"
            && row[3] != candidate_root
        {
            return Err(format!(
                "DOCS_REVIEW_{kind}_DECISION_ROOT_MISMATCH: {}: {}: expected {candidate_root}, got {}",
                transition.file, row[0], row[3]
            ));
        }
        if decisions
            .insert(
                row[0].clone(),
                (row[1].clone(), row[2].clone(), row[3].clone()),
            )
            .is_some()
        {
            return Err(format!(
                "DOCS_REVIEW_{kind}_DECISION_DUPLICATE: {}: {}",
                transition.file, row[0]
            ));
        }
    }
    for name in required {
        let Some((decision, _, _)) = decisions.get(*name) else {
            return Err(format!(
                "DOCS_REVIEW_{kind}_DECISION_MISSING: {}: {name}",
                transition.file
            ));
        };
        if record_status == "Approved" && decision != "Approved" {
            return Err(format!(
                "DOCS_REVIEW_{kind}_DECISION_NOT_APPROVED: {}: {name}: {decision}",
                transition.file
            ));
        }
    }
    Ok(decisions)
}

fn architecture_candidate_root(manifest: &BTreeMap<String, String>) -> String {
    let mut input = Vec::new();
    for (path, file_hash) in manifest {
        input.extend_from_slice(path.as_bytes());
        input.push(0);
        input.extend_from_slice(file_hash.as_bytes());
        input.push(b'\n');
    }
    sha256_hex(&input)
}

fn is_lower_sha256(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

fn is_safe_manifest_path(value: &str) -> bool {
    !value.is_empty()
        && !value.contains('\\')
        && !value.contains('\0')
        && !Path::new(value).is_absolute()
        && Path::new(value)
            .components()
            .all(|component| matches!(component, Component::Normal(_)))
}

fn is_absent(value: &str) -> bool {
    value.is_empty() || value == "absent"
}

#[derive(Clone, Debug)]
struct MarkdownTable {
    header: Vec<String>,
    rows: Vec<Vec<String>>,
}

#[derive(Clone, Debug)]
struct TraceabilityRow {
    id: String,
    owner: String,
    document_ids: BTreeSet<String>,
    gates: BTreeSet<String>,
    closure: BTreeSet<String>,
    evidence: String,
}

#[derive(Clone, Debug)]
struct ReservedAllocation {
    id: String,
    status: String,
    document: String,
    reason: String,
}

#[derive(Clone, Debug, Default)]
struct TraceabilityContract {
    requirements: BTreeMap<String, TraceabilityRow>,
    failures: BTreeMap<String, TraceabilityRow>,
    reservations: BTreeMap<String, ReservedAllocation>,
}

#[derive(Clone, Debug)]
struct GateDefinition {
    document_id: String,
    document_status: String,
    source: String,
    owner: String,
    threshold: String,
    evidence: String,
    fallback: String,
    alias_of: String,
}

impl GateDefinition {
    fn is_complete(&self) -> bool {
        [
            self.owner.as_str(),
            self.threshold.as_str(),
            self.evidence.as_str(),
            self.fallback.as_str(),
        ]
        .iter()
        .all(|value| !is_blank_contract_value(value))
    }
}

#[derive(Clone, Debug, Default)]
struct GateCatalog {
    definitions: BTreeMap<String, Vec<GateDefinition>>,
    candidate_only: BTreeSet<String>,
    candidate_subjects: BTreeMap<String, String>,
    canonical_owners: BTreeMap<String, String>,
}

#[derive(Clone, Debug)]
struct TechnologyEntry {
    status: String,
    gates: BTreeSet<String>,
}

#[derive(Clone, Debug)]
struct RequirementDefinition {
    id: String,
    document_id: String,
    document_status: String,
    owner: String,
    gates: BTreeSet<String>,
}

#[derive(Clone, Debug)]
struct ReservedRequirementDefinition {
    id: String,
    document_id: String,
    owner: String,
}

pub fn validate_traceability_allocations(traceability: &str) -> Result<(usize, usize), String> {
    let contract = parse_traceability_contract(traceability)?;
    validate_traceability_allocations_contract(&contract)?;
    Ok((contract.requirements.len(), contract.failures.len()))
}

pub fn validate_packet_16_traceability_allocations(
    traceability: &str,
) -> Result<(usize, usize), String> {
    let contract = parse_traceability_contract(traceability)?;
    validate_traceability_allocations_contract(&contract)?;
    validate_packet_16_allocation_contract(&contract)?;
    Ok((contract.requirements.len(), contract.failures.len()))
}

pub fn validate_packet_17_traceability_allocations(
    traceability: &str,
) -> Result<(usize, usize), String> {
    let contract = parse_traceability_contract(traceability)?;
    validate_traceability_allocations_contract(&contract)?;
    validate_packet_17_allocation_contract(&contract)?;
    Ok((contract.requirements.len(), contract.failures.len()))
}

pub fn validate_technology_registry(evidence: &str) -> Result<(usize, usize), String> {
    let registry = parse_technology_registry(evidence)?;
    Ok((
        registry.len(),
        registry
            .values()
            .filter(|entry| entry.status == "Proposed")
            .count(),
    ))
}

fn parse_technology_registry(evidence: &str) -> Result<BTreeMap<String, TechnologyEntry>, String> {
    let mut registry: Option<BTreeMap<String, TechnologyEntry>> = None;
    for table in markdown_tables(evidence) {
        if table.header.first().map(String::as_str) != Some("Technology ID") {
            continue;
        }
        if registry.is_some() {
            return Err("DOCS_TECHNOLOGY_TABLE_DUPLICATE".to_owned());
        }
        let status_index = exact_header_index(&table.header, "Статус")
            .or_else(|| exact_header_index(&table.header, "Status"))
            .ok_or_else(|| "DOCS_TECHNOLOGY_STATUS_COLUMN_MISSING".to_owned())?;
        let gate_index = exact_header_index(&table.header, "Gate / fallback")
            .ok_or_else(|| "DOCS_TECHNOLOGY_GATE_COLUMN_MISSING".to_owned())?;
        let mut rows = BTreeMap::new();
        for row in table.rows {
            if row.len() != table.header.len() {
                return Err("DOCS_TECHNOLOGY_ROW_INVALID".to_owned());
            }
            let id = strip_code_ticks(&row[0]);
            validate_sequential_id_syntax(&id, "TECH-", 3)?;
            let status = row[status_index].clone();
            if !matches!(
                status.as_str(),
                "Accepted" | "Proposed" | "Rejected" | "Superseded"
            ) {
                return Err(format!("DOCS_TECHNOLOGY_STATUS_INVALID: {id}: {status}"));
            }
            let gates = identifier_tokens(&row[gate_index])
                .into_iter()
                .filter(|token| is_gate_id(token))
                .collect();
            if rows
                .insert(id.clone(), TechnologyEntry { status, gates })
                .is_some()
            {
                return Err(format!("DOCS_TECHNOLOGY_ID_DUPLICATE: {id}"));
            }
        }
        registry = Some(rows);
    }
    let registry = registry.ok_or_else(|| "DOCS_TECHNOLOGY_TABLE_MISSING".to_owned())?;
    let ids = registry.keys().cloned().collect::<BTreeSet<_>>();
    require_sequential(&ids, "TECH-", 3)?;
    Ok(registry)
}

pub fn validate_requirement_gate_documents(
    traceability: &str,
    documents: &[(&str, &str)],
    vertical: &str,
) -> Result<(), String> {
    let contract = parse_traceability_contract(traceability)?;
    validate_traceability_allocations_contract(&contract)?;
    let parsed = documents
        .iter()
        .map(|(relative, body)| parse_document((*relative).to_owned(), (*body).to_owned()))
        .collect::<Result<Vec<_>, _>>()?;
    let by_relative = parsed
        .into_iter()
        .map(|document| (document.relative.clone(), document))
        .collect();
    validate_requirement_gate_contract(&contract, &by_relative, vertical, None)
}

pub fn validate_requirement_gate_documents_with_evidence(
    traceability: &str,
    documents: &[(&str, &str)],
    vertical: &str,
    evidence: &str,
) -> Result<(), String> {
    let contract = parse_traceability_contract(traceability)?;
    validate_traceability_allocations_contract(&contract)?;
    let parsed = documents
        .iter()
        .map(|(relative, body)| parse_document((*relative).to_owned(), (*body).to_owned()))
        .collect::<Result<Vec<_>, _>>()?;
    let by_relative = parsed
        .into_iter()
        .map(|document| (document.relative.clone(), document))
        .collect();
    let technologies = parse_technology_registry(evidence)?;
    validate_requirement_gate_contract(&contract, &by_relative, vertical, Some(&technologies))
}

pub fn validate_human_review_decision_v2_documents(
    documents: &[(&str, &str)],
) -> Result<(), String> {
    let parsed = documents
        .iter()
        .map(|(relative, body)| parse_document((*relative).to_owned(), (*body).to_owned()))
        .collect::<Result<Vec<_>, _>>()?;
    let by_relative = parsed
        .into_iter()
        .map(|document| (document.relative.clone(), document))
        .collect();
    validate_human_review_decision_v2_contract(&by_relative)
}

pub fn validate_human_review_decision_v2_text(body: &str) -> Result<(), String> {
    const PAYLOAD_SCHEMA: &str = r#"schema = "nextengine.human-review-decision.v2"
project_id
decision_id
subject = {
  changeset_sha256
  revision_kind = "GitCommit" | "Diff"
  revision_sha256
}
evidence_bundle_sha256
baseline =
  { kind = "None" }
  | { kind = "EvidenceBaseline", sha256 }
verification_policy_sha256
impact_manifest_sha256
fixture = {
  class = "vertical-v1-neutral"
  sha256
}
automatic_gate_summary_sha256
requirement_graph_sha256
gate_descriptor_set_sha256
review_category
decision = "Approve" | "Reject" | "NeedsChanges"
reviewer_role
reason_code
issued_at_unix_seconds"#;
    const PAYLOAD_CLOSED_SCHEMA_RULE: &str = "- object, `subject`, `baseline` и `fixture` являются closed schemas: unknown, duplicate, missing, `null` и implicit-default fields rejected;";
    const ENVELOPE_SCHEMA: &str = r#"schema = "nextengine.attestation-envelope.v2"
algorithm = "ed25519"
domain = "nextengine.human-review-decision.v2"
project_id
key_id
payload_type = "HumanReviewDecisionV2"
payload_hash
trust_policy_hash
trust_manifest_hash
revocation_snapshot_hash
signature"#;
    const ENVELOPE_CLOSED_SCHEMA_RULE: &str = "`domain` MUST equal exact ASCII literal `nextengine.human-review-decision.v2`. `payload_hash`, `trust_policy_hash`, `trust_manifest_hash` и `revocation_snapshot_hash` are exactly 64 lowercase SHA-256 hexadecimal characters. `payload_hash` is SHA-256 of exact canonical payload bytes; the other three hashes are SHA-256 of exact canonical referenced policy/manifest/snapshot bytes. `signature` is canonical unpadded base64url and MUST decode to exactly 64 bytes. `project_id` is Unicode NFC; `key_id` is a non-empty ASCII identifier from the referenced trust manifest. Unknown, duplicate, missing or `null` fields are rejected.";
    const SIGNATURE_PREIMAGE: &str = r#""nextengine.human-attestation.v2\0"
|| u32_le(schema.len) || schema_ascii
|| u32_le(algorithm.len) || algorithm_ascii
|| u32_le(domain.len) || domain_ascii
|| u32_le(project_id.len) || project_id_nfc_utf8
|| u32_le(key_id.len) || key_id_ascii
|| u32_le(payload_type.len) || payload_type_ascii
|| payload_hash[32]
|| trust_policy_hash[32]
|| trust_manifest_hash[32]
|| revocation_snapshot_hash[32]"#;
    const VERIFICATION_ADMISSION_STEPS: [&str; 6] = [
        "every persisted V2 review decision MUST have an `AttestationEnvelopeV2`; verifier validates canonical V2 payload/envelope, signature, project/hash closure and offline trust;",
        "a successful verifier returns exactly one of `VerifiedDecision::Approve`, `VerifiedDecision::Reject` or `VerifiedDecision::NeedsChanges`;",
        "admission evaluates the verified result together with required automatic gates, evidence completeness, baseline and redaction results;",
        "only `VerifiedDecision::Approve` with every required automatic gate `PASS` and complete valid evidence returns `Admit`;",
        "`VerifiedDecision::Reject` and `VerifiedDecision::NeedsChanges` are signed, cryptographically valid, machine-readable non-admitting feedback;",
        "`Approve` with any automatic result other than `PASS` is rejected as `AUTO_GATE_NOT_PASS`; human decision cannot waive failure or missing capability.",
    ];
    const SIGNATURE_NON_AUTHORITY_RULE: &str = "Success of signature verification alone MUST NOT emit gate `PASS`, mutate project state, merge, promote a baseline/model/package or publish a release.";
    const V1_HISTORICAL_PARAGRAPHS: [&str; 2] = [
        "`HumanReviewDecisionV1` and `AttestationEnvelopeV1` remain readable only in an explicit read-only `historical-audit` verifier mode. Successful legacy verification returns `HistoricalVerified`, never `VerifiedDecision`, `PASS`, `Admit`, merge, promotion or baseline acceptance.",
        "Current admission policy MUST set `review_contract_major = 2`. Supplying V1 to an admission path fails closed as `REVIEW_SCHEMA_HISTORICAL_ONLY`. Generic verifier API requires an explicit known mode; gate and mutation-capable workflows are hard-wired to `admission`, while `historical-audit` is always explicit. V1 payload/envelope MUST NOT be relabelled, field-renamed, automatically upgraded or automatically re-signed. A current decision requires a fresh V2 payload reviewed and signed by a human.",
    ];

    let payload_section = markdown_section(body, "## `HumanReviewDecisionV2`")?;
    let payload_blocks = fenced_code_blocks(&payload_section)?;
    validate_exact_review_code_blocks(
        &payload_blocks,
        &[PAYLOAD_SCHEMA],
        "DOCS_REVIEW_V2_PAYLOAD_SCHEMA_DRIFT",
    )?;
    require_exact_section_line(
        &payload_section,
        "- object, `subject`, `baseline` и `fixture` являются closed schemas:",
        PAYLOAD_CLOSED_SCHEMA_RULE,
        "DOCS_REVIEW_V2_PAYLOAD_CLOSURE_DRIFT",
    )?;

    let envelope_section = markdown_section(body, "## `AttestationEnvelopeV2`")?;
    let envelope_blocks = fenced_code_blocks(&envelope_section)?;
    validate_exact_review_code_blocks(
        &envelope_blocks,
        &[ENVELOPE_SCHEMA, SIGNATURE_PREIMAGE],
        "DOCS_REVIEW_V2_ENVELOPE_OR_PREIMAGE_DRIFT",
    )?;
    require_exact_section_line(
        &envelope_section,
        "`domain` MUST equal exact ASCII literal",
        ENVELOPE_CLOSED_SCHEMA_RULE,
        "DOCS_REVIEW_V2_ENVELOPE_CLOSURE_DRIFT",
    )?;

    let admission_section = markdown_section(body, "## Verification и admission")?;
    let steps = markdown_ordered_list_items(&admission_section)?;
    if steps != VERIFICATION_ADMISSION_STEPS {
        return Err(format!(
            "DOCS_REVIEW_V2_ADMISSION_MATRIX_DRIFT: expected {:?}, got {:?}",
            VERIFICATION_ADMISSION_STEPS, steps
        ));
    }
    require_exact_section_line(
        &admission_section,
        "Success of signature verification alone",
        SIGNATURE_NON_AUTHORITY_RULE,
        "DOCS_REVIEW_V2_SIGNATURE_AUTHORITY_DRIFT",
    )?;

    let v1_section = markdown_section(body, "## V1 historical verification only")?;
    let v1_paragraphs = markdown_paragraphs(&v1_section);
    if v1_paragraphs != V1_HISTORICAL_PARAGRAPHS {
        return Err(format!(
            "DOCS_REVIEW_V1_ADMISSION_DENIAL_DRIFT: expected explicit historical-only denial, got {:?}",
            v1_paragraphs
        ));
    }
    Ok(())
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct MarkdownCodeBlock {
    language: String,
    body: String,
}

fn markdown_section(body: &str, heading: &str) -> Result<String, String> {
    let target_level = markdown_heading_level(heading).ok_or_else(|| {
        format!("DOCS_REVIEW_V2_SECTION_HEADING_INVALID: expected heading {heading}")
    })?;
    let mut section = Vec::new();
    let mut target_count = 0_usize;
    let mut capture = false;
    let mut fence: Option<&str> = None;

    for line in body.lines() {
        let trimmed = line.trim_start();
        if let Some(marker) = fence {
            if capture {
                section.push(line);
            }
            if trimmed == marker {
                fence = None;
            }
            continue;
        }
        if let Some(marker) = markdown_fence_marker(trimmed) {
            if capture {
                section.push(line);
            }
            fence = Some(marker);
            continue;
        }
        if line.trim() == heading {
            target_count += 1;
            capture = target_count == 1;
            continue;
        }
        if capture && markdown_heading_level(line).is_some_and(|level| level <= target_level) {
            capture = false;
        }
        if capture {
            section.push(line);
        }
    }

    match target_count {
        0 => Err(format!("DOCS_REVIEW_V2_SECTION_MISSING: {heading}")),
        1 => Ok(section.join("\n")),
        _ => Err(format!("DOCS_REVIEW_V2_SECTION_DUPLICATE: {heading}")),
    }
}

fn markdown_heading_level(line: &str) -> Option<usize> {
    let trimmed = line.trim_start();
    let level = trimmed.bytes().take_while(|byte| *byte == b'#').count();
    if level == 0 || level > 6 {
        return None;
    }
    trimmed
        .as_bytes()
        .get(level)
        .is_some_and(u8::is_ascii_whitespace)
        .then_some(level)
}

fn markdown_fence_marker(line: &str) -> Option<&'static str> {
    if line.starts_with("```") {
        Some("```")
    } else if line.starts_with("~~~") {
        Some("~~~")
    } else {
        None
    }
}

fn fenced_code_blocks(section: &str) -> Result<Vec<MarkdownCodeBlock>, String> {
    let mut blocks = Vec::new();
    let mut current: Option<(String, Vec<String>)> = None;

    for line in section.lines() {
        let trimmed = line.trim_start();
        if let Some((_, body)) = current.as_mut() {
            if trimmed == "```" {
                let (language, body) = current
                    .take()
                    .expect("current fenced block exists while closing it");
                blocks.push(MarkdownCodeBlock {
                    language,
                    body: body.join("\n"),
                });
            } else {
                body.push(line.trim_end().to_owned());
            }
            continue;
        }
        if let Some(language) = trimmed.strip_prefix("```") {
            current = Some((language.trim().to_owned(), Vec::new()));
        }
    }

    if current.is_some() {
        return Err("DOCS_REVIEW_V2_CODE_BLOCK_UNCLOSED".to_owned());
    }
    Ok(blocks)
}

fn validate_exact_review_code_blocks(
    actual: &[MarkdownCodeBlock],
    expected: &[&str],
    diagnostic: &str,
) -> Result<(), String> {
    if actual.len() != expected.len() {
        return Err(format!(
            "{diagnostic}: expected {} text blocks, got {}",
            expected.len(),
            actual.len()
        ));
    }
    for (index, (actual, expected)) in actual.iter().zip(expected).enumerate() {
        if actual.language != "text" || actual.body != *expected {
            return Err(format!(
                "{diagnostic}: block {} expected language=text and exact closed schema/preimage",
                index + 1
            ));
        }
    }
    Ok(())
}

fn require_exact_section_line(
    section: &str,
    prefix: &str,
    expected: &str,
    diagnostic: &str,
) -> Result<(), String> {
    let matching = section
        .lines()
        .map(str::trim)
        .filter(|line| line.starts_with(prefix))
        .collect::<Vec<_>>();
    if matching == [expected] {
        Ok(())
    } else {
        Err(format!(
            "{diagnostic}: expected exactly `{expected}`, got {matching:?}"
        ))
    }
}

fn markdown_ordered_list_items(section: &str) -> Result<Vec<&str>, String> {
    let mut items = Vec::new();
    for line in section.lines() {
        let trimmed = line.trim();
        let Some((number, item)) = trimmed.split_once(". ") else {
            continue;
        };
        if !number.bytes().all(|byte| byte.is_ascii_digit()) {
            continue;
        }
        let expected_number = items.len() + 1;
        if number.parse::<usize>() != Ok(expected_number) {
            return Err(format!(
                "DOCS_REVIEW_V2_ADMISSION_MATRIX_ORDER: expected {expected_number}, got {number}"
            ));
        }
        items.push(item);
    }
    Ok(items)
}

fn markdown_paragraphs(section: &str) -> Vec<String> {
    let mut paragraphs = Vec::new();
    let mut current = Vec::new();
    for line in section.lines() {
        if line.trim().is_empty() {
            if !current.is_empty() {
                paragraphs.push(current.join(" "));
                current.clear();
            }
        } else {
            current.push(line.trim().to_owned());
        }
    }
    if !current.is_empty() {
        paragraphs.push(current.join(" "));
    }
    paragraphs
}

fn parse_traceability_contract(body: &str) -> Result<TraceabilityContract, String> {
    const RESERVED_HEADER: [&str; 4] = [
        "Reserved ID",
        "Allocation status",
        "Owning document",
        "Reason",
    ];
    let mut contract = TraceabilityContract::default();
    let mut accepted_table_count = 0_usize;
    let mut reserved_table_count = 0_usize;

    for table in markdown_tables(body) {
        if table.header == RESERVED_HEADER {
            reserved_table_count += 1;
            for row in table.rows {
                if row.len() != RESERVED_HEADER.len() {
                    return Err("DOCS_RESERVED_ID_ROW_INVALID".to_owned());
                }
                let id = parse_exact_requirement_or_failure_id(&row[0], "reserved ID")?;
                let allocation = ReservedAllocation {
                    id: id.clone(),
                    status: row[1].clone(),
                    document: row[2].clone(),
                    reason: row[3].clone(),
                };
                if contract
                    .reservations
                    .insert(id.clone(), allocation)
                    .is_some()
                {
                    return Err(format!("DOCS_RESERVED_ID_DUPLICATE: {id}"));
                }
            }
            continue;
        }

        let row_prefix = match table.header.first().map(String::as_str) {
            Some("Requirement") => "REQ-",
            Some("Failure requirement") => "FAIL-",
            _ => continue,
        };
        accepted_table_count += 1;
        let owner_index = exact_header_index(&table.header, "Primary owner")
            .ok_or_else(|| format!("DOCS_TRACEABILITY_OWNER_COLUMN_MISSING: {row_prefix}"))?;
        let document_index = exact_header_index(&table.header, "RFC / ADR")
            .ok_or_else(|| format!("DOCS_TRACEABILITY_DOCUMENT_COLUMN_MISSING: {row_prefix}"))?;
        let gate_index = exact_header_index(&table.header, "Gate")
            .ok_or_else(|| format!("DOCS_TRACEABILITY_GATE_COLUMN_MISSING: {row_prefix}"))?;
        let evidence_index = exact_header_index(&table.header, "Evidence")
            .ok_or_else(|| format!("DOCS_TRACEABILITY_EVIDENCE_COLUMN_MISSING: {row_prefix}"))?;
        let closure_index = exact_header_index(&table.header, "VS / profile closure")
            .ok_or_else(|| format!("DOCS_TRACEABILITY_CLOSURE_COLUMN_MISSING: {row_prefix}"))?;

        for row in table.rows {
            if row.len() != table.header.len() {
                return Err(format!("DOCS_TRACEABILITY_ROW_INVALID: {}", row[0]));
            }
            let id = parse_exact_requirement_or_failure_id(
                &row[0],
                &format!("traceability {row_prefix} row"),
            )?;
            if !id.starts_with(row_prefix) {
                return Err(format!(
                    "DOCS_TRACEABILITY_ID_KIND_MISMATCH: expected {row_prefix}, got {id}"
                ));
            }
            let document_ids =
                parse_exact_document_id_list(&row[document_index], &format!("{id}: RFC / ADR"))?;
            let gates = parse_exact_gate_id_list(&row[gate_index], &format!("{id}: Gate"))?;
            let closure =
                parse_exact_closure_id_list(&row[closure_index], &format!("{id}: closure"))?;
            let trace_row = TraceabilityRow {
                id: id.clone(),
                owner: row[owner_index].clone(),
                document_ids,
                gates,
                closure,
                evidence: row[evidence_index].clone(),
            };
            let target = if row_prefix == "REQ-" {
                &mut contract.requirements
            } else {
                &mut contract.failures
            };
            if target.insert(id.clone(), trace_row).is_some() {
                return Err(format!("DOCS_TRACEABILITY_ID_DUPLICATE: {id}"));
            }
        }
    }

    if accepted_table_count == 0 {
        return Err("DOCS_TRACEABILITY_ACCEPTED_TABLE_MISSING".to_owned());
    }
    if reserved_table_count != 1 {
        return Err(format!(
            "DOCS_RESERVED_ID_TABLE_COUNT: expected 1, got {reserved_table_count}"
        ));
    }
    Ok(contract)
}

fn validate_traceability_allocations_contract(
    contract: &TraceabilityContract,
) -> Result<(), String> {
    let mut requirement_ids = contract
        .requirements
        .keys()
        .cloned()
        .collect::<BTreeSet<_>>();
    let mut failure_ids = contract.failures.keys().cloned().collect::<BTreeSet<_>>();

    for allocation in contract.reservations.values() {
        if allocation.status != "Reserved" {
            return Err(format!(
                "DOCS_RESERVED_ID_STATUS_INVALID: {}: expected Reserved, got {}",
                allocation.id, allocation.status
            ));
        }
        if is_blank_contract_value(&allocation.document)
            || is_blank_contract_value(&allocation.reason)
        {
            return Err(format!(
                "DOCS_RESERVED_ID_CLOSURE_MISSING: {}",
                allocation.id
            ));
        }
        let conditional = format!(
            "{} {} {}",
            allocation.status, allocation.document, allocation.reason
        )
        .to_ascii_lowercase();
        if conditional.contains("conditional")
            || conditional.contains("depends on")
            || conditional.contains("if accepted")
            || conditional.contains("если ")
        {
            return Err(format!("DOCS_RESERVED_ID_CONDITIONAL: {}", allocation.id));
        }

        let target = if allocation.id.starts_with("REQ-") {
            &mut requirement_ids
        } else {
            &mut failure_ids
        };
        if !target.insert(allocation.id.clone()) {
            return Err(format!("DOCS_ID_ALLOCATION_REUSED: {}", allocation.id));
        }
    }

    require_sequential(&requirement_ids, "REQ-", 3)?;
    require_sequential(&failure_ids, "FAIL-", 3)?;
    Ok(())
}

fn validate_versioned_allocation_contract(
    contract: &TraceabilityContract,
    architecture_version: &str,
) -> Result<(), String> {
    match architecture_version {
        "1.6" => validate_packet_16_allocation_contract(contract),
        "1.7" => validate_packet_17_allocation_contract(contract),
        "1.8" => validate_packet_18_allocation_contract(contract),
        _ => Err(format!(
            "DOCS_ALLOCATION_VERSION_UNSUPPORTED: {architecture_version}"
        )),
    }
}

fn validate_packet_16_allocation_contract(contract: &TraceabilityContract) -> Result<(), String> {
    let expected_requirements = id_set("REQ-", (1..=78).chain(103..=111));
    let expected_failures = id_set("FAIL-", (1..=24).chain(39..=43));
    require_exact_id_set(
        "DOCS_PACKET_16_ACCEPTED_REQUIREMENT_SET_MISMATCH",
        contract.requirements.keys(),
        &expected_requirements,
    )?;
    require_exact_id_set(
        "DOCS_PACKET_16_ACCEPTED_FAILURE_SET_MISMATCH",
        contract.failures.keys(),
        &expected_failures,
    )?;

    let mut expected_reservations = BTreeMap::new();
    add_reserved_range(&mut expected_reservations, "REQ-", 79, 86, "SPEC-16");
    add_reserved_range(&mut expected_reservations, "REQ-", 87, 90, "SPEC-17");
    add_reserved_range(&mut expected_reservations, "REQ-", 91, 94, "SPEC-18");
    add_reserved_range(&mut expected_reservations, "REQ-", 95, 98, "SPEC-19");
    add_reserved_range(&mut expected_reservations, "REQ-", 99, 102, "SPEC-20");
    for (offset, document) in (22..=30)
        .map(|number| format!("SPEC-{number:02}"))
        .enumerate()
    {
        let first = 112 + offset * 4;
        add_reserved_range(
            &mut expected_reservations,
            "REQ-",
            first,
            first + 3,
            &document,
        );
    }
    add_reserved_range(&mut expected_reservations, "FAIL-", 25, 30, "SPEC-16");
    add_reserved_range(&mut expected_reservations, "FAIL-", 31, 32, "SPEC-17");
    add_reserved_range(&mut expected_reservations, "FAIL-", 33, 34, "SPEC-18");
    add_reserved_range(&mut expected_reservations, "FAIL-", 35, 36, "SPEC-19");
    add_reserved_range(&mut expected_reservations, "FAIL-", 37, 38, "SPEC-20");
    for (offset, document) in (22..=30)
        .map(|number| format!("SPEC-{number:02}"))
        .enumerate()
    {
        let first = 44 + offset * 2;
        add_reserved_range(
            &mut expected_reservations,
            "FAIL-",
            first,
            first + 1,
            &document,
        );
    }

    let actual_ids = contract
        .reservations
        .keys()
        .cloned()
        .collect::<BTreeSet<_>>();
    let expected_ids = expected_reservations
        .keys()
        .cloned()
        .collect::<BTreeSet<_>>();
    if actual_ids != expected_ids {
        let missing = expected_ids
            .difference(&actual_ids)
            .next()
            .map_or("none", String::as_str);
        let unexpected = actual_ids
            .difference(&expected_ids)
            .next()
            .map_or("none", String::as_str);
        return Err(format!(
            "DOCS_PACKET_16_RESERVED_SET_MISMATCH: missing {missing}, unexpected {unexpected}"
        ));
    }
    for (id, expected_document) in expected_reservations {
        let actual_document = &contract.reservations[&id].document;
        if actual_document != &expected_document {
            return Err(format!(
                "DOCS_PACKET_16_RESERVED_OWNER_MISMATCH: {id}: expected {expected_document}, got {actual_document}"
            ));
        }
    }
    Ok(())
}

fn validate_packet_17_allocation_contract(contract: &TraceabilityContract) -> Result<(), String> {
    let expected_requirements = id_set("REQ-", (1..=78).chain(87..=111));
    let expected_failures = id_set("FAIL-", (1..=24).chain(31..=43));
    require_exact_id_set(
        "DOCS_PACKET_17_ACCEPTED_REQUIREMENT_SET_MISMATCH",
        contract.requirements.keys(),
        &expected_requirements,
    )?;
    require_exact_id_set(
        "DOCS_PACKET_17_ACCEPTED_FAILURE_SET_MISMATCH",
        contract.failures.keys(),
        &expected_failures,
    )?;

    let mut expected_reservations = BTreeMap::new();
    add_reserved_range(&mut expected_reservations, "REQ-", 79, 86, "SPEC-16");
    for (offset, document) in (22..=30)
        .map(|number| format!("SPEC-{number:02}"))
        .enumerate()
    {
        let first = 112 + offset * 4;
        add_reserved_range(
            &mut expected_reservations,
            "REQ-",
            first,
            first + 3,
            &document,
        );
    }
    add_reserved_range(&mut expected_reservations, "FAIL-", 25, 30, "SPEC-16");
    for (offset, document) in (22..=30)
        .map(|number| format!("SPEC-{number:02}"))
        .enumerate()
    {
        let first = 44 + offset * 2;
        add_reserved_range(
            &mut expected_reservations,
            "FAIL-",
            first,
            first + 1,
            &document,
        );
    }

    let actual_ids = contract
        .reservations
        .keys()
        .cloned()
        .collect::<BTreeSet<_>>();
    let expected_ids = expected_reservations
        .keys()
        .cloned()
        .collect::<BTreeSet<_>>();
    if actual_ids != expected_ids {
        let missing = expected_ids
            .difference(&actual_ids)
            .next()
            .map_or("none", String::as_str);
        let unexpected = actual_ids
            .difference(&expected_ids)
            .next()
            .map_or("none", String::as_str);
        return Err(format!(
            "DOCS_PACKET_17_RESERVED_SET_MISMATCH: missing {missing}, unexpected {unexpected}"
        ));
    }
    for (id, expected_document) in expected_reservations {
        let actual_document = &contract.reservations[&id].document;
        if actual_document != &expected_document {
            return Err(format!(
                "DOCS_PACKET_17_RESERVED_OWNER_MISMATCH: {id}: expected {expected_document}, got {actual_document}"
            ));
        }
    }

    for (prefix, first, last, document) in [
        ("REQ-", 87, 90, "SPEC-17"),
        ("REQ-", 91, 94, "SPEC-18"),
        ("REQ-", 95, 98, "SPEC-19"),
        ("REQ-", 99, 102, "SPEC-20"),
        ("REQ-", 103, 110, "SPEC-21"),
        ("FAIL-", 31, 32, "SPEC-17"),
        ("FAIL-", 33, 34, "SPEC-18"),
        ("FAIL-", 35, 36, "SPEC-19"),
        ("FAIL-", 37, 38, "SPEC-20"),
        ("FAIL-", 39, 42, "SPEC-21"),
    ] {
        let rows = if prefix == "REQ-" {
            &contract.requirements
        } else {
            &contract.failures
        };
        for number in first..=last {
            let id = format!("{prefix}{number:03}");
            let row = &rows[&id];
            if !row.document_ids.contains(document) {
                return Err(format!(
                    "DOCS_PACKET_17_ACCEPTED_OWNER_MISMATCH: {id}: expected {document}"
                ));
            }
        }
    }
    Ok(())
}

fn validate_packet_18_allocation_contract(contract: &TraceabilityContract) -> Result<(), String> {
    let expected_requirements = id_set("REQ-", (1..=78).chain(87..=147));
    let expected_failures = id_set("FAIL-", (1..=24).chain(31..=61));
    require_exact_id_set(
        "DOCS_PACKET_18_ACCEPTED_REQUIREMENT_SET_MISMATCH",
        contract.requirements.keys(),
        &expected_requirements,
    )?;
    require_exact_id_set(
        "DOCS_PACKET_18_ACCEPTED_FAILURE_SET_MISMATCH",
        contract.failures.keys(),
        &expected_failures,
    )?;

    let mut expected_reservations = BTreeMap::new();
    add_reserved_range(&mut expected_reservations, "REQ-", 79, 86, "SPEC-16");
    add_reserved_range(&mut expected_reservations, "FAIL-", 25, 30, "SPEC-16");

    let actual_ids = contract
        .reservations
        .keys()
        .cloned()
        .collect::<BTreeSet<_>>();
    let expected_ids = expected_reservations
        .keys()
        .cloned()
        .collect::<BTreeSet<_>>();
    if actual_ids != expected_ids {
        let missing = expected_ids
            .difference(&actual_ids)
            .next()
            .map_or("none", String::as_str);
        let unexpected = actual_ids
            .difference(&expected_ids)
            .next()
            .map_or("none", String::as_str);
        return Err(format!(
            "DOCS_PACKET_18_RESERVED_SET_MISMATCH: missing {missing}, unexpected {unexpected}"
        ));
    }
    for (id, expected_document) in expected_reservations {
        let actual_document = &contract.reservations[&id].document;
        if actual_document != &expected_document {
            return Err(format!(
                "DOCS_PACKET_18_RESERVED_OWNER_MISMATCH: {id}: expected {expected_document}, got {actual_document}"
            ));
        }
    }

    for (prefix, first, last, document) in [
        ("REQ-", 87, 90, "SPEC-17"),
        ("REQ-", 91, 94, "SPEC-18"),
        ("REQ-", 95, 98, "SPEC-19"),
        ("REQ-", 99, 102, "SPEC-20"),
        ("REQ-", 103, 110, "SPEC-21"),
        ("REQ-", 112, 115, "SPEC-22"),
        ("REQ-", 116, 119, "SPEC-23"),
        ("REQ-", 120, 123, "SPEC-24"),
        ("REQ-", 124, 127, "SPEC-25"),
        ("REQ-", 128, 131, "SPEC-26"),
        ("REQ-", 132, 135, "SPEC-27"),
        ("REQ-", 136, 139, "SPEC-28"),
        ("REQ-", 140, 143, "SPEC-29"),
        ("REQ-", 144, 147, "SPEC-30"),
        ("FAIL-", 31, 32, "SPEC-17"),
        ("FAIL-", 33, 34, "SPEC-18"),
        ("FAIL-", 35, 36, "SPEC-19"),
        ("FAIL-", 37, 38, "SPEC-20"),
        ("FAIL-", 39, 42, "SPEC-21"),
        ("FAIL-", 44, 45, "SPEC-22"),
        ("FAIL-", 46, 47, "SPEC-23"),
        ("FAIL-", 48, 49, "SPEC-24"),
        ("FAIL-", 50, 51, "SPEC-25"),
        ("FAIL-", 52, 53, "SPEC-26"),
        ("FAIL-", 54, 55, "SPEC-27"),
        ("FAIL-", 56, 57, "SPEC-28"),
        ("FAIL-", 58, 59, "SPEC-29"),
        ("FAIL-", 60, 61, "SPEC-30"),
    ] {
        let rows = if prefix == "REQ-" {
            &contract.requirements
        } else {
            &contract.failures
        };
        for number in first..=last {
            let id = format!("{prefix}{number:03}");
            let row = &rows[&id];
            if !row.document_ids.contains(document) {
                return Err(format!(
                    "DOCS_PACKET_18_ACCEPTED_OWNER_MISMATCH: {id}: expected {document}"
                ));
            }
        }
    }
    Ok(())
}

fn id_set(prefix: &str, numbers: impl IntoIterator<Item = usize>) -> BTreeSet<String> {
    numbers
        .into_iter()
        .map(|number| format!("{prefix}{number:03}"))
        .collect()
}

fn require_exact_id_set<'a>(
    code: &str,
    actual: impl Iterator<Item = &'a String>,
    expected: &BTreeSet<String>,
) -> Result<(), String> {
    let actual = actual.cloned().collect::<BTreeSet<_>>();
    if actual == *expected {
        return Ok(());
    }
    let missing = expected
        .difference(&actual)
        .next()
        .map_or("none", String::as_str);
    let unexpected = actual
        .difference(expected)
        .next()
        .map_or("none", String::as_str);
    Err(format!(
        "{code}: missing {missing}, unexpected {unexpected}"
    ))
}

fn add_reserved_range(
    reservations: &mut BTreeMap<String, String>,
    prefix: &str,
    first: usize,
    last: usize,
    document: &str,
) {
    for number in first..=last {
        reservations.insert(format!("{prefix}{number:03}"), document.to_owned());
    }
}

fn validate_deterministic_substrate_contract(
    documents: &BTreeMap<String, Document>,
) -> Result<(), String> {
    let accepted_body = |id: &str| {
        documents
            .values()
            .find(|document| document.id == id && document.status == "Accepted")
            .map(|document| document.body.as_str())
            .ok_or_else(|| format!("DOCS_DETERMINISTIC_CONTRACT_DOCUMENT_MISSING: {id}"))
    };
    validate_deterministic_substrate_documents(
        accepted_body("SPEC-21")?,
        accepted_body("ADR-022")?,
        accepted_body("SPEC-07")?,
        accepted_body("ADR-014")?,
    )
}

pub fn validate_deterministic_substrate_documents(
    spec_21: &str,
    adr_022: &str,
    spec_07: &str,
    adr_014: &str,
) -> Result<(), String> {
    for (expected_id, body) in [
        ("SPEC-21", spec_21),
        ("ADR-022", adr_022),
        ("SPEC-07", spec_07),
        ("ADR-014", adr_014),
    ] {
        let actual_id = table_field(body, "ID").unwrap_or_default();
        let status = table_field(body, "Статус").unwrap_or_default();
        if actual_id != expected_id || status != "Accepted" {
            return Err(format!(
                "DOCS_DETERMINISTIC_CONTRACT_DOCUMENT_INVALID: expected {expected_id}/Accepted, got {actual_id}/{status}"
            ));
        }
    }

    validate_command_identity_contract(spec_21, adr_022)?;
    validate_causal_identity_contract(spec_21, adr_022)?;
    validate_command_ledger_contract(spec_21, adr_022)?;
    validate_ingress_cutoff_contract(spec_21, adr_022)?;
    validate_rng_contract(spec_21, adr_022)?;
    validate_schedule_contract(spec_21, adr_022)?;
    validate_numeric_contract(spec_21, adr_022)?;
    validate_script_breaker_contract(spec_07, adr_014)?;
    Ok(())
}

fn validate_command_identity_contract(spec_21: &str, adr_022: &str) -> Result<(), String> {
    let spec_body = contract_section(
        spec_21,
        "### CanonicalCommandBodyV2",
        "### WorldCommandEnvelopeV2",
        "SPEC-21",
    )?;
    let adr_body = contract_section(
        adr_022,
        "### Несамоссылочная command identity",
        "### Canonical encoding, paths и state root",
        "ADR-022",
    )?;
    for (document, section) in [("SPEC-21", spec_body), ("ADR-022", adr_body)] {
        if !section.contains("MUST NOT")
            || !(section.contains("`command_id` либо `claimed_command_id`")
                || section.contains("`command_id` или `claimed_command_id`"))
        {
            return Err(format!(
                "DOCS_DETERMINISTIC_COMMAND_BODY_SELF_FIELD: {document}"
            ));
        }
    }

    let expected_body_hash = "body_hash=SHA256(body_bytes)";
    let expected_command_id = r#"command_id=left128(SHA256("nextengine.command-id.v2\0"||u64_le(body_bytes.len)||body_bytes))"#;
    for (document, body) in [("SPEC-21", spec_21), ("ADR-022", adr_022)] {
        let compact = compact_contract_text(body);
        if !compact.contains(expected_body_hash) {
            return Err(format!(
                "DOCS_DETERMINISTIC_BODY_HASH_CONTRACT_DRIFT: {document}"
            ));
        }
        if !compact.contains(expected_command_id) {
            return Err(format!(
                "DOCS_DETERMINISTIC_COMMAND_ID_PREIMAGE_DRIFT: {document}"
            ));
        }
    }

    let identity = contract_section(
        spec_21,
        "### Command identity",
        "### Validator-owned ordering",
        "SPEC-21",
    )?;
    require_contract_literals(
        identity,
        "SPEC-21",
        "DOCS_DETERMINISTIC_COMMAND_ID_VALIDATION_DRIFT",
        &[
            "Validator MUST повторно построить `body_bytes`",
            "предоставление caller-owned canonical bytes без schema decode/re-encode запрещено",
            "`COMMAND_ID_MISMATCH`",
            "до ledger reservation и без gameplay mutation",
        ],
    )?;
    let adr_identity = contract_section(
        adr_022,
        "### Несамоссылочная command identity",
        "### Canonical encoding, paths и state root",
        "ADR-022",
    )?;
    require_contract_literals(
        adr_identity,
        "ADR-022",
        "DOCS_DETERMINISTIC_COMMAND_ID_VALIDATION_DRIFT",
        &[
            "Переданный claim MUST совпасть с вычисленным ID",
            "mismatch отклоняется до reservation или gameplay mutation",
        ],
    )?;
    Ok(())
}

fn validate_causal_identity_contract(spec_21: &str, adr_022: &str) -> Result<(), String> {
    let causal = contract_section(
        spec_21,
        "## World и causal identity",
        "## Command ledger schemas",
        "SPEC-21",
    )?;
    require_contract_literals(
        causal,
        "SPEC-21",
        "DOCS_DETERMINISTIC_CAUSAL_IDENTITY_DRIFT",
        &[
            "WorldIdentityManifestV1 {",
            "\"nextengine.world-namespace.v1\\0\"",
            "`Save As`, autosave, replay и recovery сохраняют namespace",
            "CommandStreamRegistryV1 {",
            "\"nextengine.command-stream.v1\\0\"",
            "Stream навсегда bound к principal",
            "COMMAND_STREAM_EPOCH_EXHAUSTED",
            "\"nextengine.event-body.v1\\0\"",
            "\"nextengine.event-id.v1\\0\"",
            "Exact retry возвращает прежний receipt и MUST NOT повторно публиковать event",
            "\"nextengine.persistent-id.v2\\0\"",
            "`PERSISTENT_ID_COLLISION`, не выбирая другой ID",
            "CausalIdentityRegistryV1 {",
            "compare-or-insert",
            "Collision не получает salt/random fallback",
            "\"nextengine.causal-provenance.v1\\0\"",
            "для event дополнительно включают full event-body hash",
            "для durable record — causal full command body hash",
            "`WORLD_NAMESPACE_COLLISION`",
        ],
    )?;
    for code in [
        "PLAYER_PRINCIPAL_COLLISION",
        "COMMAND_STREAM_ID_COLLISION",
        "DOMAIN_EVENT_ID_COLLISION",
        "PERSISTENT_ID_COLLISION",
        "RNG_STREAM_ID_COLLISION",
    ] {
        if !causal.contains(code) {
            return Err(format!(
                "DOCS_DETERMINISTIC_CAUSAL_IDENTITY_DRIFT: SPEC-21: {code}"
            ));
        }
    }

    let adr_causal = contract_section(
        adr_022,
        "### Causal identities",
        "### Persisted command ledger",
        "ADR-022",
    )?;
    require_contract_literals(
        adr_causal,
        "ADR-022",
        "DOCS_DETERMINISTIC_CAUSAL_IDENTITY_DRIFT",
        &[
            "WorldIdentityManifestV1",
            "`Save As` сохраняет namespace",
            "`CommandStreamId` детерминированно выводится",
            "wall clock, process/session/task/worker IDs запрещены",
            "`DomainEventId` выводится из causal command",
            "full event-body hash",
            "Runtime-created `PersistentId` выводится",
            "collision с другой provenance является fatal",
            "Explicit runtime ID недоступен script/plugin/AI/scenario input",
        ],
    )
}

fn validate_command_ledger_contract(spec_21: &str, adr_022: &str) -> Result<(), String> {
    let schema = contract_section(
        spec_21,
        "## Command ledger schemas",
        "## Admission, retry и collision algorithm",
        "SPEC-21",
    )?;
    require_contract_literals(
        schema,
        "SPEC-21",
        "DOCS_DETERMINISTIC_LEDGER_CONTRACT_DRIFT",
        &[
            "CommandBodyArchiveManifestV1 {",
            "CommandIdentityIndexBodyV1 {",
            "CommandIdentityIndexV1 {",
            "CommandStreamLedgerV2 {",
            "admission_high_watermark: Option<u64>",
            "receipt_window: Vec<CommandReceiptV1>",
            "finalized_receipt_count: u64",
            "receipt_chain_root: Hash256",
            "COMMAND_ID_COLLISION",
            "После 4096 finalizations published snapshot/save MUST содержать **ровно 4096**",
            "capacity не конфигурируется",
        ],
    )?;
    require_contract_literals(
        schema,
        "SPEC-21",
        "DOCS_DETERMINISTIC_LEDGER_ARCHIVE_CLOSURE_DRIFT",
        &[
            "Body archive является logical immutable content-addressed map `body_hash → exact canonical body bytes`",
            "`canonical_body_ref` равен `body_hash`",
            "Exact bytes сохраняются на lifetime world identity и не удаляются при receipt eviction",
            "`index_root` находится только во внешнем `CommandIdentityIndexV1` и не входит в собственный preimage",
            "Union всех `identity_index.body.bindings.values().occurrences.body_hash` MUST точно совпадать с key set body archive",
            "`body.command_id_count == body.bindings.len`",
            "`body.occurrence_count` равен сумме occurrence lengths",
            "`Unique` имеет ровно одну occurrence, `Collision` — не меньше двух",
            "`COMMAND_BODY_ARCHIVE_CORRUPT`",
        ],
    )?;
    let index_body = contract_section(
        schema,
        "CommandIdentityIndexBodyV1 {",
        "CommandIdentityIndexV1 {",
        "SPEC-21",
    )?;
    if index_body.contains("index_root") {
        return Err(
            "DOCS_DETERMINISTIC_LEDGER_IDENTITY_INDEX_SELF_REFERENCE: SPEC-21 body contains index_root"
                .to_owned(),
        );
    }
    let index_preimage = contract_section(
        schema,
        "identity_index_root = SHA256(",
        "Leaves сортируются",
        "SPEC-21",
    )?;
    let compact_index_preimage = compact_contract_text(index_preimage);
    if !compact_index_preimage.contains("||u64_le(index_body_bytes.len)||index_body_bytes)")
        || compact_index_preimage
            .split_once('=')
            .is_some_and(|(_, preimage)| preimage.contains("index_root"))
    {
        return Err(
            "DOCS_DETERMINISTIC_LEDGER_IDENTITY_INDEX_SELF_REFERENCE: SPEC-21 preimage".to_owned(),
        );
    }
    if !compact_contract_text(schema)
        .contains("receipt_window.len=min(finalized_receipt_count,4096)")
    {
        return Err("DOCS_DETERMINISTIC_LEDGER_RECEIPT_WINDOW_DRIFT: SPEC-21".to_owned());
    }
    require_contract_literals(
        schema,
        "SPEC-21",
        "DOCS_DETERMINISTIC_LEDGER_FINALIZATION_DRIFT",
        &[
            "`finalized_receipt_count = 0`",
            "следующий\nordinal `0`",
            "`finalization_ordinal == finalized_receipt_count`",
            "последний\nдопустимый receipt имеет ordinal `u64::MAX - 1`",
            "`COMMAND_FINALIZATION_ORDINAL_EXHAUSTED`",
            "без\nreservation, identity/archive insert, receipt или gameplay mutation",
            "`COMMAND_SEQUENCE_FINALIZED`; оба пути не требуют нового ordinal",
            "`COMMAND_LEDGER_CORRUPT` и fail-closed",
        ],
    )?;

    let admission = contract_section(
        spec_21,
        "## Admission, retry и collision algorithm",
        "## Ingress cutoff, clocks и async completion",
        "SPEC-21",
    )?;
    require_contract_literals(
        admission,
        "SPEC-21",
        "DOCS_DETERMINISTIC_LEDGER_RETRY_DRIFT",
        &[
            "CommandCollisionIncidentV1",
            "CollisionLocked",
            "Если sequence уже закрыта high-watermark и отсутствует в retained state, вернуть `COMMAND_SEQUENCE_FINALIZED`",
            "High-watermark может только увеличиваться",
            "все пропущенные sequence закрываются",
            "COMMAND_SEQUENCE_FINALIZED",
            "Exact retry означает:",
            "retry после eviction возвращает только `COMMAND_SEQUENCE_FINALIZED`",
            "ни один body не выбирается",
            "создаётся один terminal collision receipt",
        ],
    )?;
    let failure_contract = contract_section(
        spec_21,
        "## Failure paths",
        "## Verification gates",
        "SPEC-21",
    )?;
    require_contract_literals(
        failure_contract,
        "SPEC-21",
        "DOCS_DETERMINISTIC_COMMAND_COLLISION_MUTATION_DRIFT",
        &[
            "Claimed-ID mismatch, command/body collision",
            "no partial gameplay mutation",
            "exact prior result preserved",
        ],
    )?;

    let save_restart = contract_section(
        spec_21,
        "### Ingress и Outcome после save/restart",
        "## Deterministic RNG",
        "SPEC-21",
    )?;
    require_contract_literals(
        save_restart,
        "SPEC-21",
        "DOCS_DETERMINISTIC_LEDGER_RESTART_DRIFT",
        &[
            "Crash после atomic checkpoint",
            "exact retry retained command возвращает receipt без republish events",
            "Published save MUST NOT содержать незавершённую `Outcome` reservation",
            "LEDGER_OUTCOME_PARTIAL",
        ],
    )?;

    let profiles = contract_section(
        spec_21,
        "### Hash-bound component profiles",
        "## Requirements",
        "SPEC-21",
    )?;
    require_contract_literals(
        profiles,
        "SPEC-21",
        "DOCS_DETERMINISTIC_LEDGER_RESTART_DRIFT",
        &[
            "Published save MUST атомарно содержать:",
            "`CommandLedgerV2`, включая body archive/index",
            "каждый полный stream ledger, pending, exact 4096 receipt rule, chain count/root и state",
        ],
    )?;
    let compact_profiles = compact_contract_text(profiles);
    for policy in [
        "SequencePolicy=StrictHighWatermarkGapsFinal",
        "RetryPolicy=PendingOrRetainedExactElseFinalized",
        "CollisionPolicy=RejectAllAndLockExternalFatalInternal",
    ] {
        if !compact_profiles.contains(policy) {
            return Err(format!(
                "DOCS_DETERMINISTIC_LEDGER_POLICY_DRIFT: SPEC-21: {policy}"
            ));
        }
    }

    let adr_ledger = contract_section(
        adr_022,
        "### Persisted command ledger",
        "### Deterministic runtime primitives",
        "ADR-022",
    )?;
    require_contract_literals(
        adr_ledger,
        "ADR-022",
        "DOCS_DETERMINISTIC_LEDGER_CONTRACT_DRIFT",
        &[
            "monotonic admission high-watermark",
            "COMMAND_SEQUENCE_FINALIZED",
            "CollisionLocked",
            "Save/restart не может повторно публиковать events",
            "ровно 4096",
        ],
    )?;
    if !compact_contract_text(adr_ledger).contains("min(finalized_receipt_count,4096)") {
        return Err("DOCS_DETERMINISTIC_LEDGER_RECEIPT_WINDOW_DRIFT: ADR-022".to_owned());
    }
    Ok(())
}

fn validate_ingress_cutoff_contract(spec_21: &str, adr_022: &str) -> Result<(), String> {
    let ingress = contract_section(
        spec_21,
        "## Ingress cutoff, clocks и async completion",
        "## Deterministic RNG",
        "SPEC-21",
    )?;
    require_contract_literals(
        ingress,
        "SPEC-21",
        "DOCS_DETERMINISTIC_INGRESS_CUTOFF_DRIFT",
        &[
            "IngressAssignmentV1 {",
            "CompletionAssignmentV1 {",
            "ClosedIngressBatchBodyV1 {",
            "Ingress adapter имеет две logical queues: `current(T)` и `next(T + 1)`",
            "linearized до barrier",
            "linearized в barrier или после него",
            "wall-clock timestamp не решает сторону cutoff",
            "Replay хранит closed batch и не пересчитывает assignment по timestamp",
            "Async result публикуется в тот же current/next cutoff и получает `CompletionAssignmentV1`",
            "один request с разными result hashes получает persisted `IngressEquivalenceReceiptV1(TASK_RESULT_COLLISION)`",
            "save/restart сохраняет",
        ],
    )?;

    let compact = compact_contract_text(ingress);
    for invariant in [
        "`current(first_gameplay_tick)`имеет`queue_generation=0`;`next`имеетещёнеопубликованнуюgeneration`1`",
        "всеегоinput/completionassignmentsобязаныиметьровнотожезначение",
        "Atomicpublish/close/swapвыполняетровноодинcheckedincrementиделает`g+1`новойcurrentgeneration",
        "При`g=u64::MAX`новыйbarrierненачинается",
        "`INGRESS_QUEUE_GENERATION_EXHAUSTED`",
        "непубликуетbatch,assignment,command,eventилиpartialtick",
        "Generationgap,duplicategeneration",
        "mismatchbatch/assignment",
    ] {
        if !compact.contains(invariant) {
            return Err(format!(
                "DOCS_DETERMINISTIC_QUEUE_GENERATION_DRIFT: SPEC-21: {invariant}"
            ));
        }
    }

    let adr_runtime = contract_section(
        adr_022,
        "### Deterministic runtime primitives",
        "## Рассмотренные варианты",
        "ADR-022",
    )?;
    require_contract_literals(
        adr_runtime,
        "ADR-022",
        "DOCS_DETERMINISTIC_INGRESS_CUTOFF_DRIFT",
        &[
            "Input ingress использует атомарный current/next queue cutoff",
            "linearized до close barrier stage 1",
            "элемент после barrier — next tick",
            "Wall-clock timestamp остаётся metadata",
            "Async/task completion становится authoritative только как immutable staged result",
        ],
    )
}

fn validate_rng_contract(spec_21: &str, adr_022: &str) -> Result<(), String> {
    let rng = contract_section(
        spec_21,
        "## Deterministic RNG",
        "## Stable schedule, queries, shards и task merge",
        "SPEC-21",
    )?;
    require_contract_literals(
        rng,
        "SPEC-21",
        "DOCS_DETERMINISTIC_RNG_CONTRACT_DRIFT",
        &[
            "RngStreamDescriptorV1",
            "stream_name: NamespacedId",
            "algorithm_id: NamespacedId = \"nextengine.chacha12-ietf.v1\"",
            "\"nextengine.rng-key.chacha12.v1\\0\"",
            "\"nextengine.rng-nonce.chacha12.v1\\0\"",
            "Один descriptor всегда обозначает один stream",
            "State состоит из 16 little-endian `u32` words",
            "0..3   = 0x61707865, 0x3320646e, 0x79622d32, 0x6b206574",
            "12     = block_counter",
            "13..15 = three nonce words",
            "a = a +% b; d = rotl(d xor a, 16)",
            "c = c +% d; b = rotl(b xor c, 12)",
            "a = a +% b; d = rotl(d xor a, 8)",
            "c = c +% d; b = rotl(b xor c, 7)",
            "QR(0,4,8,12); QR(1,5,9,13); QR(2,6,10,14); QR(3,7,11,15)",
            "QR(0,5,10,15); QR(1,6,11,12); QR(2,7,8,13); QR(3,4,9,14)",
            "ChaCha12 выполняет ровно 6 double rounds",
            "word-wise wrapping-add initial state",
            "64 bytes как 16 little-endian words",
            "Первый block counter равен 0",
            "Единственный valid exhausted representation — `(u32::MAX, 0, true)`",
            "Следующий draw возвращает `RNG_STREAM_EXHAUSTED` без state mutation",
            "RNG_STREAM_EXHAUSTED",
        ],
    )?;
    let implementation = contract_section(
        spec_21,
        "## Implementation admission и migration",
        "",
        "SPEC-21",
    )?;
    require_contract_literals(
        implementation,
        "SPEC-21",
        "DOCS_DETERMINISTIC_RNG_GOLDEN_VECTOR_MISSING",
        &["checked-in neutral golden vectors", "RNG"],
    )?;
    require_contract_literals(
        adr_022,
        "ADR-022",
        "DOCS_DETERMINISTIC_RNG_CONTRACT_DRIFT",
        &[
            "engine-owned exact `ChaCha12IetfV1`",
            "canonical stream identity",
            "Неявный global/worker RNG запрещён",
        ],
    )
}

fn validate_schedule_contract(spec_21: &str, adr_022: &str) -> Result<(), String> {
    let schedule = contract_section(
        spec_21,
        "## Stable schedule, queries, shards и task merge",
        "## Authoritative numerics и physics quantization",
        "SPEC-21",
    )?;
    require_contract_literals(
        schedule,
        "SPEC-21",
        "DOCS_DETERMINISTIC_SCHEDULE_CONTRACT_DRIFT",
        &[
            "ScheduleManifestV1",
            "SCHEDULE_ACCESS_AMBIGUOUS",
            "SCHEDULE_CYCLE",
            "lexicographically smallest canonical `SystemId`",
            "zero-indegree nodes",
            "Authoritative query MUST объявить tag `0x01 PersistentId` либо `0x02 StableRecordKey`",
            "ReducerDescriptorV1",
            "checked fixed-point/integer sum",
            "Float associative reduction, unordered fold",
            "Task result merge происходит только на declared commit stage",
        ],
    )?;
    let compact = compact_contract_text(schedule);
    for ordering in [
        "(owner_id,schema_id,stable_record_key,field_id,system_id,shard_index)",
        "(commit_stage,owner_id,request_id,result_hash)",
    ] {
        if !compact.contains(ordering) {
            return Err(format!(
                "DOCS_DETERMINISTIC_SCHEDULE_ORDER_DRIFT: SPEC-21: {ordering}"
            ));
        }
    }
    require_contract_literals(
        adr_022,
        "ADR-022",
        "DOCS_DETERMINISTIC_SCHEDULE_CONTRACT_DRIFT",
        &[
            "versioned stable DAG",
            "stable topological tie-break",
            "ordered authoritative queries",
            "deterministic delta/reducer merge",
        ],
    )
}

fn validate_numeric_contract(spec_21: &str, adr_022: &str) -> Result<(), String> {
    let numeric = contract_section(
        spec_21,
        "## Authoritative numerics и physics quantization",
        "## RuntimeDeterminismProfileV1 и persistence",
        "SPEC-21",
    )?;
    require_contract_literals(
        numeric,
        "SPEC-21",
        "DOCS_DETERMINISTIC_NUMERIC_CONTRACT_DRIFT",
        &[
            "FixedPointDescriptorV1",
            "AuthoritativeNumericProfileV1",
            "PhysicsQuantizationRuleV1",
            "PhysicsQuantizationProfileV1",
            "NearestTiesToEven",
            "RejectTransaction",
            "checked signed `i128` numerator/denominator",
            "одно** round-to-nearest-ties-to-even",
            "reject NaN/infinity",
            "Fast-math, implicit epsilon, unordered float sum",
        ],
    )?;
    require_contract_literals(
        adr_022,
        "ADR-022",
        "DOCS_DETERMINISTIC_NUMERIC_CONTRACT_DRIFT",
        &[
            "checked integer/fixed-point rules",
            "Raw backend physics float проходит declared exact quantization",
            "NaN, infinity, unchecked overflow, authoritative float reduction и implicit epsilon запрещены",
        ],
    )
}

fn validate_script_breaker_contract(spec_07: &str, adr_014: &str) -> Result<(), String> {
    for (document, body, start, end) in [
        (
            "SPEC-07",
            spec_07,
            "## Failure semantics",
            "## Verification gates",
        ),
        (
            "ADR-014",
            adr_014,
            "## Circuit breaker",
            "## Package identity, signatures и capabilities",
        ),
    ] {
        let breaker = contract_section(body, start, end, document)?;
        require_contract_literals(
            breaker,
            document,
            "DOCS_SCRIPT_BREAKER_CONTRACT_DRIFT",
            &[
                "после третьего authoritative violation",
                "inclusive sliding window `1_800` gameplay ticks",
                "window_start_tick = current_tick.saturating_sub(1_799)",
                "entry_tick < window_start_tick",
                "добавляет current entry",
                "затем сравнивает count",
                "ticks `0…1_798` lower bound равен `0`",
                "начиная с tick `1_799` inclusive window всегда содержит не более `1_800` gameplay ticks",
                "wall seconds",
            ],
        )?;
    }

    let failure = contract_section(
        spec_07,
        "## Failure semantics",
        "## Verification gates",
        "SPEC-07",
    )?;
    require_contract_literals(
        failure,
        "SPEC-07",
        "DOCS_SCRIPT_BREAKER_CONTRACT_DRIFT",
        &[
            "ExtensionBudgetPolicyV1",
            "count ≥3 открывает circuit",
            "implicit reload и new-session reset запрещены",
        ],
    )?;

    let luau = contract_section(spec_07, "## Luau scripting", "## Wasm plugins", "SPEC-07")?;
    require_contract_literals(
        luau,
        "SPEC-07",
        "DOCS_SCRIPT_WATCHDOG_CONTRACT_DRIFT",
        &[
            "NonConforming(EXTENSION_WALL_WATCHDOG)",
            "не добавляет authoritative violation/strike",
            "не может дать PASS",
        ],
    )?;
    require_contract_literals(
        spec_07,
        "SPEC-07",
        "DOCS_SCRIPT_WATCHDOG_CONTRACT_DRIFT",
        &["every injected wall trip is `NonConforming`, never PASS"],
    )?;
    let adr_watchdog = contract_section(
        adr_014,
        "## Deterministic authoritative budgets",
        "## Circuit breaker",
        "ADR-014",
    )?;
    require_contract_literals(
        adr_watchdog,
        "ADR-014",
        "DOCS_SCRIPT_WATCHDOG_CONTRACT_DRIFT",
        &[
            "NonConforming(EXTENSION_WALL_WATCHDOG)",
            "не создавать обычный gameplay strike",
            "запретить PASS",
        ],
    )
}

fn contract_section<'a>(
    body: &'a str,
    start: &str,
    end: &str,
    document: &str,
) -> Result<&'a str, String> {
    let start_index = body.find(start).ok_or_else(|| {
        format!("DOCS_DETERMINISTIC_CONTRACT_SECTION_MISSING: {document}: {start}")
    })?;
    let remainder = &body[start_index..];
    if end.is_empty() {
        return Ok(remainder);
    }
    let end_index = remainder
        .find(end)
        .ok_or_else(|| format!("DOCS_DETERMINISTIC_CONTRACT_SECTION_MISSING: {document}: {end}"))?;
    Ok(&remainder[..end_index])
}

fn compact_contract_text(body: &str) -> String {
    body.chars()
        .filter(|character| !character.is_whitespace())
        .collect()
}

fn require_contract_literals(
    body: &str,
    document: &str,
    code: &str,
    required: &[&str],
) -> Result<(), String> {
    for literal in required {
        if !body.contains(literal) {
            return Err(format!("{code}: {document}: {literal}"));
        }
    }
    Ok(())
}

fn validate_foundation_packet_17_contract(
    documents: &BTreeMap<String, Document>,
) -> Result<(), String> {
    const IDS: [&str; 8] = [
        "SPEC-17", "ADR-018", "SPEC-18", "ADR-019", "SPEC-19", "ADR-020", "SPEC-20", "ADR-021",
    ];
    let mut packet_documents = Vec::new();
    for id in IDS {
        let document = documents
            .values()
            .find(|document| document.id == id && document.status == "Accepted")
            .ok_or_else(|| {
                format!("DOCS_PACKET_17_FOUNDATION_DOCUMENT_MISSING_OR_NOT_ACCEPTED: {id}")
            })?;
        packet_documents.push((id, document.body.as_str()));
    }
    validate_foundation_packet_17_documents(&packet_documents)
}

fn validate_data_substrate_packet_18_contract(
    documents: &BTreeMap<String, Document>,
) -> Result<(), String> {
    const IDS: [&str; 13] = [
        "SPEC-22", "SPEC-23", "SPEC-24", "SPEC-25", "SPEC-26", "SPEC-27", "SPEC-28", "SPEC-29",
        "SPEC-30", "ADR-025", "ADR-026", "ADR-027", "ADR-028",
    ];
    let mut packet_documents = Vec::new();
    for id in IDS {
        let document = documents
            .values()
            .find(|document| document.id == id && document.status == "Accepted")
            .ok_or_else(|| format!("DOCS_PACKET_18_P0_DOCUMENT_MISSING_OR_NOT_ACCEPTED: {id}"))?;
        packet_documents.push((id, document.body.as_str()));
    }
    validate_p0_packet_18_documents(&packet_documents)?;

    let spec_14 = documents
        .values()
        .find(|document| document.id == "SPEC-14" && document.status == "Accepted")
        .ok_or_else(|| {
            "DOCS_PACKET_18_INTEGRATION_DOCUMENT_MISSING_OR_NOT_ACCEPTED: SPEC-14".to_owned()
        })?;
    require_contract_literals(
        &spec_14.body,
        "SPEC-14",
        "DOCS_PACKET_18_INTEGRATION_CONTRACT_DRIFT",
        &[
            "`PolicyStateRecordV1`",
            "`authoritative_state_hash`",
            "`PolicyStateCommitV1`",
            "`S = 0`",
        ],
    )?;
    if spec_14.body.contains("MotorRecurrentStateV1") {
        return Err(
            "DOCS_PACKET_18_INTEGRATION_CONTRACT_DRIFT: SPEC-14: stale MotorRecurrentStateV1"
                .to_owned(),
        );
    }

    let spec_17 = documents
        .values()
        .find(|document| document.id == "SPEC-17" && document.status == "Accepted")
        .ok_or_else(|| {
            "DOCS_PACKET_18_INTEGRATION_DOCUMENT_MISSING_OR_NOT_ACCEPTED: SPEC-17".to_owned()
        })?;
    require_contract_literals(
        &spec_17.body,
        "SPEC-17",
        "DOCS_PACKET_18_INTEGRATION_CONTRACT_DRIFT",
        &[
            "`RecoverySessionLinkV1`",
            "same exact `ProjectCompositionLock`",
            "not another project activation",
        ],
    )
}

pub fn validate_p0_packet_18_documents(documents: &[(&str, &str)]) -> Result<(), String> {
    const IDS: [&str; 13] = [
        "SPEC-22", "SPEC-23", "SPEC-24", "SPEC-25", "SPEC-26", "SPEC-27", "SPEC-28", "SPEC-29",
        "SPEC-30", "ADR-025", "ADR-026", "ADR-027", "ADR-028",
    ];
    let mut by_id = BTreeMap::new();
    for (declared_id, body) in documents {
        let metadata_id = table_field(body, "ID").unwrap_or_default();
        let status = table_field(body, "Статус").unwrap_or_default();
        if metadata_id != *declared_id {
            return Err(format!(
                "DOCS_PACKET_18_P0_DOCUMENT_ID_MISMATCH: {declared_id}: {metadata_id}"
            ));
        }
        if status != "Accepted" {
            return Err(format!(
                "DOCS_PACKET_18_P0_DOCUMENT_STATUS_INVALID: {declared_id}: {status}"
            ));
        }
        if by_id.insert(*declared_id, *body).is_some() {
            return Err(format!(
                "DOCS_PACKET_18_P0_DOCUMENT_DUPLICATE: {declared_id}"
            ));
        }
    }
    for id in IDS {
        if !by_id.contains_key(id) {
            return Err(format!("DOCS_PACKET_18_P0_DOCUMENT_MISSING: {id}"));
        }
    }

    for (id, literals) in [
        (
            "SPEC-22",
            &[
                "`SchemaDescriptorV1`",
                "`SchemaRegistryManifestV1`",
                "`SchemaCompatibilityClassV1`",
                "`MigrationPlanV1`",
                "N-2",
                "copy-on-write",
                "SCHEMA-P1",
                "COMPAT-P1",
                "MIGRATION-P1",
                "REQ-112",
                "REQ-115",
                "FAIL-044",
                "FAIL-045",
            ][..],
        ),
        (
            "ADR-025",
            &[
                "Schema",
                "content",
                "migration",
                "`field_id`",
                "copy-on-write",
            ][..],
        ),
        (
            "SPEC-23",
            &[
                "`JobClassV1`",
                "`MemoryBudgetProfileV1`",
                "`ComputeResourcePinV1`",
                "`ComputeResourceLeaseV1`",
                "`ArchiveDecompressionLimitsV1`",
                "JOB-P1",
                "RESOURCE-MEMORY-P1",
                "RESOURCE-RESIDENCY-P1",
                "IO-BACKPRESSURE-P1",
                "REQ-116",
                "REQ-119",
                "FAIL-046",
                "FAIL-047",
            ][..],
        ),
        (
            "ADR-026",
            &[
                "deterministic",
                "job",
                "resource",
                "backpressure",
                "authoritative",
            ][..],
        ),
        (
            "SPEC-24",
            &[
                "`ContentManifestV1`",
                "`ContentBundleV1`",
                "`NeutralSceneV1`",
                "`NeutralMeshV1`",
                "`NeutralMaterialV1`",
                "`NeutralTextureV1`",
                "`NeutralSkeletonV1`",
                "`NeutralAnimationV1`",
                "`NeutralAudioV1`",
                "`NeutralCollisionV1`",
                "`NeutralNavigationV1`",
                "`NeutralWorldChunkV1`",
                "`TargetCapabilitySetV1`",
                "`VariantFallbackPlanV1`",
                "`fallback_target_profile_id`",
                "`CONTENT_VARIANT_CAPABILITY_UNSATISFIED`",
                "CONTENT-P1",
                "BUNDLE-P1",
                "VARIANT-P1",
                "REQ-120",
                "REQ-123",
                "FAIL-048",
                "FAIL-049",
            ][..],
        ),
        (
            "SPEC-25",
            &[
                "`WorldPartitionManifestV1`",
                "`StreamingAdmissionPlanV1`",
                "`PersistentSpatialObjectV1`",
                "`CrossChunkReferenceV1`",
                "`TierExecutionProfile`",
                "`initial_placement_catalog_hash`",
                "`WorldPlacementStateV1`",
                "`SpatialPlacementStateRootV1`",
                "`WORLD_PLACEMENT_ROOT_MISMATCH`",
                "WORLD-TOPOLOGY-P1",
                "WORLD-OBJECT-P1",
                "WORLD-ABSTRACT-P1",
                "WORLD-STREAM-P1",
                "REQ-124",
                "REQ-127",
                "FAIL-050",
                "FAIL-051",
            ][..],
        ),
        (
            "SPEC-26",
            &[
                "`PhysicsWorldDescriptorV1`",
                "`PhysicsBodyDescriptorV1`",
                "`PhysicsShapeDescriptorV1`",
                "`PhysicsJointDescriptorV1`",
                "`PhysicsQueryBatchV1`",
                "`ContactEventV1`",
                "`PhysicsCanonicalSnapshotV1`",
                "`ExactCanonical`",
                "`QuantizedExact`",
                "`ToleranceDiagnosticOnly`",
                "PHYS-API-P1",
                "PHYS-COLLISION-P1",
                "PHYS-JOINT-P1",
                "PHYS-QUERY-P1",
                "PHYS-SNAPSHOT-P1",
                "REQ-128",
                "REQ-131",
                "FAIL-052",
                "FAIL-053",
            ][..],
        ),
        (
            "ADR-027",
            &["physics", "motor", "animation", "authority", "presentation"][..],
        ),
        (
            "SPEC-27",
            &[
                "`MotorObservationSchemaV1`",
                "`MotorActionSchemaV1`",
                "`MotorInferenceBatchV1`",
                "`PolicyStateRecordV1`",
                "`MotorSafetyEnvelopeV1`",
                "`ProceduralMotorControllerV1`",
                "`authoritative_state_hash`",
                "`PolicyStateCommitV1`",
                "`MOTOR_STATE_HASH_MISMATCH`",
                "(motor_tick, PersistentId, PolicyId)",
                "MOTOR-SCHEMA-P1",
                "MOTOR-SCHEDULE-P1",
                "MOTOR-SAFETY-P1",
                "MOTOR-STATE-P1",
                "MOTOR-ROUTE-P1",
                "REQ-132",
                "REQ-135",
                "FAIL-054",
                "FAIL-055",
            ][..],
        ),
        (
            "SPEC-28",
            &[
                "`SkeletonDescriptorV1`",
                "`AnimationClipDescriptorV1`",
                "`AnimationGraphDescriptorV1`",
                "`RootMotionIntentV1`",
                "`RetargetProfileV1`",
                "`PhysicalIkConstraintV1`",
                "`PresentationIkRequestV1`",
                "`AnimationLodProfileV1`",
                "ANIM-GRAPH-P1",
                "ANIM-ROOT-MOTION-P1",
                "ANIM-RETARGET-P1",
                "ANIM-IK-P1",
                "ANIM-LOD-P1",
                "REQ-136",
                "REQ-139",
                "FAIL-056",
                "FAIL-057",
            ][..],
        ),
        (
            "SPEC-29",
            &[
                "`PlatformCapabilitySetV1`",
                "`PlatformEventV1`",
                "`NormalizedControlEventV1`",
                "`PlatformTimebaseV1`",
                "`ApplicationSessionManifestV1`",
                "`ApplicationSessionStateV1`",
                "`CloseSessionOperationJournalV1`",
                "`SessionFinalSaveLedgerV1`",
                "`canonical_close_request_hash`",
                "`FinalSaveReservationBodyV1`",
                "nextengine.final-save-reservation.v1",
                "nextengine.session-final-save-ledger.v1",
                "`attempt_count: u16 = 0`",
                "`k = attempt_count + 1`",
                "`attempt_count = k`",
                "`FinalSaveReceiptV1`",
                "`RecoverySessionLinkV1`",
                "`prior_application_session_manifest_hash`",
                "nextengine.application-session-state.v1",
                "`CloseSessionResultV1`",
                "`RetryPending`",
                "`FinalSaveRequiredFailed`",
                "`ClosedUsingLastSafeGeneration`",
                "CloseSessionReceiptV1",
                "CompositionStaged",
                "RuntimeStaged",
                "Quiescing",
                "Finalizing",
                "Closed",
                "PLATFORM-HOST-P1",
                "PLATFORM-INPUT-P1",
                "SESSION-P1",
                "SESSION-RECOVERY-P1",
                "REQ-140",
                "REQ-143",
                "FAIL-058",
                "FAIL-059",
            ][..],
        ),
        (
            "SPEC-30",
            &[
                "`PresentationSnapshotV2`",
                "PresentationObjectKeyV1",
                "CanonicalPresentationBatchV1",
                "`PresentationCueEnvelopeV1`",
                "`cue_kind_rank`",
                "`MaterialDefinitionV1`",
                "`MaterialInstanceV1`",
                "`ShaderInterfaceManifestV1`",
                "`PresentationPipelineKeyV1`",
                "`SdrColorProfileV1`",
                "`HdrPresentationBoundaryV1`",
                "VfxCueV1",
                "`PresentationConsumptionStateV1`",
                "`continuous_instance_id_or_none`",
                "`target_cue_id_or_none`",
                "`acknowledgment_key`",
                "`pending_one_shots`",
                "`pending_one_shot_root`",
                "`PrimaryEligible`",
                "`PresentationRealizationOutcomeV1`",
                "`PresentationConsumptionEvidenceV1`",
                "`SubmittedNotObserved`",
                "`presentation_snapshot_hash`",
                "`presentation_consumption_state_hash`",
                "nextengine.vfx-cue-prefix-empty.v1",
                "`PresentationCacheManifestV1`",
                "PRESENTATION-P1",
                "MATERIAL-P1",
                "COLOR-P1",
                "VFX-P1",
                "PRESENTATION-CACHE-P1",
                "REQ-144",
                "REQ-147",
                "FAIL-060",
                "FAIL-061",
            ][..],
        ),
        (
            "ADR-028",
            &[
                "Platform",
                "session",
                "presentation",
                "Created",
                "CompositionStaged",
                "RuntimeStaged",
                "Quiescing",
                "Finalizing",
                "Closed",
                "`CloseSessionOperationJournalV1`",
                "`FinalSaveReceiptV1`",
                "`RecoverySessionLinkV1`",
                "CloseSessionResultV1",
                "`PresentationConsumptionStateV1`",
            ][..],
        ),
    ] {
        require_contract_literals(by_id[id], id, "DOCS_PACKET_18_P0_CONTRACT_DRIFT", literals)?;
    }
    Ok(())
}

pub fn validate_foundation_packet_17_documents(documents: &[(&str, &str)]) -> Result<(), String> {
    const IDS: [&str; 8] = [
        "SPEC-17", "ADR-018", "SPEC-18", "ADR-019", "SPEC-19", "ADR-020", "SPEC-20", "ADR-021",
    ];
    let mut by_id = BTreeMap::new();
    for (declared_id, body) in documents {
        let metadata_id = table_field(body, "ID").unwrap_or_default();
        let status = table_field(body, "Статус").unwrap_or_default();
        if metadata_id != *declared_id {
            return Err(format!(
                "DOCS_PACKET_17_FOUNDATION_DOCUMENT_ID_MISMATCH: {declared_id}: {metadata_id}"
            ));
        }
        if status != "Accepted" {
            return Err(format!(
                "DOCS_PACKET_17_FOUNDATION_DOCUMENT_STATUS_INVALID: {declared_id}: {status}"
            ));
        }
        if by_id.insert(*declared_id, *body).is_some() {
            return Err(format!(
                "DOCS_PACKET_17_FOUNDATION_DOCUMENT_DUPLICATE: {declared_id}"
            ));
        }
    }
    for id in IDS {
        if !by_id.contains_key(id) {
            return Err(format!("DOCS_PACKET_17_FOUNDATION_DOCUMENT_MISSING: {id}"));
        }
    }

    validate_project_composition_contract(by_id["SPEC-17"], by_id["ADR-018"])?;
    validate_player_experience_contract(by_id["SPEC-18"], by_id["ADR-019"])?;
    validate_rpg_domain_contract(by_id["SPEC-19"], by_id["ADR-020"])?;
    validate_world_services_contract(by_id["SPEC-20"], by_id["ADR-021"])
}

fn validate_project_composition_contract(spec: &str, adr: &str) -> Result<(), String> {
    let resolver = foundation_contract_section(
        spec,
        "### ProjectCatalogSnapshot и deterministic resolver",
        "### ProjectCompositionLock",
        "SPEC-17",
        "DOCS_PACKET_17_PROJECT_CONTRACT_DRIFT",
    )?;
    require_contract_literals(
        resolver,
        "SPEC-17",
        "DOCS_PACKET_17_PROJECT_CONTRACT_DRIFT",
        &[
            "`ProjectCatalogSnapshot` является immutable JCS-canonical input resolver",
            "`catalog_snapshot_sha256 = SHA-256(JCS(body))`",
            "`record_sha256 = SHA-256(JCS(record_body))`",
            "удовлетворяет всем accumulated SemVer/engine/schema/target/capability/trust/budget constraints и `yanked = false`",
            "exact full prerelease SemVer в closed `prerelease_admissions`",
            "SemVer 2.0.0 precedence descending",
            "lexicographically smallest 32-byte `record_sha256`",
            "deterministically backtrack-ит до первой полной compatible closure",
            "`ProjectResolutionConflictReport`",
            "equal invalid inputs MUST produce byte-identical report and diagnostic",
            "Runtime roots MUST NOT читать registry, network/cache state, выбирать package version или повторно разрешать floating range",
        ],
    )?;

    let lock = foundation_contract_section(
        spec,
        "### ProjectCompositionLock",
        "### LaunchProfile и ConfigurationClass",
        "SPEC-17",
        "DOCS_PACKET_17_PROJECT_CONTRACT_DRIFT",
    )?;
    require_contract_literals(
        lock,
        "SPEC-17",
        "DOCS_PACKET_17_PROJECT_CONTRACT_DRIFT",
        &[
            "project/manifest identity и canonical hash",
            "catalog snapshot, resolver profile and canonical resolution-trace hashes",
            "engine build/contract, schema registry and runtime-determinism profile hashes",
            "ContentManifest/content closure, MechanicsLock/package dependency DAG and selected catalog-record hashes",
            "Luau/Wasm artifact, WIT/API compatibility, `PackageTrustManifestV1`, trust-policy and effective-capability hashes",
            "`GameplayBudgetMatrix`, extension/resource budget policy and configuration-schema hashes",
            "canonical authoritative configuration and selected `LaunchProfile` authoritative-subset hashes",
            "physical archetype, policy/model catalog and required state-schema hashes",
            "ordered migration set, target-independent project data",
            "resolved fallback for every optional dependency",
            "lock schema version и canonical closure hash",
        ],
    )?;
    require_contract_literals(
        adr,
        "ADR-018",
        "DOCS_PACKET_17_PROJECT_CONTRACT_DRIFT",
        &[
            "exact immutable `ProjectCatalogSnapshot`",
            "ranked by SemVer 2.0.0 precedence descending",
            "exact full prerelease SemVer explicitly admitted",
            "lexicographically smallest canonical catalog-record SHA-256 as tie-break",
            "one canonical `ProjectResolutionConflictReport`",
            "Runtime MUST NOT resolve floating ranges",
            "Lock MUST bind exact catalog/resolver trace; engine/schema/runtime profile; content; package/script/plugin/model; trust/capability; budget; authoritative configuration; migration; fallback; and target presentation hashes",
        ],
    )
}

fn validate_player_experience_contract(spec: &str, adr: &str) -> Result<(), String> {
    let action_frame = foundation_contract_section(
        spec,
        "### PlayerActionFrame",
        "## Camera, aiming и targeting",
        "SPEC-18",
        "DOCS_PACKET_17_PLAYER_CONTRACT_DRIFT",
    )?;
    require_contract_literals(
        action_frame,
        "SPEC-18",
        "DOCS_PACKET_17_PLAYER_CONTRACT_DRIFT",
        &[
            "`PlayerActionFrame` contains schema version",
            "Physical device/source identity, native sample reference, wall-clock timestamp, target tick, camera transform and backend object MUST NOT appear in canonical frame bytes",
            "Interactive input service and headless scenario producer enqueue the same `PlayerActionFrame` schema through the same gateway",
            "enqueue before the barrier belongs to current tick, while enqueue linearized in or after the barrier belongs to next tick",
            "The production mapper derives command target tick only from the persisted assignment",
            "The frame is production input, not committed gameplay",
            "common capability/schema/domain/physics validation still decides outcome",
        ],
    )?;
    require_contract_literals(
        spec,
        "SPEC-18",
        "DOCS_PACKET_17_PLAYER_CONTRACT_DRIFT",
        &[
            "`CameraIntent`",
            "`TargetingIntent`",
            "`AuthoritativeTargetingQueryV1`",
            "`UiSemanticSnapshot`",
            "`PlayerPreferenceProfile`",
            "the data flow never reverses",
            "Localized dialogue presentation cannot alter Dialogue node, choice ID, commitment or command payload",
            "Accessibility alternative MUST invoke the same action ID and validation path as default input",
        ],
    )?;
    require_contract_literals(
        adr,
        "ADR-019",
        "DOCS_PACKET_17_PLAYER_CONTRACT_DRIFT",
        &[
            "canonical device-independent immutable `PlayerActionFrame` bytes",
            "ADR-022 current/next close barrier alone creates persisted `IngressAssignmentV1`",
            "frame, wall clock, renderer cadence and scenario cannot choose target tick",
            "Player action is an input proposal, not authority",
            "Camera state is presentation-only",
            "UI reads immutable `UiSemanticSnapshot` projections",
            "Localization/accessibility/preferences cannot change command ordering, simulation schedule or authoritative outcome",
        ],
    )
}

fn validate_rpg_domain_contract(spec: &str, adr: &str) -> Result<(), String> {
    let aggregate = foundation_contract_section(
        spec,
        "## Common aggregate contract",
        "## Aggregate payloads",
        "SPEC-19",
        "DOCS_PACKET_17_RPG_CONTRACT_DRIFT",
    )?;
    require_contract_literals(
        aggregate,
        "SPEC-19",
        "DOCS_PACKET_17_RPG_CONTRACT_DRIFT",
        &[
            "`RpgAggregateEnvelopeV1`",
            "closed enum `Character`, `Item`, `Inventory`, `Equipment`, `Quest`, `Dialogue`, `Faction`, `FactionMembership`, `Relationship` or `InteractiveObject`",
            "`persistent_id` is a `PersistentId`",
            "`schema_version` is a positive `u32`",
            "`revision` is a `u64` optimistic-concurrency counter",
            "increments every changed aggregate from `r` to `r + 1` exactly once",
            "`RPG_REVISION_EXHAUSTED`",
        ],
    )?;

    let operations = foundation_contract_section(
        spec,
        "## Typed engine-owned operations",
        "## Immutable `RpgTransactionPlan`",
        "SPEC-19",
        "DOCS_PACKET_17_RPG_CONTRACT_DRIFT",
    )?;
    require_contract_literals(
        operations,
        "SPEC-19",
        "DOCS_PACKET_17_RPG_CONTRACT_DRIFT",
        &[
            "`RpgCommandV1`",
            "`RpgOperationV1`",
            "contiguous `operation_slot: u32`",
            "sorted unique target refs `(aggregate_kind, PersistentId, expected_revision)`",
            "`RpgCommandV1` and `CanonicalCommandBodyV2` MUST NOT contain `command_id`",
            "authoritative causal command ID is a validator result computed under ADR-022",
        ],
    )?;

    let plan = foundation_contract_section(
        spec,
        "## Immutable `RpgTransactionPlan`",
        "## Stable event order",
        "SPEC-19",
        "DOCS_PACKET_17_RPG_CONTRACT_DRIFT",
    )?;
    require_contract_literals(
        plan,
        "SPEC-19",
        "DOCS_PACKET_17_RPG_CONTRACT_DRIFT",
        &[
            "versioned engine-owned `crates/contracts` handoff",
            "schema = \"nextengine.rpg-transaction-plan.v1\"",
            "ordered_read_set[]",
            "ordered_write_set[]",
            "ordered_event_drafts[]",
            "The plan contains no mutable reference, ECS/entity handle, database transaction, task handle, VM object, callback or backend/vendor value",
            "staged copy-on-write in an isolated transaction buffer",
            "atomically publishes the whole write set, terminal receipt and committed event batch",
        ],
    )?;

    let event_order = foundation_contract_section(
        spec,
        "## Stable event order",
        "## Proposal and read-only boundaries",
        "SPEC-19",
        "DOCS_PACKET_17_RPG_CONTRACT_DRIFT",
    )?;
    let compact_event_order = compact_contract_text(event_order);
    let expected_event_order = compact_contract_text(
        "(operation_slot, event_local_slot, event_schema_id_nfc_utf8, primary_aggregate_kind_tag, primary_persistent_id_bytes)",
    );
    if !compact_event_order.contains(&expected_event_order)
        || !event_order.contains("canonical ADR-022 event slot")
        || !event_order.contains("`RPG_EVENT_ORDER_INVALID`")
    {
        return Err("DOCS_PACKET_17_RPG_CONTRACT_DRIFT: SPEC-19: canonical event order".to_owned());
    }

    let migrations = foundation_contract_section(
        spec,
        "## Persistence и copy-on-write migrations",
        "## Compositional budget ownership",
        "SPEC-19",
        "DOCS_PACKET_17_RPG_CONTRACT_DRIFT",
    )?;
    require_contract_literals(
        migrations,
        "SPEC-19",
        "DOCS_PACKET_17_RPG_CONTRACT_DRIFT",
        &[
            "`RpgMigrationRegistryV1`",
            "N-2 --migration[N-2,N-1]--> N-1 --migration[N-1,N]--> N",
            "records are copied into an isolated working generation",
            "migration preserves aggregate domain revisions, causal/idempotency history and tombstones and emits no gameplay `DomainEvent`",
            "only then may Persistence atomically publish a new save generation",
            "leaves original bytes/hash and prior published generation unchanged",
        ],
    )?;
    require_contract_literals(
        adr,
        "ADR-020",
        "DOCS_PACKET_17_RPG_CONTRACT_DRIFT",
        &[
            "sole owner of revisioned `Character`, `Item`, `Inventory`, `Equipment`, `Quest`, `Dialogue`, `Faction`, `FactionMembership`, `Relationship` and `InteractiveObject`",
            "closed typed engine-owned operation inside `WorldCommand`",
            "Only RPG validation constructs immutable `RpgTransactionPlan`",
            "`validate → stage copy-on-write → commit atomically`",
            "complete write set, terminal receipt and DomainEvent batch become visible together or not at all",
            "exact adjacent copy-on-write chain `N-2 → N-1 → N`",
            "Any failure preserves original bytes and the prior generation",
        ],
    )
}

fn validate_world_services_contract(spec: &str, adr: &str) -> Result<(), String> {
    let calendar = foundation_contract_section(
        spec,
        "## World calendar contract",
        "## Population contracts",
        "SPEC-20",
        "DOCS_PACKET_17_WORLD_CONTRACT_DRIFT",
    )?;
    require_contract_literals(
        calendar,
        "SPEC-20",
        "DOCS_PACKET_17_WORLD_CONTRACT_DRIFT",
        &[
            "`WorldCalendarStateV1` is an engine-owned public value stored only in the World Services save segment",
            "world_tick: u64",
            "calendar_definition_id: AssetId",
            "calendar_definition_hash: ContentHash",
            "calendar_units_per_world_tick_num: u64",
            "calendar_units_per_world_tick_den: u64",
            "revision: u64",
            "last_causal_command_id: Option<CommandId>",
            "Calendar revision increments exactly once per committed advance plan",
        ],
    )?;
    let compact_calendar = compact_contract_text(calendar);
    let exact_calendar_formula = compact_contract_text(
        "calendar_unit(t) = epoch_calendar_unit + floor((t - epoch_world_tick) * calendar_units_per_world_tick_num / calendar_units_per_world_tick_den)",
    );
    if !compact_calendar.contains(&exact_calendar_formula) {
        return Err(
            "DOCS_PACKET_17_WORLD_CONTRACT_DRIFT: SPEC-20: exact calendar formula".to_owned(),
        );
    }

    let unsupported = foundation_contract_section(
        spec,
        "### AbstractActivityState и unsupported outcomes",
        "### WorldAdvancePlanV1",
        "SPEC-20",
        "DOCS_PACKET_17_WORLD_CONTRACT_DRIFT",
    )?;
    require_contract_literals(
        unsupported,
        "SPEC-20",
        "DOCS_PACKET_17_WORLD_CONTRACT_DRIFT",
        &[
            "emit one revision-bound upgrade proposal and leave the activity outcome/cursor uncommitted",
            "`DeferredUnsupported`",
            "`WORLD_ABSTRACT_OUTCOME_BLOCKED`",
            "No branch may read elapsed wall time",
            "emit a success event",
            "advance the schedule cursor",
        ],
    )?;
    let compact_unsupported = compact_contract_text(unsupported);
    if !compact_unsupported.contains(
        "next_retry_world_tick=current_world_tick+min(retry_interval,maximum_deferral_tick-current_world_tick)",
    ) {
        return Err(
            "DOCS_PACKET_17_WORLD_CONTRACT_DRIFT: SPEC-20: deterministic upgrade/defer formula"
                .to_owned(),
        );
    }

    let advance = foundation_contract_section(
        spec,
        "### WorldAdvancePlanV1",
        "### RegionTransfer",
        "SPEC-20",
        "DOCS_PACKET_17_WORLD_CONTRACT_DRIFT",
    )?;
    require_contract_literals(
        advance,
        "SPEC-20",
        "DOCS_PACKET_17_WORLD_CONTRACT_DRIFT",
        &[
            "Stepped and bulk time use the same boundary evaluator, operation order and canonical plan partition",
            "same command/event/cursor/state hashes as stepped execution",
            "Each bounded plan commits atomically and increments calendar revision once",
        ],
    )?;

    let migration = foundation_contract_section(
        spec,
        "## Calendar ownership migration",
        "## Save, replay и concurrency",
        "SPEC-20",
        "DOCS_PACKET_17_WORLD_CONTRACT_DRIFT",
    )?;
    require_contract_literals(
        migration,
        "SPEC-20",
        "DOCS_PACKET_17_WORLD_CONTRACT_DRIFT",
        &[
            "`world-calendar-rpg-to-world-services-v1`",
            "pure copy transform",
            "stage a World Services segment",
            "stage an RPG segment with all legacy calendar value fields removed",
            "atomically publish a new save generation",
            "The source generation is immutable and remains the recovery generation",
            "`WORLD_CALENDAR_AUTHORITY_CONFLICT`",
            "`WORLD_CALENDAR_STATE_MISSING`",
            "`WORLD_CALENDAR_MIGRATION_INCOMPATIBLE`",
            "`WORLD_CALENDAR_MIGRATION_ABORTED`",
        ],
    )?;
    require_contract_literals(
        adr,
        "ADR-021",
        "DOCS_PACKET_17_WORLD_CONTRACT_DRIFT",
        &[
            "World Services Team является единственным владельцем `WorldCalendarStateV1`",
            "Migration ID is `world-calendar-rpg-to-world-services-v1`",
            "This matrix is exhaustive",
            "Unsupported work never emits success",
            "Stepped and bulk modes call the same boundary evaluator, canonical plan partition and command validators",
            "MUST produce exact equal calendar revision, command, event, owner-state and schedule-cursor hashes",
            "Any authoritative failure before `Commit` retains the complete source tier",
        ],
    )
}

fn foundation_contract_section<'a>(
    body: &'a str,
    start: &str,
    end: &str,
    document: &str,
    code: &str,
) -> Result<&'a str, String> {
    contract_section(body, start, end, document)
        .map_err(|_| format!("{code}: {document}: missing section {start}"))
}

fn validate_requirement_gate_contract(
    traceability: &TraceabilityContract,
    documents: &BTreeMap<String, Document>,
    vertical: &str,
    technology_registry: Option<&BTreeMap<String, TechnologyEntry>>,
) -> Result<(), String> {
    let catalog = collect_gate_catalog(documents)?;
    if let Some(technology_registry) = technology_registry {
        validate_candidate_only_technology_links(&catalog, technology_registry)?;
    }
    validate_gate_descriptor_ownership(&catalog)?;
    validate_traceability_document_ownership(traceability, documents)?;
    validate_requirement_definition_bijection(traceability, documents)?;
    let vertical_children = validate_vertical_slice_children(vertical, &catalog)?;
    let vertical_ids = vertical_children.keys().cloned().collect::<BTreeSet<_>>();
    let declared_profiles = collect_declared_profiles(documents)?;

    let perf_row = traceability
        .requirements
        .get("REQ-111")
        .ok_or_else(|| "DOCS_PERF_REQUIREMENT_MISSING: REQ-111".to_owned())?;
    if !perf_row.gates.contains("PERF-01") {
        return Err("DOCS_PERF_REQUIREMENT_MAPPING_MISSING: REQ-111 -> PERF-01".to_owned());
    }
    if !vertical_children
        .get("VS-12")
        .is_some_and(|children| children.contains("PERF-01"))
    {
        return Err("DOCS_PERF_BLOCKING_CHILD_MISSING: VS-12 -> PERF-01".to_owned());
    }

    let accepted_definitions = catalog
        .definitions
        .iter()
        .filter_map(|(gate_id, definitions)| {
            definitions
                .iter()
                .any(|definition| definition.document_status == "Accepted")
                .then_some(gate_id.clone())
        })
        .collect::<BTreeSet<_>>();
    let mut referenced = BTreeSet::new();
    let mut trace_gate_references = BTreeSet::new();

    for row in traceability
        .requirements
        .values()
        .chain(traceability.failures.values())
    {
        validate_traceability_closure_row(row)?;
        for closure_id in &row.closure {
            if !vertical_ids.contains(closure_id) && !declared_profiles.contains(closure_id) {
                return Err(format!(
                    "DOCS_TRACEABILITY_CLOSURE_UNKNOWN: {} -> {closure_id}",
                    row.id
                ));
            }
            if catalog.definitions.contains_key(closure_id) {
                referenced.insert(closure_id.clone());
            }
        }
        let mut has_baseline_gate = false;
        for gate_id in &row.gates {
            referenced.insert(gate_id.clone());
            trace_gate_references.insert(gate_id.clone());
            let Some(definitions) = catalog.definitions.get(gate_id) else {
                return Err(format!("DOCS_UNKNOWN_GATE: {} -> {gate_id}", row.id));
            };
            let has_accepted_definition = definitions
                .iter()
                .any(|definition| definition.document_status == "Accepted");
            if has_accepted_definition {
                if !catalog.candidate_only.contains(gate_id) && row.gates.contains(gate_id) {
                    has_baseline_gate = true;
                }
                continue;
            }
            if catalog.candidate_only.contains(gate_id)
                && definitions.iter().any(|definition| {
                    definition.document_status == "Proposed" && definition.is_complete()
                })
            {
                continue;
            }
            if !definitions
                .iter()
                .any(|definition| definition.document_status == "Proposed")
            {
                return Err(format!(
                    "DOCS_GATE_DEFINITION_STATUS_INCOMPATIBLE: {} -> {gate_id}",
                    row.id
                ));
            }
            return Err(format!("DOCS_PROPOSED_ONLY_GATE: {} -> {gate_id}", row.id));
        }
        if !has_baseline_gate {
            return Err(format!("DOCS_CANDIDATE_ONLY_GATE_CANNOT_CLOSE: {}", row.id));
        }
    }

    for (vertical_id, children) in &vertical_children {
        for child in children {
            referenced.insert(child.clone());
            let Some(definitions) = catalog.definitions.get(child) else {
                return Err(format!(
                    "DOCS_VERTICAL_CHILD_UNKNOWN: {vertical_id} -> {child}"
                ));
            };
            if !definitions
                .iter()
                .any(|definition| definition.document_status == "Accepted")
            {
                return Err(format!(
                    "DOCS_VERTICAL_CHILD_STATUS_INCOMPATIBLE: {vertical_id} -> {child}"
                ));
            }
            if !trace_gate_references.contains(child) {
                return Err(format!(
                    "DOCS_VERTICAL_CHILD_REVERSE_MAPPING_MISSING: {vertical_id} -> {child}"
                ));
            }
        }
    }

    for gate_id in &accepted_definitions {
        let definitions = &catalog.definitions[gate_id];
        if !definitions
            .iter()
            .any(|definition| definition.document_status == "Accepted" && definition.is_complete())
        {
            let sources = definitions
                .iter()
                .map(|definition| format!("{}:{}", definition.document_id, definition.source))
                .collect::<Vec<_>>()
                .join(", ");
            return Err(format!(
                "DOCS_GATE_DESCRIPTOR_INCOMPLETE: {gate_id}: {sources}"
            ));
        }
        if !referenced.contains(gate_id) && !catalog.candidate_only.contains(gate_id) {
            return Err(format!("DOCS_ORPHAN_GATE: {gate_id}"));
        }
    }

    for gate_id in &catalog.candidate_only {
        let Some(definitions) = catalog.definitions.get(gate_id) else {
            return Err(format!("DOCS_CANDIDATE_ONLY_DESCRIPTOR_MISSING: {gate_id}"));
        };
        if !definitions.iter().any(|definition| {
            matches!(definition.document_status.as_str(), "Accepted" | "Proposed")
                && definition.is_complete()
        }) {
            return Err(format!(
                "DOCS_CANDIDATE_ONLY_DESCRIPTOR_INCOMPLETE: {gate_id}"
            ));
        }
    }

    Ok(())
}

fn validate_candidate_only_technology_links(
    catalog: &GateCatalog,
    technologies: &BTreeMap<String, TechnologyEntry>,
) -> Result<(), String> {
    for gate_id in &catalog.candidate_only {
        let subject = catalog
            .candidate_subjects
            .get(gate_id)
            .ok_or_else(|| format!("DOCS_CANDIDATE_ONLY_SUBJECT_MISSING: {gate_id}"))?;
        let technology = technologies.get(subject).ok_or_else(|| {
            format!("DOCS_CANDIDATE_ONLY_TECHNOLOGY_MISSING: {gate_id} -> {subject}")
        })?;
        if technology.status != "Proposed" {
            return Err(format!(
                "DOCS_CANDIDATE_ONLY_TECHNOLOGY_STATUS_INVALID: {gate_id} -> {subject} ({})",
                technology.status
            ));
        }
        if !technology.gates.contains(gate_id) {
            return Err(format!(
                "DOCS_CANDIDATE_ONLY_TECHNOLOGY_GATE_MISMATCH: {gate_id} -> {subject}"
            ));
        }
    }
    Ok(())
}

fn validate_traceability_document_ownership(
    traceability: &TraceabilityContract,
    documents: &BTreeMap<String, Document>,
) -> Result<(), String> {
    let documents_by_id = documents
        .values()
        .map(|document| (document.id.as_str(), document))
        .collect::<BTreeMap<_, _>>();
    for row in traceability
        .requirements
        .values()
        .chain(traceability.failures.values())
    {
        let mut accepted_count = 0_usize;
        for document_id in &row.document_ids {
            let document = documents_by_id.get(document_id.as_str()).ok_or_else(|| {
                format!(
                    "DOCS_TRACEABILITY_DOCUMENT_UNKNOWN: {} -> {document_id}",
                    row.id
                )
            })?;
            if document.status != "Accepted" {
                return Err(format!(
                    "DOCS_TRACEABILITY_DOCUMENT_STATUS_INCOMPATIBLE: {} -> {document_id} ({})",
                    row.id, document.status
                ));
            }
            accepted_count += 1;
        }
        if accepted_count == 0 {
            return Err(format!(
                "DOCS_TRACEABILITY_ACCEPTED_OWNER_DOCUMENT_MISSING: {}",
                row.id
            ));
        }
    }
    Ok(())
}

fn validate_requirement_definition_bijection(
    traceability: &TraceabilityContract,
    documents: &BTreeMap<String, Document>,
) -> Result<(), String> {
    let definitions = collect_requirement_definitions(documents)?;
    let mut seen = BTreeMap::new();
    let mut definitions_by_document: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    for definition in definitions {
        if let Some(previous) = seen.insert(definition.id.clone(), definition.document_id.clone()) {
            return Err(format!(
                "DOCS_REQUIREMENT_DEFINITION_DUPLICATE: {}: {previous}, {}",
                definition.id, definition.document_id
            ));
        }
        definitions_by_document
            .entry(definition.document_id.clone())
            .or_default()
            .insert(definition.id.clone());

        if definition.document_status == "Accepted" {
            let row = if definition.id.starts_with("REQ-") {
                traceability.requirements.get(&definition.id)
            } else {
                traceability.failures.get(&definition.id)
            }
            .ok_or_else(|| {
                format!(
                    "DOCS_REQUIREMENT_DEFINITION_TRACE_MISSING: {} -> {}",
                    definition.document_id, definition.id
                )
            })?;
            if row.owner != definition.owner {
                return Err(format!(
                    "DOCS_REQUIREMENT_DEFINITION_OWNER_MISMATCH: {} in {}: expected {}, got {}",
                    definition.id, definition.document_id, row.owner, definition.owner
                ));
            }
            if row.gates != definition.gates {
                return Err(format!(
                    "DOCS_REQUIREMENT_DEFINITION_GATE_MISMATCH: {} in {}",
                    definition.id, definition.document_id
                ));
            }
            if !row.document_ids.contains(&definition.document_id) {
                return Err(format!(
                    "DOCS_REQUIREMENT_DEFINITION_DOCUMENT_EDGE_MISSING: {} -> {}",
                    definition.id, definition.document_id
                ));
            }
        } else if definition.document_status == "Proposed" {
            let reservation = traceability
                .reservations
                .get(&definition.id)
                .ok_or_else(|| {
                    format!(
                        "DOCS_PROPOSED_REQUIREMENT_RESERVATION_MISSING: {} -> {}",
                        definition.document_id, definition.id
                    )
                })?;
            if reservation.document != definition.document_id {
                return Err(format!(
                    "DOCS_PROPOSED_REQUIREMENT_RESERVATION_OWNER_MISMATCH: {}: expected {}, got {}",
                    definition.id, definition.document_id, reservation.document
                ));
            }
        }
    }

    if documents
        .values()
        .any(|document| document.id == "SPEC-21" && document.status == "Accepted")
    {
        let expected_spec_21 = id_set("REQ-", 103..=110)
            .into_iter()
            .chain(id_set("FAIL-", 39..=42))
            .collect::<BTreeSet<_>>();
        if definitions_by_document.get("SPEC-21") != Some(&expected_spec_21) {
            return Err(format!(
                "DOCS_SPEC_21_DEFINITION_SET_MISMATCH: expected {expected_spec_21:?}, got {:?}",
                definitions_by_document.get("SPEC-21")
            ));
        }
    }
    for (document_id, requirement_range, failure_range) in [
        ("SPEC-17", 87..=90, 31..=32),
        ("SPEC-18", 91..=94, 33..=34),
        ("SPEC-19", 95..=98, 35..=36),
        ("SPEC-20", 99..=102, 37..=38),
    ] {
        if !documents
            .values()
            .any(|document| document.id == document_id && document.status == "Accepted")
        {
            continue;
        }
        let expected = id_set("REQ-", requirement_range)
            .into_iter()
            .chain(id_set("FAIL-", failure_range))
            .collect::<BTreeSet<_>>();
        if definitions_by_document.get(document_id) != Some(&expected) {
            return Err(format!(
                "DOCS_PACKET_17_DEFINITION_SET_MISMATCH: {document_id}: expected {expected:?}, got {:?}",
                definitions_by_document.get(document_id)
            ));
        }
    }
    validate_proposed_reserved_definitions(traceability, documents)?;
    Ok(())
}

fn validate_proposed_reserved_definitions(
    traceability: &TraceabilityContract,
    documents: &BTreeMap<String, Document>,
) -> Result<(), String> {
    let definitions = collect_reserved_requirement_definitions(documents)?;
    let mut by_document: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    let mut seen = BTreeMap::new();
    for definition in definitions {
        if is_blank_contract_value(&definition.owner) {
            return Err(format!(
                "DOCS_PROPOSED_REQUIREMENT_OWNER_MISSING: {} -> {}",
                definition.document_id, definition.id
            ));
        }
        if let Some(previous) = seen.insert(definition.id.clone(), definition.document_id.clone()) {
            return Err(format!(
                "DOCS_PROPOSED_REQUIREMENT_DEFINITION_DUPLICATE: {}: {previous}, {}",
                definition.id, definition.document_id
            ));
        }
        let reservation = traceability
            .reservations
            .get(&definition.id)
            .ok_or_else(|| {
                format!(
                    "DOCS_PROPOSED_REQUIREMENT_RESERVATION_MISSING: {} -> {}",
                    definition.document_id, definition.id
                )
            })?;
        if reservation.document != definition.document_id {
            return Err(format!(
                "DOCS_PROPOSED_REQUIREMENT_RESERVATION_OWNER_MISMATCH: {}: expected {}, got {}",
                definition.id, definition.document_id, reservation.document
            ));
        }
        by_document
            .entry(definition.document_id)
            .or_default()
            .insert(definition.id);
    }

    let expected = [
        (
            "SPEC-16",
            id_set("REQ-", 79..=86)
                .into_iter()
                .chain(id_set("FAIL-", 25..=30))
                .collect::<BTreeSet<_>>(),
        ),
        (
            "SPEC-17",
            id_set("REQ-", 87..=90)
                .into_iter()
                .chain(id_set("FAIL-", 31..=32))
                .collect(),
        ),
        (
            "SPEC-18",
            id_set("REQ-", 91..=94)
                .into_iter()
                .chain(id_set("FAIL-", 33..=34))
                .collect(),
        ),
        (
            "SPEC-19",
            id_set("REQ-", 95..=98)
                .into_iter()
                .chain(id_set("FAIL-", 35..=36))
                .collect(),
        ),
        (
            "SPEC-20",
            id_set("REQ-", 99..=102)
                .into_iter()
                .chain(id_set("FAIL-", 37..=38))
                .collect(),
        ),
    ];
    for (document_id, expected_ids) in expected {
        let document_is_present = documents
            .values()
            .any(|document| document.id == document_id && document.status == "Proposed");
        if document_is_present && by_document.get(document_id) != Some(&expected_ids) {
            return Err(format!(
                "DOCS_PROPOSED_REQUIREMENT_DEFINITION_SET_MISMATCH: {document_id}: expected {expected_ids:?}, got {:?}",
                by_document.get(document_id)
            ));
        }
    }
    Ok(())
}

fn collect_reserved_requirement_definitions(
    documents: &BTreeMap<String, Document>,
) -> Result<Vec<ReservedRequirementDefinition>, String> {
    let mut definitions = Vec::new();
    for document in documents
        .values()
        .filter(|document| document.status == "Proposed")
    {
        for table in markdown_tables(&document.body) {
            if !matches!(
                table.header.first().map(String::as_str),
                Some("Reserved row") | Some("Reserved ID")
            ) {
                continue;
            }
            let owner_index =
                exact_header_index(&table.header, "Primary owner").ok_or_else(|| {
                    format!(
                        "DOCS_PROPOSED_REQUIREMENT_OWNER_COLUMN_MISSING: {}",
                        document.relative
                    )
                })?;
            for row in table.rows {
                if row.len() != table.header.len() {
                    return Err(format!(
                        "DOCS_PROPOSED_REQUIREMENT_DEFINITION_ROW_INVALID: {}",
                        document.relative
                    ));
                }
                definitions.push(ReservedRequirementDefinition {
                    id: parse_exact_requirement_or_failure_id(
                        &row[0],
                        &format!("{} reserved definition", document.relative),
                    )?,
                    document_id: document.id.clone(),
                    owner: row[owner_index].clone(),
                });
            }
        }
    }
    Ok(definitions)
}

fn collect_requirement_definitions(
    documents: &BTreeMap<String, Document>,
) -> Result<Vec<RequirementDefinition>, String> {
    let mut definitions = Vec::new();
    for document in documents
        .values()
        .filter(|document| document.id != "TRACE-001")
    {
        for table in markdown_tables(&document.body) {
            let Some(owner_index) = exact_header_index(&table.header, "Primary owner") else {
                continue;
            };
            let gate_index = exact_header_index(&table.header, "Blocking gates")
                .or_else(|| exact_header_index(&table.header, "Gates"));
            let Some(gate_index) = gate_index else {
                continue;
            };
            for row in table.rows {
                if row.len() != table.header.len() || row.is_empty() {
                    return Err(format!(
                        "DOCS_REQUIREMENT_DEFINITION_ROW_INVALID: {}",
                        document.relative
                    ));
                }
                let raw_id = strip_code_ticks(&row[0]);
                if !raw_id.starts_with("REQ-") && !raw_id.starts_with("FAIL-") {
                    continue;
                }
                let id = parse_exact_requirement_or_failure_id(
                    &raw_id,
                    &format!("{} definition", document.relative),
                )?;
                definitions.push(RequirementDefinition {
                    id,
                    document_id: document.id.clone(),
                    document_status: document.status.clone(),
                    owner: row[owner_index].clone(),
                    gates: parse_exact_gate_id_list(
                        &row[gate_index],
                        &format!("{} definition gates", document.relative),
                    )?,
                });
            }
        }
    }
    Ok(definitions)
}

fn collect_declared_profiles(
    documents: &BTreeMap<String, Document>,
) -> Result<BTreeSet<String>, String> {
    let mut profiles = BTreeSet::new();
    for document in documents
        .values()
        .filter(|document| document.status == "Accepted")
    {
        for table in markdown_tables(&document.body) {
            if !matches!(
                table.header.first().map(String::as_str),
                Some("Profile") | Some("Profile ID")
            ) {
                continue;
            }
            for row in table.rows {
                let profile = strip_code_ticks(row.first().map_or("", String::as_str));
                if !is_named_profile_id(&profile) {
                    return Err(format!(
                        "DOCS_PROFILE_ID_INVALID: {}: {profile}",
                        document.relative
                    ));
                }
                profiles.insert(profile);
            }
        }
    }
    Ok(profiles)
}

fn validate_traceability_closure_row(row: &TraceabilityRow) -> Result<(), String> {
    if is_blank_contract_value(&row.owner) {
        return Err(format!("DOCS_TRACEABILITY_OWNER_MISSING: {}", row.id));
    }
    if row.owner.contains(" + ")
        || row.owner.contains(" / ")
        || row.owner.contains(';')
        || row.owner.contains(',')
    {
        return Err(format!("DOCS_COMPOSITE_PRIMARY_OWNER: {}", row.id));
    }
    if row.gates.is_empty() {
        return Err(format!("DOCS_TRACEABILITY_GATE_MISSING: {}", row.id));
    }
    if row.closure.is_empty() {
        return Err(format!(
            "DOCS_TRACEABILITY_VS_PROFILE_CLOSURE_MISSING: {}",
            row.id
        ));
    }
    if is_blank_contract_value(&row.evidence) {
        return Err(format!("DOCS_TRACEABILITY_EVIDENCE_MISSING: {}", row.id));
    }
    Ok(())
}

fn validate_gate_descriptor_ownership(catalog: &GateCatalog) -> Result<(), String> {
    let mut conflicting_gate_ids = BTreeSet::new();
    for (gate_id, definitions) in &catalog.definitions {
        let accepted = definitions
            .iter()
            .filter(|definition| definition.document_status == "Accepted")
            .collect::<Vec<_>>();
        if let Some(expected_document) = catalog.canonical_owners.get(gate_id) {
            let complete_in_owner = accepted
                .iter()
                .filter(|definition| {
                    definition.document_id == *expected_document && definition.is_complete()
                })
                .count();
            if complete_in_owner != 1 {
                return Err(format!(
                    "DOCS_GATE_CANONICAL_DESCRIPTOR_COUNT: {gate_id}: expected exactly one complete descriptor in {expected_document}, got {complete_in_owner}"
                ));
            }
            continue;
        }
        if accepted.len() > 1 {
            let primaries = accepted
                .iter()
                .copied()
                .filter(|definition| is_blank_contract_value(&definition.alias_of))
                .collect::<Vec<_>>();
            if let Some(primary) = primaries.first() {
                let primary_signature = canonical_gate_signature(primary);
                for definition in &accepted {
                    if !is_blank_contract_value(&definition.alias_of) {
                        let alias_target = strip_code_ticks(&definition.alias_of);
                        if alias_target != primary.document_id {
                            return Err(format!(
                                "DOCS_GATE_DESCRIPTOR_ALIAS_INVALID: {gate_id}: {} aliases {alias_target}, expected {}",
                                definition.document_id, primary.document_id
                            ));
                        }
                        continue;
                    }
                    if canonical_gate_signature(definition) != primary_signature {
                        conflicting_gate_ids.insert(gate_id.clone());
                    }
                }
            } else {
                return Err(format!("DOCS_GATE_DESCRIPTOR_PRIMARY_MISSING: {gate_id}"));
            }
        }
    }
    for (gate_id, expected_document) in &catalog.canonical_owners {
        if !catalog.definitions.contains_key(gate_id) {
            return Err(format!(
                "DOCS_GATE_CANONICAL_DESCRIPTOR_MISSING: {gate_id} -> {expected_document}"
            ));
        }
    }
    if !conflicting_gate_ids.is_empty() {
        return Err(format!(
            "DOCS_GATE_DESCRIPTOR_CONFLICT: {}",
            conflicting_gate_ids
                .into_iter()
                .collect::<Vec<_>>()
                .join(", ")
        ));
    }
    Ok(())
}

fn canonical_gate_signature(definition: &GateDefinition) -> String {
    [
        definition.owner.as_str(),
        definition.threshold.as_str(),
        definition.evidence.as_str(),
        definition.fallback.as_str(),
    ]
    .map(|value| value.split_whitespace().collect::<Vec<_>>().join(" "))
    .join("\0")
}

fn validate_vertical_slice_children(
    vertical: &str,
    catalog: &GateCatalog,
) -> Result<BTreeMap<String, BTreeSet<String>>, String> {
    let mut descriptors: Option<BTreeMap<String, BTreeSet<String>>> = None;
    for table in markdown_tables(vertical) {
        if table.header.first().map(String::as_str) != Some("Gate") {
            continue;
        }
        let Some(child_index) = exact_header_index(&table.header, "Blocking child gates") else {
            continue;
        };
        if descriptors.is_some() {
            return Err("DOCS_VERTICAL_DESCRIPTOR_TABLE_DUPLICATE".to_owned());
        }
        let mut rows = BTreeMap::new();
        for row in table.rows {
            if row.len() != table.header.len() {
                return Err("DOCS_VERTICAL_DESCRIPTOR_ROW_INVALID".to_owned());
            }
            let gate_id = parse_gate_definition_id(&row[0], "Gate", "SPEC-12 blocking children")?;
            if !is_vertical_id(&gate_id) {
                return Err(format!("DOCS_VERTICAL_ID_INVALID: {gate_id}"));
            }
            let children = parse_exact_gate_id_list(
                row.get(child_index).map_or("", String::as_str),
                &format!("{gate_id} Blocking child gates"),
            )?;
            if rows.insert(gate_id.clone(), children).is_some() {
                return Err(format!("DOCS_VERTICAL_ID_DUPLICATE: {gate_id}"));
            }
        }
        descriptors = Some(rows);
    }
    let descriptors =
        descriptors.ok_or_else(|| "DOCS_VERTICAL_BLOCKING_CHILD_COLUMN_MISSING".to_owned())?;
    let expected = (1..=15)
        .map(|number| format!("VS-{number:02}"))
        .collect::<BTreeSet<_>>();
    require_exact_id_set(
        "DOCS_VERTICAL_ID_SET_MISMATCH",
        descriptors.keys(),
        &expected,
    )?;
    for (vertical_id, children) in &descriptors {
        for child in children {
            let Some(definitions) = catalog.definitions.get(child) else {
                return Err(format!(
                    "DOCS_VERTICAL_CHILD_UNKNOWN: {vertical_id} -> {child}"
                ));
            };
            if !definitions
                .iter()
                .any(|definition| definition.document_status == "Accepted")
            {
                return Err(format!(
                    "DOCS_VERTICAL_CHILD_STATUS_INCOMPATIBLE: {vertical_id} -> {child}"
                ));
            }
        }
    }
    Ok(descriptors)
}

fn collect_gate_catalog(documents: &BTreeMap<String, Document>) -> Result<GateCatalog, String> {
    const CANDIDATE_ONLY_HEADER: [&str; 4] = ["Gate ID", "Classification", "Subject", "Rationale"];
    let mut catalog = GateCatalog::default();

    for document in documents.values() {
        for (table_index, table) in markdown_tables(&document.body).into_iter().enumerate() {
            if document.id == "ADR-024"
                && table.header
                    == [
                        "Catalog reference",
                        "Canonical descriptor source",
                        "Classification / closure role",
                    ]
            {
                for row in &table.rows {
                    if row.len() != table.header.len() {
                        return Err("DOCS_GATE_CANONICAL_OWNER_ROW_INVALID: ADR-024".to_owned());
                    }
                    let gate_id = parse_exact_gate_id(&row[0], "ADR-024 canonical gate ownership")?;
                    let owner_document = strip_code_ticks(&row[1]);
                    if !is_architecture_document_id(&owner_document) {
                        return Err(format!(
                            "DOCS_GATE_CANONICAL_OWNER_INVALID: {gate_id}: {owner_document}"
                        ));
                    }
                    if catalog
                        .canonical_owners
                        .insert(gate_id.clone(), owner_document)
                        .is_some()
                    {
                        return Err(format!("DOCS_GATE_CANONICAL_OWNER_DUPLICATE: {gate_id}"));
                    }
                }
                continue;
            }
            if document.id == "ADR-024" && table.header == CANDIDATE_ONLY_HEADER {
                for row in &table.rows {
                    if row.len() != CANDIDATE_ONLY_HEADER.len() {
                        return Err("DOCS_CANDIDATE_ONLY_ROW_INVALID: ADR-024".to_owned());
                    }
                    let gate_id = parse_exact_gate_id(&row[0], "ADR-024 CandidateOnly")?;
                    if row[1] != "CandidateOnly" {
                        return Err(format!(
                            "DOCS_CANDIDATE_ONLY_CLASS_INVALID: {gate_id}: {}",
                            row[1]
                        ));
                    }
                    if is_blank_contract_value(&row[2]) || is_blank_contract_value(&row[3]) {
                        return Err(format!("DOCS_CANDIDATE_ONLY_SUBJECT_MISSING: {gate_id}"));
                    }
                    let subject_id = row[2]
                        .split_whitespace()
                        .next()
                        .ok_or_else(|| format!("DOCS_CANDIDATE_ONLY_SUBJECT_MISSING: {gate_id}"))?;
                    validate_sequential_id_syntax(subject_id, "TECH-", 3).map_err(|_| {
                        format!("DOCS_CANDIDATE_ONLY_SUBJECT_INVALID: {gate_id}: {}", row[2])
                    })?;
                    if !catalog.candidate_only.insert(gate_id.clone()) {
                        return Err(format!("DOCS_CANDIDATE_ONLY_DUPLICATE: {gate_id}"));
                    }
                    catalog
                        .candidate_subjects
                        .insert(gate_id, subject_id.to_owned());
                }
                continue;
            }

            if !matches!(
                table.header.first().map(String::as_str),
                Some("Gate") | Some("Gate ID")
            ) {
                continue;
            }
            if exact_header_index(&table.header, "Blocking child gates").is_some() {
                continue;
            }
            if document.id == "ADR-024" {
                let classification_index = exact_header_index(&table.header, "Classification");
                let owning_document_index = exact_header_index(&table.header, "Owning document");
                if let (Some(classification_index), Some(owning_document_index)) =
                    (classification_index, owning_document_index)
                {
                    for row in &table.rows {
                        if row.len() != table.header.len() {
                            return Err("DOCS_GATE_CANONICAL_OWNER_ROW_INVALID: ADR-024".to_owned());
                        }
                        let gate_id =
                            parse_exact_gate_id(&row[0], "ADR-024 canonical gate ownership")?;
                        let classification = row[classification_index].as_str();
                        if !matches!(
                            classification,
                            "AcceptedBaseline"
                                | "CandidateOnly"
                                | "Rejected"
                                | "ImplementationChoice"
                        ) {
                            return Err(format!(
                                "DOCS_GATE_CLASSIFICATION_INVALID: {gate_id}: {classification}"
                            ));
                        }
                        let owner_document = strip_code_ticks(&row[owning_document_index]);
                        if !is_architecture_document_id(&owner_document) {
                            return Err(format!(
                                "DOCS_GATE_CANONICAL_OWNER_INVALID: {gate_id}: {owner_document}"
                            ));
                        }
                        if catalog
                            .canonical_owners
                            .insert(gate_id.clone(), owner_document.clone())
                            .is_some()
                        {
                            return Err(format!("DOCS_GATE_CANONICAL_OWNER_DUPLICATE: {gate_id}"));
                        }
                        if classification == "CandidateOnly" {
                            catalog.candidate_only.insert(gate_id);
                        }
                    }
                    continue;
                }
            }
            let owner_index = header_index(&table.header, |header| {
                matches!(header, "owner" | "владелец")
            });
            let threshold_index = header_index(&table.header, |header| {
                header.contains("threshold")
                    || header.contains("pass/fail")
                    || header.contains("blocking contract")
                    || header.contains("порог")
            });
            let evidence_index = header_index(&table.header, |header| {
                header.contains("evidence")
                    || header.contains("negative corpus")
                    || header.contains("доказатель")
            });
            let fallback_index = header_index(&table.header, |header| {
                header.contains("fallback") || header.contains("rollback")
            });
            if threshold_index.is_none() || evidence_index.is_none() || fallback_index.is_none() {
                continue;
            }
            let alias_index = header_index(&table.header, |header| {
                matches!(header, "alias of" | "descriptor alias")
            });

            for (row_index, row) in table.rows.iter().enumerate() {
                if row.len() != table.header.len() || row.is_empty() {
                    return Err(format!(
                        "DOCS_GATE_DESCRIPTOR_ROW_INVALID: {} table {} row {}",
                        document.relative,
                        table_index + 1,
                        row_index + 1
                    ));
                }
                let gate_id = parse_gate_definition_id(
                    &row[0],
                    table.header[0].as_str(),
                    &format!("{} table {}", document.relative, table_index + 1),
                )?;
                let value = |index: Option<usize>| {
                    index
                        .and_then(|index| row.get(index))
                        .cloned()
                        .unwrap_or_default()
                };
                let owner = owner_index
                    .and_then(|index| row.get(index))
                    .filter(|owner| !is_blank_contract_value(owner))
                    .cloned()
                    .unwrap_or_else(|| document.owner.clone());
                let definition = GateDefinition {
                    document_id: document.id.clone(),
                    document_status: document.status.clone(),
                    source: format!("table {} row {}", table_index + 1, row_index + 1),
                    owner,
                    threshold: value(threshold_index),
                    evidence: value(evidence_index),
                    fallback: value(fallback_index),
                    alias_of: value(alias_index),
                };
                catalog
                    .definitions
                    .entry(gate_id)
                    .or_default()
                    .push(definition);
            }
        }
    }
    Ok(catalog)
}

fn validate_human_review_decision_v2_contract(
    documents: &BTreeMap<String, Document>,
) -> Result<(), String> {
    let adr = documents
        .values()
        .find(|document| document.id == "ADR-023" && document.status == "Accepted")
        .ok_or_else(|| "DOCS_REVIEW_V2_ACCEPTED_ADR_MISSING: ADR-023".to_owned())?;
    validate_human_review_decision_v2_text(&adr.body)?;

    for document in documents
        .values()
        .filter(|document| document.status == "Accepted" && document.id != "ADR-023")
    {
        for paragraph in markdown_paragraphs(&document.body)
            .into_iter()
            .filter(|paragraph| {
                paragraph.contains("HumanReviewDecisionV1")
                    || paragraph.contains("AttestationEnvelopeV1")
            })
        {
            if !is_explicit_historical_only_v1_reference(&paragraph) {
                return Err(format!(
                    "DOCS_REVIEW_CONTRACT_VERSION_DRIFT: {}: {}",
                    document.relative,
                    paragraph.trim()
                ));
            }
        }
    }
    Ok(())
}

fn is_explicit_historical_only_v1_reference(paragraph: &str) -> bool {
    let normalized = paragraph.to_ascii_lowercase();
    let Some(v1_start) = ["humanreviewdecisionv1", "attestationenvelopev1"]
        .iter()
        .filter_map(|contract| normalized.find(contract))
        .min()
    else {
        return false;
    };
    // A paragraph may first describe the current V2 admission contract and then
    // contrast V1 with it. Only claims at or after the first V1 reference decide
    // whether the legacy contract is historical-only.
    let v1_scope = &normalized[v1_start..];
    let names_historical_mode =
        v1_scope.contains("historical-audit") || v1_scope.contains("historical audit");
    let is_read_only = v1_scope.contains("read-only");
    if !names_historical_mode || !is_read_only {
        return false;
    }

    let authority_words = [
        "admission",
        "admit",
        "`pass`",
        "promotion",
        "merge",
        "mutation",
        "baseline acceptance",
    ];
    let positive_authority_claims = [
        "may admit",
        "can admit",
        "may authorize",
        "can authorize",
        "authorizes admission",
        "admission-eligible",
        "admission token",
        "accepted for admission",
        "used for admission",
        "допускает admission",
        "разрешает admission",
        "может допустить",
    ];
    if positive_authority_claims
        .iter()
        .any(|claim| v1_scope.contains(claim))
    {
        return false;
    }
    if !authority_words
        .iter()
        .any(|authority| v1_scope.contains(authority))
    {
        return true;
    }

    [
        "never",
        "cannot",
        "must not",
        "fails closed",
        "rejected",
        "non-admitting",
        "никогда",
        "не созда",
        "не может",
        "не допуска",
        "отклон",
    ]
    .iter()
    .any(|denial| v1_scope.contains(denial))
}

fn markdown_tables(body: &str) -> Vec<MarkdownTable> {
    let lines = body.lines().collect::<Vec<_>>();
    let mut tables = Vec::new();
    let mut index = 0_usize;
    while index + 1 < lines.len() {
        let header = table_cells(lines[index]);
        let separator = table_cells(lines[index + 1]);
        if header.is_empty()
            || separator.len() != header.len()
            || !separator.iter().all(|cell| {
                !cell.is_empty() && cell.bytes().all(|byte| matches!(byte, b'-' | b':'))
            })
        {
            index += 1;
            continue;
        }
        let mut rows = Vec::new();
        index += 2;
        while index < lines.len() {
            let row = table_cells(lines[index]);
            if row.is_empty() {
                break;
            }
            rows.push(row);
            index += 1;
        }
        tables.push(MarkdownTable { header, rows });
    }
    tables
}

fn exact_header_index(header: &[String], expected: &str) -> Option<usize> {
    header.iter().position(|cell| cell == expected)
}

fn header_index(header: &[String], predicate: impl Fn(&str) -> bool) -> Option<usize> {
    header
        .iter()
        .position(|cell| predicate(&cell.to_ascii_lowercase()))
}

fn parse_exact_requirement_or_failure_id(value: &str, context: &str) -> Result<String, String> {
    let value = strip_code_ticks(value);
    let prefix = if value.starts_with("REQ-") {
        "REQ-"
    } else if value.starts_with("FAIL-") {
        "FAIL-"
    } else {
        return Err(format!("DOCS_ID_INVALID: {context}: {value}"));
    };
    validate_sequential_id_syntax(&value, prefix, 3)?;
    Ok(value)
}

fn parse_exact_document_id_list(value: &str, context: &str) -> Result<BTreeSet<String>, String> {
    if is_blank_contract_value(value) {
        return Err(format!("DOCS_DOCUMENT_ID_LIST_EMPTY: {context}"));
    }
    if value.contains('/')
        || value.contains('…')
        || value.contains("...")
        || value.contains("..")
        || value.contains('–')
    {
        return Err(format!("DOCS_DOCUMENT_ID_SHORTHAND: {context}: {value}"));
    }
    let normalized = value
        .replace("<br />", ",")
        .replace("<br/>", ",")
        .replace("<br>", ",");
    let mut ids = BTreeSet::new();
    for item in normalized.split(',') {
        let id = strip_code_ticks(item);
        if !is_architecture_document_id(&id) {
            return Err(format!("DOCS_DOCUMENT_ID_INVALID: {context}: {id}"));
        }
        if !ids.insert(id.clone()) {
            return Err(format!("DOCS_DOCUMENT_ID_DUPLICATE: {context}: {id}"));
        }
    }
    Ok(ids)
}

fn is_architecture_document_id(value: &str) -> bool {
    (value.strip_prefix("SPEC-").is_some_and(|suffix| {
        suffix.len() == 2 && suffix.bytes().all(|byte| byte.is_ascii_digit())
    })) || (value.strip_prefix("ADR-").is_some_and(|suffix| {
        suffix.len() == 3 && suffix.bytes().all(|byte| byte.is_ascii_digit())
    })) || matches!(value, "EVIDENCE-001" | "TRACE-001")
}

fn validate_sequential_id_syntax(value: &str, prefix: &str, width: usize) -> Result<(), String> {
    let suffix = value
        .strip_prefix(prefix)
        .ok_or_else(|| format!("DOCS_ID_INVALID: expected {prefix}*, got {value}"))?;
    if suffix.len() != width || !suffix.bytes().all(|byte| byte.is_ascii_digit()) {
        return Err(format!("DOCS_ID_INVALID: {value}"));
    }
    Ok(())
}

fn parse_exact_gate_id_list(value: &str, context: &str) -> Result<BTreeSet<String>, String> {
    if is_blank_contract_value(value) {
        return Err(format!("DOCS_GATE_LIST_EMPTY: {context}"));
    }
    if value.contains('/')
        || value.contains('…')
        || value.contains("...")
        || value.contains("..")
        || value.contains('–')
    {
        return Err(format!("DOCS_GATE_ID_SHORTHAND: {context}: {value}"));
    }
    let normalized = value
        .replace("<br />", ",")
        .replace("<br/>", ",")
        .replace("<br>", ",");
    let mut ids = BTreeSet::new();
    for item in normalized.split(',') {
        let id = parse_exact_gate_id(item, context)?;
        if !ids.insert(id.clone()) {
            return Err(format!("DOCS_GATE_ID_DUPLICATE: {context}: {id}"));
        }
    }
    Ok(ids)
}

fn parse_exact_closure_id_list(value: &str, context: &str) -> Result<BTreeSet<String>, String> {
    if is_blank_contract_value(value) {
        return Err(format!("DOCS_CLOSURE_ID_LIST_EMPTY: {context}"));
    }
    if value.contains('/')
        || value.contains('…')
        || value.contains("...")
        || value.contains("..")
        || value.contains('–')
    {
        return Err(format!("DOCS_CLOSURE_ID_SHORTHAND: {context}: {value}"));
    }
    let normalized = value
        .replace("<br />", ",")
        .replace("<br/>", ",")
        .replace("<br>", ",");
    let mut ids = BTreeSet::new();
    for item in normalized.split(',') {
        let id = strip_code_ticks(item);
        if !is_vertical_id(&id) && !is_named_profile_id(&id) {
            return Err(format!("DOCS_CLOSURE_ID_INVALID: {context}: {id}"));
        }
        if !ids.insert(id.clone()) {
            return Err(format!("DOCS_CLOSURE_ID_DUPLICATE: {context}: {id}"));
        }
    }
    Ok(ids)
}

fn is_vertical_id(value: &str) -> bool {
    value
        .strip_prefix("VS-")
        .is_some_and(|suffix| suffix.len() == 2 && suffix.bytes().all(|byte| byte.is_ascii_digit()))
}

fn is_named_profile_id(value: &str) -> bool {
    let mut bytes = value.bytes();
    bytes
        .next()
        .is_some_and(|byte| byte.is_ascii_alphanumeric())
        && bytes
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.' | b':'))
}

fn parse_exact_gate_id(value: &str, context: &str) -> Result<String, String> {
    let value = strip_code_ticks(value);
    if !is_gate_id(&value) {
        return Err(format!("DOCS_GATE_ID_INVALID: {context}: {value}"));
    }
    Ok(value)
}

fn parse_gate_definition_id(value: &str, header: &str, context: &str) -> Result<String, String> {
    let value = value.trim();
    if header == "Gate ID" {
        return parse_exact_gate_id(value, context);
    }
    let first = value
        .split_whitespace()
        .next()
        .ok_or_else(|| format!("DOCS_GATE_ID_INVALID: {context}: empty"))?;
    if first.contains('/') || first.contains('…') || first.contains("...") {
        return Err(format!("DOCS_GATE_ID_SHORTHAND: {context}: {first}"));
    }
    parse_exact_gate_id(first, context)
}

fn strip_code_ticks(value: &str) -> String {
    value.trim().trim_matches('`').trim().to_owned()
}

fn is_blank_contract_value(value: &str) -> bool {
    let value = value.trim();
    value.is_empty() || matches!(value, "—" | "-" | "absent" | "none")
}

pub fn validate_traceability_text(
    traceability: &str,
    defined_gates: &BTreeSet<String>,
) -> Result<(), String> {
    validate_traceability_text_at(traceability, defined_gates, false)
}

fn validate_traceability_text_at(
    traceability: &str,
    defined_gates: &BTreeSet<String>,
    allow_packet_14_legacy: bool,
) -> Result<(), String> {
    let has_contributors = traceability.contains("| Primary owner |")
        && traceability.contains("Contributors / required approvers");
    let legacy_composite_owners = BTreeMap::from([
        ("REQ-032", "RPG + Security"),
        ("REQ-043", "Gameplay Extensibility + Runtime"),
        ("REQ-045", "Persistence + Gameplay Extensibility"),
        ("REQ-048", "Developer Experience + Security"),
        ("REQ-049", "Gameplay Extensibility + Physical Embodiment"),
        ("REQ-051", "Agent + Gameplay Extensibility"),
        ("FAIL-003", "RPG + Security"),
        ("FAIL-007", "Importer + Security"),
        ("FAIL-009", "Persistence + Gameplay Extensibility"),
        ("FAIL-010", "Developer Experience + Security"),
    ]);
    for line in traceability.lines() {
        let cells = table_cells(line);
        if cells.len() < 5 || !(cells[0].starts_with("REQ-") || cells[0].starts_with("FAIL-")) {
            continue;
        }
        let row_id = cells[0].split_whitespace().next().unwrap_or(&cells[0]);
        let owner_index = if cells[0].starts_with("REQ-") { 2 } else { 1 };
        let owner = &cells[owner_index];
        let composite = owner.is_empty()
            || owner.contains(" + ")
            || owner.contains(" / ")
            || owner.contains(';')
            || owner.contains(',');
        let grandfathered =
            allow_packet_14_legacy && legacy_composite_owners.get(row_id) == Some(&owner.as_str());
        if composite && !grandfathered {
            return Err(format!("DOCS_COMPOSITE_PRIMARY_OWNER: {row_id}"));
        }
        if has_contributors {
            let required_len = if cells[0].starts_with("REQ-") { 7 } else { 6 };
            if cells.len() < required_len {
                return Err(format!("DOCS_CONTRIBUTORS_COLUMN_MISSING: {row_id}"));
            }
        }
        let gate_index = match (cells[0].starts_with("REQ-"), has_contributors) {
            (true, true) => 5,
            (true, false) => 4,
            (false, true) => 4,
            (false, false) => 3,
        };
        for token in identifier_tokens(&cells[gate_index]) {
            if is_gate_id(&token) && !defined_gates.contains(&token) {
                return Err(format!("DOCS_UNKNOWN_GATE: {row_id} -> {token}"));
            }
        }
    }
    Ok(())
}

pub fn validate_gameplay_budget_matrix(body: &str) -> Result<(), String> {
    let expected = BTreeMap::from([
        ("core-command-rpg", (1_500_usize, 2_000_usize)),
        ("mechanics", (2_000, 4_000)),
        ("perception-world-services", (1_000, 1_500)),
        ("agent-planning", (1_250, 1_500)),
        ("navigation", (1_250, 1_500)),
        ("state-hash-save-delta-snapshot", (500, 500)),
        ("streaming-staging-commit", (250, 500)),
        ("reserved-headroom", (250, 500)),
    ]);
    let mut rows = BTreeMap::new();
    let mut declared_total = None;
    for line in body.lines() {
        let cells = table_cells(line);
        if cells.len() != 3 || (!expected.contains_key(cells[0].as_str()) && cells[0] != "TOTAL") {
            continue;
        }
        let p95 = cells[1]
            .parse::<usize>()
            .map_err(|error| format!("DOCS_BUDGET_INTEGER_REQUIRED: {}: {error}", cells[0]))?;
        let p99 = cells[2]
            .parse::<usize>()
            .map_err(|error| format!("DOCS_BUDGET_INTEGER_REQUIRED: {}: {error}", cells[0]))?;
        if cells[0] == "TOTAL" {
            if declared_total.replace((p95, p99)).is_some() {
                return Err("DOCS_BUDGET_DUPLICATE_TOTAL".to_owned());
            }
        } else if rows.insert(cells[0].clone(), (p95, p99)).is_some() {
            return Err(format!("DOCS_BUDGET_DUPLICATE_ROW: {}", cells[0]));
        }
    }
    if rows.len() != expected.len() {
        return Err(format!(
            "DOCS_BUDGET_ROWS_MISSING: expected {}, got {}",
            expected.len(),
            rows.len()
        ));
    }
    for (owner, values) in &expected {
        if rows.get(*owner) != Some(values) {
            return Err(format!(
                "DOCS_BUDGET_DEFAULT_MISMATCH: {owner}: expected {}/{}, got {:?}",
                values.0,
                values.1,
                rows.get(*owner)
            ));
        }
    }
    let sums = rows.values().fold((0_usize, 0_usize), |acc, value| {
        (acc.0 + value.0, acc.1 + value.1)
    });
    if sums != (8_000, 12_000) || declared_total != Some(sums) {
        return Err(format!(
            "DOCS_BUDGET_SUM_MISMATCH: rows={}/{}, total={declared_total:?}",
            sums.0, sums.1
        ));
    }
    Ok(())
}

fn identifier_tokens(value: &str) -> Vec<String> {
    value
        .split(|character: char| {
            !(character.is_ascii_uppercase()
                || character.is_ascii_digit()
                || character == '-'
                || character == '_')
        })
        .filter(|token| !token.is_empty())
        .map(str::to_owned)
        .collect()
}

fn is_gate_id(token: &str) -> bool {
    if ["TRACE-", "EVIDENCE-"].iter().any(|prefix| {
        token.strip_prefix(prefix).is_some_and(|suffix| {
            suffix.len() == 2 && suffix.bytes().all(|byte| byte.is_ascii_digit())
        })
    }) {
        return true;
    }
    let excluded = [
        "ADR-",
        "SPEC-",
        "REQ-",
        "FAIL-",
        "INDEX-",
        "TRACE-",
        "GLOSSARY-",
        "EVIDENCE-",
        "RESEARCH-",
        "TECH-",
    ];
    if excluded.iter().any(|prefix| token.starts_with(prefix))
        || !token.chars().any(|character| character.is_ascii_digit())
    {
        return false;
    }
    let mut segments = token.split('-');
    let Some(first) = segments.next() else {
        return false;
    };
    if first.is_empty()
        || !first
            .bytes()
            .all(|byte| byte.is_ascii_uppercase() || byte.is_ascii_digit())
        || !first
            .bytes()
            .next()
            .is_some_and(|byte| byte.is_ascii_uppercase())
    {
        return false;
    }
    let remainder = segments.collect::<Vec<_>>();
    !remainder.is_empty()
        && remainder.iter().all(|segment| {
            !segment.is_empty()
                && segment
                    .bytes()
                    .all(|byte| byte.is_ascii_uppercase() || byte.is_ascii_digit())
        })
}

fn packet_summary(readme: &str) -> Result<BTreeMap<String, usize>, String> {
    let expected = [
        "Markdown documents",
        "Subsystem SPEC files",
        "Decision ADR files",
        "Vertical gates",
        "Requirements",
        "Failure paths",
        "Technology rows",
        "Proposed technology rows",
    ];
    let mut values = BTreeMap::new();
    for line in readme.lines() {
        let cells = table_cells(line);
        if cells.len() == 2 && expected.contains(&cells[0].as_str()) {
            let value = cells[1]
                .parse::<usize>()
                .map_err(|error| format!("DOCS_SUMMARY_INVALID: {}: {error}", cells[0]))?;
            values.insert(cells[0].clone(), value);
        }
    }
    for metric in expected {
        if !values.contains_key(metric) {
            return Err(format!("DOCS_SUMMARY_MISSING: {metric}"));
        }
    }
    Ok(values)
}

fn summary_value(summary: &BTreeMap<String, usize>, metric: &str) -> Result<usize, String> {
    summary
        .get(metric)
        .copied()
        .ok_or_else(|| format!("DOCS_SUMMARY_MISSING: {metric}"))
}

fn reject_markers(file: &Path, body: &str) -> Result<(), String> {
    for marker in ["TODO", "TBD", "FIXME"] {
        if body
            .split(|character: char| !character.is_ascii_alphanumeric())
            .any(|word| word == marker)
        {
            return Err(format!(
                "DOCS_UNRESOLVED_MARKER: {marker} in {}",
                file.display()
            ));
        }
    }
    Ok(())
}

fn validate_relative_links(file: &Path, body: &str) -> Result<(), String> {
    for target in markdown_targets(body) {
        let path_part = target.split('#').next().unwrap_or_default();
        if !path_part.is_empty()
            && !path_part.starts_with("http://")
            && !path_part.starts_with("https://")
            && !path_part.starts_with("mailto:")
        {
            let resolved = file
                .parent()
                .ok_or_else(|| format!("no parent for {}", file.display()))?
                .join(path_part);
            if !resolved.exists() {
                return Err(format!(
                    "DOCS_RELATIVE_LINK_BROKEN: {}: {target}",
                    file.display()
                ));
            }
        }
    }
    Ok(())
}

fn markdown_targets(body: &str) -> Vec<String> {
    let mut targets = Vec::new();
    let mut remainder = body;
    while let Some(start) = remainder.find("](") {
        let after = &remainder[start + 2..];
        let Some(end) = after.find(')') else {
            break;
        };
        targets.push(after[..end].trim().trim_matches(['<', '>']).to_owned());
        remainder = &after[end + 1..];
    }
    targets
}

fn table_cells(line: &str) -> Vec<String> {
    if !line.trim_start().starts_with('|') {
        return Vec::new();
    }
    line.trim()
        .trim_matches('|')
        .split('|')
        .map(str::trim)
        .map(str::to_owned)
        .collect()
}

fn table_ids(body: &str, prefix: &str) -> BTreeSet<String> {
    body.lines()
        .flat_map(table_cells)
        .filter(|cell| cell.starts_with(prefix))
        .filter_map(|cell| cell.split_whitespace().next().map(str::to_owned))
        .collect()
}

fn require_sequential(ids: &BTreeSet<String>, prefix: &str, width: usize) -> Result<(), String> {
    for (index, id) in ids.iter().enumerate() {
        let expected = format!("{prefix}{:0width$}", index + 1);
        if id != &expected {
            return Err(format!(
                "DOCS_NON_SEQUENTIAL_ID: expected {expected}, got {id}"
            ));
        }
    }
    Ok(())
}

fn require_count(label: &str, actual: usize, expected: usize) -> Result<(), String> {
    if actual == expected {
        Ok(())
    } else {
        Err(format!(
            "DOCS_COUNT_MISMATCH: {label}: expected {expected}, got {actual}"
        ))
    }
}

fn is_spec_file(path: &Path) -> bool {
    path.file_name()
        .and_then(OsStr::to_str)
        .is_some_and(|name| {
            name.len() > 3
                && name.as_bytes()[0].is_ascii_digit()
                && name.as_bytes()[1].is_ascii_digit()
                && name.as_bytes()[2] == b'-'
        })
}

fn relative_path(root: &Path, path: &Path) -> Result<String, String> {
    path.strip_prefix(root)
        .map_err(|error| error.to_string())
        .map(|relative| relative.to_string_lossy().replace('\\', "/"))
}

fn normalize_relative(path: &str) -> String {
    normalize_path(Path::new(path))
        .to_string_lossy()
        .replace('\\', "/")
        .trim_start_matches("./")
        .to_owned()
}

fn normalize_path(path: &Path) -> PathBuf {
    let mut normalized = PathBuf::new();
    for component in path.components() {
        match component {
            Component::CurDir => {}
            Component::ParentDir => {
                normalized.pop();
            }
            other => normalized.push(other.as_os_str()),
        }
    }
    normalized
}

fn read(path: &Path) -> Result<String, String> {
    fs::read_to_string(path).map_err(|error| format!("{}: {error}", path.display()))
}

fn collect_files(
    root: &Path,
    extension: Option<&str>,
    output: &mut Vec<PathBuf>,
) -> Result<(), String> {
    for entry in fs::read_dir(root).map_err(|error| format!("{}: {error}", root.display()))? {
        let entry = entry.map_err(|error| error.to_string())?;
        let path = entry.path();
        if path.is_dir() {
            collect_files(&path, extension, output)?;
        } else if extension.is_none_or(|expected| path.extension() == Some(OsStr::new(expected))) {
            output.push(path);
        }
    }
    Ok(())
}

pub fn sha256_hex(input: &[u8]) -> String {
    const INITIAL: [u32; 8] = [
        0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a, 0x510e527f, 0x9b05688c, 0x1f83d9ab,
        0x5be0cd19,
    ];
    const K: [u32; 64] = [
        0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1, 0x923f82a4,
        0xab1c5ed5, 0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3, 0x72be5d74, 0x80deb1fe,
        0x9bdc06a7, 0xc19bf174, 0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc, 0x2de92c6f,
        0x4a7484aa, 0x5cb0a9dc, 0x76f988da, 0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7,
        0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967, 0x27b70a85, 0x2e1b2138, 0x4d2c6dfc,
        0x53380d13, 0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85, 0xa2bfe8a1, 0xa81a664b,
        0xc24b8b70, 0xc76c51a3, 0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070, 0x19a4c116,
        0x1e376c08, 0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3,
        0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208, 0x90befffa, 0xa4506ceb, 0xbef9a3f7,
        0xc67178f2,
    ];

    let bit_len = (input.len() as u64).wrapping_mul(8);
    let mut padded = input.to_vec();
    padded.push(0x80);
    while padded.len() % 64 != 56 {
        padded.push(0);
    }
    padded.extend_from_slice(&bit_len.to_be_bytes());

    let mut state = INITIAL;
    for block in padded.chunks_exact(64) {
        let mut words = [0_u32; 64];
        for (index, chunk) in block.chunks_exact(4).enumerate() {
            words[index] = u32::from_be_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]);
        }
        for index in 16..64 {
            let s0 = words[index - 15].rotate_right(7)
                ^ words[index - 15].rotate_right(18)
                ^ (words[index - 15] >> 3);
            let s1 = words[index - 2].rotate_right(17)
                ^ words[index - 2].rotate_right(19)
                ^ (words[index - 2] >> 10);
            words[index] = words[index - 16]
                .wrapping_add(s0)
                .wrapping_add(words[index - 7])
                .wrapping_add(s1);
        }

        let [mut a, mut b, mut c, mut d, mut e, mut f, mut g, mut h] = state;
        for index in 0..64 {
            let sigma1 = e.rotate_right(6) ^ e.rotate_right(11) ^ e.rotate_right(25);
            let choice = (e & f) ^ ((!e) & g);
            let temp1 = h
                .wrapping_add(sigma1)
                .wrapping_add(choice)
                .wrapping_add(K[index])
                .wrapping_add(words[index]);
            let sigma0 = a.rotate_right(2) ^ a.rotate_right(13) ^ a.rotate_right(22);
            let majority = (a & b) ^ (a & c) ^ (b & c);
            let temp2 = sigma0.wrapping_add(majority);
            h = g;
            g = f;
            f = e;
            e = d.wrapping_add(temp1);
            d = c;
            c = b;
            b = a;
            a = temp1.wrapping_add(temp2);
        }
        for (slot, value) in state.iter_mut().zip([a, b, c, d, e, f, g, h].into_iter()) {
            *slot = slot.wrapping_add(value);
        }
    }
    state.iter().map(|word| format!("{word:08x}")).collect()
}

#[cfg(test)]
mod tests {
    use super::{
        ARCHITECTURE_REVIEW_ROOT, ARCHITECTURE_REVIEW_TRANSITIONS, ArchitectureReviewMode,
        ArchitectureReviewTransition, architecture_candidate_root, is_frozen_annex_id, is_index_id,
        sha256_hex, table_field, validate_architecture_review_records_with_mode_and_anchors,
        validate_normative_graph_documents_at, validate_review_candidate_version,
    };
    use std::collections::BTreeMap;
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::sync::atomic::{AtomicU64, Ordering};

    static TEST_ROOT_COUNTER: AtomicU64 = AtomicU64::new(0);

    struct TestRoot(PathBuf);

    impl TestRoot {
        fn path(&self) -> &Path {
            &self.0
        }
    }

    impl Drop for TestRoot {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    fn test_root(label: &str) -> TestRoot {
        let serial = TEST_ROOT_COUNTER.fetch_add(1, Ordering::Relaxed);
        let path = std::env::temp_dir().join(format!(
            "nextengine-docs-check-{label}-{}-{serial}",
            std::process::id()
        ));
        fs::create_dir_all(path.join("docs/architecture"))
            .expect("test architecture directory should be created");
        fs::write(
            path.join("docs/architecture/README.md"),
            b"architecture candidate\n",
        )
        .expect("test candidate should be written");
        TestRoot(path)
    }

    fn test_approved_roots(root: &Path) -> BTreeMap<String, String> {
        ARCHITECTURE_REVIEW_TRANSITIONS
            .iter()
            .filter_map(|transition| {
                let body =
                    fs::read_to_string(root.join(ARCHITECTURE_REVIEW_ROOT).join(transition.file))
                        .ok()?;
                (table_field(&body, "Status").as_deref() == Some("Approved")).then(|| {
                    (
                        transition.to.to_owned(),
                        table_field(&body, "Candidate root SHA-256")
                            .expect("approved test record should contain a candidate root"),
                    )
                })
            })
            .collect()
    }

    fn validate_architecture_review_records_at(
        root: &Path,
        authoritative_version: &str,
    ) -> Result<(), String> {
        let approved_roots = test_approved_roots(root);
        validate_architecture_review_records_with_mode_and_anchors(
            root,
            authoritative_version,
            ArchitectureReviewMode::AuthoritativeAdmission,
            &approved_roots,
        )?;
        Ok(())
    }

    fn validate_architecture_review_records_with_mode(
        root: &Path,
        authoritative_version: &str,
        mode: ArchitectureReviewMode<'_>,
    ) -> Result<Option<String>, String> {
        let approved_roots = test_approved_roots(root);
        validate_architecture_review_records_with_mode_and_anchors(
            root,
            authoritative_version,
            mode,
            &approved_roots,
        )
    }

    fn render_approved_record(root: &Path, transition: ArchitectureReviewTransition) -> String {
        let candidate_path = "docs/architecture/README.md";
        let candidate_hash = sha256_hex(
            &fs::read(root.join(candidate_path)).expect("test candidate should be readable"),
        );
        let manifest = BTreeMap::from([(candidate_path.to_owned(), candidate_hash.clone())]);
        let candidate_root = architecture_candidate_root(&manifest);
        let record = format!(
            "# Test architecture review\n\n\
             | Field | Value |\n\
             |---|---|\n\
             | Record ID | ARCH-REVIEW-{} |\n\
             | From packet | {} |\n\
             | To packet | {} |\n\
             | Status | Approved |\n\
             | Candidate root algorithm | sha256-path-nul-file-sha256-lf-v1 |\n\
             | Candidate scope | docs/architecture/**/*.md |\n\
             | Candidate root SHA-256 | {candidate_root} |\n\n\
             ## Candidate file manifest\n\n\
             | Path | SHA-256 |\n\
             |---|---|\n\
             | {candidate_path} | {candidate_hash} |\n\n\
             ## Automatic checks\n\n\
             | Check | Result | Evidence reference |\n\
             |---|---|---|\n\
             | cargo fmt --all -- --check | PASS | evidence/fmt |\n\
             | cargo clippy --workspace --all-targets -- -D warnings | PASS | evidence/clippy |\n\
             | cargo test --workspace | PASS | evidence/workspace-test |\n\
             | cargo run -p xtask -- boundary-scan | PASS | evidence/boundary-scan |\n\
             | git diff --check | PASS | evidence/diff-check |\n\
             | cargo run -p xtask -- architecture-review-preflight {} | PASS | evidence/preflight |\n\n\
             ## Bootstrap capability decisions\n\n\
             | Capability | Decision | Reviewer | Decision reference |\n\
             |---|---|---|---|\n\
             | architecture.promote | Approved | repository-owner | {candidate_root} |\n",
            transition.to, transition.from, transition.to, transition.to
        );
        record
    }

    fn render_pending_preflight_record(
        root: &Path,
        transition: ArchitectureReviewTransition,
    ) -> String {
        let approved = render_approved_record(root, transition);
        let candidate_root = table_field(&approved, "Candidate root SHA-256")
            .expect("rendered approved record should contain a candidate root");
        approved
            .replace("| Status | Approved |", "| Status | Pending |")
            .replace(
                &format!(
                    "| cargo run -p xtask -- architecture-review-preflight {} | PASS | evidence/preflight |",
                    transition.to
                ),
                &format!(
                    "| cargo run -p xtask -- architecture-review-preflight {} | Pending | absent |",
                    transition.to
                ),
            )
            .replace(
                &format!(
                    "| architecture.promote | Approved | repository-owner | {candidate_root} |"
                ),
                "| architecture.promote | Pending | absent | absent |",
            )
    }

    fn write_review_record(root: &Path, transition: ArchitectureReviewTransition, body: &str) {
        let review_root = root.join(ARCHITECTURE_REVIEW_ROOT);
        fs::create_dir_all(&review_root).expect("test review directory should be created");
        fs::write(review_root.join(transition.file), body)
            .expect("test review record should be written");
    }

    fn write_packet_15_record(root: &Path, body: &str) {
        write_review_record(root, ARCHITECTURE_REVIEW_TRANSITIONS[0], body);
    }

    fn without_row(body: &str, row_prefix: &str) -> String {
        let mut filtered = body
            .lines()
            .filter(|line| !line.starts_with(row_prefix))
            .collect::<Vec<_>>()
            .join("\n");
        filtered.push('\n');
        filtered
    }

    #[test]
    fn all_research_documents_are_index_ids() {
        assert!(is_index_id("RESEARCH-001"));
        assert!(is_index_id("RESEARCH-002"));
        assert!(is_index_id("RESEARCH-123"));
        assert!(!is_index_id("RESEARCH"));
        assert!(!is_index_id("RESEARCH-draft"));
    }

    #[test]
    fn only_research_001_has_the_frozen_annex_exception() {
        assert!(is_frozen_annex_id("RESEARCH-001"));
        assert!(!is_frozen_annex_id("RESEARCH-002"));
    }

    #[test]
    fn w1_accepted_cycle_is_rejected_without_rejecting_legacy_accepted_scc() {
        let document = |id: &str, dependency: &str| {
            format!(
                "| ID | {id} |\n\
                 | Статус | Accepted |\n\
                 | Версия | 1.0 |\n\
                 | Владелец | Architecture |\n\
                 | Нормативные зависимости | {dependency} |\n\
                 | Заменяет | отсутствует |\n"
            )
        };
        let spec_21 = document("SPEC-21", "[ADR-022](adr/022-test.md)");
        let adr_022 = document("ADR-022", "[SPEC-21](../21-test.md)");
        let error = validate_normative_graph_documents_at(
            Path::new("."),
            &[
                ("21-test.md", spec_21.as_str()),
                ("adr/022-test.md", adr_022.as_str()),
            ],
            true,
        )
        .expect_err("new packet 1.6 Accepted documents must not create a cycle");
        assert!(error.contains("DOCS_GRAPH_CYCLE: W1 Accepted"));

        let legacy_a = document("SPEC-01", "[SPEC-02](02-test.md)");
        let legacy_b = document("SPEC-02", "[SPEC-01](01-test.md)");
        validate_normative_graph_documents_at(
            Path::new("."),
            &[
                ("01-test.md", legacy_a.as_str()),
                ("02-test.md", legacy_b.as_str()),
            ],
            true,
        )
        .expect("the explicitly bounded W1 check must not re-adjudicate the admitted legacy SCC");
    }

    #[test]
    fn missing_review_record_is_rejected_for_accepted_packet() {
        let root = test_root("missing-record");
        let error = validate_architecture_review_records_at(root.path(), "1.5")
            .expect_err("packet 1.5 must require its review record");
        assert!(error.contains("DOCS_REVIEW_RECORD_MISSING"));
    }

    #[test]
    fn pending_review_record_is_rejected_for_accepted_packet() {
        let root = test_root("pending-record");
        let transition = ARCHITECTURE_REVIEW_TRANSITIONS[0];
        let pending = render_approved_record(root.path(), transition)
            .replace("| Status | Approved |", "| Status | Pending |");
        write_packet_15_record(root.path(), &pending);
        let error = validate_architecture_review_records_at(root.path(), "1.5")
            .expect_err("Pending record must not authorize packet 1.5");
        assert!(error.contains("DOCS_REVIEW_RECORD_NOT_APPROVED"));
    }

    #[test]
    fn invalid_review_record_status_is_rejected() {
        let root = test_root("invalid-review-status");
        let transition = ARCHITECTURE_REVIEW_TRANSITIONS[0];
        let invalid = render_approved_record(root.path(), transition)
            .replace("| Status | Approved |", "| Status | Rejected |");
        write_packet_15_record(root.path(), &invalid);
        let error = validate_architecture_review_records_at(root.path(), "1.5")
            .expect_err("architecture review records accept only Pending or Approved");
        assert!(error.contains("DOCS_REVIEW_STATUS_INVALID"));
    }

    #[test]
    fn pending_review_record_passes_candidate_preflight() {
        let root = test_root("pending-preflight");
        let transition = ARCHITECTURE_REVIEW_TRANSITIONS[0];
        let pending = render_pending_preflight_record(root.path(), transition);
        let expected_root = table_field(&pending, "Candidate root SHA-256")
            .expect("test review record should contain candidate root");
        write_packet_15_record(root.path(), &pending);

        let actual_root = validate_architecture_review_records_with_mode(
            root.path(),
            "1.5",
            ArchitectureReviewMode::CandidatePreflight { target: "1.5" },
        )
        .expect("pending record with passing component checks should pass preflight");

        assert_eq!(actual_root.as_deref(), Some(expected_root.as_str()));
    }

    #[test]
    fn candidate_preflight_rejects_absent_root() {
        let root = test_root("preflight-absent-root");
        let transition = ARCHITECTURE_REVIEW_TRANSITIONS[0];
        let pending = render_pending_preflight_record(root.path(), transition);
        let candidate_root =
            table_field(&pending, "Candidate root SHA-256").expect("candidate root");
        let pending = pending
            .replace(
                &format!("| Candidate root SHA-256 | {candidate_root} |"),
                "| Candidate root SHA-256 | absent |",
            )
            .lines()
            .map(|line| {
                if line.starts_with("| docs/architecture/") {
                    "| none | absent |"
                } else {
                    line
                }
            })
            .collect::<Vec<_>>()
            .join("\n");
        write_packet_15_record(root.path(), &pending);

        let error = validate_architecture_review_records_with_mode(
            root.path(),
            "1.5",
            ArchitectureReviewMode::CandidatePreflight { target: "1.5" },
        )
        .expect_err("candidate preflight must reject an absent root");
        assert!(error.contains("DOCS_REVIEW_PREFLIGHT_CANDIDATE_ROOT_MISSING"));
    }

    #[test]
    fn candidate_preflight_rejects_empty_manifest() {
        let root = test_root("preflight-empty-manifest");
        let transition = ARCHITECTURE_REVIEW_TRANSITIONS[0];
        let pending = render_pending_preflight_record(root.path(), transition)
            .lines()
            .filter(|line| !line.starts_with("| docs/architecture/"))
            .collect::<Vec<_>>()
            .join("\n");
        write_packet_15_record(root.path(), &pending);

        let error = validate_architecture_review_records_with_mode(
            root.path(),
            "1.5",
            ArchitectureReviewMode::CandidatePreflight { target: "1.5" },
        )
        .expect_err("candidate preflight must reject an empty manifest");
        assert!(error.contains("DOCS_REVIEW_MANIFEST_EMPTY"));
    }

    #[test]
    fn candidate_preflight_rejects_manifest_without_readme() {
        let root = test_root("preflight-readme-missing");
        let transition = ARCHITECTURE_REVIEW_TRANSITIONS[0];
        let alternative_path = "docs/architecture/other.md";
        fs::write(
            root.path().join(alternative_path),
            b"other architecture document\n",
        )
        .expect("alternative architecture document should be written");
        let alternative_hash =
            sha256_hex(&fs::read(root.path().join(alternative_path)).expect("alternative file"));
        let manifest = BTreeMap::from([(alternative_path.to_owned(), alternative_hash.clone())]);
        let alternative_root = architecture_candidate_root(&manifest);

        let pending = render_pending_preflight_record(root.path(), transition);
        let original_root =
            table_field(&pending, "Candidate root SHA-256").expect("candidate root");
        let pending = pending
            .replace(
                &format!("| Candidate root SHA-256 | {original_root} |"),
                &format!("| Candidate root SHA-256 | {alternative_root} |"),
            )
            .lines()
            .map(|line| {
                if line.starts_with("| docs/architecture/") {
                    format!("| {alternative_path} | {alternative_hash} |")
                } else {
                    line.to_owned()
                }
            })
            .collect::<Vec<_>>()
            .join("\n");
        write_packet_15_record(root.path(), &pending);

        let error = validate_architecture_review_records_with_mode(
            root.path(),
            "1.5",
            ArchitectureReviewMode::CandidatePreflight { target: "1.5" },
        )
        .expect_err("candidate preflight must hash the packet README");
        assert!(error.contains("DOCS_REVIEW_PACKET_INDEX_NOT_HASHED"));
    }

    #[test]
    fn candidate_preflight_rejects_pending_component_check() {
        let root = test_root("pending-component");
        let transition = ARCHITECTURE_REVIEW_TRANSITIONS[0];
        let pending = render_pending_preflight_record(root.path(), transition).replace(
            "| cargo fmt --all -- --check | PASS | evidence/fmt |",
            "| cargo fmt --all -- --check | Pending | absent |",
        );
        write_packet_15_record(root.path(), &pending);

        let error = validate_architecture_review_records_with_mode(
            root.path(),
            "1.5",
            ArchitectureReviewMode::CandidatePreflight { target: "1.5" },
        )
        .expect_err("preflight must require every component check to pass");

        assert!(error.contains("DOCS_REVIEW_PREFLIGHT_CHECK_NOT_PASS"));
    }

    #[test]
    fn candidate_preflight_rejects_wrong_target() {
        let root = test_root("wrong-preflight-target");
        let transition = ARCHITECTURE_REVIEW_TRANSITIONS[0];
        let pending = render_pending_preflight_record(root.path(), transition);
        write_packet_15_record(root.path(), &pending);

        let error = validate_architecture_review_records_with_mode(
            root.path(),
            "1.5",
            ArchitectureReviewMode::CandidatePreflight { target: "1.6" },
        )
        .expect_err("preflight target must match the candidate packet version");

        assert!(error.contains("DOCS_REVIEW_PREFLIGHT_TARGET_MISMATCH"));
    }

    #[test]
    fn candidate_tree_preflight_targets_version_while_review_candidate_is_next() {
        let root = test_root("candidate-tree-version");
        for transition in &ARCHITECTURE_REVIEW_TRANSITIONS[..2] {
            let approved = render_approved_record(root.path(), *transition);
            write_review_record(root.path(), *transition, &approved);
        }
        let packet_16 = ARCHITECTURE_REVIEW_TRANSITIONS[2];
        let pending = render_pending_preflight_record(root.path(), packet_16);
        write_review_record(root.path(), packet_16, &pending);

        validate_review_candidate_version("1.6", "1.7")
            .expect("README review candidate must be the immediate successor");
        validate_architecture_review_records_with_mode(
            root.path(),
            "1.6",
            ArchitectureReviewMode::CandidatePreflight { target: "1.6" },
        )
        .expect("preflight command targets the candidate tree Version, not Review candidate");

        let error = validate_architecture_review_records_with_mode(
            root.path(),
            "1.6",
            ArchitectureReviewMode::CandidatePreflight { target: "1.7" },
        )
        .expect_err("preflight target must not use the next Review candidate");
        assert!(error.contains("DOCS_REVIEW_PREFLIGHT_TARGET_MISMATCH"));
    }

    #[test]
    fn changed_candidate_file_invalidates_review_record() {
        let root = test_root("changed-candidate");
        let transition = ARCHITECTURE_REVIEW_TRANSITIONS[0];
        let record = render_approved_record(root.path(), transition);
        write_packet_15_record(root.path(), &record);
        fs::write(
            root.path().join("docs/architecture/README.md"),
            b"changed architecture candidate\n",
        )
        .expect("test candidate should be changed");
        let error = validate_architecture_review_records_at(root.path(), "1.5")
            .expect_err("changed file must invalidate its review record");
        assert!(error.contains("DOCS_REVIEW_FILE_HASH_MISMATCH"));
    }

    #[test]
    fn wrong_candidate_root_is_rejected() {
        let root = test_root("wrong-root");
        let transition = ARCHITECTURE_REVIEW_TRANSITIONS[0];
        let record = render_approved_record(root.path(), transition);
        let root_hash = table_field(&record, "Candidate root SHA-256")
            .expect("test record should contain candidate root");
        let record = record.replace(&root_hash, &"0".repeat(64));
        write_packet_15_record(root.path(), &record);
        let error = validate_architecture_review_records_at(root.path(), "1.5")
            .expect_err("wrong candidate root must fail");
        assert!(error.contains("DOCS_REVIEW_CANDIDATE_ROOT_MISMATCH"));
    }

    #[test]
    fn manifest_must_cover_the_complete_architecture_packet() {
        let root = test_root("incomplete-scope");
        let transition = ARCHITECTURE_REVIEW_TRANSITIONS[0];
        let record = render_approved_record(root.path(), transition);
        fs::write(
            root.path().join("docs/architecture/glossary.md"),
            b"additional candidate file\n",
        )
        .expect("additional candidate file should be written");
        write_packet_15_record(root.path(), &record);
        let error = validate_architecture_review_records_at(root.path(), "1.5")
            .expect_err("manifest must cover every architecture Markdown file");
        assert!(error.contains("DOCS_REVIEW_MANIFEST_SCOPE_MISMATCH"));
        assert!(error.contains("docs/architecture/glossary.md"));
    }

    #[test]
    fn missing_capability_decision_is_rejected() {
        let root = test_root("missing-capability");
        let transition = ARCHITECTURE_REVIEW_TRANSITIONS[0];
        let record = without_row(
            &render_approved_record(root.path(), transition),
            "| architecture.promote |",
        );
        write_packet_15_record(root.path(), &record);
        let error = validate_architecture_review_records_at(root.path(), "1.5")
            .expect_err("architecture.promote must be present");
        assert!(error.contains("DOCS_REVIEW_CAPABILITY_DECISION_MISSING"));
        assert!(error.contains("architecture.promote"));
    }

    #[test]
    fn architecture_promotion_decision_must_reference_exact_candidate_root() {
        let root = test_root("wrong-decision-root");
        let packet_15 = ARCHITECTURE_REVIEW_TRANSITIONS[0];
        let admitted_packet_15 = render_approved_record(root.path(), packet_15);
        write_review_record(root.path(), packet_15, &admitted_packet_15);

        let transition = ARCHITECTURE_REVIEW_TRANSITIONS[1];
        let record = render_approved_record(root.path(), transition);
        let candidate_root = table_field(&record, "Candidate root SHA-256")
            .expect("test record should contain candidate root");
        let record = record.replace(
            &format!("| architecture.promote | Approved | repository-owner | {candidate_root} |"),
            &format!(
                "| architecture.promote | Approved | repository-owner | {} |",
                "0".repeat(64)
            ),
        );
        write_review_record(root.path(), transition, &record);
        let error = validate_architecture_review_records_at(root.path(), "1.5.1")
            .expect_err("promotion decision must bind the exact candidate root");
        assert!(error.contains("DOCS_REVIEW_CAPABILITY_DECISION_ROOT_MISMATCH"));
        assert!(error.contains("architecture.promote"));
    }

    #[test]
    fn skipped_review_transition_is_rejected() {
        let root = test_root("skipped-transition");
        let transition = ARCHITECTURE_REVIEW_TRANSITIONS[0];
        let record = render_approved_record(root.path(), transition)
            .replace("| To packet | 1.5 |", "| To packet | 1.6 |");
        write_packet_15_record(root.path(), &record);
        let error = validate_architecture_review_records_at(root.path(), "1.5")
            .expect_err("1.4 -> 1.6 must not replace the 1.4 -> 1.5 transition");
        assert!(error.contains("DOCS_REVIEW_TRANSITION_MISMATCH"));
    }

    #[test]
    fn valid_hash_bound_review_record_passes() {
        let root = test_root("valid-record");
        let transition = ARCHITECTURE_REVIEW_TRANSITIONS[0];
        let record = render_approved_record(root.path(), transition);
        write_packet_15_record(root.path(), &record);
        validate_architecture_review_records_at(root.path(), "1.5")
            .expect("complete hash-bound review record should pass");
    }

    #[test]
    fn approved_15_and_pending_151_passes_candidate_preflight_after_candidate_change() {
        let root = test_root("editorial-patch-preflight");
        let packet_15 = ARCHITECTURE_REVIEW_TRANSITIONS[0];
        let packet_151 = ARCHITECTURE_REVIEW_TRANSITIONS[1];
        let approved_15 = render_approved_record(root.path(), packet_15);
        write_review_record(root.path(), packet_15, &approved_15);

        fs::write(
            root.path().join("docs/architecture/README.md"),
            b"editorial architecture candidate\n",
        )
        .expect("test candidate should be changed for packet 1.5.1");
        let pending_151 = render_pending_preflight_record(root.path(), packet_151);
        let expected_root = table_field(&pending_151, "Candidate root SHA-256")
            .expect("packet 1.5.1 should contain a candidate root");
        write_review_record(root.path(), packet_151, &pending_151);

        let actual_root = validate_architecture_review_records_with_mode(
            root.path(),
            "1.5.1",
            ArchitectureReviewMode::CandidatePreflight { target: "1.5.1" },
        )
        .expect("historical packet 1.5 must not be compared with the packet 1.5.1 tree");

        assert_eq!(actual_root.as_deref(), Some(expected_root.as_str()));
    }

    #[test]
    fn tampered_historical_candidate_root_is_rejected() {
        let root = test_root("tampered-historical-root");
        let packet_15 = ARCHITECTURE_REVIEW_TRANSITIONS[0];
        let packet_151 = ARCHITECTURE_REVIEW_TRANSITIONS[1];
        let approved_15 = render_approved_record(root.path(), packet_15);
        let historical_root = table_field(&approved_15, "Candidate root SHA-256")
            .expect("packet 1.5 should contain a candidate root");
        let tampered_15 = approved_15.replace(&historical_root, &"0".repeat(64));
        write_review_record(root.path(), packet_15, &tampered_15);

        fs::write(
            root.path().join("docs/architecture/README.md"),
            b"editorial architecture candidate\n",
        )
        .expect("test candidate should be changed for packet 1.5.1");
        let pending_151 = render_pending_preflight_record(root.path(), packet_151);
        write_review_record(root.path(), packet_151, &pending_151);

        let error = validate_architecture_review_records_with_mode(
            root.path(),
            "1.5.1",
            ArchitectureReviewMode::CandidatePreflight { target: "1.5.1" },
        )
        .expect_err("historical record roots must remain bound to their recorded manifests");

        assert!(error.contains("DOCS_REVIEW_CANDIDATE_ROOT_MISMATCH"));
        assert!(error.contains("packet-1.5.md"));
    }

    #[test]
    fn coordinated_historical_manifest_and_root_tampering_is_rejected() {
        let root = test_root("coordinated-historical-tampering");
        let packet_15 = ARCHITECTURE_REVIEW_TRANSITIONS[0];
        let packet_151 = ARCHITECTURE_REVIEW_TRANSITIONS[1];
        let approved_15 = render_approved_record(root.path(), packet_15);
        let approved_root = table_field(&approved_15, "Candidate root SHA-256")
            .expect("packet 1.5 should contain a candidate root");
        let candidate_path = "docs/architecture/README.md";
        let candidate_hash = sha256_hex(
            &fs::read(root.path().join(candidate_path)).expect("test candidate should be readable"),
        );
        let tampered_hash = "f".repeat(64);
        let tampered_manifest =
            BTreeMap::from([(candidate_path.to_owned(), tampered_hash.clone())]);
        let tampered_root = architecture_candidate_root(&tampered_manifest);
        let tampered_15 = approved_15
            .replace(&candidate_hash, &tampered_hash)
            .replace(&approved_root, &tampered_root);
        write_review_record(root.path(), packet_15, &tampered_15);

        fs::write(
            root.path().join("docs/architecture/README.md"),
            b"editorial architecture candidate\n",
        )
        .expect("test candidate should be changed for packet 1.5.1");
        let pending_151 = render_pending_preflight_record(root.path(), packet_151);
        write_review_record(root.path(), packet_151, &pending_151);

        let approved_roots = BTreeMap::from([("1.5".to_owned(), approved_root)]);
        let error = validate_architecture_review_records_with_mode_and_anchors(
            root.path(),
            "1.5.1",
            ArchitectureReviewMode::CandidatePreflight { target: "1.5.1" },
            &approved_roots,
        )
        .expect_err("historical record must remain bound to its independently anchored root");

        assert!(error.contains("DOCS_REVIEW_APPROVED_ROOT_MISMATCH"));
        assert!(error.contains("packet-1.5.md"));
    }

    #[test]
    fn changed_current_151_candidate_is_rejected_by_preflight() {
        let root = test_root("changed-current-151-candidate");
        let packet_15 = ARCHITECTURE_REVIEW_TRANSITIONS[0];
        let packet_151 = ARCHITECTURE_REVIEW_TRANSITIONS[1];
        let approved_15 = render_approved_record(root.path(), packet_15);
        write_review_record(root.path(), packet_15, &approved_15);

        fs::write(
            root.path().join("docs/architecture/README.md"),
            b"editorial architecture candidate\n",
        )
        .expect("test candidate should be changed for packet 1.5.1");
        let pending_151 = render_pending_preflight_record(root.path(), packet_151);
        write_review_record(root.path(), packet_151, &pending_151);
        fs::write(
            root.path().join("docs/architecture/README.md"),
            b"drifted editorial architecture candidate\n",
        )
        .expect("test candidate should drift after its manifest is recorded");

        let error = validate_architecture_review_records_with_mode(
            root.path(),
            "1.5.1",
            ArchitectureReviewMode::CandidatePreflight { target: "1.5.1" },
        )
        .expect_err("preflight must compare the packet 1.5.1 record with the current tree");

        assert!(error.contains("DOCS_REVIEW_FILE_HASH_MISMATCH"));
        assert!(error.contains("packet-1.5.1.md"));
    }

    #[test]
    fn future_16_approval_is_rejected_while_151_is_authoritative() {
        let root = test_root("future-approval");
        let packet_15 = ARCHITECTURE_REVIEW_TRANSITIONS[0];
        let packet_151 = ARCHITECTURE_REVIEW_TRANSITIONS[1];
        let packet_16 = ARCHITECTURE_REVIEW_TRANSITIONS[2];
        let approved_15 = render_approved_record(root.path(), packet_15);
        write_review_record(root.path(), packet_15, &approved_15);

        fs::write(
            root.path().join("docs/architecture/README.md"),
            b"editorial architecture candidate\n",
        )
        .expect("test candidate should be changed for packet 1.5.1");
        let approved_151 = render_approved_record(root.path(), packet_151);
        write_review_record(root.path(), packet_151, &approved_151);
        let approved_16 = render_approved_record(root.path(), packet_16);
        write_review_record(root.path(), packet_16, &approved_16);

        let error = validate_architecture_review_records_at(root.path(), "1.5.1")
            .expect_err("packet 1.6 approval must not skip the authoritative sequence");

        assert!(error.contains("DOCS_REVIEW_TRANSITION_OUT_OF_SEQUENCE"));
        assert!(error.contains("1.5.1 -> 1.6"));
    }

    #[test]
    fn future_pending_records_are_allowed_without_later_speculative_files() {
        let root = test_root("future-pending");
        for transition in &ARCHITECTURE_REVIEW_TRANSITIONS[..2] {
            let approved = render_approved_record(root.path(), *transition);
            write_review_record(root.path(), *transition, &approved);
        }
        for transition in &ARCHITECTURE_REVIEW_TRANSITIONS[2..4] {
            let pending = render_pending_preflight_record(root.path(), *transition);
            write_review_record(root.path(), *transition, &pending);
        }

        validate_architecture_review_records_at(root.path(), "1.5.1")
            .expect("future Pending records are allowed and packets 1.8 through 1.10 are optional");
    }
}
