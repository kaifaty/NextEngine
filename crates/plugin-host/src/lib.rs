#![forbid(unsafe_code)]

use std::collections::BTreeMap;
use std::error::Error;
use std::fmt::{Display, Formatter};

use next_contracts::{
    CapabilityId, ContentHash, ExtensionContractError, PluginId, RpgDefinitionRegistryV1,
    RpgPhysicalContactFactV1, RpgSnapshotV2, SchemaId, WasmPluginManifestV1, WasmPluginStateV1,
    WasmResourceLimitsV1, WitInterfaceSelectionV1, WitResourceHandleV1, content_hash_from_bytes,
    negotiate_wit_interface_v1, sha256,
};
use next_mechanics::{
    AbilityInvocationV1, CompiledAbilityEffectV1, MechanicsHostError, compile_contact_ability_v1,
};
use wasmtime::component::{Component, Linker};
use wasmtime::{Config, Engine, Store, StoreLimits, StoreLimitsBuilder, Trap};

#[allow(dead_code)]
mod wit_v2 {
    wasmtime::component::bindgen!({
        path: "wit/v2",
        world: "plugin-v2",
    });
}

#[allow(dead_code)]
mod wit_v3 {
    wasmtime::component::bindgen!({
        path: "wit/v3",
        world: "plugin-v3",
    });
}

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
        next_contracts::WASM_HOST_CURRENT_API_MAJOR,
        next_contracts::WASM_HOST_CURRENT_API_MAJOR,
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

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u8)]
pub enum WasmDiagnosticCodeV1 {
    OptionalPluginUnavailable = 1,
    ComponentHashMismatch = 2,
    CapabilityDenied = 3,
    ComponentRejected = 4,
    FuelExhausted = 5,
    MemoryLimitExceeded = 6,
    Trap = 7,
    CircuitOpen = 8,
    StateBindingMismatch = 9,
    InvalidResourceHandle = 10,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WasmCallbackOutcomeV1 {
    pub value: i32,
    pub fuel_consumed: u64,
    pub selected_host_api_major: u16,
    pub used_compatibility_adapter: bool,
    pub plugin_state_hash: ContentHash,
}

pub fn compile_reference_wasm_action_v1(
    outcome: &WasmCallbackOutcomeV1,
    registry: &RpgDefinitionRegistryV1,
    snapshot: &RpgSnapshotV2,
    gameplay_tick: u64,
    source_character_id: next_contracts::PersistentId,
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
            semantic_action_id: SchemaId::new(next_contracts::CORE_MELEE_ACTION_ID)
                .expect("engine-owned melee action ID is canonical"),
            prior_cooldown_commit_tick: None,
            physical_contact_facts,
        },
    )?)
}

pub enum WasmPluginStartupV1 {
    Active(Box<WasmPluginRuntimeV1>),
    OptionalDisabled {
        plugin_id: PluginId,
        diagnostic: WasmDiagnosticCodeV1,
    },
}

struct StoreData {
    limits: StoreLimits,
}

pub struct WasmPluginRuntimeV1 {
    manifest: WasmPluginManifestV1,
    selection: WitInterfaceSelectionV1,
    engine: Engine,
    component: Component,
    state: WasmPluginStateV1,
    fuel_tick: Option<u64>,
    fuel_consumed_at_tick: u64,
}

impl WasmPluginRuntimeV1 {
    pub fn activate(
        manifest: WasmPluginManifestV1,
        component_bytes: Option<Vec<u8>>,
        granted_capabilities: Vec<CapabilityId>,
    ) -> Result<WasmPluginStartupV1, WasmHostError> {
        manifest.validate()?;
        let selection = negotiate_wit_interface_v1(&manifest)?;
        validate_wit_world(&manifest, selection)?;
        if !manifest
            .requested_capabilities
            .iter()
            .all(|capability| granted_capabilities.contains(capability))
        {
            return startup_failure(
                &manifest,
                WasmDiagnosticCodeV1::CapabilityDenied,
                WasmHostError::CapabilityDenied,
            );
        }
        let Some(component_bytes) = component_bytes else {
            return startup_failure(
                &manifest,
                WasmDiagnosticCodeV1::OptionalPluginUnavailable,
                WasmHostError::RequiredPluginUnavailable,
            );
        };
        if content_hash_from_bytes(sha256(&component_bytes)) != manifest.component_hash {
            return startup_failure(
                &manifest,
                WasmDiagnosticCodeV1::ComponentHashMismatch,
                WasmHostError::ComponentHashMismatch,
            );
        }
        let engine = component_engine()?;
        let component = match Component::new(&engine, &component_bytes) {
            Ok(component) => component,
            Err(error) => {
                return startup_failure(
                    &manifest,
                    WasmDiagnosticCodeV1::ComponentRejected,
                    WasmHostError::ComponentRejected(stable_wasmtime_error(&error)),
                );
            }
        };
        if let Err(error) = instantiate_component(&engine, &component, &manifest) {
            return startup_failure(
                &manifest,
                WasmDiagnosticCodeV1::ComponentRejected,
                WasmHostError::ComponentRejected(stable_wasmtime_error(&error)),
            );
        }
        let state = WasmPluginStateV1::new(&manifest, Vec::new())?;
        Ok(WasmPluginStartupV1::Active(Box::new(Self {
            manifest,
            selection,
            engine,
            component,
            state,
            fuel_tick: None,
            fuel_consumed_at_tick: 0,
        })))
    }

    pub fn restore(
        manifest: WasmPluginManifestV1,
        component_bytes: Vec<u8>,
        granted_capabilities: Vec<CapabilityId>,
        state: WasmPluginStateV1,
    ) -> Result<Self, WasmHostError> {
        state.validate()?;
        let startup = Self::activate(
            manifest.clone(),
            Some(component_bytes),
            granted_capabilities,
        )?;
        let WasmPluginStartupV1::Active(mut runtime) = startup else {
            return Err(WasmHostError::StateBindingMismatch);
        };
        if state.plugin_id != manifest.plugin_id
            || state.plugin_manifest_hash != manifest.manifest_hash
            || state.state_schema_id != manifest.state_schema_id
            || state.state_schema_version != manifest.state_schema_version
        {
            return Err(WasmHostError::StateBindingMismatch);
        }
        runtime.state = state;
        Ok(*runtime)
    }

    #[must_use]
    pub fn manifest(&self) -> &WasmPluginManifestV1 {
        &self.manifest
    }

    #[must_use]
    pub fn state(&self) -> &WasmPluginStateV1 {
        &self.state
    }

    #[must_use]
    pub const fn selection(&self) -> WitInterfaceSelectionV1 {
        self.selection
    }

    pub fn execute_i32(
        &mut self,
        gameplay_tick: u64,
        input: i32,
    ) -> Result<WasmCallbackOutcomeV1, WasmHostError> {
        if self.state.circuit_opened_at_tick.is_some() {
            return Err(WasmHostError::CircuitOpen);
        }
        if self.fuel_tick != Some(gameplay_tick) {
            self.fuel_tick = Some(gameplay_tick);
            self.fuel_consumed_at_tick = 0;
        }
        let tick_remaining = self
            .manifest
            .limits
            .fuel_per_gameplay_tick
            .saturating_sub(self.fuel_consumed_at_tick);
        let callback_fuel = self.manifest.limits.fuel_per_call.min(tick_remaining);
        if callback_fuel == 0 {
            self.record_violation(gameplay_tick)?;
            return Err(WasmHostError::Execution {
                code: WasmDiagnosticCodeV1::FuelExhausted,
                detail: "WASM_FUEL_PER_TICK_EXHAUSTED".to_owned(),
            });
        }

        let mut store = bounded_store(&self.engine, &self.manifest)?;
        store
            .set_fuel(callback_fuel)
            .map_err(|error| WasmHostError::HostConfiguration(stable_wasmtime_error(&error)))?;
        let linker = Linker::new(&self.engine);
        let instance = linker
            .instantiate(&mut store, &self.component)
            .map_err(|error| WasmHostError::ComponentRejected(stable_wasmtime_error(&error)))?;
        let run = instance
            .get_typed_func::<(i32,), (i32,)>(&mut store, "run")
            .map_err(|error| WasmHostError::ExportMismatch(stable_wasmtime_error(&error)))?;
        let call_result = run.call(&mut store, (input,));
        let remaining = store
            .get_fuel()
            .map_err(|error| WasmHostError::HostConfiguration(stable_wasmtime_error(&error)))?;
        let consumed = callback_fuel.saturating_sub(remaining);
        self.fuel_consumed_at_tick = self.fuel_consumed_at_tick.saturating_add(consumed);
        match call_result {
            Ok((value,)) => {
                self.state
                    .replace_state_bytes(value.to_le_bytes().to_vec())?;
                Ok(WasmCallbackOutcomeV1 {
                    value,
                    fuel_consumed: consumed,
                    selected_host_api_major: self.selection.host_api_major,
                    used_compatibility_adapter: self.selection.uses_compatibility_adapter,
                    plugin_state_hash: self.state.state_hash,
                })
            }
            Err(error) => {
                let code = classify_execution_error(&error);
                self.record_violation(gameplay_tick)?;
                Err(WasmHostError::Execution {
                    code,
                    detail: stable_wasmtime_error(&error),
                })
            }
        }
    }

    fn record_violation(&mut self, gameplay_tick: u64) -> Result<(), WasmHostError> {
        self.state.record_violation(gameplay_tick)?;
        Ok(())
    }
}

pub struct WasmResourceHandleRegistryV1 {
    plugin_id: PluginId,
    binding_secret: ContentHash,
    next_slot: u32,
    records: BTreeMap<u32, (u32, SchemaId, ContentHash)>,
}

impl WasmResourceHandleRegistryV1 {
    pub fn new(plugin_id: PluginId, binding_secret: ContentHash) -> Self {
        Self {
            plugin_id,
            binding_secret,
            next_slot: 0,
            records: BTreeMap::new(),
        }
    }

    pub fn mint(&mut self, resource_kind: SchemaId) -> Result<WitResourceHandleV1, WasmHostError> {
        let slot = self.next_slot;
        self.next_slot = self
            .next_slot
            .checked_add(1)
            .ok_or(WasmHostError::ResourceHandleExhausted)?;
        let generation = 1;
        let binding_hash = resource_binding_hash(
            &self.binding_secret,
            &self.plugin_id,
            &resource_kind,
            slot,
            generation,
        );
        self.records
            .insert(slot, (generation, resource_kind.clone(), binding_hash));
        Ok(WitResourceHandleV1 {
            plugin_id: self.plugin_id.clone(),
            resource_kind,
            slot,
            generation,
            binding_hash,
        })
    }

    pub fn validate(&self, handle: &WitResourceHandleV1) -> Result<(), WasmHostError> {
        let expected = self.records.get(&handle.slot);
        if handle.plugin_id != self.plugin_id
            || expected
                != Some(&(
                    handle.generation,
                    handle.resource_kind.clone(),
                    handle.binding_hash,
                ))
            || resource_binding_hash(
                &self.binding_secret,
                &handle.plugin_id,
                &handle.resource_kind,
                handle.slot,
                handle.generation,
            ) != handle.binding_hash
        {
            return Err(WasmHostError::InvalidResourceHandle);
        }
        Ok(())
    }
}

fn component_engine() -> Result<Engine, WasmHostError> {
    let mut config = Config::new();
    config
        .consume_fuel(true)
        .wasm_component_model(true)
        .wasm_relaxed_simd(false)
        .wasm_simd(false)
        .wasm_multi_memory(false)
        .wasm_memory64(false)
        .debug_info(false);
    Engine::new(&config)
        .map_err(|error| WasmHostError::HostConfiguration(stable_wasmtime_error(&error)))
}

fn bounded_store(
    engine: &Engine,
    manifest: &WasmPluginManifestV1,
) -> Result<Store<StoreData>, WasmHostError> {
    let memory_limit = usize::try_from(manifest.limits.linear_memory_limit_bytes)
        .map_err(|_| WasmHostError::HostConfiguration("WASM_MEMORY_LIMIT_RANGE".to_owned()))?;
    let limits = StoreLimitsBuilder::new()
        .memory_size(memory_limit)
        .instances(
            usize::try_from(manifest.limits.instance_limit).map_err(|_| {
                WasmHostError::HostConfiguration("WASM_INSTANCE_LIMIT_RANGE".into())
            })?,
        )
        .tables(
            usize::try_from(manifest.limits.table_limit)
                .map_err(|_| WasmHostError::HostConfiguration("WASM_TABLE_LIMIT_RANGE".into()))?,
        )
        .memories(1)
        .trap_on_grow_failure(true)
        .build();
    let mut store = Store::new(engine, StoreData { limits });
    store.limiter(|state| &mut state.limits);
    Ok(store)
}

fn instantiate_component(
    engine: &Engine,
    component: &Component,
    manifest: &WasmPluginManifestV1,
) -> Result<(), wasmtime::Error> {
    let mut store = bounded_store(engine, manifest).map_err(|error| {
        wasmtime::Error::msg(format!("NEXTENGINE_STORE_CONFIGURATION: {error}"))
    })?;
    store.set_fuel(manifest.limits.fuel_per_call)?;
    Linker::new(engine).instantiate(&mut store, component)?;
    Ok(())
}

fn validate_wit_world(
    manifest: &WasmPluginManifestV1,
    selection: WitInterfaceSelectionV1,
) -> Result<(), WasmHostError> {
    let expected = if selection.uses_compatibility_adapter {
        WIT_WORLD_V2_ID
    } else {
        WIT_WORLD_V3_ID
    };
    if manifest.wit_world_id.as_str() != expected {
        return Err(WasmHostError::WitWorldMismatch);
    }
    Ok(())
}

fn startup_failure(
    manifest: &WasmPluginManifestV1,
    diagnostic: WasmDiagnosticCodeV1,
    required_error: WasmHostError,
) -> Result<WasmPluginStartupV1, WasmHostError> {
    if manifest.required_at_startup {
        Err(required_error)
    } else {
        Ok(WasmPluginStartupV1::OptionalDisabled {
            plugin_id: manifest.plugin_id.clone(),
            diagnostic,
        })
    }
}

fn classify_execution_error(error: &wasmtime::Error) -> WasmDiagnosticCodeV1 {
    if matches!(error.downcast_ref::<Trap>(), Some(Trap::OutOfFuel)) {
        return WasmDiagnosticCodeV1::FuelExhausted;
    }
    if matches!(error.downcast_ref::<Trap>(), Some(Trap::MemoryOutOfBounds)) {
        return WasmDiagnosticCodeV1::MemoryLimitExceeded;
    }
    let message = wasmtime_error_chain(error);
    if message.contains("memory") || message.contains("grow") {
        WasmDiagnosticCodeV1::MemoryLimitExceeded
    } else {
        WasmDiagnosticCodeV1::Trap
    }
}

fn stable_wasmtime_error(error: &wasmtime::Error) -> String {
    if let Some(trap) = error.downcast_ref::<Trap>() {
        return format!("WASM_TRAP_{trap:?}");
    }
    let message = wasmtime_error_chain(error);
    if message.contains("resource limit") || message.contains("memory") {
        "WASM_RESOURCE_LIMIT".to_owned()
    } else if message.contains("unknown import") || message.contains("not defined") {
        "WASM_IMPORT_DENIED".to_owned()
    } else if message.contains("type mismatch") || message.contains("failed to parse") {
        "WASM_COMPONENT_INVALID".to_owned()
    } else {
        "WASM_COMPONENT_ERROR".to_owned()
    }
}

fn wasmtime_error_chain(error: &wasmtime::Error) -> String {
    format!("{error:#}").to_lowercase()
}

fn resource_binding_hash(
    secret: &ContentHash,
    plugin_id: &PluginId,
    resource_kind: &SchemaId,
    slot: u32,
    generation: u32,
) -> ContentHash {
    let mut bytes = b"nextengine.wit-resource-handle.v1\0".to_vec();
    bytes.extend_from_slice(secret.as_bytes());
    extend_text(&mut bytes, plugin_id.as_str());
    extend_text(&mut bytes, resource_kind.as_str());
    bytes.extend_from_slice(&slot.to_le_bytes());
    bytes.extend_from_slice(&generation.to_le_bytes());
    content_hash_from_bytes(sha256(&bytes))
}

fn extend_text(bytes: &mut Vec<u8>, value: &str) {
    let length = u32::try_from(value.len())
        .expect("engine-owned identifier construction enforces canonical u32 length");
    bytes.extend_from_slice(&length.to_le_bytes());
    bytes.extend_from_slice(value.as_bytes());
}

#[derive(Debug)]
#[non_exhaustive]
pub enum WasmHostError {
    Contract(ExtensionContractError),
    RequiredPluginUnavailable,
    ComponentHashMismatch,
    CapabilityDenied,
    WitWorldMismatch,
    ComponentRejected(String),
    ExportMismatch(String),
    HostConfiguration(String),
    Execution {
        code: WasmDiagnosticCodeV1,
        detail: String,
    },
    CircuitOpen,
    StateBindingMismatch,
    InvalidResourceHandle,
    ResourceHandleExhausted,
    ProposalInvalid,
    Mechanics(MechanicsHostError),
}

impl Display for WasmHostError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Contract(error) => write!(formatter, "Wasm contract failed: {error}"),
            Self::RequiredPluginUnavailable => {
                formatter.write_str("required Wasm plugin is unavailable")
            }
            Self::ComponentHashMismatch => formatter.write_str("Wasm component hash mismatch"),
            Self::CapabilityDenied => formatter.write_str("Wasm plugin capability denied"),
            Self::WitWorldMismatch => formatter.write_str("Wasm plugin WIT world mismatch"),
            Self::ComponentRejected(detail) => {
                write!(formatter, "Wasm component rejected: {detail}")
            }
            Self::ExportMismatch(detail) => write!(formatter, "Wasm export mismatch: {detail}"),
            Self::HostConfiguration(detail) => {
                write!(formatter, "Wasm host configuration failed: {detail}")
            }
            Self::Execution { code, detail } => {
                write!(formatter, "Wasm execution failed ({code:?}): {detail}")
            }
            Self::CircuitOpen => formatter.write_str("Wasm plugin circuit is open"),
            Self::StateBindingMismatch => formatter.write_str("Wasm plugin state binding mismatch"),
            Self::InvalidResourceHandle => {
                formatter.write_str("Wasm resource handle is invalid or forged")
            }
            Self::ResourceHandleExhausted => {
                formatter.write_str("Wasm resource handle registry exhausted")
            }
            Self::ProposalInvalid => formatter.write_str("Wasm proposal is invalid"),
            Self::Mechanics(error) => write!(formatter, "Wasm mechanics proposal failed: {error}"),
        }
    }
}

impl Error for WasmHostError {}

impl From<ExtensionContractError> for WasmHostError {
    fn from(error: ExtensionContractError) -> Self {
        Self::Contract(error)
    }
}

impl From<MechanicsHostError> for WasmHostError {
    fn from(error: MechanicsHostError) -> Self {
        Self::Mechanics(error)
    }
}

#[cfg(test)]
mod tests {
    use super::{
        REFERENCE_COMPONENT_WAT, WIT_WORLD_V2_ID, WasmDiagnosticCodeV1, WasmHostError,
        WasmPluginRuntimeV1, WasmPluginStartupV1, WasmResourceHandleRegistryV1,
        reference_wasm_manifest_v1,
    };
    use next_contracts::{
        CanonicalDecodeLimits, CapabilityId, ContentHash, PluginId, SchemaId, WasmPluginManifestV1,
        WasmPluginStateV1, WasmResourceLimitsV1, WitResourceHandleV1, content_hash_from_bytes,
        sha256,
    };

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
        let state =
            WasmPluginStateV1::from_canonical_bytes(&bytes, CanonicalDecodeLimits::default())
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
                next_contracts::ExtensionContractError::IncompatibleInterface
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
}
