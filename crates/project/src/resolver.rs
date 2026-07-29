use std::collections::{BTreeMap, BTreeSet};
use std::error::Error;
use std::fmt::{Display, Formatter};

use next_contracts::ids::SchemaId;
use next_contracts::project::{
    ProjectCatalogRecordV1, ProjectCatalogSnapshotV1, ProjectDependencyKindV1, ProjectManifestV1,
    ProjectRequirementV1, ResolvedProjectRecordV1, SemanticVersionV1,
};

type DependencyKey = (ProjectDependencyKindV1, SchemaId);

pub fn resolve_project_records_v1(
    manifest: &ProjectManifestV1,
    catalog: &ProjectCatalogSnapshotV1,
) -> Result<Vec<ResolvedProjectRecordV1>, ProjectResolutionError> {
    ProjectManifestV1::from_jcs_bytes(
        &manifest.to_jcs_bytes(),
        next_contracts::canonical::CanonicalDecodeLimits::default(),
    )?;
    ProjectCatalogSnapshotV1::from_jcs_bytes(
        &catalog.to_jcs_bytes(),
        next_contracts::canonical::CanonicalDecodeLimits::default(),
    )?;

    let mut constraints = BTreeMap::<DependencyKey, SemanticVersionV1>::new();
    let mut optional = BTreeMap::<DependencyKey, bool>::new();
    for requirement in &manifest.requirements {
        merge_requirement(&mut constraints, &mut optional, requirement);
    }
    let mut selected = BTreeMap::<DependencyKey, ProjectCatalogRecordV1>::new();
    let mut skipped = BTreeSet::<DependencyKey>::new();

    loop {
        let unresolved = constraints
            .keys()
            .find(|key| !selected.contains_key(*key) && !skipped.contains(*key))
            .cloned();
        let Some(key) = unresolved else {
            break;
        };
        let minimum = constraints[&key];
        let mut candidates: Vec<_> = catalog
            .records
            .iter()
            .filter(|record| {
                record.kind == key.0
                    && record.identity == key.1
                    && record.version >= minimum
                    && !record.yanked
            })
            .cloned()
            .collect();
        candidates.sort_by(|left, right| {
            right
                .version
                .cmp(&left.version)
                .then_with(|| left.record_sha256.cmp(&right.record_sha256))
        });
        let Some(candidate) = candidates.into_iter().next() else {
            if optional[&key] {
                skipped.insert(key);
                continue;
            }
            return Err(ProjectResolutionError::NoCandidate {
                kind: key.0,
                identity: key.1,
                minimum,
            });
        };
        for requirement in &candidate.requirements {
            let requirement_key = (requirement.kind, requirement.identity.clone());
            if let Some(existing) = selected.get(&requirement_key)
                && existing.version < requirement.minimum_version
            {
                return Err(ProjectResolutionError::SelectedVersionConflict {
                    identity: requirement.identity.clone(),
                });
            }
            merge_requirement(&mut constraints, &mut optional, requirement);
        }
        selected.insert(key, candidate);
    }

    ensure_no_dependency_cycle(&selected)?;
    Ok(selected
        .into_values()
        .map(|record| ResolvedProjectRecordV1 {
            kind: record.kind,
            identity: record.identity,
            version: record.version,
            record_sha256: record.record_sha256,
            artifact_sha256: record.artifact_sha256,
        })
        .collect())
}

fn merge_requirement(
    constraints: &mut BTreeMap<DependencyKey, SemanticVersionV1>,
    optional: &mut BTreeMap<DependencyKey, bool>,
    requirement: &ProjectRequirementV1,
) {
    let key = (requirement.kind, requirement.identity.clone());
    constraints
        .entry(key.clone())
        .and_modify(|version| *version = (*version).max(requirement.minimum_version))
        .or_insert(requirement.minimum_version);
    optional
        .entry(key)
        .and_modify(|value| *value &= requirement.optional)
        .or_insert(requirement.optional);
}

fn ensure_no_dependency_cycle(
    selected: &BTreeMap<DependencyKey, ProjectCatalogRecordV1>,
) -> Result<(), ProjectResolutionError> {
    let mut indegree: BTreeMap<DependencyKey, usize> =
        selected.keys().cloned().map(|key| (key, 0)).collect();
    let mut successors: BTreeMap<DependencyKey, Vec<DependencyKey>> = BTreeMap::new();
    for (key, record) in selected {
        for requirement in &record.requirements {
            let target = (requirement.kind, requirement.identity.clone());
            if !selected.contains_key(&target) {
                if requirement.optional {
                    continue;
                }
                return Err(ProjectResolutionError::MissingTransitive {
                    identity: requirement.identity.clone(),
                });
            }
            *indegree
                .get_mut(&target)
                .expect("selected dependency has indegree") += 1;
            successors.entry(key.clone()).or_default().push(target);
        }
    }
    let mut ready: BTreeSet<_> = indegree
        .iter()
        .filter_map(|(key, indegree)| (*indegree == 0).then_some(key.clone()))
        .collect();
    let mut visited = 0_usize;
    while let Some(key) = ready.pop_first() {
        visited += 1;
        if let Some(children) = successors.get(&key) {
            for child in children {
                let value = indegree.get_mut(child).expect("selected dependency");
                *value -= 1;
                if *value == 0 {
                    ready.insert(child.clone());
                }
            }
        }
    }
    if visited != selected.len() {
        return Err(ProjectResolutionError::DependencyCycle);
    }
    Ok(())
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum ProjectResolutionError {
    Contract(next_contracts::project::ProjectContractError),
    NoCandidate {
        kind: ProjectDependencyKindV1,
        identity: SchemaId,
        minimum: SemanticVersionV1,
    },
    SelectedVersionConflict {
        identity: SchemaId,
    },
    MissingTransitive {
        identity: SchemaId,
    },
    DependencyCycle,
}

impl Display for ProjectResolutionError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Contract(error) => write!(formatter, "project resolution input invalid: {error}"),
            Self::NoCandidate {
                identity, minimum, ..
            } => write!(
                formatter,
                "no project candidate for {identity} at least {}",
                minimum.canonical_text()
            ),
            Self::SelectedVersionConflict { identity } => {
                write!(formatter, "selected version conflicts for {identity}")
            }
            Self::MissingTransitive { identity } => {
                write!(
                    formatter,
                    "required transitive dependency is missing: {identity}"
                )
            }
            Self::DependencyCycle => formatter.write_str("project dependency cycle"),
        }
    }
}

impl Error for ProjectResolutionError {}

impl From<next_contracts::project::ProjectContractError> for ProjectResolutionError {
    fn from(error: next_contracts::project::ProjectContractError) -> Self {
        Self::Contract(error)
    }
}
