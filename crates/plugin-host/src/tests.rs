use super::{
    REFERENCE_COMPONENT_WAT, WIT_WORLD_V2_ID, WasmDiagnosticCodeV1, WasmHostError,
    WasmPluginRuntimeV1, WasmPluginStartupV1, WasmResourceHandleRegistryV1,
    reference_wasm_manifest_v1,
};
use next_contracts::canonical::{CanonicalDecodeLimits, sha256};
use next_contracts::extension::{
    WasmPluginManifestV1, WasmPluginStateV1, WasmResourceLimitsV1, WitResourceHandleV1,
};
use next_contracts::ids::{CapabilityId, ContentHash, PluginId, SchemaId, content_hash_from_bytes};

const LOOP_COMPONENT_WAT: &str = r#"
(component
  (core module $module
(func (export "run") (param i32) (result i32)
  (loop $forever
    br $forever)
  i32.const 0))
  (core instance $instance (instantiate $module))
  (func (export "run") (param "input" s32) (result s32)
(canon lift (core func $instance "run"))))
"#;

const MEMORY_COMPONENT_WAT: &str = r#"
(component
  (core module $module
(memory 1)
(func (export "run") (param i32) (result i32)
  i32.const 1
  memory.grow))
  (core instance $instance (instantiate $module))
  (func (export "run") (param "input" s32) (result s32)
(canon lift (core func $instance "run"))))
"#;

const AMBIENT_IMPORT_COMPONENT_WAT: &str = r#"
(component
  (import "ambient-time" (func $time (result u64)))
  (core func $time-core (canon lower (func $time)))
  (core module $module
(import "" "time" (func $time (result i64)))
(func (export "run") (param i32) (result i32)
  call $time
  drop
  local.get 0))
  (core instance $instance
(instantiate $module
  (with "" (instance (export "time" (func $time-core))))))
  (func (export "run") (param "input" s32) (result s32)
(canon lift (core func $instance "run"))))
"#;

#[test]
fn component_executes_with_fuel_and_state_round_trip() {
    let manifest = reference_wasm_manifest_v1(true).expect("manifest");
    let mut runtime = active(
        manifest.clone(),
        REFERENCE_COMPONENT_WAT,
        manifest.requested_capabilities.clone(),
    );
    let outcome = runtime.execute_i32(7, 41).expect("execute");
    assert_eq!(outcome.value, 42);
    assert!(outcome.fuel_consumed > 0);
    assert_eq!(outcome.selected_host_api_major, 3);
    assert!(!outcome.used_compatibility_adapter);

    let bytes = runtime.state().canonical_bytes().expect("state bytes");
    let state = WasmPluginStateV1::from_canonical_bytes(&bytes, CanonicalDecodeLimits::default())
        .expect("decode");
    let mut restored = WasmPluginRuntimeV1::restore(
        manifest.clone(),
        REFERENCE_COMPONENT_WAT.as_bytes().to_vec(),
        manifest.requested_capabilities.clone(),
        state,
    )
    .expect("restore");
    assert_eq!(restored.execute_i32(8, 9).expect("retry").value, 10);

    let mut mismatched = manifest.clone();
    mismatched.state_schema_version = 2;
    assert!(matches!(
        WasmPluginRuntimeV1::restore(
            mismatched,
            REFERENCE_COMPONENT_WAT.as_bytes().to_vec(),
            manifest.requested_capabilities,
            runtime.state().clone(),
        ),
        Err(WasmHostError::Contract(_)) | Err(WasmHostError::StateBindingMismatch)
    ));
}

#[test]
fn n_n_minus_one_and_n_minus_two_are_exact() {
    let current = reference_wasm_manifest_v1(true).expect("current");
    let current_runtime = active(
        current.clone(),
        REFERENCE_COMPONENT_WAT,
        current.requested_capabilities.clone(),
    );
    assert_eq!(current_runtime.selection().host_api_major, 3);

    let previous = manifest_for(
        REFERENCE_COMPONENT_WAT,
        2,
        WIT_WORLD_V2_ID,
        true,
        100_000,
        1_048_576,
    );
    let previous_runtime = active(
        previous.clone(),
        REFERENCE_COMPONENT_WAT,
        previous.requested_capabilities.clone(),
    );
    assert_eq!(previous_runtime.selection().host_api_major, 2);
    assert!(previous_runtime.selection().uses_compatibility_adapter);

    let n_minus_two = manifest_for(
        REFERENCE_COMPONENT_WAT,
        1,
        "nextengine.extension.plugin-v1",
        true,
        100_000,
        1_048_576,
    );
    assert!(matches!(
        WasmPluginRuntimeV1::activate(
            n_minus_two.clone(),
            Some(REFERENCE_COMPONENT_WAT.as_bytes().to_vec()),
            n_minus_two.requested_capabilities,
        ),
        Err(WasmHostError::Contract(
            next_contracts::extension::ExtensionContractError::IncompatibleInterface
        ))
    ));
}

#[test]
fn fuel_memory_capability_and_ambient_imports_fail_closed() {
    let fuel_manifest = manifest_for(
        LOOP_COMPONENT_WAT,
        3,
        super::WIT_WORLD_V3_ID,
        true,
        10,
        65_536,
    );
    let mut fuel_runtime = active(
        fuel_manifest.clone(),
        LOOP_COMPONENT_WAT,
        fuel_manifest.requested_capabilities.clone(),
    );
    assert!(matches!(
        fuel_runtime.execute_i32(0, 0),
        Err(WasmHostError::Execution {
            code: WasmDiagnosticCodeV1::FuelExhausted,
            ..
        })
    ));
    assert!(fuel_runtime.state().state_bytes.is_empty());
    assert!(fuel_runtime.execute_i32(1, 0).is_err());
    assert!(fuel_runtime.execute_i32(2, 0).is_err());
    assert!(matches!(
        fuel_runtime.execute_i32(3, 0),
        Err(WasmHostError::CircuitOpen)
    ));

    let memory_manifest = manifest_for(
        MEMORY_COMPONENT_WAT,
        3,
        super::WIT_WORLD_V3_ID,
        true,
        100_000,
        65_536,
    );
    let mut memory_runtime = active(
        memory_manifest.clone(),
        MEMORY_COMPONENT_WAT,
        memory_manifest.requested_capabilities.clone(),
    );
    let memory_error = memory_runtime
        .execute_i32(0, 0)
        .expect_err("growth beyond the declared memory limit must fail");
    assert!(
        matches!(
            memory_error,
            WasmHostError::Execution {
                code: WasmDiagnosticCodeV1::MemoryLimitExceeded,
                ..
            }
        ),
        "unexpected memory failure: {memory_error:?}"
    );

    let manifest = reference_wasm_manifest_v1(true).expect("manifest");
    assert!(matches!(
        WasmPluginRuntimeV1::activate(
            manifest.clone(),
            Some(REFERENCE_COMPONENT_WAT.as_bytes().to_vec()),
            Vec::new(),
        ),
        Err(WasmHostError::CapabilityDenied)
    ));

    let ambient = manifest_for(
        AMBIENT_IMPORT_COMPONENT_WAT,
        3,
        super::WIT_WORLD_V3_ID,
        true,
        100_000,
        65_536,
    );
    assert!(matches!(
        WasmPluginRuntimeV1::activate(
            ambient.clone(),
            Some(AMBIENT_IMPORT_COMPONENT_WAT.as_bytes().to_vec()),
            ambient.requested_capabilities,
        ),
        Err(WasmHostError::ComponentRejected(_))
    ));
}

#[test]
fn forged_handle_and_required_optional_startup_are_isolated() {
    let plugin_id = PluginId::new("org.nextengine.test.handles").expect("plugin");
    let mut registry =
        WasmResourceHandleRegistryV1::new(plugin_id.clone(), ContentHash::from_bytes([7; 32]));
    let handle = registry
        .mint(SchemaId::new("nextengine.query.character").expect("kind"))
        .expect("mint");
    registry.validate(&handle).expect("valid");
    let forged = WitResourceHandleV1 {
        slot: handle.slot + 1,
        ..handle
    };
    assert!(matches!(
        registry.validate(&forged),
        Err(WasmHostError::InvalidResourceHandle)
    ));

    let optional = reference_wasm_manifest_v1(false).expect("optional");
    assert!(matches!(
        WasmPluginRuntimeV1::activate(optional.clone(), None, optional.requested_capabilities,),
        Ok(WasmPluginStartupV1::OptionalDisabled {
            diagnostic: WasmDiagnosticCodeV1::OptionalPluginUnavailable,
            ..
        })
    ));
    let required = reference_wasm_manifest_v1(true).expect("required");
    assert!(matches!(
        WasmPluginRuntimeV1::activate(required.clone(), None, required.requested_capabilities,),
        Err(WasmHostError::RequiredPluginUnavailable)
    ));
}

fn active(
    manifest: WasmPluginManifestV1,
    component: &str,
    grants: Vec<CapabilityId>,
) -> WasmPluginRuntimeV1 {
    match WasmPluginRuntimeV1::activate(manifest, Some(component.as_bytes().to_vec()), grants)
        .expect("activation")
    {
        WasmPluginStartupV1::Active(runtime) => *runtime,
        WasmPluginStartupV1::OptionalDisabled { .. } => panic!("unexpected optional disable"),
    }
}

fn manifest_for(
    component: &str,
    api_major: u16,
    wit_world: &str,
    required: bool,
    fuel: u64,
    memory: u64,
) -> WasmPluginManifestV1 {
    WasmPluginManifestV1::new(
        PluginId::new("org.nextengine.test.wasm").expect("plugin"),
        1,
        content_hash_from_bytes(sha256(component.as_bytes())),
        SchemaId::new(wit_world).expect("world"),
        api_major,
        api_major,
        vec![CapabilityId::new("mechanics.effect.propose").expect("capability")],
        SchemaId::new("org.nextengine.test.wasm.state").expect("state"),
        1,
        required,
        true,
        "engine://test/wasm",
        "Apache-2.0 test fixture",
        WasmResourceLimitsV1 {
            linear_memory_limit_bytes: memory,
            table_limit: 1,
            instance_limit: 1,
            fuel_per_call: fuel,
            fuel_per_gameplay_tick: fuel.saturating_mul(2),
        },
    )
    .expect("manifest")
}
