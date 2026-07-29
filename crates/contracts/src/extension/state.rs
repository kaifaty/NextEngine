use super::*;

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
