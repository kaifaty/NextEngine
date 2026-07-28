use std::collections::BTreeMap;
use std::error::Error;
use std::fmt::{Display, Formatter};

use crate::manifest_jcs::{JcsValue, encode_canonical_jcs};
use crate::{CapabilityId, ContentHash, PersistentId, SchemaId, domain_hash};

pub const PLATFORM_CAPABILITY_SET_SCHEMA_VERSION: u32 = 1;
pub const PLATFORM_TIMEBASE_SCHEMA_VERSION: u32 = 1;
pub const PLATFORM_EVENT_SCHEMA_VERSION: u32 = 1;
pub const NORMALIZED_CONTROL_EVENT_SCHEMA_VERSION: u32 = 1;
pub const PLATFORM_MAX_CAPABILITIES: usize = 128;
pub const PLATFORM_MAX_CONTROL_COMPONENTS: usize = 8;
pub const PLATFORM_MAX_MODIFIERS: usize = 16;

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
#[repr(u8)]
pub enum PresentationTargetKindV1 {
    None = 0,
    Interactive = 1,
    DisplaylessOffscreen = 2,
}

impl PresentationTargetKindV1 {
    const fn token(self) -> &'static str {
        match self {
            Self::None => "None",
            Self::Interactive => "Interactive",
            Self::DisplaylessOffscreen => "DisplaylessOffscreen",
        }
    }
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct NormalizedPlatformCapabilityV1 {
    pub capability_id: CapabilityId,
    pub limit: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PlatformCapabilitySetV1 {
    pub schema_version: u32,
    pub capability_set_id: SchemaId,
    pub host_target_profile_id: SchemaId,
    pub normalized_capabilities: Vec<NormalizedPlatformCapabilityV1>,
    pub required_capability_failures: Vec<CapabilityId>,
    pub presentation_target_kinds: Vec<PresentationTargetKindV1>,
    pub input_classes: Vec<SchemaId>,
    pub timebase_profile_id: SchemaId,
    pub canonical_hash: ContentHash,
}

impl PlatformCapabilitySetV1 {
    pub fn new(
        capability_set_id: SchemaId,
        host_target_profile_id: SchemaId,
        mut normalized_capabilities: Vec<NormalizedPlatformCapabilityV1>,
        mut required_capability_failures: Vec<CapabilityId>,
        mut presentation_target_kinds: Vec<PresentationTargetKindV1>,
        mut input_classes: Vec<SchemaId>,
        timebase_profile_id: SchemaId,
    ) -> Result<Self, PlatformContractError> {
        normalized_capabilities.sort();
        required_capability_failures.sort();
        presentation_target_kinds.sort();
        input_classes.sort();
        enforce_unique(&normalized_capabilities)?;
        enforce_capability_identities_unique(&normalized_capabilities)?;
        enforce_unique(&required_capability_failures)?;
        enforce_unique(&presentation_target_kinds)?;
        enforce_unique(&input_classes)?;
        enforce_limit(normalized_capabilities.len(), PLATFORM_MAX_CAPABILITIES)?;
        enforce_limit(
            required_capability_failures.len(),
            PLATFORM_MAX_CAPABILITIES,
        )?;
        let mut value = Self {
            schema_version: PLATFORM_CAPABILITY_SET_SCHEMA_VERSION,
            capability_set_id,
            host_target_profile_id,
            normalized_capabilities,
            required_capability_failures,
            presentation_target_kinds,
            input_classes,
            timebase_profile_id,
            canonical_hash: ContentHash::default(),
        };
        value.canonical_hash = value.computed_hash();
        Ok(value)
    }

    pub fn validate(&self) -> Result<(), PlatformContractError> {
        if self.schema_version != PLATFORM_CAPABILITY_SET_SCHEMA_VERSION {
            return Err(PlatformContractError::UnsupportedVersion);
        }
        enforce_canonical(&self.normalized_capabilities)?;
        enforce_capability_identities_unique(&self.normalized_capabilities)?;
        enforce_canonical(&self.required_capability_failures)?;
        enforce_canonical(&self.presentation_target_kinds)?;
        enforce_canonical(&self.input_classes)?;
        enforce_limit(
            self.normalized_capabilities.len(),
            PLATFORM_MAX_CAPABILITIES,
        )?;
        enforce_limit(
            self.required_capability_failures.len(),
            PLATFORM_MAX_CAPABILITIES,
        )?;
        if self.computed_hash() != self.canonical_hash {
            return Err(PlatformContractError::HashMismatch);
        }
        Ok(())
    }

    fn computed_hash(&self) -> ContentHash {
        domain_hash(
            "nextengine.platform-capability-set.v1",
            &encode_canonical_jcs(&self.body_value()),
        )
    }

    fn body_value(&self) -> JcsValue {
        object([
            ("capability_set_id", string(self.capability_set_id.as_str())),
            (
                "host_target_profile_id",
                string(self.host_target_profile_id.as_str()),
            ),
            (
                "input_classes",
                JcsValue::Array(
                    self.input_classes
                        .iter()
                        .map(|value| string(value.as_str()))
                        .collect(),
                ),
            ),
            (
                "normalized_capabilities",
                JcsValue::Array(
                    self.normalized_capabilities
                        .iter()
                        .map(|capability| {
                            object([
                                ("capability_id", string(capability.capability_id.as_str())),
                                ("limit", JcsValue::Number(capability.limit)),
                            ])
                        })
                        .collect(),
                ),
            ),
            (
                "presentation_target_kinds",
                JcsValue::Array(
                    self.presentation_target_kinds
                        .iter()
                        .map(|kind| string(kind.token()))
                        .collect(),
                ),
            ),
            (
                "required_capability_failures",
                JcsValue::Array(
                    self.required_capability_failures
                        .iter()
                        .map(|value| string(value.as_str()))
                        .collect(),
                ),
            ),
            (
                "schema_version",
                JcsValue::Number(u64::from(self.schema_version)),
            ),
            (
                "timebase_profile_id",
                string(self.timebase_profile_id.as_str()),
            ),
        ])
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PlatformTimebaseWrapPolicyV1 {
    Reject,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PlatformTimebaseV1 {
    pub schema_version: u32,
    pub timebase_id: SchemaId,
    pub epoch_id: ContentHash,
    pub ticks_per_second_num: u64,
    pub ticks_per_second_den: u64,
    pub maximum_sample_delta: u64,
    pub wrap_policy: PlatformTimebaseWrapPolicyV1,
    pub canonical_hash: ContentHash,
}

impl PlatformTimebaseV1 {
    pub fn new(
        timebase_id: SchemaId,
        epoch_id: ContentHash,
        ticks_per_second_num: u64,
        ticks_per_second_den: u64,
        maximum_sample_delta: u64,
    ) -> Result<Self, PlatformContractError> {
        if ticks_per_second_num == 0 || ticks_per_second_den == 0 || maximum_sample_delta == 0 {
            return Err(PlatformContractError::InvalidTimebase);
        }
        let mut value = Self {
            schema_version: PLATFORM_TIMEBASE_SCHEMA_VERSION,
            timebase_id,
            epoch_id,
            ticks_per_second_num,
            ticks_per_second_den,
            maximum_sample_delta,
            wrap_policy: PlatformTimebaseWrapPolicyV1::Reject,
            canonical_hash: ContentHash::default(),
        };
        value.canonical_hash = value.computed_hash();
        Ok(value)
    }

    pub fn validate(&self) -> Result<(), PlatformContractError> {
        if self.schema_version != PLATFORM_TIMEBASE_SCHEMA_VERSION {
            return Err(PlatformContractError::UnsupportedVersion);
        }
        if self.ticks_per_second_num == 0
            || self.ticks_per_second_den == 0
            || self.maximum_sample_delta == 0
            || self.computed_hash() != self.canonical_hash
        {
            return Err(PlatformContractError::InvalidTimebase);
        }
        Ok(())
    }

    fn computed_hash(&self) -> ContentHash {
        domain_hash(
            "nextengine.platform-timebase.v1",
            &encode_canonical_jcs(&object([
                ("epoch_id", string(self.epoch_id.to_hex())),
                (
                    "maximum_sample_delta",
                    JcsValue::Number(self.maximum_sample_delta),
                ),
                (
                    "schema_version",
                    JcsValue::Number(u64::from(self.schema_version)),
                ),
                (
                    "ticks_per_second_den",
                    JcsValue::Number(self.ticks_per_second_den),
                ),
                (
                    "ticks_per_second_num",
                    JcsValue::Number(self.ticks_per_second_num),
                ),
                ("timebase_id", string(self.timebase_id.as_str())),
                ("wrap_policy", string("Reject")),
            ])),
        )
    }
}

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
    FocusChanged { focused: bool },
    WindowExtentChanged { width: u32, height: u32 },
    Reason { reason: SchemaId },
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

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum PlatformContractError {
    UnsupportedVersion,
    DuplicateIdentity,
    NonCanonicalOrder,
    LimitExceeded { actual: usize, limit: usize },
    HashMismatch,
    InvalidTimebase,
    KindPayloadMismatch,
}

impl Display for PlatformContractError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnsupportedVersion => formatter.write_str("platform schema version unsupported"),
            Self::DuplicateIdentity => formatter.write_str("platform identity duplicated"),
            Self::NonCanonicalOrder => formatter.write_str("platform collection is not canonical"),
            Self::LimitExceeded { actual, limit } => {
                write!(formatter, "platform limit exceeded: {actual} > {limit}")
            }
            Self::HashMismatch => formatter.write_str("platform canonical hash mismatch"),
            Self::InvalidTimebase => formatter.write_str("platform timebase invalid"),
            Self::KindPayloadMismatch => {
                formatter.write_str("platform event kind/payload mismatch")
            }
        }
    }
}

impl Error for PlatformContractError {}

fn enforce_unique<T: Ord>(values: &[T]) -> Result<(), PlatformContractError> {
    if values.windows(2).any(|pair| pair[0] == pair[1]) {
        Err(PlatformContractError::DuplicateIdentity)
    } else {
        Ok(())
    }
}

fn enforce_capability_identities_unique(
    values: &[NormalizedPlatformCapabilityV1],
) -> Result<(), PlatformContractError> {
    if values
        .windows(2)
        .any(|pair| pair[0].capability_id == pair[1].capability_id)
    {
        Err(PlatformContractError::DuplicateIdentity)
    } else {
        Ok(())
    }
}

fn enforce_canonical<T: Ord>(values: &[T]) -> Result<(), PlatformContractError> {
    if values.windows(2).any(|pair| pair[0] >= pair[1]) {
        Err(PlatformContractError::NonCanonicalOrder)
    } else {
        Ok(())
    }
}

fn enforce_limit(actual: usize, limit: usize) -> Result<(), PlatformContractError> {
    if actual > limit {
        Err(PlatformContractError::LimitExceeded { actual, limit })
    } else {
        Ok(())
    }
}

fn object<const N: usize>(entries: [(&str, JcsValue); N]) -> JcsValue {
    JcsValue::Object(
        entries
            .into_iter()
            .map(|(key, value)| (key.to_owned(), value))
            .collect::<BTreeMap<_, _>>(),
    )
}

fn string(value: impl Into<String>) -> JcsValue {
    JcsValue::String(value.into())
}

fn hex_bytes(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut value = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        value.push(char::from(HEX[usize::from(*byte >> 4)]));
        value.push(char::from(HEX[usize::from(*byte & 0x0f)]));
    }
    value
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn capability_set_is_order_independent_and_hash_bound() {
        let a = CapabilityId::new("nextengine.platform.capability.a").expect("ID");
        let z = CapabilityId::new("nextengine.platform.capability.z").expect("ID");
        let first = capability_set(vec![z.clone(), a.clone()]);
        let second = capability_set(vec![a, z]);
        assert_eq!(first, second);

        let mut corrupt = first;
        corrupt.normalized_capabilities[0].limit += 1;
        assert_eq!(corrupt.validate(), Err(PlatformContractError::HashMismatch));
    }

    #[test]
    fn duplicate_capability_identity_is_rejected_even_when_limits_differ() {
        let capability_id =
            CapabilityId::new("nextengine.platform.capability.duplicate").expect("ID");
        assert_eq!(
            PlatformCapabilitySetV1::new(
                SchemaId::new("nextengine.test.duplicate-capability-set").expect("ID"),
                SchemaId::new("nextengine.test.host").expect("ID"),
                vec![
                    NormalizedPlatformCapabilityV1 {
                        capability_id: capability_id.clone(),
                        limit: 1,
                    },
                    NormalizedPlatformCapabilityV1 {
                        capability_id,
                        limit: 2,
                    },
                ],
                Vec::new(),
                vec![PresentationTargetKindV1::Interactive],
                Vec::new(),
                SchemaId::new("nextengine.timebase.test").expect("ID"),
            ),
            Err(PlatformContractError::DuplicateIdentity)
        );
    }

    #[test]
    fn control_and_event_kind_are_closed_and_hash_bound() {
        let capabilities = capability_set(Vec::new());
        let control = NormalizedControlEventV1::new(
            SchemaId::new("nextengine.input.keyboard").expect("ID"),
            PersistentId::from_bytes([1; 16]),
            SchemaId::new("nextengine.input.key.forward").expect("ID"),
            NormalizedControlPhaseV1::Started,
            vec![i16::MAX],
            Vec::new(),
            7,
            3,
        )
        .expect("control");
        let event = PlatformEventV1::new(
            PersistentId::from_bytes([2; 16]),
            SchemaId::new("nextengine.platform.source.keyboard").expect("ID"),
            3,
            7,
            PlatformEventKindV1::Control,
            PlatformEventPayloadV1::Control(control),
            capabilities.canonical_hash,
        )
        .expect("event");
        event.validate().expect("valid event");

        assert_eq!(
            PlatformEventV1::new(
                PersistentId::from_bytes([2; 16]),
                SchemaId::new("nextengine.platform.source.window").expect("ID"),
                4,
                8,
                PlatformEventKindV1::CloseRequested,
                PlatformEventPayloadV1::FocusChanged { focused: false },
                capabilities.canonical_hash,
            ),
            Err(PlatformContractError::KindPayloadMismatch)
        );
    }

    fn capability_set(capabilities: Vec<CapabilityId>) -> PlatformCapabilitySetV1 {
        PlatformCapabilitySetV1::new(
            SchemaId::new("nextengine.test.platform-capability-set").expect("ID"),
            SchemaId::new("nextengine.test.host").expect("ID"),
            capabilities
                .into_iter()
                .map(|capability_id| NormalizedPlatformCapabilityV1 {
                    capability_id,
                    limit: 1,
                })
                .collect(),
            Vec::new(),
            vec![PresentationTargetKindV1::Interactive],
            vec![SchemaId::new("nextengine.input.keyboard").expect("ID")],
            SchemaId::new("nextengine.timebase.test").expect("ID"),
        )
        .expect("capabilities")
    }
}
