#![forbid(unsafe_code)]

use serde::Serialize;

use super::*;

#[derive(Clone, Serialize)]
pub(super) struct W1OutputMetric {
    #[serde(flatten)]
    pub(super) common: OutputMetric,
    pub(super) signed_energy_balance_ppb: i64,
    pub(super) energy_excess_ppb: i64,
    pub(super) energy_deficit_ppb: i64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct EnergyPublication {
    pub(super) signed_balance_ppb: i64,
    pub(super) excess_ppb: i64,
    pub(super) deficit_ppb: i64,
}

#[derive(Serialize)]
pub(super) struct EnergyStageEvidence {
    stage: &'static str,
    cumulative_delta_bits: String,
    normalized_delta_ppb: i64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum EnergyClass {
    ReversibleEquilibrium,
    StaticImpactDissipative,
}

const ENERGY_STAGES: [&str; 6] = [
    "divergence-pressure",
    "gravity",
    "density-pressure",
    "static-contact",
    "position-integration",
    "canonical-publication",
];

pub(super) fn initial_energy_stage_evidence() -> Vec<EnergyStageEvidence> {
    ENERGY_STAGES
        .into_iter()
        .map(|stage| EnergyStageEvidence {
            stage,
            cumulative_delta_bits: "0x0000000000000000".to_owned(),
            normalized_delta_ppb: 0,
        })
        .collect()
}

pub(super) fn step_energy_deltas(trace: solver::StepEnergyTrace) -> [f64; 6] {
    [
        trace.after_divergence - trace.decoded,
        trace.after_gravity - trace.after_divergence,
        trace.after_density - trace.after_gravity,
        trace.after_contact - trace.after_density,
        trace.after_integration - trace.after_contact,
        trace.after_publication - trace.after_integration,
    ]
}

pub(super) fn checked_stage_energy_sum(deltas: &[f64; 6]) -> Result<f64, WaterError> {
    deltas.iter().try_fold(0.0_f64, |total, delta| {
        checked_scalar(total + delta, "W1 stage energy sum")
    })
}

pub(super) fn update_energy_stage_evidence(
    evidence: &mut [EnergyStageEvidence],
    deltas: [f64; 6],
    denominator: f64,
) -> Result<(), WaterError> {
    if evidence.len() != ENERGY_STAGES.len() {
        return Err(WaterError::new(
            INVARIANT_MISMATCH,
            "W1 stage energy evidence has the wrong shape",
        ));
    }
    for (row, delta) in evidence.iter_mut().zip(deltas) {
        row.cumulative_delta_bits = format!("0x{:016x}", delta.to_bits());
        row.normalized_delta_ppb = quantize_ppb(checked_scalar(
            delta / denominator,
            "W1 normalized stage energy delta",
        )?)?;
    }
    Ok(())
}

pub(super) fn w1_output_metric(
    scenario: &Scenario,
    frame: &crate::model::AcceptedFrame,
    centre_of_mass_um: Vec3i,
    baseline: super::super::PhysicalTotals,
    gravity_impulse: Vec3f,
    boundary_impulse: Vec3f,
) -> Result<W1OutputMetric, WaterError> {
    let common = output_metric(
        scenario,
        frame,
        centre_of_mass_um,
        baseline,
        gravity_impulse,
        boundary_impulse,
    )?;
    let current = physical_totals(frame)?;
    let energy = publish_energy_metrics(
        baseline.kinetic,
        baseline.potential,
        current.kinetic,
        current.potential,
    )?;
    let signed_absolute =
        i64::try_from(energy.signed_balance_ppb.unsigned_abs()).map_err(|_| {
            WaterError::new(
                NUMERIC_OVERFLOW,
                "W1 signed energy absolute value does not fit i64",
            )
        })?;
    if (common.energy_residual_ppb - signed_absolute).unsigned_abs() > 1 {
        return Err(WaterError::new(
            INVARIANT_MISMATCH,
            format!(
                "W1 signed and absolute energy metrics disagree at step {}",
                common.step
            ),
        ));
    }
    Ok(W1OutputMetric {
        common,
        signed_energy_balance_ppb: energy.signed_balance_ppb,
        energy_excess_ppb: energy.excess_ppb,
        energy_deficit_ppb: energy.deficit_ppb,
    })
}

pub(super) fn publish_energy_metrics(
    baseline_kinetic: f64,
    baseline_potential: f64,
    current_kinetic: f64,
    current_potential: f64,
) -> Result<EnergyPublication, WaterError> {
    let baseline_energy = checked_scalar(
        baseline_kinetic + baseline_potential,
        "W1 baseline signed mechanical energy",
    )?;
    let current_energy = checked_scalar(
        current_kinetic + current_potential,
        "W1 current signed mechanical energy",
    )?;
    let denominator = checked_scalar(
        (baseline_kinetic.abs() + baseline_potential.abs()).max(1.0),
        "W1 signed energy denominator",
    )?;
    let signed_balance_ppb = quantize_ppb(checked_scalar(
        (current_energy - baseline_energy) / denominator,
        "W1 signed energy balance",
    )?)?;
    Ok(EnergyPublication {
        signed_balance_ppb,
        excess_ppb: signed_balance_ppb.max(0),
        deficit_ppb: signed_balance_ppb.checked_neg().unwrap_or(i64::MAX).max(0),
    })
}

pub(super) fn energy_class(scenario_id: &str) -> Result<EnergyClass, WaterError> {
    match scenario_id {
        "CW-HYDRO-001" | "CW-FREEFALL-001" | "CW-STILL-001" | "CW-ORDER-001" => {
            Ok(EnergyClass::ReversibleEquilibrium)
        }
        "CW-DAMBREAK-001" | "CW-ORIFICE-001" | "CW-SEALED-001" => {
            Ok(EnergyClass::StaticImpactDissipative)
        }
        _ => Err(WaterError::new(
            SCENARIO_INVALID,
            format!("scenario {scenario_id:?} has no W0G energy class"),
        )),
    }
}

pub(super) fn energy_contract_projection(scenario_id: &str) -> Result<&'static str, WaterError> {
    match scenario_id {
        "CW-HYDRO-001" => Ok(
            "scenario.CW-HYDRO-001.energy-class=reversible-equilibrium;metric=absolute-drift;maximum-ppb=10000000\n",
        ),
        "CW-FREEFALL-001" => Ok(
            "scenario.CW-FREEFALL-001.energy-class=reversible-equilibrium;metric=absolute-drift;maximum-ppb=10000000\n",
        ),
        "CW-DAMBREAK-001" => Ok(
            "scenario.CW-DAMBREAK-001.energy-class=static-impact-dissipative;metric=energy-excess;maximum-ppb=10000000;mechanical-deficit=diagnostic;reference=front,height\n",
        ),
        "CW-STILL-001" => Ok(
            "scenario.CW-STILL-001.energy-class=reversible-equilibrium;metric=absolute-drift;maximum-ppb=10000000\n",
        ),
        "CW-ORIFICE-001" => Ok(
            "scenario.CW-ORIFICE-001.energy-class=static-impact-dissipative;metric=energy-excess;maximum-ppb=10000000;mechanical-deficit=diagnostic;reference=transfer\n",
        ),
        "CW-SEALED-001" => Ok(
            "scenario.CW-SEALED-001.energy-class=static-impact-dissipative;metric=energy-excess;maximum-ppb=10000000;mechanical-deficit=diagnostic;reference=none\n",
        ),
        "CW-ORDER-001" => Ok(
            "scenario.CW-ORDER-001.energy-class=reversible-equilibrium;metric=absolute-drift;maximum-ppb=10000000\n",
        ),
        _ => Err(WaterError::new(
            SCENARIO_INVALID,
            format!("scenario {scenario_id:?} has no W0G energy projection"),
        )),
    }
}
