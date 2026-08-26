use super::*;

#[test]
fn arguments_require_external_profile_and_output() {
    assert!(
        parse_arguments(
            [
                "--profile".to_owned(),
                "/tmp/profile.json".to_owned(),
                "--output".to_owned(),
                "/tmp/output".to_owned(),
            ]
            .into_iter(),
        )
        .is_ok()
    );
    assert!(parse_arguments(std::iter::empty()).is_err());
    assert!(
        parse_arguments(
            [
                "--profile".to_owned(),
                "/tmp/profile.json".to_owned(),
                "--profile".to_owned(),
                "/tmp/other.json".to_owned(),
                "--output".to_owned(),
                "/tmp/output".to_owned(),
            ]
            .into_iter(),
        )
        .is_err()
    );
}

#[test]
fn control_relations_distinguish_position_and_preserve_force() {
    let profile = control_profile();
    let rendered = profile
        .conditions
        .iter()
        .map(|condition| {
            let reference = condition
                .amplitudes
                .iter()
                .flat_map(|amplitude| [*amplitude, -*amplitude])
                .collect::<Vec<_>>();
            (
                condition.id.clone(),
                RenderedCondition {
                    reference,
                    reference_file: String::new(),
                    reference_sha256: String::new(),
                    repeated_q30: true,
                },
            )
        })
        .collect::<BTreeMap<_, _>>();

    let force = evaluate_force_control(&profile, &rendered).expect("evaluate force relation");
    let position = evaluate_position_control(&profile).expect("evaluate position relation");

    assert_eq!(force.status, "PASS");
    assert_eq!(position.status, "PASS");
    assert!(position.minimum_cosine_distance >= POSITION_COSINE_DISTANCE_MINIMUM);
}

#[test]
fn residual_and_audition_helpers_are_bounded() {
    let residual =
        calculate_residual(&[0.5, -0.5], &[0.5, -0.25]).expect("calculate finite residual");
    assert_eq!(residual.maximum_absolute, 0.25);
    assert!(residual.rms > 0.0);
    assert!(calculate_residual(&[], &[]).is_err());

    let audition =
        concatenate_mono(&[&[0.5], &[-0.5]], 48_000, 1).expect("concatenate bounded clips");
    assert_eq!(audition.len(), 50);
    assert_eq!(audition[0], 0.5);
    assert!(audition[1..49].iter().all(|sample| *sample == 0.0));
    assert_eq!(audition[49], -0.5);
}

fn control_profile() -> ExternalProfile {
    let impulses = vec![
        ExternalImpulse {
            id: "low".to_owned(),
            newton_seconds: 1.0,
        },
        ExternalImpulse {
            id: "medium".to_owned(),
            newton_seconds: 2.0,
        },
        ExternalImpulse {
            id: "high".to_owned(),
            newton_seconds: 4.0,
        },
    ];
    let strikes = vec![
        ExternalStrike {
            id: "left".to_owned(),
            role: "train".to_owned(),
            point_m: [1.0, 0.0, 0.0],
            direction: [-1.0, 0.0, 0.0],
            mesh_node_index: 0,
        },
        ExternalStrike {
            id: "right".to_owned(),
            role: "train".to_owned(),
            point_m: [1.0, 1.0, 0.0],
            direction: [-1.0, 0.0, 0.0],
            mesh_node_index: 1,
        },
        ExternalStrike {
            id: "heldout".to_owned(),
            role: "heldout".to_owned(),
            point_m: [1.0, 0.5, 0.0],
            direction: [-1.0, 0.0, 0.0],
            mesh_node_index: 2,
        },
    ];
    let base = BTreeMap::from([
        ("left", [1.0, 0.0]),
        ("right", [0.0, 1.0]),
        ("heldout", [0.8, 0.6]),
    ]);
    let mut conditions = Vec::new();
    for strike in &strikes {
        for impulse in &impulses {
            conditions.push(ExternalCondition {
                id: format!("{}--{}", strike.id, impulse.id),
                split: expected_split(&strike.role, &impulse.id).to_owned(),
                strike_id: strike.id.clone(),
                force_id: impulse.id.clone(),
                impulse_newton_seconds: impulse.newton_seconds,
                amplitudes: base[strike.id.as_str()]
                    .map(|amplitude| amplitude * impulse.newton_seconds)
                    .to_vec(),
            });
        }
    }
    conditions.sort_by(|left, right| left.id.cmp(&right.id));
    ExternalProfile {
        schema: PROFILE_SCHEMA.to_owned(),
        claim: REQUIRED_CLAIM.to_owned(),
        object_id: "test-glass".to_owned(),
        source_recipe: ExternalFile {
            file: "recipe.json".to_owned(),
            sha256: "a".repeat(64),
        },
        solver: ExternalSolver {
            id: "solver".to_owned(),
            python: "3.12".to_owned(),
            numpy: "2".to_owned(),
            scipy: "1".to_owned(),
            method: "test".to_owned(),
        },
        geometry: ExternalGeometry {
            kind: "open_rectangular_vessel".to_owned(),
            outer_size_m: [2.0, 2.0, 2.0],
            wall_thickness_m: 0.1,
            bottom_thickness_m: 0.1,
            cell_size_m: 0.1,
            axes: "+X right, +Y up, +Z forward".to_owned(),
            tetrahedra_per_cell: 6,
            node_count: 4,
            tetrahedron_count: 1,
            fixed_node_count: 4,
            node_file: "nodes.npy".to_owned(),
            node_sha256: "a".repeat(64),
            tetrahedron_file: "tetrahedra.npy".to_owned(),
            tetrahedron_sha256: "a".repeat(64),
            fixed_node_file: "fixed.npy".to_owned(),
            fixed_node_sha256: "a".repeat(64),
        },
        material: ExternalMaterial {
            name: "glass".to_owned(),
            density_kg_m3: 2_500.0,
            youngs_modulus_pa: 70.0e9,
            poisson_ratio: 0.22,
            damping: ExternalDamping {
                kind: "explicit_frequency_calibration".to_owned(),
                base_per_second: 50.0,
                slope_per_hz: 0.002,
            },
        },
        boundary: ExternalBoundary {
            kind: "fixed_bottom_back_left_patch".to_owned(),
            patch_size_m: 0.1,
        },
        modal_basis: ExternalModalBasis {
            file: "basis.npy".to_owned(),
            sha256: "a".repeat(64),
            normalization: "test".to_owned(),
            sign_rule: "test".to_owned(),
        },
        pickup: ExternalPoint {
            point_m: [-1.0, 0.0, 0.0],
            direction: [1.0, 0.0, 0.0],
            mesh_node_index: 3,
        },
        sample_rate_hz: 48_000,
        frame_count: 4_800,
        global_amplitude_gain: 1.0,
        target_global_peak: 0.5,
        modes: vec![
            ExternalMode {
                index: 0,
                undamped_frequency_hz: 1_600.0,
                damped_frequency_hz: 1_599.9,
                damping_per_second: 53.2,
                t60_seconds: 1000.0_f64.ln() / 53.2,
                relative_eigen_residual: 1.0e-9,
            },
            ExternalMode {
                index: 1,
                undamped_frequency_hz: 4_200.0,
                damped_frequency_hz: 4_199.9,
                damping_per_second: 58.4,
                t60_seconds: 1000.0_f64.ln() / 58.4,
                relative_eigen_residual: 1.0e-9,
            },
        ],
        strikes,
        impulses,
        heldout: ExternalHeldout {
            position_interpolator: "inverse_distance_squared".to_owned(),
            train_force_ids: vec!["low".to_owned(), "high".to_owned()],
            force_holdout_id: "medium".to_owned(),
            position_holdout_strike_id: "heldout".to_owned(),
        },
        conditions,
    }
}
