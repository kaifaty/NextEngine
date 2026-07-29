use super::*;

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
#[repr(u8)]
pub enum PresentationTargetKindV1 {
    None = 0,
    Interactive = 1,
    DisplaylessOffscreen = 2,
}

impl PresentationTargetKindV1 {
    pub(crate) const fn token(self) -> &'static str {
        match self {
            Self::None => "None",
            Self::Interactive => "Interactive",
            Self::DisplaylessOffscreen => "DisplaylessOffscreen",
        }
    }

    pub(crate) fn parse(value: &str) -> Result<Self, PlatformContractError> {
        match value {
            "None" => Ok(Self::None),
            "Interactive" => Ok(Self::Interactive),
            "DisplaylessOffscreen" => Ok(Self::DisplaylessOffscreen),
            _ => Err(PlatformContractError::KindPayloadMismatch),
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
