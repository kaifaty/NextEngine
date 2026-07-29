use super::*;

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum NormalizedControlPhaseV1 {
    Started,
    Changed,
    Completed,
    Cancelled,
}

impl NormalizedControlPhaseV1 {
    const fn token(self) -> &'static str {
        match self {
            Self::Started => "Started",
            Self::Changed => "Changed",
            Self::Completed => "Completed",
            Self::Cancelled => "Cancelled",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NormalizedControlEventV1 {
    pub schema_version: u32,
    pub control_sample_id: ContentHash,
    pub device_class: SchemaId,
    pub device_instance_nonce: PersistentId,
    pub control_path_id: SchemaId,
    pub phase: NormalizedControlPhaseV1,
    pub quantized_value: Vec<i16>,
    pub modifier_set: Vec<SchemaId>,
    pub platform_sample_tick: u64,
    pub source_sequence: u64,
}

impl NormalizedControlEventV1 {
    #[allow(
        clippy::too_many_arguments,
        reason = "the closed contract constructor keeps every identity and ordering field explicit"
    )]
    pub fn new(
        device_class: SchemaId,
        device_instance_nonce: PersistentId,
        control_path_id: SchemaId,
        phase: NormalizedControlPhaseV1,
        quantized_value: Vec<i16>,
        mut modifier_set: Vec<SchemaId>,
        platform_sample_tick: u64,
        source_sequence: u64,
    ) -> Result<Self, PlatformContractError> {
        enforce_limit(quantized_value.len(), PLATFORM_MAX_CONTROL_COMPONENTS)?;
        enforce_limit(modifier_set.len(), PLATFORM_MAX_MODIFIERS)?;
        modifier_set.sort();
        enforce_unique(&modifier_set)?;
        let mut value = Self {
            schema_version: NORMALIZED_CONTROL_EVENT_SCHEMA_VERSION,
            control_sample_id: ContentHash::default(),
            device_class,
            device_instance_nonce,
            control_path_id,
            phase,
            quantized_value,
            modifier_set,
            platform_sample_tick,
            source_sequence,
        };
        value.control_sample_id = value.computed_id();
        Ok(value)
    }

    pub fn validate(&self) -> Result<(), PlatformContractError> {
        if self.schema_version != NORMALIZED_CONTROL_EVENT_SCHEMA_VERSION {
            return Err(PlatformContractError::UnsupportedVersion);
        }
        enforce_limit(self.quantized_value.len(), PLATFORM_MAX_CONTROL_COMPONENTS)?;
        enforce_limit(self.modifier_set.len(), PLATFORM_MAX_MODIFIERS)?;
        enforce_canonical(&self.modifier_set)?;
        if self.computed_id() != self.control_sample_id {
            return Err(PlatformContractError::HashMismatch);
        }
        Ok(())
    }

    fn computed_id(&self) -> ContentHash {
        domain_hash(
            "nextengine.normalized-control-event.v1",
            &encode_canonical_jcs(&object([
                ("control_path_id", string(self.control_path_id.as_str())),
                ("device_class", string(self.device_class.as_str())),
                (
                    "device_instance_nonce",
                    string(hex_bytes(self.device_instance_nonce.as_bytes())),
                ),
                (
                    "modifier_set",
                    JcsValue::Array(
                        self.modifier_set
                            .iter()
                            .map(|value| string(value.as_str()))
                            .collect(),
                    ),
                ),
                ("phase", string(self.phase.token())),
                (
                    "platform_sample_tick",
                    JcsValue::Number(self.platform_sample_tick),
                ),
                (
                    "quantized_value",
                    JcsValue::Array(
                        self.quantized_value
                            .iter()
                            .map(|value| string(format!("{:04x}", *value as u16)))
                            .collect(),
                    ),
                ),
                (
                    "schema_version",
                    JcsValue::Number(u64::from(self.schema_version)),
                ),
                ("source_sequence", JcsValue::Number(self.source_sequence)),
            ])),
        )
    }
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum PlatformEventKindV1 {
    Control,
    FocusChanged,
    SuspendRequested,
    ResumeRequested,
    CloseRequested,
    DeviceConnected,
    DeviceDisconnected,
    PresentationDeviceLost,
    PresentationDeviceRestored,
    CapabilityChanged,
    FatalHostFault,
}

impl PlatformEventKindV1 {
    const fn token(self) -> &'static str {
        match self {
            Self::Control => "Control",
            Self::FocusChanged => "FocusChanged",
            Self::SuspendRequested => "SuspendRequested",
            Self::ResumeRequested => "ResumeRequested",
            Self::CloseRequested => "CloseRequested",
            Self::DeviceConnected => "DeviceConnected",
            Self::DeviceDisconnected => "DeviceDisconnected",
            Self::PresentationDeviceLost => "PresentationDeviceLost",
            Self::PresentationDeviceRestored => "PresentationDeviceRestored",
            Self::CapabilityChanged => "CapabilityChanged",
            Self::FatalHostFault => "FatalHostFault",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PlatformEventPayloadV1 {
    Control(NormalizedControlEventV1),
    FocusChanged {
        focused: bool,
    },
    WindowExtentChanged {
        width: u32,
        height: u32,
    },
    Device {
        device_class: SchemaId,
        device_instance_nonce: PersistentId,
    },
    Reason {
        reason: SchemaId,
    },
}

impl PlatformEventPayloadV1 {
    fn value(&self) -> JcsValue {
        match self {
            Self::Control(control) => object([
                (
                    "control_sample_id",
                    string(control.control_sample_id.to_hex()),
                ),
                ("payload_kind", string("Control")),
            ]),
            Self::FocusChanged { focused } => object([
                ("focused", string(if *focused { "true" } else { "false" })),
                ("payload_kind", string("FocusChanged")),
            ]),
            Self::WindowExtentChanged { width, height } => object([
                ("height", JcsValue::Number(u64::from(*height))),
                ("payload_kind", string("WindowExtentChanged")),
                ("width", JcsValue::Number(u64::from(*width))),
            ]),
            Self::Device {
                device_class,
                device_instance_nonce,
            } => object([
                ("device_class", string(device_class.as_str())),
                (
                    "device_instance_nonce",
                    string(hex_bytes(device_instance_nonce.as_bytes())),
                ),
                ("payload_kind", string("Device")),
            ]),
            Self::Reason { reason } => object([
                ("payload_kind", string("Reason")),
                ("reason", string(reason.as_str())),
            ]),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PlatformEventV1 {
    pub schema_version: u32,
    pub platform_event_id: ContentHash,
    pub host_instance_id: PersistentId,
    pub source_class: SchemaId,
    pub source_sequence: u64,
    pub platform_sample_tick: u64,
    pub kind: PlatformEventKindV1,
    pub payload: PlatformEventPayloadV1,
    pub capability_set_hash: ContentHash,
}

impl PlatformEventV1 {
    pub fn new(
        host_instance_id: PersistentId,
        source_class: SchemaId,
        source_sequence: u64,
        platform_sample_tick: u64,
        kind: PlatformEventKindV1,
        payload: PlatformEventPayloadV1,
        capability_set_hash: ContentHash,
    ) -> Result<Self, PlatformContractError> {
        validate_kind_payload(kind, &payload)?;
        let mut value = Self {
            schema_version: PLATFORM_EVENT_SCHEMA_VERSION,
            platform_event_id: ContentHash::default(),
            host_instance_id,
            source_class,
            source_sequence,
            platform_sample_tick,
            kind,
            payload,
            capability_set_hash,
        };
        value.platform_event_id = value.computed_id();
        Ok(value)
    }

    pub fn validate(&self) -> Result<(), PlatformContractError> {
        if self.schema_version != PLATFORM_EVENT_SCHEMA_VERSION {
            return Err(PlatformContractError::UnsupportedVersion);
        }
        validate_kind_payload(self.kind, &self.payload)?;
        if let PlatformEventPayloadV1::Control(control) = &self.payload {
            control.validate()?;
            if control.platform_sample_tick != self.platform_sample_tick
                || control.source_sequence != self.source_sequence
            {
                return Err(PlatformContractError::KindPayloadMismatch);
            }
        }
        if self.computed_id() != self.platform_event_id {
            return Err(PlatformContractError::HashMismatch);
        }
        Ok(())
    }

    fn computed_id(&self) -> ContentHash {
        domain_hash(
            "nextengine.platform-event.v1",
            &encode_canonical_jcs(&object([
                (
                    "capability_set_hash",
                    string(self.capability_set_hash.to_hex()),
                ),
                (
                    "host_instance_id",
                    string(hex_bytes(self.host_instance_id.as_bytes())),
                ),
                ("kind", string(self.kind.token())),
                ("payload", self.payload.value()),
                (
                    "platform_sample_tick",
                    JcsValue::Number(self.platform_sample_tick),
                ),
                (
                    "schema_version",
                    JcsValue::Number(u64::from(self.schema_version)),
                ),
                ("source_class", string(self.source_class.as_str())),
                ("source_sequence", JcsValue::Number(self.source_sequence)),
            ])),
        )
    }
}

fn validate_kind_payload(
    kind: PlatformEventKindV1,
    payload: &PlatformEventPayloadV1,
) -> Result<(), PlatformContractError> {
    let valid = matches!(
        (kind, payload),
        (
            PlatformEventKindV1::Control,
            PlatformEventPayloadV1::Control(_)
        ) | (
            PlatformEventKindV1::FocusChanged,
            PlatformEventPayloadV1::FocusChanged { .. }
        ) | (
            PlatformEventKindV1::CapabilityChanged,
            PlatformEventPayloadV1::WindowExtentChanged { .. }
        ) | (
            PlatformEventKindV1::DeviceConnected | PlatformEventKindV1::DeviceDisconnected,
            PlatformEventPayloadV1::Device { .. }
        ) | (
            PlatformEventKindV1::SuspendRequested
                | PlatformEventKindV1::ResumeRequested
                | PlatformEventKindV1::CloseRequested
                | PlatformEventKindV1::PresentationDeviceLost
                | PlatformEventKindV1::PresentationDeviceRestored
                | PlatformEventKindV1::FatalHostFault,
            PlatformEventPayloadV1::Reason { .. }
        )
    );
    if valid {
        Ok(())
    } else {
        Err(PlatformContractError::KindPayloadMismatch)
    }
}
