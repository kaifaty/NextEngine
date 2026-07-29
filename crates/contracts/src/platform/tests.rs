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
    let capability_id = CapabilityId::new("nextengine.platform.capability.duplicate").expect("ID");
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

#[test]
fn device_events_require_device_payloads() {
    let capabilities = capability_set(Vec::new());
    let device = PlatformEventPayloadV1::Device {
        device_class: SchemaId::new("nextengine.input.gamepad").expect("ID"),
        device_instance_nonce: PersistentId::from_bytes([4; 16]),
    };
    PlatformEventV1::new(
        PersistentId::from_bytes([2; 16]),
        SchemaId::new("nextengine.platform.source.device").expect("ID"),
        4,
        8,
        PlatformEventKindV1::DeviceConnected,
        device,
        capabilities.canonical_hash,
    )
    .expect("device event")
    .validate()
    .expect("valid device event");
    assert_eq!(
        PlatformEventV1::new(
            PersistentId::from_bytes([2; 16]),
            SchemaId::new("nextengine.platform.source.device").expect("ID"),
            5,
            9,
            PlatformEventKindV1::DeviceDisconnected,
            PlatformEventPayloadV1::Reason {
                reason: SchemaId::new("nextengine.device.removed").expect("ID")
            },
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
