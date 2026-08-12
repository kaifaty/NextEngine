use next_contracts::ids::PersistentId;
use next_contracts::motor::MotorTerminalDispositionV1;
use next_physics_physx::{CanonicalPhysXSnapshotV2, PhysXArticulationWorldV2};

use crate::{
    BiomechanicsContactClassifier, BiomechanicsProceduralStandingControllerV1,
    BiomechanicsSafetyController, BiomechanicsSkillContactProfileV1, BiomechanicsTerminalEvaluator,
    CompiledBodySchemaV2, JointControlStateV1, PROCEDURAL_STANDING_SCENARIO_MOTOR_TICKS,
    biomechanics_humanoid_body_schema_v2,
};

#[test]
fn native_procedural_standing_reaches_exact_thirty_second_timeout() {
    let compiled = CompiledBodySchemaV2::compile(
        &biomechanics_humanoid_body_schema_v2(),
        PersistentId::from_bytes([61; 16]),
    )
    .expect("compile biomechanics profile");
    let mut world =
        PhysXArticulationWorldV2::create(compiled.physx_scene_profile, &compiled.physx_catalog)
            .expect("create PhysX articulation");
    let mut safety = BiomechanicsSafetyController::new(&compiled).expect("safety controller");
    let mut contacts = BiomechanicsContactClassifier::new(&compiled).expect("contact classifier");
    let mut terminal = BiomechanicsTerminalEvaluator::new(
        &compiled,
        BiomechanicsSkillContactProfileV1::Locomotion,
        PROCEDURAL_STANDING_SCENARIO_MOTOR_TICKS,
    )
    .expect("terminal evaluator");
    let residual = vec![0; compiled.actuator_definitions.len()];
    let envelopes = safety.default_skill_envelopes();
    let mut snapshot = world.capture().expect("initial snapshot");
    let standing = BiomechanicsProceduralStandingControllerV1::new(&compiled, &snapshot)
        .expect("procedural standing controller");
    let mut last_decision = None;
    let mut last_joint_error = None;
    let mut last_motor_tick = 0;
    for motor_tick in 1..=PROCEDURAL_STANDING_SCENARIO_MOTOR_TICKS {
        let reference = standing
            .reference_targets(&snapshot)
            .expect("procedural standing reference");
        safety
            .begin_motor_tick(&reference, &residual, &envelopes)
            .expect("prepare neutral target");
        let mut contact_frames = Vec::new();
        let mut joint_error = None;
        for _ in 0..4 {
            let states = actuator_states(&compiled, &snapshot);
            let efforts = match safety.step_substep(&states) {
                Ok(efforts) => efforts,
                Err(error) => {
                    joint_error = Some(error);
                    break;
                }
            };
            let mut dof_efforts = vec![0; efforts.len()];
            for (effort, dof) in efforts.iter().zip(&compiled.actuator_dof_ordinals) {
                dof_efforts[*dof as usize] = effort.effort_micronewton_metres;
            }
            snapshot = world
                .apply_efforts_and_step(&dof_efforts)
                .expect("native safety substep");
            contact_frames.push(
                contacts
                    .classify_substep(&snapshot, BiomechanicsSkillContactProfileV1::Locomotion)
                    .unwrap_or_else(|error| {
                        panic!(
                            "classify native contacts: {error:?}; {:?}",
                            snapshot.contacts
                        )
                    }),
            );
        }
        while contact_frames.len() < 4 {
            contact_frames.push(
                contacts
                    .classify_substep(&snapshot, BiomechanicsSkillContactProfileV1::Locomotion)
                    .expect("complete terminal contact frame"),
            );
        }
        let decision = terminal
            .evaluate_motor_tick(motor_tick, &snapshot, &contact_frames, false, joint_error)
            .expect("evaluate native terminal state");
        last_joint_error = joint_error;
        last_motor_tick = motor_tick;
        last_decision = Some(decision.clone());
        if decision.disposition != MotorTerminalDispositionV1::Running {
            break;
        }
    }
    let decision = last_decision.expect("at least one motor tick");
    assert_eq!(
        decision.disposition,
        MotorTerminalDispositionV1::Truncated,
        "neutral safety path terminated early at tick {last_motor_tick}: {:?}; joint={last_joint_error:?}; root={:?}; joints={:?}",
        decision.reason,
        snapshot.links.first(),
        snapshot.joints,
    );
}

fn actuator_states(
    compiled: &CompiledBodySchemaV2,
    snapshot: &CanonicalPhysXSnapshotV2,
) -> Vec<JointControlStateV1> {
    compiled
        .actuator_dof_ordinals
        .iter()
        .map(|dof| {
            let state = snapshot
                .joints
                .iter()
                .find(|state| state.ordinal == *dof)
                .expect("snapshot contains every actuator DoF");
            JointControlStateV1 {
                position_microradians: state.position_microradians,
                velocity_microradians_per_second: state.velocity_microradians_per_second,
            }
        })
        .collect()
}
