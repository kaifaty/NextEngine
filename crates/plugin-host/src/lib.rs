#![forbid(unsafe_code)]

use std::collections::BTreeMap;
use std::error::Error;
use std::fmt::{Display, Formatter};

use next_contracts::canonical::sha256;
use next_contracts::extension::{
    ExtensionContractError, WasmPluginManifestV1, WasmPluginStateV1, WasmResourceLimitsV1,
    WitInterfaceSelectionV1, WitResourceHandleV1, negotiate_wit_interface_v1,
};
use next_contracts::ids::{CapabilityId, ContentHash, PluginId, SchemaId, content_hash_from_bytes};
use next_contracts::mechanics::RpgDefinitionRegistryV2;
use next_contracts::rpg::{RpgPhysicalContactFactV1, RpgSnapshotV2};
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

mod reference;
pub use reference::*;

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
mod tests;
