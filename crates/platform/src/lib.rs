#![forbid(unsafe_code)]

use std::collections::{BTreeMap, VecDeque};
use std::error::Error;
use std::fmt::{Display, Formatter};

use next_contracts::{
    CapabilityId, NormalizedPlatformCapabilityV1, PersistentId, PlatformCapabilitySetV1,
    PlatformContractError, PlatformEventV1, PlatformTimebaseV1, PresentationTargetKindV1, SchemaId,
    domain_hash,
};

pub const PLATFORM_EVENT_BATCH_LIMIT: usize = 4_096;

pub trait PlatformHost {
    fn capability_set(&self) -> &PlatformCapabilitySetV1;
    fn timebase(&self) -> &PlatformTimebaseV1;
    fn presentation_target_kind(&self) -> PresentationTargetKindV1;
    fn drawable_extent(&self) -> [u32; 2];
    fn poll_events(&mut self) -> Result<Vec<PlatformEventV1>, PlatformHostError>;
}

#[derive(Clone, Debug)]
pub struct ReferencePlatformHost {
    capabilities: PlatformCapabilitySetV1,
    timebase: PlatformTimebaseV1,
    target_kind: PresentationTargetKindV1,
    drawable_extent: [u32; 2],
    queued_events: VecDeque<PlatformEventV1>,
    last_sequences: BTreeMap<(PersistentId, SchemaId), u64>,
}

impl ReferencePlatformHost {
    pub fn interactive() -> Result<Self, PlatformHostError> {
        Self::new(PresentationTargetKindV1::Interactive, [960, 540])
    }

    pub fn headless() -> Result<Self, PlatformHostError> {
        Self::new(PresentationTargetKindV1::None, [0, 0])
    }

    pub fn new(
        target_kind: PresentationTargetKindV1,
        drawable_extent: [u32; 2],
    ) -> Result<Self, PlatformHostError> {
        if target_kind == PresentationTargetKindV1::Interactive
            && (drawable_extent[0] == 0 || drawable_extent[1] == 0)
        {
            return Err(PlatformHostError::InvalidExtent);
        }
        if target_kind == PresentationTargetKindV1::None && drawable_extent != [0, 0] {
            return Err(PlatformHostError::ForbiddenTarget);
        }
        let target_limit = match target_kind {
            PresentationTargetKindV1::None => 0,
            PresentationTargetKindV1::Interactive
            | PresentationTargetKindV1::DisplaylessOffscreen => 1,
        };
        let capabilities = PlatformCapabilitySetV1::new(
            SchemaId::new("nextengine.platform.reference-capabilities")?,
            SchemaId::new("nextengine.platform.reference-host")?,
            vec![NormalizedPlatformCapabilityV1 {
                capability_id: CapabilityId::new("nextengine.platform.presentation-target")?,
                limit: target_limit,
            }],
            Vec::new(),
            vec![target_kind],
            vec![SchemaId::new("nextengine.input.keyboard")?],
            SchemaId::new("nextengine.platform.timebase.logical")?,
        )?;
        let timebase = PlatformTimebaseV1::new(
            SchemaId::new("nextengine.platform.timebase.logical")?,
            domain_hash("nextengine.platform.reference-epoch", b"epoch"),
            1_000,
            1,
            1_000,
        )?;
        Ok(Self {
            capabilities,
            timebase,
            target_kind,
            drawable_extent,
            queued_events: VecDeque::new(),
            last_sequences: BTreeMap::new(),
        })
    }

    pub fn inject_event(&mut self, event: PlatformEventV1) -> Result<(), PlatformHostError> {
        event.validate()?;
        if event.capability_set_hash != self.capabilities.canonical_hash {
            return Err(PlatformHostError::CapabilityMismatch);
        }
        if self.queued_events.len() >= PLATFORM_EVENT_BATCH_LIMIT {
            return Err(PlatformHostError::BatchLimitExceeded);
        }
        self.queued_events.push_back(event);
        Ok(())
    }
}

impl PlatformHost for ReferencePlatformHost {
    fn capability_set(&self) -> &PlatformCapabilitySetV1 {
        &self.capabilities
    }

    fn timebase(&self) -> &PlatformTimebaseV1 {
        &self.timebase
    }

    fn presentation_target_kind(&self) -> PresentationTargetKindV1 {
        self.target_kind
    }

    fn drawable_extent(&self) -> [u32; 2] {
        self.drawable_extent
    }

    fn poll_events(&mut self) -> Result<Vec<PlatformEventV1>, PlatformHostError> {
        let mut events: Vec<_> = self.queued_events.drain(..).collect();
        events.sort_by(|left, right| {
            (
                left.host_instance_id,
                left.source_class.as_str(),
                left.source_sequence,
                left.platform_event_id,
            )
                .cmp(&(
                    right.host_instance_id,
                    right.source_class.as_str(),
                    right.source_sequence,
                    right.platform_event_id,
                ))
        });
        let mut deduplicated = Vec::with_capacity(events.len());
        let mut identities = BTreeMap::new();
        let mut staged_sequences = self.last_sequences.clone();
        for event in events {
            if let Some(prior) = identities.get(&event.platform_event_id) {
                if prior != &event {
                    return Err(PlatformHostError::IdentityCollision);
                }
                continue;
            }
            identities.insert(event.platform_event_id, event.clone());
            let sequence_key = (event.host_instance_id, event.source_class.clone());
            let expected = staged_sequences
                .get(&sequence_key)
                .map_or(0, |value| value.saturating_add(1));
            if event.source_sequence != expected {
                return Err(PlatformHostError::SequenceGap {
                    expected,
                    actual: event.source_sequence,
                });
            }
            staged_sequences.insert(sequence_key, event.source_sequence);
            deduplicated.push(event);
        }
        self.last_sequences = staged_sequences;
        Ok(deduplicated)
    }
}

#[derive(Debug)]
#[non_exhaustive]
pub enum PlatformHostError {
    Contract(PlatformContractError),
    Identifier(next_contracts::IdentifierError),
    ForbiddenTarget,
    InvalidExtent,
    CapabilityMismatch,
    BatchLimitExceeded,
    IdentityCollision,
    SequenceGap { expected: u64, actual: u64 },
}

impl Display for PlatformHostError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Contract(error) => write!(formatter, "{error}"),
            Self::Identifier(error) => write!(formatter, "{error}"),
            Self::ForbiddenTarget => formatter.write_str("platform presentation target forbidden"),
            Self::InvalidExtent => formatter.write_str("platform drawable extent invalid"),
            Self::CapabilityMismatch => formatter.write_str("platform capability hash mismatch"),
            Self::BatchLimitExceeded => formatter.write_str("platform event batch limit exceeded"),
            Self::IdentityCollision => formatter.write_str("platform event identity collision"),
            Self::SequenceGap { expected, actual } => write!(
                formatter,
                "platform event sequence gap: expected {expected}, got {actual}"
            ),
        }
    }
}

impl Error for PlatformHostError {}

impl From<PlatformContractError> for PlatformHostError {
    fn from(error: PlatformContractError) -> Self {
        Self::Contract(error)
    }
}

impl From<next_contracts::IdentifierError> for PlatformHostError {
    fn from(error: next_contracts::IdentifierError) -> Self {
        Self::Identifier(error)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use next_contracts::{PlatformEventKindV1, PlatformEventPayloadV1};

    #[test]
    fn event_arrival_permutations_close_to_one_order() {
        let mut forward = ReferencePlatformHost::interactive().expect("host");
        let mut reverse = forward.clone();
        let events = [
            event(&forward, 0, "nextengine.platform.source.a"),
            event(&forward, 0, "nextengine.platform.source.b"),
        ];
        for event in events.clone() {
            forward.inject_event(event).expect("inject");
        }
        for event in events.into_iter().rev() {
            reverse.inject_event(event).expect("inject");
        }
        assert_eq!(
            forward.poll_events().expect("forward"),
            reverse.poll_events().expect("reverse")
        );
    }

    #[test]
    fn headless_has_no_extent_or_interactive_target() {
        let host = ReferencePlatformHost::headless().expect("host");
        assert_eq!(host.drawable_extent(), [0, 0]);
        assert_eq!(
            host.presentation_target_kind(),
            PresentationTargetKindV1::None
        );
    }

    #[test]
    fn rejected_batch_does_not_advance_sequence_authority() {
        let mut host = ReferencePlatformHost::interactive().expect("host");
        let first = event(&host, 0, "nextengine.platform.source.a");
        let gap = event(&host, 2, "nextengine.platform.source.a");
        host.inject_event(first).expect("first");
        host.inject_event(gap).expect("gap");
        assert!(matches!(
            host.poll_events(),
            Err(PlatformHostError::SequenceGap {
                expected: 1,
                actual: 2
            })
        ));

        let retry = event(&host, 0, "nextengine.platform.source.a");
        host.inject_event(retry).expect("retry");
        assert_eq!(host.poll_events().expect("retry accepted").len(), 1);
    }

    fn event(host: &ReferencePlatformHost, sequence: u64, source: &str) -> PlatformEventV1 {
        PlatformEventV1::new(
            PersistentId::from_bytes([3; 16]),
            SchemaId::new(source).expect("source"),
            sequence,
            sequence,
            PlatformEventKindV1::FocusChanged,
            PlatformEventPayloadV1::FocusChanged { focused: true },
            host.capability_set().canonical_hash,
        )
        .expect("event")
    }
}
