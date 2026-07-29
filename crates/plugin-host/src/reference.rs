use super::*;

pub const WASMTIME_ADAPTER_VERSION: &str = "wasmtime-45.0.0";
pub const WASM_COMPONENT_MODEL_FEATURE_SET: &str =
    "component-model+cranelift+runtime+std+wat;no-wasi;no-async;no-threads";
pub const WASM_AMBIENT_WASI_ENABLED: bool = false;
pub const WIT_WORLD_V2_ID: &str = "nextengine.extension.plugin-v2";
pub const WIT_WORLD_V3_ID: &str = "nextengine.extension.plugin-v3";
pub const REFERENCE_COMPONENT_WAT: &str = r#"
(component
  (core module $module
    (func (export "run") (param i32) (result i32)
      local.get 0
      i32.const 1
      i32.add))
  (core instance $instance (instantiate $module))
  (func (export "run") (param "input" s32) (result s32)
    (canon lift (core func $instance "run"))))
"#;
pub const WIT_V2: &str = include_str!("../wit/v2/nextengine-extension.wit");
pub const WIT_V3: &str = include_str!("../wit/v3/nextengine-extension.wit");
const MECHANICS_PROPOSE_CAPABILITY: &str = "mechanics.effect.propose";

pub fn reference_wasm_manifest_v1(
    required_at_startup: bool,
) -> Result<WasmPluginManifestV1, WasmHostError> {
    Ok(WasmPluginManifestV1::new(
        PluginId::new("org.nextengine.reference.wasm-increment")
            .expect("engine-owned plugin ID is canonical"),
        1,
        content_hash_from_bytes(sha256(REFERENCE_COMPONENT_WAT.as_bytes())),
        SchemaId::new(WIT_WORLD_V3_ID).expect("engine-owned WIT world ID is canonical"),
        next_contracts::extension::WASM_HOST_CURRENT_API_MAJOR,
        next_contracts::extension::WASM_HOST_CURRENT_API_MAJOR,
        vec![
            CapabilityId::new(MECHANICS_PROPOSE_CAPABILITY)
                .expect("engine-owned capability is canonical"),
        ],
        SchemaId::new("org.nextengine.reference.wasm-increment.state")
            .expect("engine-owned state schema is canonical"),
        1,
        required_at_startup,
        true,
        "engine://reference/wasm-increment",
        "Apache-2.0 engine-owned reference fixture",
        WasmResourceLimitsV1 {
            linear_memory_limit_bytes: 1_048_576,
            table_limit: 1,
            instance_limit: 1,
            fuel_per_call: 100_000,
            fuel_per_gameplay_tick: 200_000,
        },
    )?)
}

pub fn compile_reference_wasm_action_v1(
    outcome: &WasmCallbackOutcomeV1,
    registry: &RpgDefinitionRegistryV1,
    snapshot: &RpgSnapshotV2,
    gameplay_tick: u64,
    source_character_id: next_contracts::ids::PersistentId,
    physical_contact_facts: Vec<RpgPhysicalContactFactV1>,
) -> Result<CompiledAbilityEffectV1, WasmHostError> {
    if outcome.value != 1 {
        return Err(WasmHostError::ProposalInvalid);
    }
    Ok(compile_contact_ability_v1(
        registry,
        snapshot,
        AbilityInvocationV1 {
            gameplay_tick,
            source_character_id,
            semantic_action_id: SchemaId::new(next_contracts::input::CORE_MELEE_ACTION_ID)
                .expect("engine-owned melee action ID is canonical"),
            prior_cooldown_commit_tick: None,
            physical_contact_facts,
        },
    )?)
}
