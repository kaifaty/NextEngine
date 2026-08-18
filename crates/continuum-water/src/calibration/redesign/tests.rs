#![forbid(unsafe_code)]

use super::*;
use crate::audit::scalar_bits;

#[test]
fn predictive_contact_pcg_passes_the_local_soak() {
    let hydro = scenario::find("CW-HYDRO-001").unwrap();
    let samples = scenario::initial_samples(&hydro, StorageOrder::Reverse).unwrap();
    let boundary = support_complete_lattice_complement(hydro.geometry).unwrap();
    let (mut frame, _) =
        solver::initial_frame(samples, hydro.geometry, &boundary, &[0; 32], &[1; 32]).unwrap();
    let mut completed = 0_u32;
    for step in 1..=LOCAL_HYDRO_STEPS {
        let constrained = match solver::contact_pcg_constrained_substep(
            &frame,
            hydro.geometry,
            &boundary,
            &[0; 32],
            &[1; 32],
            PCG_MAXIMUM_ITERATIONS,
        ) {
            Ok(value) => value,
            Err(error) => {
                eprintln!("step {step} failed: {error}");
                break;
            }
        };
        completed = step;
        frame = constrained.outcome.frame;
    }
    assert_eq!(completed, LOCAL_HYDRO_STEPS);
}

#[test]
fn pressure_operator_matches_independent_support_complete_calculator() {
    let hydro = scenario::find("CW-HYDRO-001").unwrap();
    let samples = scenario::initial_samples(&hydro, StorageOrder::Reverse).unwrap();
    let boundary = support_complete_lattice_complement(hydro.geometry).unwrap();
    let production = solver::production_pressure_operator_probe(&samples, &boundary).unwrap();
    let independent = independent_support_complete_pressure_probe().unwrap();

    assert_eq!(production, independent);
    assert!(production.u_dot_b_u_bits != scalar_bits(0.0));
    assert!(production.v_dot_b_v_bits != scalar_bits(0.0));
    assert!(production.symmetry_relative_difference_ppb <= 1);
    assert!(
        production
            .diagonals
            .iter()
            .all(|probe| probe.relative_difference_ppb <= 1)
    );
}

#[test]
fn projected_pcg_first_step_matches_independent_calculator() {
    let hydro = scenario::find("CW-HYDRO-001").unwrap();
    let samples = scenario::initial_samples(&hydro, StorageOrder::Reverse).unwrap();
    let boundary = support_complete_lattice_complement(hydro.geometry).unwrap();
    let production =
        solver::production_projected_pcg_first_step_probe(&samples, &boundary).unwrap();
    let independent = independent_support_complete_projected_pcg_first_step_probe().unwrap();

    assert_eq!(production, independent);
    assert_eq!(production.density_iterations, 2);
    assert_eq!(production.density_error_ppb, 99_723);
}

#[test]
fn predictive_contact_matches_independent_analytical_calculator() {
    let production = solver::production_contact_projection_probe().unwrap();
    let independent = independent_contact_projection_probe().unwrap();

    assert_eq!(production, independent);
    assert_eq!(production.cases.len(), 8);
    assert_eq!(production.cases[0].active_components, 1);
    assert_eq!(production.cases[1].active_components, 0);
    assert_eq!(production.cases[3].active_components, 2);
    assert_eq!(production.cases[4].active_components, 3);
}

#[test]
fn redesign_cli_rejects_an_unknown_candidate() {
    let repository_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .unwrap();
    let error = crate::run_xtask(
        repository_root,
        [
            "water".to_owned(),
            "evaluate-hydro-redesign".to_owned(),
            "--candidate".to_owned(),
            "unknown".to_owned(),
            "--output".to_owned(),
            "/tmp/not-created.json".to_owned(),
        ]
        .into_iter(),
    )
    .unwrap_err();
    assert!(error.starts_with("WATER_SCENARIO_INVALID:"));
}
