use super::*;

fn parse(arguments: &[&str]) -> Result<Request, WaterError> {
    parse_arguments(arguments.iter().map(|value| (*value).to_owned()))
}

#[test]
fn parser_separates_scenario_and_full_corpus_reference_modes() {
    assert!(
        parse(&[
            "--scenario",
            "CW-HYDRO-001",
            "--reference",
            "/tmp/hydro.bin",
            "--output",
            "/tmp/report.json",
        ])
        .is_ok()
    );
    assert!(
        parse(&[
            "--hydro-reference",
            "/tmp/hydro.bin",
            "--output",
            "/tmp/report.json",
        ])
        .is_ok()
    );
    assert!(
        parse(&[
            "--scenario",
            "CW-HYDRO-001",
            "--hydro-reference",
            "/tmp/hydro.bin",
            "--output",
            "/tmp/report.json",
        ])
        .is_err()
    );
    assert!(
        parse(&[
            "--reference",
            "/tmp/hydro.bin",
            "--output",
            "/tmp/report.json",
        ])
        .is_err()
    );
}

#[test]
fn parser_rejects_unknown_scenarios_and_duplicate_outputs() {
    assert!(
        parse(&[
            "--scenario",
            "SMOKE-CW-HYDRO-001",
            "--output",
            "/tmp/report.json",
        ])
        .is_err()
    );
    assert!(parse(&["--output", "/tmp/one.json", "--output", "/tmp/two.json",]).is_err());
}

#[test]
fn successor_capacity_matrix_accepts_equal_and_rejects_above() {
    for threshold in capacity_thresholds() {
        assert_eq!(threshold.below, "PASS");
        assert_eq!(threshold.equal, "PASS");
        assert_eq!(threshold.above, "EXPECTED_REJECTION");
    }
}

#[test]
fn w0g_energy_publication_separates_excess_deficit_and_zero() {
    assert_eq!(
        publish_energy_metrics(0.0, 100.0, 0.0, 101.0).unwrap(),
        EnergyPublication {
            signed_balance_ppb: 10_000_000,
            excess_ppb: 10_000_000,
            deficit_ppb: 0,
        }
    );
    assert_eq!(
        publish_energy_metrics(0.0, 100.0, 0.0, 99.0).unwrap(),
        EnergyPublication {
            signed_balance_ppb: -10_000_000,
            excess_ppb: 0,
            deficit_ppb: 10_000_000,
        }
    );
    assert_eq!(
        publish_energy_metrics(25.0, 75.0, 25.0, 75.0).unwrap(),
        EnergyPublication {
            signed_balance_ppb: 0,
            excess_ppb: 0,
            deficit_ppb: 0,
        }
    );
}

#[test]
fn every_w1_scenario_has_one_exact_w0g_energy_projection() {
    let repository_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .unwrap();
    let roots = ImpactEnergyRoots::verify(repository_root).unwrap();
    let mut reversible = 0;
    let mut impact = 0;
    for scenario_id in SCENARIO_IDS {
        assert_eq!(
            roots.scenario_projection(scenario_id).unwrap(),
            energy_contract_projection(scenario_id).unwrap().as_bytes()
        );
        match energy_class(scenario_id).unwrap() {
            EnergyClass::ReversibleEquilibrium => reversible += 1,
            EnergyClass::StaticImpactDissipative => impact += 1,
        }
    }
    assert_eq!((reversible, impact), (4, 3));
    assert!(energy_class("CW-UNKNOWN-001").is_err());
}

#[test]
fn diagnostic_solver_requires_one_explicit_scenario() {
    assert!(
        parse(&[
            "--solver",
            "diagnostic-frozen-observe-energy",
            "--output",
            "/tmp/report.json",
        ])
        .is_err()
    );
    assert!(
        parse(&[
            "--scenario",
            "CW-DAMBREAK-001",
            "--solver",
            "diagnostic-frozen-observe-energy",
            "--output",
            "/tmp/report.json",
        ])
        .is_ok()
    );
}
