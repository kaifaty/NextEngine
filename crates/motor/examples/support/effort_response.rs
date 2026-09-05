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
    measure_from_history(
        compiled,
        expected,
        history,
        base_efforts,
        "r8b-coupled-effort-response.v1",
    )
}

fn measure_from_history(
    compiled: &CompiledBodySchemaV4,
    expected: &CanonicalPhysXSnapshotV2,
    history: &[[i64; 23]],
    base_efforts: [i64; 23],
    contract: &str,
) -> ProbeResult<Value> {
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
        "contract": contract, "physics_hz": 240,
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

fn raised_schema(
    mut schema: next_contracts::body::BodySchemaV2,
) -> ProbeResult<next_contracts::body::BodySchemaV2> {
    use next_contracts::canonical::sha256;
    use next_contracts::ids::{SchemaId, content_hash_from_bytes};
    schema.schema_id = SchemaId::new(format!("{}.cold-lift-500mm", schema.schema_id.as_str()))?;
    let mut source = b"nextengine.probe.cold-lift-500mm.v1\0".to_vec();
    source.extend_from_slice(schema.source_provenance_hash.as_bytes());
    schema.source_provenance_hash = content_hash_from_bytes(sha256(&source));
    let mut count = 0;
    for body in &mut schema.bodies {
        if body.parent_body_id.is_none() {
            body.local_bind_pose.translation_micrometres[1] += 500_000;
            count += 1;
        }
    }
    if count != 1 {
        return Err("expected unique body root".into());
    }
    schema.validate()?;
    Ok(schema)
}

pub fn measure_cold_pair(schema: &next_contracts::body::BodySchemaV2) -> ProbeResult<Value> {
    use next_contracts::ids::PersistentId;
    let base = CompiledBodySchemaV4::compile(schema, PersistentId::from_bytes([0; 16]))?;
    let raised = CompiledBodySchemaV4::compile(
        &raised_schema(schema.clone())?,
        PersistentId::from_bytes([0; 16]),
    )?;
    let grounded_state = base.create_world()?.capture()?;
    let raised_state = raised.create_world()?.capture()?;
    if grounded_state.links.len() != raised_state.links.len()
        || grounded_state.joints != raised_state.joints
        || grounded_state
            .joints
            .iter()
            .any(|j| j.position_microradians != 0 || j.velocity_microradians_per_second != 0)
    {
        return Err("cold joint-state control failed".into());
    }
    for (grounded, raised) in grounded_state.links.iter().zip(&raised_state.links) {
        if grounded.user_token != raised.user_token
            || grounded.rotation_q1_30 != raised.rotation_q1_30
            || grounded.linear_velocity_micrometres_per_second != [0; 3]
            || raised.linear_velocity_micrometres_per_second != [0; 3]
            || grounded.angular_velocity_microradians_per_second != [0; 3]
            || raised.angular_velocity_microradians_per_second != [0; 3]
            || (0..3).any(|axis| {
                (raised.position_micrometres[axis]
                    - grounded.position_micrometres[axis]
                    - if axis == 1 { 500_000 } else { 0 })
                .abs()
                    > 1
            })
        {
            return Err("cold rigid translation control failed".into());
        }
    }
    let grounded = measure_from_history(
        &base,
        &grounded_state,
        &[],
        [0; 23],
        "r8b-cold-contact-response.v1",
    )?;
    let raised = measure_from_history(
        &raised,
        &raised_state,
        &[],
        [0; 23],
        "r8b-cold-contact-response.v1",
    )?;
    Ok(json!({
        "schema_version": 1, "probe": "native-cold-contact-response-v1",
        "source_body_schema_hash": base.base.base.body_schema_hash.to_hex(),
        "translation_um": [0, 500_000, 0],
        "cold_response": { "grounded": grounded, "raised": raised },
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cold_schema_changes_only_identified_root_translation() {
        let original = next_motor::biomechanics_humanoid_body_schema_v6();
        let mut raised = raised_schema(original.clone()).unwrap();
        assert_ne!(raised.schema_id, original.schema_id);
        assert_ne!(
            raised.source_provenance_hash,
            original.source_provenance_hash
        );
        raised.schema_id = original.schema_id.clone();
        raised.source_provenance_hash = original.source_provenance_hash;
        for body in &mut raised.bodies {
            if body.parent_body_id.is_none() {
                body.local_bind_pose.translation_micrometres[1] -= 500_000;
            }
        }
        assert_eq!(raised, original);
    }

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
