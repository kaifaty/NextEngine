use super::*;

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
