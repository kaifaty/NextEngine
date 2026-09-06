//! ADR-126: explicit corrected-body canonical environment; no old identity changes.
use super::*;

pub const BIOMECHANICS_FORWARD_START_STOP_ENVIRONMENT_PROFILE_ID_V9: &str =
    "nextengine.motor.env.humanoid-biomechanics-forward-start-stop.v9";

pub(crate) fn corrected_reward_q16(
    compiled: &CompiledBodySchemaV3,
    facts: &BiomechanicsForwardStartStopRewardFactsV1<'_>,
) -> Result<([i64; 11], i64), crate::TrainingEnvironmentError> {
    biomechanics_forward_start_stop_reward_q16_v2_or_v3(compiled, facts, true, 25)
}

pub fn biomechanics_forward_start_stop_environment_manifest_v9()
-> Result<MotorTrainingEnvironmentManifestV2, MotorCompileError> {
    let body = crate::CompiledBodySchemaV4::compile(
        &crate::biomechanics_humanoid_body_schema_v11(),
        PersistentId::from_bytes([0; 16]),
    )?;
    let mut manifest = biomechanics_forward_start_stop_environment_manifest_v8()?;
    let bind = |meaning: &str, prior: ContentHash| {
        let mut bytes = b"nextengine.corrected-walking.v9\0".to_vec();
        bytes.extend_from_slice(meaning.as_bytes());
        bytes.push(0);
        bytes.extend_from_slice(prior.as_bytes());
        bytes.extend_from_slice(body.compiled_descriptor_hash.as_bytes());
        bytes.extend_from_slice(crate::bandwidth_contact_profile_hash().as_bytes());
        content_hash_from_bytes(sha256(&bytes))
    };
    manifest.environment_id = id(BIOMECHANICS_FORWARD_START_STOP_ENVIRONMENT_PROFILE_ID_V9);
    manifest.body_schema_hash = body.base.base.body_schema_hash;
    manifest.body_instance_projection_hash = bind(
        "neutral-body-instance",
        manifest.body_instance_projection_hash,
    );
    manifest.physics_catalog_hash = body.compiled_descriptor_hash;
    manifest.observation_layout_hash = bind(
        "94=quat4+root-local-v6+q25+v25+targets25+command3+anatomical-active2+clock2+whole-foot-minY2",
        manifest.observation_layout_hash,
    );
    manifest.action_layout_hash = bind(
        "25-Q30-residual;walking-reference-v1;scale-multiplier=262144;unchanged-complete-PD-safety;no-support-feedforward",
        manifest.action_layout_hash,
    );
    manifest.physics_build_profile_hash =
        bind("locked-native-build", manifest.physics_build_profile_hash);
    manifest.scene_profile_hash = bind(
        "compiled-v4-per-iteration-forces",
        manifest.scene_profile_hash,
    );
    manifest.bridge_abi_hash = bind("unchanged-bridge", manifest.bridge_abi_hash);
    manifest.quantization_profile_hash =
        bind("unchanged-quantization", manifest.quantization_profile_hash);
    manifest.translator_version_hash = bind(
        "canonical-only-no-isaac-admission",
        manifest.translator_version_hash,
    );
    manifest.command_schedule_profile_hash = bind(
        "unchanged-v8-applied-command-schedule",
        manifest.command_schedule_profile_hash,
    );
    manifest.reward_profile_hash = bind(
        "v8-formulas;25-channel-normalizations;foot=rear+mtp;sum-ground-oriented-Y-then-abs-per-substep;slip=max-active-member-link-planar-L1-per-foot;load-speed=max-member-link-planar-L1;stop=anatomical-active;min-box-Y-over-both-members;no-padding-impulses",
        manifest.reward_profile_hash,
    );
    manifest.termination_profile_hash = bind(
        "bandwidth-anatomical-contact-v4;1200-ticks;unchanged-limits;freeze-padding-without-continuity-advance",
        manifest.termination_profile_hash,
    );
    manifest.rng_derivation_profile_hash = bind(
        "unchanged-episode-seeds",
        manifest.rng_derivation_profile_hash,
    );
    manifest.correspondence_profile_hash =
        bind("canonical-v9-only", manifest.correspondence_profile_hash);
    manifest.validate_for_protocol_v2()?;
    Ok(manifest)
}

pub fn biomechanics_forward_start_stop_canonical_descriptor_json_v9()
-> Result<String, MotorCompileError> {
    let body = crate::CompiledBodySchemaV4::compile(
        &crate::biomechanics_humanoid_body_schema_v11(),
        PersistentId::from_bytes([0; 16]),
    )?;
    let manifest = biomechanics_forward_start_stop_environment_manifest_v9()?;
    let mut descriptor: Value =
        serde_json::from_str(&crate::biomechanics_body_diagnostic_descriptor_json_v11()?)
            .expect("engine body descriptor");
    let legacy: Value =
        serde_json::from_str(&biomechanics_forward_start_stop_canonical_descriptor_json_v8()?)
            .expect("engine environment descriptor");
    let mut profile = legacy["environment_profiles"][0].clone();
    for (name, hash) in [
        ("manifest_hash", manifest.manifest_hash()?),
        ("observation_layout_hash", manifest.observation_layout_hash),
        ("action_layout_hash", manifest.action_layout_hash),
        (
            "command_schedule_profile_hash",
            manifest.command_schedule_profile_hash,
        ),
        ("reward_profile_hash", manifest.reward_profile_hash),
        ("translator_version_hash", manifest.translator_version_hash),
        (
            "termination_profile_hash",
            manifest.termination_profile_hash,
        ),
        (
            "rng_derivation_profile_hash",
            manifest.rng_derivation_profile_hash,
        ),
        (
            "correspondence_profile_hash",
            manifest.correspondence_profile_hash,
        ),
    ] {
        profile[name] = json!(hash.to_hex());
    }
    profile["profile_id"] = json!(manifest.environment_id.as_str());
    profile["observation_layout_id"] = json!("nextengine.motor.observation.corrected-walking.v1");
    profile["action_layout_id"] = json!("nextengine.motor.action.corrected-walking-residual.v1");
    profile["observation"]["channel_count"] = json!(94);
    profile["observation"]["contacts"] =
        json!(["anatomical.left-active", "anatomical.right-active"]);
    profile["observation"]["appended_clock"]["offset"] = json!(90);
    profile["observation"]["appended_sole_heights"]["offset"] = json!(92);
    profile["observation"]["appended_sole_heights"]["measurement"] =
        json!("minimum-native-box-world-Y-over-rearfoot-and-MTP");
    profile["action"]["channel_count"] = json!(25);
    profile["periodic_load_credit"]["impulse_measurement"] =
        json!("sum-over-actual-substeps-of-abs-ground-oriented-anatomical-foot-Y-sum");
    profile["periodic_load_credit"]["stance_speed"] =
        json!("target-load-weighted-maximum-member-link-planar-L1-speed");
    profile["foot_aggregation"] = json!({
        "members": [["body.left-ankle-roll", "body.left-mtp"], ["body.right-ankle-roll", "body.right-mtp"]],
        "flags": "classifier anatomical active continuity, not a load measurement",
        "load": "absolute ground-oriented summed vertical impulse per anatomical foot per actual substep",
        "periodic_speed": "maximum member link planar L1 speed",
        "slip": "maximum active member link planar L1 speed per contacting anatomical foot",
        "height": "minimum collider Y over both members",
        "impact": "unchanged anatomical 6Ns and per-pair limits",
    });
    let (_, effort, target_rate) = standing_normalizations(&body.base);
    profile["reward_normalizations"]["applied_effort_per_motor_tick_micronewton_metres"] =
        json!(effort);
    profile["reward_normalizations"]["applied_target_rate_microradians_per_motor_tick"] =
        json!(target_rate);
    descriptor["environment_profiles"] = json!([profile]);
    descriptor["training_descriptor_id"] =
        json!("nextengine.canonical.humanoid-biomechanics-forward-start-stop.v9");
    descriptor["translator_id"] = json!("nextengine.canonical-corrected-walking.v1");
    descriptor["backend_admission"] =
        json!("canonical-cpu-bounded-pipeline-smoke;no-Isaac-or-runtime");
    descriptor["legacy_compiled_descriptor_hash"] = descriptor["compiled_descriptor_hash"].clone();
    descriptor["compiled_descriptor_hash"] = json!(body.compiled_descriptor_hash.to_hex());
    descriptor["force_schedule_profile_id"] =
        json!(crate::BIOMECHANICS_FORCE_SCHEDULE_PROFILE_ID_V1);
    descriptor["safety_contact_profile_hash"] =
        json!(crate::bandwidth_contact_profile_hash().to_hex());
    descriptor["action_width"] = json!(25);
    descriptor["observation_width"] = json!(94);
    let mut output = serde_json::to_string_pretty(&descriptor).expect("engine descriptor");
    output.push('\n');
    Ok(output)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn corrected_descriptor_closes_new_body_layout_and_all_environment_hashes() {
        let old = biomechanics_forward_start_stop_environment_manifest_v8().unwrap();
        let manifest = biomechanics_forward_start_stop_environment_manifest_v9().unwrap();
        let value: Value = serde_json::from_str(
            &biomechanics_forward_start_stop_canonical_descriptor_json_v9().unwrap(),
        )
        .unwrap();
        let body = crate::CompiledBodySchemaV4::compile(
            &crate::biomechanics_humanoid_body_schema_v11(),
            PersistentId::from_bytes([0; 16]),
        )
        .unwrap();
        assert_eq!(manifest.physics_catalog_hash, body.compiled_descriptor_hash);
        assert_ne!(manifest.body_schema_hash, old.body_schema_hash);
        assert_ne!(manifest.action_layout_hash, old.action_layout_hash);
        assert_ne!(
            manifest.observation_layout_hash,
            old.observation_layout_hash
        );
        assert_ne!(manifest.reward_profile_hash, old.reward_profile_hash);
        assert_ne!(
            manifest.termination_profile_hash,
            old.termination_profile_hash
        );
        assert_eq!(manifest.reward_components, old.reward_components);
        assert_eq!(
            value["compiled_descriptor_hash"],
            body.compiled_descriptor_hash.to_hex()
        );
        assert_eq!(value["action_width"], 25);
        assert_eq!(value["observation_width"], 94);
        assert_eq!(value["actuators"].as_array().unwrap().len(), 25);
        assert_eq!(value["environment_profiles"].as_array().unwrap().len(), 1);
        let profile = &value["environment_profiles"][0];
        assert_eq!(
            profile["manifest_hash"],
            manifest.manifest_hash().unwrap().to_hex()
        );
        assert_eq!(profile["observation"]["appended_clock"]["offset"], 90);
        assert_eq!(
            profile["observation"]["appended_sole_heights"]["offset"],
            92
        );
        assert_eq!(
            profile["walking_reference"]["profile_id"],
            crate::PROCEDURAL_WALKING_REFERENCE_PROFILE_ID_V1
        );
        let (_, effort, rate) = standing_normalizations(&body.base);
        assert_eq!(
            profile["reward_normalizations"]["applied_effort_per_motor_tick_micronewton_metres"],
            effort
        );
        assert_eq!(
            profile["reward_normalizations"]["applied_target_rate_microradians_per_motor_tick"],
            rate
        );
    }
}
