use next_motor::CompiledBodySchemaV4;
use next_physics_physx::CanonicalPhysXSnapshotV2;
use serde_json::{Value, json};

type ProbeResult<T> = Result<T, Box<dyn std::error::Error>>;

fn require_match(
    actual: &CanonicalPhysXSnapshotV2,
    expected: &CanonicalPhysXSnapshotV2,
) -> ProbeResult<()> {
    if actual != expected {
        return Err("native reconstruction mismatch before response measurement".into());
    }
    Ok(())
}

fn snapshot_json(snapshot: &CanonicalPhysXSnapshotV2) -> Value {
    json!({
        "links": snapshot.links.iter().map(|l| json!({
            "body_token": l.user_token, "position_um": l.position_micrometres,
            "rotation_q1_30": l.rotation_q1_30,
            "linear_velocity_um_s": l.linear_velocity_micrometres_per_second,
            "angular_velocity_urad_s": l.angular_velocity_microradians_per_second,
        })).collect::<Vec<_>>(),
        "joints": snapshot.joints.iter().map(|j| json!({
            "ordinal": j.ordinal, "position_urad": j.position_microradians,
            "velocity_urad_s": j.velocity_microradians_per_second,
        })).collect::<Vec<_>>(),
        "contacts": snapshot.contacts.iter().map(|c| json!({
            "actors": [c.actor_a_token, c.actor_b_token],
            "shapes": [c.shape_a_token, c.shape_b_token],
            "position_um": c.position_micrometres, "normal_q1_30": c.normal_q1_30,
            "impulse_uns": c.impulse_micronewton_seconds,
            "separation_um": c.separation_micrometres,
        })).collect::<Vec<_>>(),
    })
}

pub fn measure(
    compiled: &CompiledBodySchemaV4,
    expected: &CanonicalPhysXSnapshotV2,
    history: &[[i64; 23]],
) -> ProbeResult<Value> {
    if history.len() != 240 {
        return Err("response requires exactly 240 recorded native steps".into());
    }
    let base_efforts = *history.last().ok_or("empty effort history")?;
    let replay_step = |efforts: &[i64; 23]| -> ProbeResult<CanonicalPhysXSnapshotV2> {
        for (actuator, dof) in compiled
            .base
            .base
            .actuator_definitions
            .iter()
            .zip(&compiled.base.base.actuator_dof_ordinals)
        {
            if !(actuator.minimum_effort_micronewton_metres
                ..=actuator.maximum_effort_micronewton_metres)
                .contains(&efforts[*dof as usize])
            {
                return Err("response command exceeds actuator effort bound".into());
            }
        }
        let mut world = compiled.create_world()?;
        for efforts in history {
            world.apply_efforts_and_step(efforts)?;
        }
        require_match(&world.capture()?, expected)?;
        Ok(world.apply_efforts_and_step(efforts)?)
    };
    let control = replay_step(&base_efforts)?;
    let repeat = replay_step(&base_efforts)?;
    require_match(&repeat, &control)?;
    let mut trials = Vec::new();
    for magnitude in [10_000_i64, 20_000] {
        for dof in 0..23 {
            for sign in [-1_i64, 1] {
                let delta = sign * magnitude;
                let mut efforts = base_efforts;
                efforts[dof] = efforts[dof].checked_add(delta).ok_or("effort overflow")?;
                let after = replay_step(&efforts)?;
                trials.push(json!({
                    "dof_ordinal": dof, "delta_effort_unm": delta,
                    "applied_efforts_by_dof_unm": efforts,
                    "after": snapshot_json(&after),
                }));
            }
        }
    }
    Ok(json!({
        "contract": "r8b-coupled-effort-response.v1", "physics_hz": 240,
        "scope": "one-step plant response, not a safety-controlled rollout or inverse mass matrix",
        "compiled_descriptor_hash": compiled.compiled_descriptor_hash.to_hex(),
        "prefix_steps": history.len(), "exact_reconstructions": trials.len() + 2,
        "before": snapshot_json(expected),
        "baseline_efforts_by_dof_unm": base_efforts,
        "control_after": snapshot_json(&control),
        "repeat_control_after": snapshot_json(&repeat),
        "trials": trials,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reconstruction_guard_rejects_a_changed_snapshot() {
        let original = CanonicalPhysXSnapshotV2 {
            links: Vec::new(),
            contacts: Vec::new(),
            joints: vec![next_physics_physx::CanonicalPhysXJointState {
                ordinal: 0,
                position_microradians: 0,
                velocity_microradians_per_second: 0,
            }],
        };
        require_match(&original, &original).unwrap();
        let mut changed = original.clone();
        changed.joints[0].velocity_microradians_per_second = 1;
        assert!(require_match(&changed, &original).is_err());
        assert_ne!(snapshot_json(&changed), snapshot_json(&original));
    }
}
