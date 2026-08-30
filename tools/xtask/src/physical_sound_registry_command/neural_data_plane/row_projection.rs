use super::*;

pub(super) fn project_row(
    root: &Path,
    manifest_directory: &Path,
    row: &NeuralRow,
    artifact_cache: &mut BTreeMap<(String, String), VerifiedArtifact>,
) -> Result<ProjectedRow, String> {
    let mut resolve = |reference: &FileRef, role: &str| {
        resolve_cached_artifact(root, manifest_directory, reference, role, artifact_cache)
    };
    let material = row
        .axes
        .material
        .as_ref()
        .map(|claim| -> Result<ProjectedLabelClaim, String> {
            Ok(ProjectedLabelClaim {
                value_id: claim.value_id.clone(),
                evidence: resolve(&claim.evidence, "material claim evidence")?,
            })
        })
        .transpose()?;
    let geometry = row
        .axes
        .geometry
        .as_ref()
        .map(|claim| -> Result<ProjectedGeometryClaim, String> {
            Ok(ProjectedGeometryClaim {
                geometry_id: claim.geometry_id.clone(),
                feature_artifact: resolve(&claim.feature_artifact, "geometry feature artifact")?,
                evidence: resolve(&claim.evidence, "geometry claim evidence")?,
            })
        })
        .transpose()?;
    let support = row
        .axes
        .support
        .as_ref()
        .map(|claim| -> Result<ProjectedLabelClaim, String> {
            Ok(ProjectedLabelClaim {
                value_id: claim.value_id.clone(),
                evidence: resolve(&claim.evidence, "support claim evidence")?,
            })
        })
        .transpose()?;
    let impact = row
        .axes
        .impact
        .as_ref()
        .map(|claim| -> Result<ProjectedImpactClaim, String> {
            Ok(ProjectedImpactClaim {
                coordinate_profile: claim.coordinate_profile.clone(),
                point_metres: claim.point_metres,
                outward_normal: claim.outward_normal,
                evidence: resolve(&claim.evidence, "impact claim evidence")?,
            })
        })
        .transpose()?;
    let listener = row
        .axes
        .listener
        .as_ref()
        .map(|claim| -> Result<ProjectedListenerClaim, String> {
            Ok(ProjectedListenerClaim {
                coordinate_profile: claim.coordinate_profile.clone(),
                point_metres: claim.point_metres,
                evidence: resolve(&claim.evidence, "listener claim evidence")?,
            })
        })
        .transpose()?;
    let excitation = row
        .axes
        .excitation
        .as_ref()
        .map(|claim| -> Result<ProjectedExcitationClaim, String> {
            Ok(ProjectedExcitationClaim {
                impulse_newton_seconds: claim.impulse_newton_seconds,
                energy_joules: claim.energy_joules,
                force_profile: claim
                    .force_profile
                    .as_ref()
                    .map(|profile| resolve(profile, "force profile artifact"))
                    .transpose()?,
                evidence: resolve(&claim.evidence, "excitation claim evidence")?,
            })
        })
        .transpose()?;
    Ok(ProjectedRow {
        row_id: row.row_id.clone(),
        split_role: row.split_role.as_str(),
        sample_role: row.sample_role.as_str(),
        corpus_role: row.corpus_role.as_str(),
        source_group_id: row.source_group_id.clone(),
        family_group_id: row.family_group_id.clone(),
        object_group_id: row.object_group_id.clone(),
        recording_parent_id: row.recording_parent_id.clone(),
        condition_group_id: row.condition_group_id.clone(),
        mutation_parent_id: row.mutation_parent_id.clone(),
        lineage_report_ids: row.lineage_report_ids.clone(),
        audio: resolve(&row.audio, "neural row audio")?,
        audio_provenance: resolve(&row.audio_provenance, "neural row audio provenance")?,
        axes: ProjectedAxes {
            material,
            geometry,
            support,
            impact,
            listener,
            excitation,
        },
    })
}

pub(super) fn resolve_cached_artifact(
    root: &Path,
    manifest_directory: &Path,
    reference: &FileRef,
    role: &str,
    cache: &mut BTreeMap<(String, String), VerifiedArtifact>,
) -> Result<VerifiedArtifact, String> {
    let key = (reference.path.clone(), reference.sha256.clone());
    if let Some(artifact) = cache.get(&key) {
        return Ok(artifact.clone());
    }
    let artifact =
        VerifiedArtifact::from(resolve_artifact(root, manifest_directory, reference, role)?);
    cache.insert(key, artifact.clone());
    Ok(artifact)
}

pub(super) fn validate_lineage_report(
    root: &Path,
    manifest_directory: &Path,
    lineage: &LineageReport,
) -> Result<(), String> {
    let path = canonical_external_file(
        root,
        &manifest_directory.join(&lineage.artifact.path),
        "neural lineage report",
    )?;
    let bytes = read_bounded_file(&path, MAX_LINEAGE_REPORT_BYTES, "neural lineage report")?;
    let actual_sha256 = sha256_hex(&bytes);
    if actual_sha256 != lineage.artifact.sha256 {
        return Err(format!(
            "neural lineage report hash mismatch for {}: expected {}, got {actual_sha256}",
            path.display(),
            lineage.artifact.sha256
        ));
    }
    let probe: LineageReportProbe = serde_json::from_slice(&bytes)
        .map_err(|error| format!("parse neural lineage report {}: {error}", path.display()))?;
    if probe.schema != lineage.expected_schema
        || probe.status != "Validated"
        || probe.claim != lineage.expected_claim
    {
        return Err(format!(
            "neural lineage report {} does not match declared schema, Validated status and claim",
            lineage.id
        ));
    }
    Ok(())
}

pub(super) fn audit_leakage(rows: &[ProjectedRow]) -> Result<LeakageAudit, String> {
    let mut object_groups = BTreeMap::new();
    let mut source_groups = BTreeMap::new();
    let mut recording_parents = BTreeMap::new();
    let mut condition_groups = BTreeMap::new();
    let mut mutation_parents = BTreeMap::new();
    let mut audio_hashes = BTreeMap::new();
    let mut family_roles = BTreeMap::<&str, BTreeSet<&str>>::new();
    for row in rows {
        bind_group(
            &mut object_groups,
            &row.object_group_id,
            row.split_role,
            "object group",
        )?;
        bind_group(
            &mut source_groups,
            &row.source_group_id,
            row.split_role,
            "source group",
        )?;
        bind_group(
            &mut recording_parents,
            &row.recording_parent_id,
            row.split_role,
            "recording parent",
        )?;
        bind_group(
            &mut condition_groups,
            &row.condition_group_id,
            row.split_role,
            "condition group",
        )?;
        if let Some(parent) = &row.mutation_parent_id {
            bind_group(
                &mut mutation_parents,
                parent,
                row.split_role,
                "mutation parent",
            )?;
        }
        bind_group(
            &mut audio_hashes,
            &row.audio.sha256,
            row.split_role,
            "identical audio",
        )?;
        family_roles
            .entry(&row.family_group_id)
            .or_default()
            .insert(row.split_role);
    }
    Ok(LeakageAudit {
        result: "NoCrossRoleLeakage",
        object_groups_disjoint: true,
        source_groups_disjoint: true,
        recording_parents_disjoint: true,
        condition_groups_disjoint: true,
        mutation_parents_disjoint: true,
        identical_audio_disjoint: true,
        family_group_cross_role_overlap_count: family_roles
            .values()
            .filter(|roles| roles.len() > 1)
            .count(),
    })
}

fn bind_group<'a>(
    groups: &mut BTreeMap<&'a str, &'a str>,
    group: &'a str,
    role: &'a str,
    label: &str,
) -> Result<(), String> {
    if let Some(previous) = groups.insert(group, role)
        && previous != role
    {
        return Err(format!(
            "{label} {group} crosses split roles {previous} and {role}"
        ));
    }
    Ok(())
}
