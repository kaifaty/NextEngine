use std::error::Error;
use std::fmt::{Display, Formatter};

use crate::{
    CanonicalDecodeLimits, CapabilityId, ContentHash, MechanicPackageId, PluginId, SchemaId,
    content_hash_from_bytes, sha256,
};

pub const EXTENSION_PACKAGE_STATE_SCHEMA_VERSION: u32 = 1;
pub const EXTENSION_CIRCUIT_THRESHOLD: usize = 3;
pub const EXTENSION_CIRCUIT_WINDOW_TICKS: u64 = 1_800;
pub const EXTENSION_MAX_CAPABILITIES: usize = 128;
pub const EXTENSION_MAX_STATE_BYTES: usize = 1_048_576;
pub const WASM_PLUGIN_STATE_SCHEMA_VERSION: u32 = 1;
pub const WASM_HOST_CURRENT_API_MAJOR: u16 = 3;
pub const WASM_HOST_PREVIOUS_API_MAJOR: u16 = 2;
pub const WASM_DEFAULT_LINEAR_MEMORY_BYTES: u64 = 64 * 1_024 * 1_024;
pub const WASM_DEFAULT_TABLE_LIMIT: u32 = 1;
pub const WASM_DEFAULT_INSTANCE_LIMIT: u32 = 1;
pub const WASM_DEFAULT_FUEL_PER_CALL: u64 = 10_000_000;
pub const WASM_DEFAULT_FUEL_PER_TICK: u64 = 20_000_000;
const WASM_MAX_SOURCE_METADATA_BYTES: usize = 4_096;
const STATE_PREFIX: &[u8] = b"nextengine.extension-package-state.v1\0";
const WASM_STATE_PREFIX: &[u8] = b"nextengine.wasm-plugin-state.v1\0";

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExtensionBudgetPolicyV1 {
    pub instruction_limit: u64,
    pub transient_allocation_limit_bytes: u64,
    pub live_memory_limit_bytes: u64,
    pub host_call_cost_limit: u64,
    pub proposed_command_count_limit: u32,
    pub proposed_command_bytes_limit: u64,
    pub policy_hash: ContentHash,
}

impl ExtensionBudgetPolicyV1 {
    pub fn new(
        instruction_limit: u64,
        transient_allocation_limit_bytes: u64,
        live_memory_limit_bytes: u64,
        host_call_cost_limit: u64,
        proposed_command_count_limit: u32,
        proposed_command_bytes_limit: u64,
    ) -> Result<Self, ExtensionContractError> {
        if instruction_limit == 0
            || transient_allocation_limit_bytes == 0
            || live_memory_limit_bytes == 0
            || host_call_cost_limit == 0
            || proposed_command_count_limit == 0
            || proposed_command_bytes_limit == 0
        {
            return Err(ExtensionContractError::InvalidBudget);
        }
        let mut value = Self {
            instruction_limit,
            transient_allocation_limit_bytes,
            live_memory_limit_bytes,
            host_call_cost_limit,
            proposed_command_count_limit,
            proposed_command_bytes_limit,
            policy_hash: ContentHash::default(),
        };
        value.policy_hash = value.computed_hash();
        Ok(value)
    }

    pub fn validate(&self) -> Result<(), ExtensionContractError> {
        if self.instruction_limit == 0
            || self.transient_allocation_limit_bytes == 0
            || self.live_memory_limit_bytes == 0
            || self.host_call_cost_limit == 0
            || self.proposed_command_count_limit == 0
            || self.proposed_command_bytes_limit == 0
            || self.computed_hash() != self.policy_hash
        {
            return Err(ExtensionContractError::InvalidBudget);
        }
        Ok(())
    }

    fn computed_hash(&self) -> ContentHash {
        let mut bytes = b"nextengine.extension-budget-policy.v1\0".to_vec();
        bytes.extend_from_slice(&self.instruction_limit.to_le_bytes());
        bytes.extend_from_slice(&self.transient_allocation_limit_bytes.to_le_bytes());
        bytes.extend_from_slice(&self.live_memory_limit_bytes.to_le_bytes());
        bytes.extend_from_slice(&self.host_call_cost_limit.to_le_bytes());
        bytes.extend_from_slice(&self.proposed_command_count_limit.to_le_bytes());
        bytes.extend_from_slice(&self.proposed_command_bytes_limit.to_le_bytes());
        content_hash_from_bytes(sha256(&bytes))
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LuauPackageManifestV1 {
    pub package_id: MechanicPackageId,
    pub package_version: u32,
    pub source_hash: ContentHash,
    pub requested_capabilities: Vec<CapabilityId>,
    pub state_schema_id: SchemaId,
    pub state_schema_version: u32,
    pub required_at_startup: bool,
    pub budget: ExtensionBudgetPolicyV1,
    pub manifest_hash: ContentHash,
}

impl LuauPackageManifestV1 {
    #[allow(
        clippy::too_many_arguments,
        reason = "the script manifest keeps compatibility, authority and budget bindings explicit"
    )]
    pub fn new(
        package_id: MechanicPackageId,
        package_version: u32,
        source_hash: ContentHash,
        mut requested_capabilities: Vec<CapabilityId>,
        state_schema_id: SchemaId,
        state_schema_version: u32,
        required_at_startup: bool,
        budget: ExtensionBudgetPolicyV1,
    ) -> Result<Self, ExtensionContractError> {
        requested_capabilities.sort();
        if package_version == 0
            || state_schema_version == 0
            || requested_capabilities.len() > EXTENSION_MAX_CAPABILITIES
            || requested_capabilities
                .windows(2)
                .any(|pair| pair[0] == pair[1])
        {
            return Err(ExtensionContractError::InvalidManifest);
        }
        budget.validate()?;
        let mut value = Self {
            package_id,
            package_version,
            source_hash,
            requested_capabilities,
            state_schema_id,
            state_schema_version,
            required_at_startup,
            budget,
            manifest_hash: ContentHash::default(),
        };
        value.manifest_hash = value.computed_hash()?;
        Ok(value)
    }

    pub fn validate(&self) -> Result<(), ExtensionContractError> {
        if self.package_version == 0
            || self.state_schema_version == 0
            || self.requested_capabilities.len() > EXTENSION_MAX_CAPABILITIES
            || self
                .requested_capabilities
                .windows(2)
                .any(|pair| pair[0] >= pair[1])
            || self.computed_hash()? != self.manifest_hash
        {
            return Err(ExtensionContractError::InvalidManifest);
        }
        self.budget.validate()
    }

    fn computed_hash(&self) -> Result<ContentHash, ExtensionContractError> {
        let mut bytes = b"nextengine.luau-package-manifest.v1\0".to_vec();
        extend_text(&mut bytes, self.package_id.as_str())?;
        bytes.extend_from_slice(&self.package_version.to_le_bytes());
        bytes.extend_from_slice(self.source_hash.as_bytes());
        extend_count(&mut bytes, self.requested_capabilities.len())?;
        for capability in &self.requested_capabilities {
            extend_text(&mut bytes, capability.as_str())?;
        }
        extend_text(&mut bytes, self.state_schema_id.as_str())?;
        bytes.extend_from_slice(&self.state_schema_version.to_le_bytes());
        bytes.push(u8::from(self.required_at_startup));
        bytes.extend_from_slice(self.budget.policy_hash.as_bytes());
        Ok(content_hash_from_bytes(sha256(&bytes)))
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WasmResourceLimitsV1 {
    pub linear_memory_limit_bytes: u64,
    pub table_limit: u32,
    pub instance_limit: u32,
    pub fuel_per_call: u64,
    pub fuel_per_gameplay_tick: u64,
}

impl WasmResourceLimitsV1 {
    pub fn untrusted_default() -> Self {
        Self {
            linear_memory_limit_bytes: WASM_DEFAULT_LINEAR_MEMORY_BYTES,
            table_limit: WASM_DEFAULT_TABLE_LIMIT,
            instance_limit: WASM_DEFAULT_INSTANCE_LIMIT,
            fuel_per_call: WASM_DEFAULT_FUEL_PER_CALL,
            fuel_per_gameplay_tick: WASM_DEFAULT_FUEL_PER_TICK,
        }
    }

    pub fn validate(&self) -> Result<(), ExtensionContractError> {
        if self.linear_memory_limit_bytes == 0
            || self.linear_memory_limit_bytes > WASM_DEFAULT_LINEAR_MEMORY_BYTES
            || self.table_limit == 0
            || self.table_limit > WASM_DEFAULT_TABLE_LIMIT
            || self.instance_limit == 0
            || self.instance_limit > WASM_DEFAULT_INSTANCE_LIMIT
            || self.fuel_per_call == 0
            || self.fuel_per_call > WASM_DEFAULT_FUEL_PER_CALL
            || self.fuel_per_gameplay_tick < self.fuel_per_call
            || self.fuel_per_gameplay_tick > WASM_DEFAULT_FUEL_PER_TICK
        {
            return Err(ExtensionContractError::InvalidBudget);
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WitInterfaceSelectionV1 {
    pub host_api_major: u16,
    pub uses_compatibility_adapter: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WasmPluginManifestV1 {
    pub plugin_id: PluginId,
    pub plugin_version: u32,
    pub component_hash: ContentHash,
    pub wit_world_id: SchemaId,
    pub minimum_host_api_major: u16,
    pub maximum_host_api_major: u16,
    pub requested_capabilities: Vec<CapabilityId>,
    pub state_schema_id: SchemaId,
    pub state_schema_version: u32,
    pub required_at_startup: bool,
    pub deterministic: bool,
    pub source: String,
    pub provenance: String,
    pub limits: WasmResourceLimitsV1,
    pub manifest_hash: ContentHash,
}

impl WasmPluginManifestV1 {
    #[allow(
        clippy::too_many_arguments,
        reason = "the plugin manifest keeps identity, WIT compatibility, provenance and quotas explicit"
    )]
    pub fn new(
        plugin_id: PluginId,
        plugin_version: u32,
        component_hash: ContentHash,
        wit_world_id: SchemaId,
        minimum_host_api_major: u16,
        maximum_host_api_major: u16,
        mut requested_capabilities: Vec<CapabilityId>,
        state_schema_id: SchemaId,
        state_schema_version: u32,
        required_at_startup: bool,
        deterministic: bool,
        source: impl Into<String>,
        provenance: impl Into<String>,
        limits: WasmResourceLimitsV1,
    ) -> Result<Self, ExtensionContractError> {
        requested_capabilities.sort();
        let mut value = Self {
            plugin_id,
            plugin_version,
            component_hash,
            wit_world_id,
            minimum_host_api_major,
            maximum_host_api_major,
            requested_capabilities,
            state_schema_id,
            state_schema_version,
            required_at_startup,
            deterministic,
            source: source.into(),
            provenance: provenance.into(),
            limits,
            manifest_hash: ContentHash::default(),
        };
        value.validate_body()?;
        value.manifest_hash = value.computed_hash()?;
        Ok(value)
    }

    pub fn validate(&self) -> Result<(), ExtensionContractError> {
        self.validate_body()?;
        if self.computed_hash()? != self.manifest_hash {
            return Err(ExtensionContractError::InvalidManifest);
        }
        Ok(())
    }

    fn validate_body(&self) -> Result<(), ExtensionContractError> {
        if self.plugin_version == 0
            || self.minimum_host_api_major == 0
            || self.minimum_host_api_major > self.maximum_host_api_major
            || self.state_schema_version == 0
            || !self.deterministic
            || self.source.is_empty()
            || self.source.len() > WASM_MAX_SOURCE_METADATA_BYTES
            || self.provenance.is_empty()
            || self.provenance.len() > WASM_MAX_SOURCE_METADATA_BYTES
            || self.requested_capabilities.len() > EXTENSION_MAX_CAPABILITIES
            || self
                .requested_capabilities
                .windows(2)
                .any(|pair| pair[0] >= pair[1])
        {
            return Err(ExtensionContractError::InvalidManifest);
        }
        self.limits.validate()
    }

    fn computed_hash(&self) -> Result<ContentHash, ExtensionContractError> {
        let mut bytes = b"nextengine.wasm-plugin-manifest.v1\0".to_vec();
        extend_text(&mut bytes, self.plugin_id.as_str())?;
        bytes.extend_from_slice(&self.plugin_version.to_le_bytes());
        bytes.extend_from_slice(self.component_hash.as_bytes());
        extend_text(&mut bytes, self.wit_world_id.as_str())?;
        bytes.extend_from_slice(&self.minimum_host_api_major.to_le_bytes());
        bytes.extend_from_slice(&self.maximum_host_api_major.to_le_bytes());
        extend_count(&mut bytes, self.requested_capabilities.len())?;
        for capability in &self.requested_capabilities {
            extend_text(&mut bytes, capability.as_str())?;
        }
        extend_text(&mut bytes, self.state_schema_id.as_str())?;
        bytes.extend_from_slice(&self.state_schema_version.to_le_bytes());
        bytes.push(u8::from(self.required_at_startup));
        bytes.push(u8::from(self.deterministic));
        extend_text(&mut bytes, &self.source)?;
        extend_text(&mut bytes, &self.provenance)?;
        bytes.extend_from_slice(&self.limits.linear_memory_limit_bytes.to_le_bytes());
        bytes.extend_from_slice(&self.limits.table_limit.to_le_bytes());
        bytes.extend_from_slice(&self.limits.instance_limit.to_le_bytes());
        bytes.extend_from_slice(&self.limits.fuel_per_call.to_le_bytes());
        bytes.extend_from_slice(&self.limits.fuel_per_gameplay_tick.to_le_bytes());
        Ok(content_hash_from_bytes(sha256(&bytes)))
    }
}

pub fn negotiate_wit_interface_v1(
    manifest: &WasmPluginManifestV1,
) -> Result<WitInterfaceSelectionV1, ExtensionContractError> {
    manifest.validate()?;
    if (manifest.minimum_host_api_major..=manifest.maximum_host_api_major)
        .contains(&WASM_HOST_CURRENT_API_MAJOR)
    {
        return Ok(WitInterfaceSelectionV1 {
            host_api_major: WASM_HOST_CURRENT_API_MAJOR,
            uses_compatibility_adapter: false,
        });
    }
    if (manifest.minimum_host_api_major..=manifest.maximum_host_api_major)
        .contains(&WASM_HOST_PREVIOUS_API_MAJOR)
    {
        return Ok(WitInterfaceSelectionV1 {
            host_api_major: WASM_HOST_PREVIOUS_API_MAJOR,
            uses_compatibility_adapter: true,
        });
    }
    Err(ExtensionContractError::IncompatibleInterface)
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WitResourceHandleV1 {
    pub plugin_id: PluginId,
    pub resource_kind: SchemaId,
    pub slot: u32,
    pub generation: u32,
    pub binding_hash: ContentHash,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WasmPluginStateV1 {
    pub plugin_id: PluginId,
    pub plugin_manifest_hash: ContentHash,
    pub state_schema_id: SchemaId,
    pub state_schema_version: u32,
    pub state_bytes: Vec<u8>,
    pub violation_ticks: Vec<u64>,
    pub circuit_opened_at_tick: Option<u64>,
    pub state_hash: ContentHash,
}

impl WasmPluginStateV1 {
    pub fn new(
        manifest: &WasmPluginManifestV1,
        state_bytes: Vec<u8>,
    ) -> Result<Self, ExtensionContractError> {
        manifest.validate()?;
        if state_bytes.len() > EXTENSION_MAX_STATE_BYTES {
            return Err(ExtensionContractError::StateLimitExceeded);
        }
        let mut value = Self {
            plugin_id: manifest.plugin_id.clone(),
            plugin_manifest_hash: manifest.manifest_hash,
            state_schema_id: manifest.state_schema_id.clone(),
            state_schema_version: manifest.state_schema_version,
            state_bytes,
            violation_ticks: Vec::new(),
            circuit_opened_at_tick: None,
            state_hash: ContentHash::default(),
        };
        value.refresh_hash()?;
        Ok(value)
    }

    pub fn validate(&self) -> Result<(), ExtensionContractError> {
        if self.state_schema_version == 0
            || self.state_bytes.len() > EXTENSION_MAX_STATE_BYTES
            || self.violation_ticks.len() > EXTENSION_CIRCUIT_THRESHOLD
            || self
                .violation_ticks
                .windows(2)
                .any(|pair| pair[0] > pair[1])
            || self.violation_ticks.first().is_some_and(|first| {
                self.violation_ticks.last().is_some_and(|last| {
                    last.saturating_sub(*first) >= EXTENSION_CIRCUIT_WINDOW_TICKS
                })
            })
            || (self.violation_ticks.len() >= EXTENSION_CIRCUIT_THRESHOLD)
                != self.circuit_opened_at_tick.is_some()
            || self.circuit_opened_at_tick.is_some_and(|opened| {
                self.violation_ticks
                    .last()
                    .is_none_or(|tick| *tick != opened)
            })
            || self.computed_hash()? != self.state_hash
        {
            return Err(ExtensionContractError::InvalidState);
        }
        Ok(())
    }

    pub fn replace_state_bytes(
        &mut self,
        state_bytes: Vec<u8>,
    ) -> Result<(), ExtensionContractError> {
        if state_bytes.len() > EXTENSION_MAX_STATE_BYTES {
            return Err(ExtensionContractError::StateLimitExceeded);
        }
        self.state_bytes = state_bytes;
        self.refresh_hash()
    }

    pub fn record_violation(&mut self, gameplay_tick: u64) -> Result<(), ExtensionContractError> {
        if self.circuit_opened_at_tick.is_some() {
            return Ok(());
        }
        let window_start = gameplay_tick.saturating_sub(EXTENSION_CIRCUIT_WINDOW_TICKS - 1);
        self.violation_ticks
            .retain(|tick| *tick >= window_start && *tick <= gameplay_tick);
        if self
            .violation_ticks
            .last()
            .is_some_and(|prior| *prior > gameplay_tick)
        {
            return Err(ExtensionContractError::InvalidState);
        }
        self.violation_ticks.push(gameplay_tick);
        if self.violation_ticks.len() >= EXTENSION_CIRCUIT_THRESHOLD {
            self.circuit_opened_at_tick = Some(gameplay_tick);
        }
        self.refresh_hash()
    }

    pub fn canonical_bytes(&self) -> Result<Vec<u8>, ExtensionContractError> {
        let mut bytes = WASM_STATE_PREFIX.to_vec();
        bytes.extend_from_slice(&WASM_PLUGIN_STATE_SCHEMA_VERSION.to_le_bytes());
        extend_text(&mut bytes, self.plugin_id.as_str())?;
        bytes.extend_from_slice(self.plugin_manifest_hash.as_bytes());
        extend_text(&mut bytes, self.state_schema_id.as_str())?;
        bytes.extend_from_slice(&self.state_schema_version.to_le_bytes());
        extend_bytes(&mut bytes, &self.state_bytes)?;
        extend_count(&mut bytes, self.violation_ticks.len())?;
        for tick in &self.violation_ticks {
            bytes.extend_from_slice(&tick.to_le_bytes());
        }
        match self.circuit_opened_at_tick {
            None => bytes.push(0),
            Some(tick) => {
                bytes.push(1);
                bytes.extend_from_slice(&tick.to_le_bytes());
            }
        }
        Ok(bytes)
    }

    pub fn from_canonical_bytes(
        bytes: &[u8],
        limits: CanonicalDecodeLimits,
    ) -> Result<Self, ExtensionContractError> {
        if bytes.len() > limits.max_total_bytes {
            return Err(ExtensionContractError::DecodeLimit);
        }
        let mut cursor = Cursor::new(bytes);
        if cursor.read_exact(WASM_STATE_PREFIX.len())? != WASM_STATE_PREFIX {
            return Err(ExtensionContractError::WrongEnvelope);
        }
        if cursor.read_u32()? != WASM_PLUGIN_STATE_SCHEMA_VERSION {
            return Err(ExtensionContractError::UnsupportedVersion);
        }
        let plugin_id = PluginId::new(cursor.read_text(limits.max_identifier_bytes)?)
            .map_err(|_| ExtensionContractError::InvalidState)?;
        let plugin_manifest_hash = ContentHash::from_bytes(cursor.read_array()?);
        let state_schema_id = SchemaId::new(cursor.read_text(limits.max_identifier_bytes)?)
            .map_err(|_| ExtensionContractError::InvalidState)?;
        let state_schema_version = cursor.read_u32()?;
        let state_bytes = cursor.read_bytes(EXTENSION_MAX_STATE_BYTES)?;
        let violation_count = cursor.read_count(EXTENSION_CIRCUIT_THRESHOLD)?;
        let mut violation_ticks = Vec::with_capacity(violation_count);
        for _ in 0..violation_count {
            violation_ticks.push(cursor.read_u64()?);
        }
        let circuit_opened_at_tick = match cursor.read_u8()? {
            0 => None,
            1 => Some(cursor.read_u64()?),
            _ => return Err(ExtensionContractError::InvalidState),
        };
        cursor.finish()?;
        let mut value = Self {
            plugin_id,
            plugin_manifest_hash,
            state_schema_id,
            state_schema_version,
            state_bytes,
            violation_ticks,
            circuit_opened_at_tick,
            state_hash: ContentHash::default(),
        };
        value.refresh_hash()?;
        value.validate()?;
        if value.canonical_bytes()? != bytes {
            return Err(ExtensionContractError::NonCanonical);
        }
        Ok(value)
    }

    fn refresh_hash(&mut self) -> Result<(), ExtensionContractError> {
        self.state_hash = self.computed_hash()?;
        Ok(())
    }

    fn computed_hash(&self) -> Result<ContentHash, ExtensionContractError> {
        Ok(content_hash_from_bytes(sha256(&self.canonical_bytes()?)))
    }
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
#[repr(u8)]
pub enum ExtensionViolationCodeV1 {
    InstructionBudget = 1,
    TransientAllocationBudget = 2,
    LiveMemoryBudget = 3,
    HostCallBudget = 4,
    CommandBudget = 5,
    CapabilityDenied = 6,
    Trap = 7,
    SchemaViolation = 8,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExtensionPackageStateV1 {
    pub package_id: MechanicPackageId,
    pub package_manifest_hash: ContentHash,
    pub state_schema_id: SchemaId,
    pub state_schema_version: u32,
    pub state_bytes: Vec<u8>,
    pub violation_ticks: Vec<u64>,
    pub circuit_opened_at_tick: Option<u64>,
    pub state_hash: ContentHash,
}

impl ExtensionPackageStateV1 {
    pub fn new(
        manifest: &LuauPackageManifestV1,
        state_bytes: Vec<u8>,
    ) -> Result<Self, ExtensionContractError> {
        manifest.validate()?;
        if state_bytes.len() > EXTENSION_MAX_STATE_BYTES {
            return Err(ExtensionContractError::StateLimitExceeded);
        }
        let mut value = Self {
            package_id: manifest.package_id.clone(),
            package_manifest_hash: manifest.manifest_hash,
            state_schema_id: manifest.state_schema_id.clone(),
            state_schema_version: manifest.state_schema_version,
            state_bytes,
            violation_ticks: Vec::new(),
            circuit_opened_at_tick: None,
            state_hash: ContentHash::default(),
        };
        value.refresh_hash()?;
        Ok(value)
    }

    pub fn validate(&self) -> Result<(), ExtensionContractError> {
        if self.state_schema_version == 0
            || self.state_bytes.len() > EXTENSION_MAX_STATE_BYTES
            || self.violation_ticks.len() > EXTENSION_CIRCUIT_THRESHOLD
            || self
                .violation_ticks
                .windows(2)
                .any(|pair| pair[0] > pair[1])
            || self.violation_ticks.first().is_some_and(|first| {
                self.violation_ticks.last().is_some_and(|last| {
                    last.saturating_sub(*first) >= EXTENSION_CIRCUIT_WINDOW_TICKS
                })
            })
            || (self.violation_ticks.len() >= EXTENSION_CIRCUIT_THRESHOLD)
                != self.circuit_opened_at_tick.is_some()
            || self.circuit_opened_at_tick.is_some_and(|opened| {
                self.violation_ticks
                    .last()
                    .is_none_or(|tick| *tick != opened)
            })
            || self.computed_hash()? != self.state_hash
        {
            return Err(ExtensionContractError::InvalidState);
        }
        Ok(())
    }

    pub fn record_violation(&mut self, gameplay_tick: u64) -> Result<(), ExtensionContractError> {
        if self.circuit_opened_at_tick.is_some() {
            return Ok(());
        }
        let window_start = gameplay_tick.saturating_sub(EXTENSION_CIRCUIT_WINDOW_TICKS - 1);
        self.violation_ticks
            .retain(|tick| *tick >= window_start && *tick <= gameplay_tick);
        if self
            .violation_ticks
            .last()
            .is_some_and(|prior| *prior > gameplay_tick)
        {
            return Err(ExtensionContractError::InvalidState);
        }
        self.violation_ticks.push(gameplay_tick);
        if self.violation_ticks.len() >= EXTENSION_CIRCUIT_THRESHOLD {
            self.circuit_opened_at_tick = Some(gameplay_tick);
        }
        self.refresh_hash()
    }

    pub fn replace_state_bytes(
        &mut self,
        state_bytes: Vec<u8>,
    ) -> Result<(), ExtensionContractError> {
        if state_bytes.len() > EXTENSION_MAX_STATE_BYTES {
            return Err(ExtensionContractError::StateLimitExceeded);
        }
        self.state_bytes = state_bytes;
        self.refresh_hash()
    }

    pub fn canonical_bytes(&self) -> Result<Vec<u8>, ExtensionContractError> {
        let mut bytes = STATE_PREFIX.to_vec();
        bytes.extend_from_slice(&EXTENSION_PACKAGE_STATE_SCHEMA_VERSION.to_le_bytes());
        extend_text(&mut bytes, self.package_id.as_str())?;
        bytes.extend_from_slice(self.package_manifest_hash.as_bytes());
        extend_text(&mut bytes, self.state_schema_id.as_str())?;
        bytes.extend_from_slice(&self.state_schema_version.to_le_bytes());
        extend_bytes(&mut bytes, &self.state_bytes)?;
        extend_count(&mut bytes, self.violation_ticks.len())?;
        for tick in &self.violation_ticks {
            bytes.extend_from_slice(&tick.to_le_bytes());
        }
        match self.circuit_opened_at_tick {
            None => bytes.push(0),
            Some(tick) => {
                bytes.push(1);
                bytes.extend_from_slice(&tick.to_le_bytes());
            }
        }
        Ok(bytes)
    }

    pub fn from_canonical_bytes(
        bytes: &[u8],
        limits: CanonicalDecodeLimits,
    ) -> Result<Self, ExtensionContractError> {
        if bytes.len() > limits.max_total_bytes {
            return Err(ExtensionContractError::DecodeLimit);
        }
        let mut cursor = Cursor::new(bytes);
        if cursor.read_exact(STATE_PREFIX.len())? != STATE_PREFIX {
            return Err(ExtensionContractError::WrongEnvelope);
        }
        if cursor.read_u32()? != EXTENSION_PACKAGE_STATE_SCHEMA_VERSION {
            return Err(ExtensionContractError::UnsupportedVersion);
        }
        let package_id = MechanicPackageId::new(cursor.read_text(limits.max_identifier_bytes)?)
            .map_err(|_| ExtensionContractError::InvalidState)?;
        let package_manifest_hash = ContentHash::from_bytes(cursor.read_array()?);
        let state_schema_id = SchemaId::new(cursor.read_text(limits.max_identifier_bytes)?)
            .map_err(|_| ExtensionContractError::InvalidState)?;
        let state_schema_version = cursor.read_u32()?;
        let state_bytes = cursor.read_bytes(EXTENSION_MAX_STATE_BYTES)?;
        let violation_count = cursor.read_count(EXTENSION_CIRCUIT_THRESHOLD)?;
        let mut violation_ticks = Vec::with_capacity(violation_count);
        for _ in 0..violation_count {
            violation_ticks.push(cursor.read_u64()?);
        }
        let circuit_opened_at_tick = match cursor.read_u8()? {
            0 => None,
            1 => Some(cursor.read_u64()?),
            _ => return Err(ExtensionContractError::InvalidState),
        };
        cursor.finish()?;
        let mut value = Self {
            package_id,
            package_manifest_hash,
            state_schema_id,
            state_schema_version,
            state_bytes,
            violation_ticks,
            circuit_opened_at_tick,
            state_hash: ContentHash::default(),
        };
        value.refresh_hash()?;
        value.validate()?;
        if value.canonical_bytes()? != bytes {
            return Err(ExtensionContractError::NonCanonical);
        }
        Ok(value)
    }

    fn refresh_hash(&mut self) -> Result<(), ExtensionContractError> {
        self.state_hash = self.computed_hash()?;
        Ok(())
    }

    fn computed_hash(&self) -> Result<ContentHash, ExtensionContractError> {
        Ok(content_hash_from_bytes(sha256(&self.canonical_bytes()?)))
    }
}

fn extend_count(bytes: &mut Vec<u8>, count: usize) -> Result<(), ExtensionContractError> {
    bytes.extend_from_slice(
        &u32::try_from(count)
            .map_err(|_| ExtensionContractError::LimitExceeded)?
            .to_le_bytes(),
    );
    Ok(())
}

fn extend_text(bytes: &mut Vec<u8>, value: &str) -> Result<(), ExtensionContractError> {
    extend_bytes(bytes, value.as_bytes())
}

fn extend_bytes(bytes: &mut Vec<u8>, value: &[u8]) -> Result<(), ExtensionContractError> {
    extend_count(bytes, value.len())?;
    bytes.extend_from_slice(value);
    Ok(())
}

struct Cursor<'a> {
    bytes: &'a [u8],
    offset: usize,
}

impl<'a> Cursor<'a> {
    const fn new(bytes: &'a [u8]) -> Self {
        Self { bytes, offset: 0 }
    }

    fn read_exact(&mut self, length: usize) -> Result<&'a [u8], ExtensionContractError> {
        let end = self
            .offset
            .checked_add(length)
            .ok_or(ExtensionContractError::DecodeLimit)?;
        let value = self
            .bytes
            .get(self.offset..end)
            .ok_or(ExtensionContractError::Truncated)?;
        self.offset = end;
        Ok(value)
    }

    fn read_array<const N: usize>(&mut self) -> Result<[u8; N], ExtensionContractError> {
        self.read_exact(N)?
            .try_into()
            .map_err(|_| ExtensionContractError::Truncated)
    }

    fn read_u8(&mut self) -> Result<u8, ExtensionContractError> {
        Ok(self.read_exact(1)?[0])
    }

    fn read_u32(&mut self) -> Result<u32, ExtensionContractError> {
        Ok(u32::from_le_bytes(self.read_array()?))
    }

    fn read_u64(&mut self) -> Result<u64, ExtensionContractError> {
        Ok(u64::from_le_bytes(self.read_array()?))
    }

    fn read_count(&mut self, maximum: usize) -> Result<usize, ExtensionContractError> {
        let count =
            usize::try_from(self.read_u32()?).map_err(|_| ExtensionContractError::DecodeLimit)?;
        if count > maximum {
            return Err(ExtensionContractError::DecodeLimit);
        }
        Ok(count)
    }

    fn read_bytes(&mut self, maximum: usize) -> Result<Vec<u8>, ExtensionContractError> {
        let length = self.read_count(maximum)?;
        Ok(self.read_exact(length)?.to_vec())
    }

    fn read_text(&mut self, maximum: usize) -> Result<String, ExtensionContractError> {
        String::from_utf8(self.read_bytes(maximum)?)
            .map_err(|_| ExtensionContractError::InvalidState)
    }

    fn finish(self) -> Result<(), ExtensionContractError> {
        if self.offset == self.bytes.len() {
            Ok(())
        } else {
            Err(ExtensionContractError::NonCanonical)
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum ExtensionContractError {
    InvalidBudget,
    InvalidManifest,
    InvalidState,
    StateLimitExceeded,
    LimitExceeded,
    DecodeLimit,
    WrongEnvelope,
    UnsupportedVersion,
    Truncated,
    NonCanonical,
    IncompatibleInterface,
    InvalidHandle,
}

impl Display for ExtensionContractError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(match self {
            Self::InvalidBudget => "extension budget policy is invalid",
            Self::InvalidManifest => "extension package manifest is invalid",
            Self::InvalidState => "extension package state is invalid",
            Self::StateLimitExceeded => "extension package state exceeds its limit",
            Self::LimitExceeded => "extension contract limit exceeded",
            Self::DecodeLimit => "extension decode limit exceeded",
            Self::WrongEnvelope => "extension state envelope is invalid",
            Self::UnsupportedVersion => "extension state version is unsupported",
            Self::Truncated => "extension state is truncated",
            Self::NonCanonical => "extension state encoding is non-canonical",
            Self::IncompatibleInterface => "WIT interface is outside the supported N/N-1 matrix",
            Self::InvalidHandle => "WIT resource handle is invalid or forged",
        })
    }
}

impl Error for ExtensionContractError {}

#[cfg(test)]
mod tests {
    use super::{
        EXTENSION_CIRCUIT_WINDOW_TICKS, ExtensionBudgetPolicyV1, ExtensionPackageStateV1,
        LuauPackageManifestV1,
    };
    use crate::{CanonicalDecodeLimits, CapabilityId, ContentHash, MechanicPackageId, SchemaId};

    #[test]
    fn circuit_window_and_package_state_round_trip_are_exact() {
        let manifest = manifest();
        let mut state = ExtensionPackageStateV1::new(&manifest, b"ready".to_vec()).expect("state");
        state.record_violation(0).expect("first");
        state
            .record_violation(EXTENSION_CIRCUIT_WINDOW_TICKS - 1)
            .expect("second inclusive");
        state
            .record_violation(EXTENSION_CIRCUIT_WINDOW_TICKS)
            .expect("oldest expires");
        assert_eq!(
            state.violation_ticks,
            [
                EXTENSION_CIRCUIT_WINDOW_TICKS - 1,
                EXTENSION_CIRCUIT_WINDOW_TICKS
            ]
        );
        state
            .record_violation(EXTENSION_CIRCUIT_WINDOW_TICKS + 1)
            .expect("third opens");
        assert_eq!(
            state.circuit_opened_at_tick,
            Some(EXTENSION_CIRCUIT_WINDOW_TICKS + 1)
        );
        let bytes = state.canonical_bytes().expect("bytes");
        assert_eq!(
            ExtensionPackageStateV1::from_canonical_bytes(&bytes, CanonicalDecodeLimits::default())
                .expect("decode"),
            state
        );
    }

    fn manifest() -> LuauPackageManifestV1 {
        LuauPackageManifestV1::new(
            MechanicPackageId::new("org.nextengine.test.luau").expect("package"),
            1,
            ContentHash::from_bytes([1; 32]),
            vec![CapabilityId::new("mechanics.effect.propose").expect("capability")],
            SchemaId::new("org.nextengine.test.state").expect("schema"),
            1,
            false,
            ExtensionBudgetPolicyV1::new(10, 100, 1_000, 3, 1, 100).expect("budget"),
        )
        .expect("manifest")
    }
}
