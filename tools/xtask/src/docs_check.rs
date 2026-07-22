use std::collections::{BTreeMap, BTreeSet};
use std::ffi::OsStr;
use std::fs;
use std::path::{Component, Path, PathBuf};

const ANNEX_PATH: &str = "research/physical-avatar-research-spec.md";
const ANNEX_SHA256: &str = "90533ed15c4c1a5ef41a24f26f4d17cf8c59f467e07619316d3c9744f4d2d79b";
const TEMPLATE_PATH: &str = "adr/000-template.md";
const LEGACY_PHYSICAL_RESEARCH_URL: &str = "https://github.com/kaifaty/OpenGothic/blob/c56e15f1fa68430eaa618dcc892edc00bff6209d/docs/physical-avatar-research-spec.md";

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
    let docs_root = root.join("docs/architecture");
    let mut files = Vec::new();
    collect_files(&docs_root, Some("md"), &mut files)?;
    files.sort();

    let readme = read(&docs_root.join("README.md"))?;
    let summary = packet_summary(&readme)?;
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
    validate_supersession_graph(&docs_root, &documents)?;
    validate_annex(root, &docs_root)?;
    validate_governance(root)?;

    let traceability = read(&docs_root.join("traceability.md"))?;
    let requirement_ids = table_ids(&traceability, "REQ-");
    let failure_ids = table_ids(&traceability, "FAIL-");
    require_count(
        "requirements",
        requirement_ids.len(),
        summary_value(&summary, "Requirements")?,
    )?;
    require_count(
        "failure paths",
        failure_ids.len(),
        summary_value(&summary, "Failure paths")?,
    )?;
    require_sequential(&requirement_ids, "REQ-", 3)?;
    require_sequential(&failure_ids, "FAIL-", 3)?;

    let vertical = read(&docs_root.join("12-vertical-slice-conformance.md"))?;
    let vertical_ids = table_ids(&vertical, "VS-");
    require_count(
        "vertical gates",
        vertical_ids.len(),
        summary_value(&summary, "Vertical gates")?,
    )?;
    require_sequential(&vertical_ids, "VS-", 2)?;

    let defined_gates = collect_defined_gates(&files)?;
    validate_traceability_text_at(&traceability, &defined_gates, true)?;

    let budget_adr = read(&docs_root.join("adr/016-compositional-gameplay-budgets.md"))?;
    validate_gameplay_budget_matrix(&budget_adr)?;

    let evidence = read(&docs_root.join("evidence-register.md"))?;
    let technology_rows: Vec<_> = evidence
        .lines()
        .filter(|line| {
            ["Accepted", "Proposed", "Rejected", "Superseded"]
                .iter()
                .any(|status| line.contains(&format!("| {status} |")))
                && line.split('|').count() == 11
        })
        .collect();
    require_count(
        "technology rows",
        technology_rows.len(),
        summary_value(&summary, "Technology rows")?,
    )?;
    require_count(
        "Proposed technology rows",
        technology_rows
            .iter()
            .filter(|line| line.contains("| Proposed |"))
            .count(),
        summary_value(&summary, "Proposed technology rows")?,
    )?;

    let authoritative_version = required_field(&readme, "Версия", "README.md")?;
    let candidate_version = required_field(&readme, "Review candidate", "README.md")?;
    println!(
        "PASS docs-check: authoritative packet {authoritative_version}, candidate {candidate_version}, {architecture_file_count} indexed architecture documents"
    );
    Ok(())
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
        if cells[0] == "RESEARCH-001" {
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
        .filter(|id| id.as_str() != "RESEARCH-001")
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
        || matches!(
            value,
            "GLOSSARY-001" | "TRACE-001" | "EVIDENCE-001" | "RESEARCH-001"
        )
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
            if document.status == "Accepted" {
                if previous.status != "Superseded" {
                    return Err(format!(
                        "DOCS_SUPERSESSION_STATUS_MISMATCH: {} accepts replacement for {} ({})",
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
            if replacement_document.status != "Accepted" {
                return Err(format!(
                    "DOCS_SUPERSESSION_REPLACEMENT_NOT_ACCEPTED: {} -> {replacement}",
                    document.relative
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
    Ok(())
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

fn collect_defined_gates(files: &[PathBuf]) -> Result<BTreeSet<String>, String> {
    let mut gates = BTreeSet::new();
    for file in files {
        if file.ends_with(ANNEX_PATH) || file.ends_with(TEMPLATE_PATH) {
            continue;
        }
        let body = read(file)?;
        for line in body.lines() {
            let cells = table_cells(line);
            let Some(first) = cells.first() else {
                continue;
            };
            for token in identifier_tokens(first) {
                if is_gate_id(&token) {
                    gates.insert(token);
                }
            }
        }
    }
    Ok(gates)
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
    ];
    token.contains('-')
        && token.chars().any(|character| character.is_ascii_digit())
        && !excluded.iter().any(|prefix| token.starts_with(prefix))
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
