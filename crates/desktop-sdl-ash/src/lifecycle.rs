use std::collections::BTreeMap;

use next_contracts::canonical::sha256;
use next_contracts::ids::{CapabilityId, ContentHash, PersistentId, SchemaId};
use next_contracts::platform::{
    NormalizedControlEventV1, NormalizedControlPhaseV1, NormalizedPlatformCapabilityV1,
    PlatformCapabilitySetV1, PlatformEventKindV1, PlatformEventPayloadV1, PlatformEventV1,
    PlatformTimebaseV1, PresentationTargetKindV1,
};

use crate::DesktopAdapterError;

pub(super) const DESKTOP_EVENT_BATCH_LIMIT: usize = 4_096;

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(super) enum DesktopEventSource {
    Graphics,
    Keyboard,
    Window,
}

impl DesktopEventSource {
    const fn schema_id(self) -> &'static str {
        match self {
            Self::Graphics => "nextengine.platform.source.graphics",
            Self::Keyboard => "nextengine.platform.source.keyboard",
            Self::Window => "nextengine.platform.source.window",
        }
    }
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(super) enum DesktopObservationKind {
    CloseRequested {
        reason: &'static str,
    },
    Control {
        control_path: &'static str,
        device_instance_nonce: PersistentId,
        modifier_set: Vec<&'static str>,
        phase: NormalizedControlPhaseV1,
        quantized_value: Vec<i16>,
    },
    FocusChanged {
        focused: bool,
    },
    PresentationDeviceLost {
        reason: &'static str,
    },
    PresentationDeviceRestored {
        reason: &'static str,
    },
    ResumeRequested {
        reason: &'static str,
    },
    SuspendRequested {
        reason: &'static str,
    },
    WindowExtentChanged {
        height: u32,
        width: u32,
    },
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(super) struct DesktopObservation {
    pub(super) source: DesktopEventSource,
    pub(super) platform_sample_tick: u64,
    pub(super) kind: DesktopObservationKind,
}

#[derive(Clone, Debug)]
pub(super) struct DesktopEventNormalizer {
    capabilities: PlatformCapabilitySetV1,
    host_instance_id: PersistentId,
    last_sample_ticks: BTreeMap<DesktopEventSource, u64>,
    next_sequences: BTreeMap<DesktopEventSource, u64>,
    timebase: PlatformTimebaseV1,
}

impl DesktopEventNormalizer {
    pub(super) fn new(host_instance_id: PersistentId) -> Result<Self, DesktopAdapterError> {
        let timebase_id = SchemaId::new("nextengine.platform.timebase.sdl-nanoseconds")?;
        let capabilities = PlatformCapabilitySetV1::new(
            SchemaId::new("nextengine.platform.desktop-b0-capabilities")?,
            SchemaId::new(host_target_profile_id())?,
            vec![
                NormalizedPlatformCapabilityV1 {
                    capability_id: CapabilityId::new(
                        "nextengine.platform.presentation-device-recovery",
                    )?,
                    limit: 1,
                },
                NormalizedPlatformCapabilityV1 {
                    capability_id: CapabilityId::new("nextengine.platform.presentation-target")?,
                    limit: 1,
                },
                NormalizedPlatformCapabilityV1 {
                    capability_id: CapabilityId::new("nextengine.platform.window-fullscreen")?,
                    limit: 1,
                },
                NormalizedPlatformCapabilityV1 {
                    capability_id: CapabilityId::new("nextengine.platform.window-resize")?,
                    limit: 1,
                },
            ],
            Vec::new(),
            vec![PresentationTargetKindV1::Interactive],
            vec![SchemaId::new("nextengine.input.keyboard")?],
            timebase_id.clone(),
        )?;
        let timebase = PlatformTimebaseV1::new(
            timebase_id,
            ContentHash::from_bytes(sha256(b"nextengine.desktop-sdl-timebase-epoch.v1")),
            1_000_000_000,
            1,
            u64::MAX,
        )?;
        Ok(Self {
            capabilities,
            host_instance_id,
            last_sample_ticks: BTreeMap::new(),
            next_sequences: BTreeMap::new(),
            timebase,
        })
    }

    pub(super) fn capability_set_hash(&self) -> ContentHash {
        self.capabilities.canonical_hash
    }

    pub(super) fn timebase_hash(&self) -> ContentHash {
        self.timebase.canonical_hash
    }

    pub(super) fn normalize(
        &mut self,
        mut observations: Vec<DesktopObservation>,
    ) -> Result<Vec<PlatformEventV1>, DesktopAdapterError> {
        if observations.len() > DESKTOP_EVENT_BATCH_LIMIT {
            return Err(DesktopAdapterError::EventBatchLimitExceeded);
        }
        observations.sort();
        observations.dedup();

        let mut events = Vec::with_capacity(observations.len());
        let mut staged_sample_ticks = self.last_sample_ticks.clone();
        let mut staged_sequences = self.next_sequences.clone();
        for observation in observations {
            if let Some(previous) = staged_sample_ticks.get(&observation.source)
                && observation.platform_sample_tick < *previous
            {
                return Err(DesktopAdapterError::TimebaseRegression {
                    previous: *previous,
                    actual: observation.platform_sample_tick,
                });
            }
            let source_sequence = *staged_sequences.entry(observation.source).or_insert(0);
            let source = observation.source;
            let sample_tick = observation.platform_sample_tick;
            let event = self.platform_event(observation, source_sequence)?;
            staged_sequences.insert(
                source,
                source_sequence
                    .checked_add(1)
                    .ok_or(DesktopAdapterError::CounterOverflow)?,
            );
            staged_sample_ticks.insert(source, sample_tick);
            events.push(event);
        }
        self.last_sample_ticks = staged_sample_ticks;
        self.next_sequences = staged_sequences;
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
        Ok(events)
    }

    fn platform_event(
        &self,
        observation: DesktopObservation,
        source_sequence: u64,
    ) -> Result<PlatformEventV1, DesktopAdapterError> {
        let source = observation.source;
        let sample_tick = observation.platform_sample_tick;
        let (kind, payload) = match observation.kind {
            DesktopObservationKind::CloseRequested { reason } => {
                (PlatformEventKindV1::CloseRequested, reason_payload(reason)?)
            }
            DesktopObservationKind::Control {
                control_path,
                device_instance_nonce,
                modifier_set,
                phase,
                quantized_value,
            } => {
                let control = NormalizedControlEventV1::new(
                    SchemaId::new("nextengine.input.keyboard")?,
                    device_instance_nonce,
                    SchemaId::new(control_path)?,
                    phase,
                    quantized_value,
                    modifier_set
                        .into_iter()
                        .map(SchemaId::new)
                        .collect::<Result<Vec<_>, _>>()?,
                    sample_tick,
                    source_sequence,
                )?;
                (
                    PlatformEventKindV1::Control,
                    PlatformEventPayloadV1::Control(control),
                )
            }
            DesktopObservationKind::FocusChanged { focused } => (
                PlatformEventKindV1::FocusChanged,
                PlatformEventPayloadV1::FocusChanged { focused },
            ),
            DesktopObservationKind::PresentationDeviceLost { reason } => (
                PlatformEventKindV1::PresentationDeviceLost,
                reason_payload(reason)?,
            ),
            DesktopObservationKind::PresentationDeviceRestored { reason } => (
                PlatformEventKindV1::PresentationDeviceRestored,
                reason_payload(reason)?,
            ),
            DesktopObservationKind::ResumeRequested { reason } => (
                PlatformEventKindV1::ResumeRequested,
                reason_payload(reason)?,
            ),
            DesktopObservationKind::SuspendRequested { reason } => (
                PlatformEventKindV1::SuspendRequested,
                reason_payload(reason)?,
            ),
            DesktopObservationKind::WindowExtentChanged { height, width } => (
                PlatformEventKindV1::CapabilityChanged,
                PlatformEventPayloadV1::WindowExtentChanged { width, height },
            ),
        };
        Ok(PlatformEventV1::new(
            self.host_instance_id,
            SchemaId::new(source.schema_id())?,
            source_sequence,
            sample_tick,
            kind,
            payload,
            self.capabilities.canonical_hash,
        )?)
    }
}

fn reason_payload(reason: &'static str) -> Result<PlatformEventPayloadV1, DesktopAdapterError> {
    Ok(PlatformEventPayloadV1::Reason {
        reason: SchemaId::new(reason)?,
    })
}

pub(super) fn keyboard_device_nonce(which: u32) -> PersistentId {
    let mut preimage = Vec::with_capacity(48);
    preimage.extend_from_slice(b"nextengine.desktop-keyboard-device.v1\0");
    preimage.extend_from_slice(&which.to_le_bytes());
    let digest = sha256(&preimage);
    let mut bytes = [0; 16];
    bytes.copy_from_slice(&digest[..16]);
    PersistentId::from_bytes(bytes)
}

const fn host_target_profile_id() -> &'static str {
    if cfg!(all(target_arch = "x86_64", target_os = "windows")) {
        "nextengine.platform.host.windows-x86-64"
    } else if cfg!(all(target_arch = "x86_64", target_os = "linux")) {
        "nextengine.platform.host.linux-x86-64"
    } else {
        "nextengine.platform.host.developer-unsupported"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn callback_permutations_normalize_to_identical_platform_events() {
        let observations = vec![
            DesktopObservation {
                source: DesktopEventSource::Window,
                platform_sample_tick: 12,
                kind: DesktopObservationKind::FocusChanged { focused: false },
            },
            DesktopObservation {
                source: DesktopEventSource::Window,
                platform_sample_tick: 11,
                kind: DesktopObservationKind::WindowExtentChanged {
                    width: 1_280,
                    height: 720,
                },
            },
            DesktopObservation {
                source: DesktopEventSource::Keyboard,
                platform_sample_tick: 12,
                kind: DesktopObservationKind::Control {
                    control_path: "nextengine.input.keyboard.w",
                    device_instance_nonce: keyboard_device_nonce(7),
                    modifier_set: Vec::new(),
                    phase: NormalizedControlPhaseV1::Started,
                    quantized_value: vec![i16::MAX],
                },
            },
        ];
        let mut forward =
            DesktopEventNormalizer::new(PersistentId::from_bytes([7; 16])).expect("normalizer");
        let mut reverse = forward.clone();

        let normalized_forward = forward
            .normalize(observations.clone())
            .expect("forward normalization");
        let normalized_reverse = reverse
            .normalize(observations.into_iter().rev().collect())
            .expect("reverse normalization");

        assert_eq!(normalized_forward, normalized_reverse);
        assert!(
            normalized_forward
                .iter()
                .all(|event| event.validate().is_ok())
        );
    }

    #[test]
    fn lifecycle_and_device_recovery_remain_typed_platform_facts() {
        let mut normalizer =
            DesktopEventNormalizer::new(PersistentId::from_bytes([8; 16])).expect("normalizer");
        let events = normalizer
            .normalize(vec![
                DesktopObservation {
                    source: DesktopEventSource::Window,
                    platform_sample_tick: 1,
                    kind: DesktopObservationKind::SuspendRequested {
                        reason: "nextengine.platform.reason.window-minimized",
                    },
                },
                DesktopObservation {
                    source: DesktopEventSource::Window,
                    platform_sample_tick: 2,
                    kind: DesktopObservationKind::ResumeRequested {
                        reason: "nextengine.platform.reason.window-restored",
                    },
                },
                DesktopObservation {
                    source: DesktopEventSource::Graphics,
                    platform_sample_tick: 3,
                    kind: DesktopObservationKind::PresentationDeviceLost {
                        reason: "nextengine.platform.reason.device-lost",
                    },
                },
                DesktopObservation {
                    source: DesktopEventSource::Graphics,
                    platform_sample_tick: 3,
                    kind: DesktopObservationKind::PresentationDeviceRestored {
                        reason: "nextengine.platform.reason.device-restored",
                    },
                },
            ])
            .expect("lifecycle normalization");

        assert_eq!(events.len(), 4);
        assert!(
            events
                .iter()
                .any(|event| { event.kind == PlatformEventKindV1::PresentationDeviceLost })
        );
        assert!(
            events
                .iter()
                .any(|event| { event.kind == PlatformEventKindV1::PresentationDeviceRestored })
        );
        assert!(
            events
                .iter()
                .all(|event| event.capability_set_hash == normalizer.capability_set_hash())
        );
        assert_ne!(normalizer.capability_set_hash(), normalizer.timebase_hash());
    }

    #[test]
    fn event_batch_limit_fails_before_sequence_progress() {
        let mut normalizer =
            DesktopEventNormalizer::new(PersistentId::from_bytes([9; 16])).expect("normalizer");
        let oversized = (0..=DESKTOP_EVENT_BATCH_LIMIT)
            .map(|tick| DesktopObservation {
                source: DesktopEventSource::Window,
                platform_sample_tick: tick as u64,
                kind: DesktopObservationKind::FocusChanged { focused: true },
            })
            .collect();

        assert!(matches!(
            normalizer.normalize(oversized),
            Err(DesktopAdapterError::EventBatchLimitExceeded)
        ));
        let accepted = normalizer
            .normalize(vec![DesktopObservation {
                source: DesktopEventSource::Window,
                platform_sample_tick: 0,
                kind: DesktopObservationKind::FocusChanged { focused: true },
            }])
            .expect("first accepted event");
        assert_eq!(accepted[0].source_sequence, 0);
    }

    #[test]
    fn timebase_regression_rejects_without_sequence_progress() {
        let mut normalizer =
            DesktopEventNormalizer::new(PersistentId::from_bytes([10; 16])).expect("normalizer");
        let accepted = normalizer
            .normalize(vec![DesktopObservation {
                source: DesktopEventSource::Window,
                platform_sample_tick: 5,
                kind: DesktopObservationKind::FocusChanged { focused: false },
            }])
            .expect("initial event");
        assert_eq!(accepted[0].source_sequence, 0);

        assert!(matches!(
            normalizer.normalize(vec![DesktopObservation {
                source: DesktopEventSource::Window,
                platform_sample_tick: 4,
                kind: DesktopObservationKind::FocusChanged { focused: true },
            }]),
            Err(DesktopAdapterError::TimebaseRegression {
                previous: 5,
                actual: 4
            })
        ));

        let retry = normalizer
            .normalize(vec![DesktopObservation {
                source: DesktopEventSource::Window,
                platform_sample_tick: 6,
                kind: DesktopObservationKind::FocusChanged { focused: true },
            }])
            .expect("valid retry");
        assert_eq!(retry[0].source_sequence, 1);
    }
}
