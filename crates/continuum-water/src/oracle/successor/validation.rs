#![forbid(unsafe_code)]

use super::*;
use crate::geometry::AxisAlignedGeometryManifest;
use crate::profile::MAXIMUM_BOUNDARY_PENETRATION_UM;

pub(super) fn serializable_root<T: Serialize>(
    domain: &[u8],
    value: &T,
) -> Result<String, WaterError> {
    let bytes = serde_json::to_vec(value).map_err(|error| {
        WaterError::new(
            REPORT_CAPACITY_EXCEEDED,
            format!("cannot serialize W1 rooted metrics: {error}"),
        )
    })?;
    let mut hasher = Sha256::new();
    hasher.update(domain);
    hasher.update(bytes);
    Ok(hash::hex(&hasher.finalize().into()))
}

pub(super) fn capacity_thresholds() -> Vec<CapacityThreshold> {
    vec![
        capacity_threshold("samples", MAXIMUM_SAMPLES, SAMPLE_CAPACITY_EXCEEDED),
        capacity_threshold(
            "static-boundary-samples",
            SUCCESSOR_MAXIMUM_STATIC_BOUNDARY_SAMPLES,
            BOUNDARY_CAPACITY_EXCEEDED,
        ),
        capacity_threshold(
            "fluid-row-neighbors",
            MAXIMUM_NEIGHBORS_PER_FLUID_ROW,
            NEIGHBOR_CAPACITY_EXCEEDED,
        ),
        capacity_threshold(
            "boundary-row-neighbors",
            MAXIMUM_NEIGHBORS_PER_BOUNDARY_ROW,
            BOUNDARY_NEIGHBOR_CAPACITY_EXCEEDED,
        ),
        capacity_threshold("steps", MAXIMUM_STEPS as usize, STEP_CAPACITY_EXCEEDED),
    ]
}

fn capacity_threshold(resource: &str, maximum: usize, code: &'static str) -> CapacityThreshold {
    CapacityThreshold {
        resource: resource.to_owned(),
        below: capacity_outcome(maximum - 1, maximum, code),
        equal: capacity_outcome(maximum, maximum, code),
        above: capacity_outcome(maximum + 1, maximum, code),
    }
}

fn capacity_outcome(value: usize, maximum: usize, code: &'static str) -> String {
    match scenario::validate_capacity(value, maximum, code, "W1 capacity threshold") {
        Ok(()) => "PASS".to_owned(),
        Err(error) if error.code() == code => "EXPECTED_REJECTION".to_owned(),
        Err(error) => format!("UNEXPECTED:{}", error.code()),
    }
}

pub(super) fn validate_step(
    selected: &Scenario,
    next: &solver::ContactConstrainedStepOutcome,
) -> Result<(), WaterError> {
    let summary = &next.outcome.summary;
    if !(2..=50).contains(&summary.density_iterations)
        || summary.density_error_ppb > 100_000
        || !summary
            .density_kkt_error_ppb
            .is_some_and(|error_ppb| error_ppb <= 100_000)
        || !(1..=20).contains(&summary.divergence_iterations)
        || summary.divergence_error_ppb > 1_000_000
        || !canonical_penetration_is_admitted(summary.maximum_penetration_um)
    {
        let clearance_witness = if summary.maximum_penetration_um == 0 {
            String::new()
        } else {
            minimum_clearance_witness(selected, &next.outcome.frame.samples)?
        };
        return Err(WaterError::new(
            INVARIANT_MISMATCH,
            format!(
                "{} step {} violates the successor solver/clearance bounds: density={}/{} ppb, density-kkt={:?} ppb, divergence={}/{} ppb, penetration={} um{clearance_witness}",
                selected.id,
                summary.step,
                summary.density_iterations,
                summary.density_error_ppb,
                summary.density_kkt_error_ppb,
                summary.divergence_iterations,
                summary.divergence_error_ppb,
                summary.maximum_penetration_um,
            ),
        ));
    }
    Ok(())
}

fn canonical_penetration_is_admitted(penetration_um: i64) -> bool {
    (0..=MAXIMUM_BOUNDARY_PENETRATION_UM).contains(&penetration_um)
}

fn minimum_clearance_witness(
    selected: &Scenario,
    samples: &[crate::model::CanonicalSample],
) -> Result<String, WaterError> {
    let manifest = AxisAlignedGeometryManifest::from_geometry(selected.geometry)?;
    let mut witness = None;
    for sample in samples {
        let observation = manifest.observe(sample.position_um)?;
        if witness.as_ref().is_none_or(
            |(_, current): &(u32, crate::geometry::GeometryObservation)| {
                observation.closest_distance_squared_um2 < current.closest_distance_squared_um2
            },
        ) {
            witness = Some((sample.id, observation));
        }
    }
    let (sample_id, observation) = witness.ok_or_else(|| {
        WaterError::new(
            INVARIANT_MISMATCH,
            "positive penetration has no canonical sample witness",
        )
    })?;
    Ok(format!(
        "; witness=sample-{sample_id}@({},{},{}),feature-{},distance-squared-{}-um2",
        observation.position_um.x,
        observation.position_um.y,
        observation.position_um.z,
        observation.closest_feature_id,
        observation.closest_distance_squared_um2,
    ))
}

pub(super) fn validate_output(
    selected: &Scenario,
    solver_mode: W1SolverMode,
    output: &W1OutputMetric,
) -> Result<(), WaterError> {
    let common = &output.common;
    let expected_count = scenario::initial_samples(selected, StorageOrder::Identity)?.len();
    let expected_mass = u64::try_from(expected_count)
        .ok()
        .and_then(|value| value.checked_mul(125_000))
        .ok_or_else(|| WaterError::new(NUMERIC_OVERFLOW, "W1 expected mass overflow"))?;
    if common.sample_count as usize != expected_count || common.mass_mg != expected_mass {
        return Err(WaterError::new(
            INVARIANT_MISMATCH,
            format!("{} step {} changed count or mass", selected.id, common.step),
        ));
    }
    let (energy_metric_name, energy_metric) = match energy_class(selected.id)? {
        EnergyClass::ReversibleEquilibrium => ("absolute energy drift", common.energy_residual_ppb),
        EnergyClass::StaticImpactDissipative => {
            ("mechanical energy excess", output.energy_excess_ppb)
        }
    };
    if energy_metric > 10_000_000 && solver_mode != W1SolverMode::FrozenObserveEnergyDiagnostic {
        return Err(WaterError::new(
            INVARIANT_MISMATCH,
            format!(
                "{} step {} {energy_metric_name} {energy_metric} ppb exceeds 10000000",
                selected.id, common.step
            ),
        ));
    }
    if common.momentum_residual_ppb > 10_000_000 {
        return Err(WaterError::new(
            INVARIANT_MISMATCH,
            format!(
                "{} step {} momentum residual {} ppb exceeds 10000000",
                selected.id, common.step, common.momentum_residual_ppb
            ),
        ));
    }
    Ok(())
}

pub(super) fn primary_order(selected: &Scenario) -> StorageOrder {
    if selected.id == "CW-ORDER-001" {
        StorageOrder::Identity
    } else {
        StorageOrder::Reverse
    }
}

pub(super) fn reference_required(scenario_id: &str) -> bool {
    matches!(
        scenario_id,
        "CW-HYDRO-001" | "CW-DAMBREAK-001" | "CW-ORIFICE-001"
    )
}

pub(super) fn reference_path<'a>(request: &'a Request, scenario_id: &str) -> Option<&'a Path> {
    if request.scenario_id.is_some() {
        return request.reference.as_deref();
    }
    match scenario_id {
        "CW-HYDRO-001" => request.hydro_reference.as_deref(),
        "CW-DAMBREAK-001" => request.dam_break_reference.as_deref(),
        "CW-ORIFICE-001" => request.orifice_reference.as_deref(),
        _ => None,
    }
}

pub(super) fn corpus_run_root(
    roots: &AcceleratedPressureRoots,
    scenarios: &[ScenarioEvidence],
) -> String {
    let mut hasher = Sha256::new();
    hasher.update(b"nextengine.continuum-water.w1-linux-corpus-run.v1\0");
    hasher.update(roots.execution_profile);
    for evidence in scenarios {
        hash_text(&mut hasher, &evidence.id);
        hash_text(&mut hasher, &evidence.scenario_root);
        if let Some(primary) = &evidence.primary {
            hash_text(
                &mut hasher,
                primary.trajectory_root.as_deref().unwrap_or("INCOMPLETE"),
            );
            hash_text(
                &mut hasher,
                primary
                    .output_metrics_root
                    .as_deref()
                    .unwrap_or("INCOMPLETE"),
            );
        }
        for root in &evidence.repeat_roots {
            hash_text(&mut hasher, &root.name);
            hash_text(&mut hasher, &root.trajectory_root);
        }
        for root in &evidence.permutation_roots {
            hash_text(&mut hasher, &root.name);
            hash_text(&mut hasher, &root.trajectory_root);
        }
        hash_text(
            &mut hasher,
            evidence
                .reference
                .sha256
                .as_deref()
                .unwrap_or("REFERENCE_MISSING"),
        );
    }
    hash::hex(&hasher.finalize().into())
}

fn hash_text(hasher: &mut Sha256, value: &str) {
    hasher.update(value.len().to_le_bytes());
    hasher.update(value.as_bytes());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn canonical_penetration_uses_the_frozen_inclusive_product_limit() {
        assert!(!canonical_penetration_is_admitted(-1));
        assert!(canonical_penetration_is_admitted(0));
        assert!(canonical_penetration_is_admitted(
            MAXIMUM_BOUNDARY_PENETRATION_UM
        ));
        assert!(!canonical_penetration_is_admitted(
            MAXIMUM_BOUNDARY_PENETRATION_UM + 1
        ));
    }
}
