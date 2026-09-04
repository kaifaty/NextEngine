//! Replay bounded Q1.30 tapes through the production runner, retaining safety diagnostics.
#[cfg(feature = "physx-sdk")]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    use std::io::{Read, Write};

    use next_contracts::canonical::sha256;
    use next_contracts::ids::ContentHash;
    use next_motor::{
        BIOMECHANICS_FORWARD_START_STOP_ENVIRONMENT_PROFILE_ID_V5,
        BIOMECHANICS_FORWARD_START_STOP_ENVIRONMENT_PROFILE_ID_V6,
        BiomechanicsStandingVectorRunner, VectorPolicyStepInput,
    };
    use serde_json::json;

    let args = std::env::args_os().skip(1).collect::<Vec<_>>();
    if args.len() != 2 {
        return Err(
            "usage: audit_biomechanics_action_tape <tapes.json> <fresh-report.json>".into(),
        );
    }
    let mut bytes = Vec::new();
    std::fs::File::open(&args[0])?
        .take(8_000_001)
        .read_to_end(&mut bytes)?;
    if bytes.len() > 8_000_000 {
        return Err("action tape exceeds 8 MB".into());
    }
    let input: serde_json::Value = serde_json::from_slice(&bytes)?;
    let profile = match input.get("profile_id") {
        None => BIOMECHANICS_FORWARD_START_STOP_ENVIRONMENT_PROFILE_ID_V5,
        Some(value) => value.as_str().ok_or("profile_id must be a string")?,
    };
    if !matches!(
        profile,
        BIOMECHANICS_FORWARD_START_STOP_ENVIRONMENT_PROFILE_ID_V5
            | BIOMECHANICS_FORWARD_START_STOP_ENVIRONMENT_PROFILE_ID_V6
    ) {
        return Err("only canonical V5/V6 tapes are supported".into());
    }
    let retain_frames = match input.get("retain_frames") {
        None => false,
        Some(value) => value.as_bool().ok_or("retain_frames must be boolean")?,
    };
    let tapes: Vec<Vec<Vec<i64>>> = serde_json::from_value(input["action_q1_30"].clone())?;
    if tapes.is_empty()
        || tapes.len() > 32
        || tapes.iter().any(|tape| {
            tape.is_empty()
                || tape.len() > 1200
                || tape.iter().any(|action| {
                    action.len() != 23
                        || action
                            .iter()
                            .any(|value| !(-(1_i64 << 30)..=(1_i64 << 30)).contains(value))
                })
        })
    {
        return Err("invalid bounded 23-channel Q1.30 tapes".into());
    }
    // Same shard identity as CanonicalVecEnv(run_root="32" * 32, shards=1).
    let mut seed_bytes = b"nextengine.canonical-ppo.shard.v1\0".to_vec();
    seed_bytes.extend_from_slice(&[0x32; 32]);
    seed_bytes.extend_from_slice(&0_u32.to_le_bytes());
    let run_root = if let Some(value) = input.get("run_root_bytes") {
        let root: [u8; 32] = serde_json::from_value(value.clone())?;
        ContentHash::from_bytes(root)
    } else {
        ContentHash::from_bytes(sha256(&seed_bytes))
    };
    let mut runner = BiomechanicsStandingVectorRunner::create_profile(
        profile,
        u32::try_from(tapes.len())?,
        run_root,
    )?;
    let slots = (0..u32::try_from(tapes.len())?).collect::<Vec<_>>();
    let mut ordinals = vec![0; tapes.len()];
    for reset in runner.reset_slots(&slots)? {
        ordinals[reset.vector_slot as usize] = reset.episode_ordinal;
    }
    let mut reports = vec![None; tapes.len()];
    let mut frames = vec![Vec::new(); tapes.len()];
    for tick in 0..tapes.iter().map(Vec::len).max().unwrap_or(0) {
        let inputs = tapes
            .iter()
            .enumerate()
            .map(|(slot, tape)| VectorPolicyStepInput {
                vector_slot: slot as u32,
                episode_ordinal: ordinals[slot],
                action_microradians: tape[tick.min(tape.len() - 1)].clone(),
            })
            .collect();
        let mut ended = Vec::new();
        for step in runner.step_actions_lockstep(inputs)? {
            let slot = step.vector_slot as usize;
            if retain_frames && reports[slot].is_none() {
                frames[slot].push(json!({
                    "tick": step.frame.motor_tick,
                    "command_raw": step.command_raw,
                    "contact_flags": step.frame.contact_flags,
                    "reward_components_q16": step.reward_components_raw.iter().map(|(_, value)| value).collect::<Vec<_>>(),
                    "links": step.frame.snapshot.links.iter().map(|link| json!({
                        "body_token": link.user_token,
                        "position_um": link.position_micrometres,
                        "rotation_q1_30": link.rotation_q1_30,
                        "linear_velocity_um_s": link.linear_velocity_micrometres_per_second,
                    })).collect::<Vec<_>>(),
                    "joint_position_urad": step.frame.snapshot.joints.iter().map(|joint| joint.position_microradians).collect::<Vec<_>>(),
                    "contacts": step.frame.snapshot.contacts.iter().map(|contact| json!({
                        "actor_tokens": [contact.actor_a_token, contact.actor_b_token],
                        "shape_tokens": [contact.shape_a_token, contact.shape_b_token],
                        "position_um": contact.position_micrometres,
                        "separation_um": contact.separation_micrometres,
                        "impulse_uns": contact.impulse_micronewton_seconds,
                    })).collect::<Vec<_>>(),
                    "physics_root": step.step_record.physics_root.to_hex(),
                    "step_root": step.step_record.step_root.to_hex(),
                }));
            }
            if reports[slot].is_none()
                && (step.terminated || step.truncated || tick + 1 == tapes[slot].len())
            {
                reports[slot] = Some(json!({
                    "case": slot,
                    "ticks": step.frame.motor_tick,
                    "terminal": step.terminal_reason_id.as_ref().map(|id| id.as_str()),
                    "safety_error": step.frame.joint_safety_error.map(|error| error.stable_code()),
                    "completed_physics_substeps": step.frame.completed_physics_substeps,
                    "previous_efforts_unm": step.frame.safety_checkpoint.as_ref().map(|checkpoint| &checkpoint.previous_efforts_micronewton_metres),
                    "positive_work_uj": step.frame.safety_checkpoint.as_ref().map(|checkpoint| &checkpoint.positive_work_microjoules),
                    "applied_targets_urad": step.frame.applied_targets_microradians,
                    "joint_position_urad": step.frame.snapshot.joints.iter().map(|joint| joint.position_microradians).collect::<Vec<_>>(),
                    "joint_velocity_urad_s": step.frame.snapshot.joints.iter().map(|joint| joint.velocity_microradians_per_second).collect::<Vec<_>>(),
                    "step_root": format!("{:?}", step.step_record.step_root),
                }));
            }
            if step.terminated || step.truncated {
                ended.push(step.vector_slot);
            }
        }
        if reports.iter().all(Option::is_some) {
            break;
        }
        if !ended.is_empty() {
            for reset in runner.reset_slots(&ended)? {
                ordinals[reset.vector_slot as usize] = reset.episode_ordinal;
            }
        }
    }
    let digest = |data: &[u8]| {
        sha256(data)
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect::<String>()
    };
    let mut report = json!({
        "schema": "nextengine.native-action-safety-audit.v1",
        "action_tape_sha256": digest(&bytes),
        "executable_sha256": digest(&std::fs::read(std::env::current_exe()?)?),
        "cases": reports,
        "profile_id": profile,
        "run_root": run_root.to_hex(),
    });
    if retain_frames {
        report["frames"] = json!(frames);
    }
    let mut output = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&args[1])?;
    output.write_all(&serde_json::to_vec_pretty(&report)?)?;
    println!("{}", report["cases"]);
    Ok(())
}

#[cfg(not(feature = "physx-sdk"))]
fn main() {
    panic!("audit_biomechanics_action_tape requires --features physx-sdk");
}
