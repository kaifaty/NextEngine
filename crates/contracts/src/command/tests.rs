use crate::{
    CANONICAL_TYPE_BYTES, CanonicalDecodeLimits, CanonicalField, CapabilityRefV1, CommandId,
    CommandStreamId, PlayerPrincipalId, decode_canonical_segment, encode_canonical_segment,
};

use super::{
    COMMAND_BODY_OWNER_ID, COMMAND_BODY_SCHEMA_ID, COMMAND_BODY_SEGMENT_ID, CommandDecodeError,
    CommandPhase, DomainEventEnvelopeV2, IssuerPrincipal, WorldCommand,
};

fn command() -> WorldCommand {
    WorldCommand::noop(
        CommandStreamId::from_bytes([1; 16]),
        IssuerPrincipal::Player(PlayerPrincipalId::from_bytes([2; 16])),
        7,
        11,
    )
    .expect("fixed command is canonical")
}

#[test]
fn command_id_v2_matches_golden_vector_and_is_body_sensitive() {
    let first = command();
    let mut second = first.clone();
    second.stream_id = CommandStreamId::from_bytes([3; 16]);
    second
        .refresh_command_id()
        .expect("command remains canonical");

    assert_eq!(
        first
            .claimed_command_id
            .expect("constructor computes the claim")
            .to_hex(),
        "c2ca2d2370d816fe888565a66bda6eeb"
    );
    assert_ne!(first.claimed_command_id, second.claimed_command_id);
}

#[test]
fn envelope_claim_is_not_part_of_canonical_body_bytes() {
    let first = command();
    let mut second = first.clone();
    second.claimed_command_id = Some(CommandId::from_bytes([9; 16]));
    assert_eq!(
        first.canonical_bytes().expect("canonical command"),
        second.canonical_bytes().expect("canonical command")
    );
    assert_ne!(
        second.claimed_command_id,
        Some(second.compute_command_id().expect("computed command ID"))
    );
}

#[test]
fn domain_event_id_is_body_sensitive_and_canonical() {
    let command_id = command().compute_command_id().expect("command ID");
    let first = DomainEventEnvelopeV2::command_committed(11, CommandPhase::Ingress, command_id, 7)
        .expect("event");
    let second = DomainEventEnvelopeV2::command_committed(11, CommandPhase::Ingress, command_id, 8)
        .expect("event");

    assert_ne!(first.event_id, second.event_id);
    assert_ne!(first.event_body_hash, second.event_body_hash);
    assert_ne!(
        first.canonical_bytes().expect("first event"),
        second.canonical_bytes().expect("second event")
    );
    first.validate().expect("first event validates");
    let mut corrupt = first;
    corrupt.event_body_hash = crate::ContentHash::from_bytes([9; 32]);
    assert!(corrupt.validate().is_err());
}

#[test]
fn capabilities_are_canonicalized_as_a_set() {
    let mut first = command();
    first.capability_claims = vec![
        CapabilityRefV1::unscoped("world.read").expect("valid capability"),
        CapabilityRefV1::unscoped("world.write").expect("valid capability"),
    ];
    let mut second = first.clone();
    second.capability_claims.reverse();

    assert_eq!(
        first.canonical_bytes().expect("canonical command"),
        second.canonical_bytes().expect("canonical command")
    );
}

#[test]
fn command_round_trip_is_byte_exact() {
    let command = command();
    let bytes = command.canonical_bytes().expect("canonical command");
    let decoded = WorldCommand::from_canonical_bytes(&bytes, CanonicalDecodeLimits::default())
        .expect("command decodes");

    assert_eq!(decoded, command);
    assert_eq!(
        decoded.canonical_bytes().expect("command re-encodes"),
        bytes
    );
    let envelope =
        decode_canonical_segment(&bytes, CanonicalDecodeLimits::default()).expect("decodes");
    assert_eq!(envelope.owner_id, COMMAND_BODY_OWNER_ID);
    assert_eq!(envelope.schema_id, COMMAND_BODY_SCHEMA_ID);
    assert_eq!(envelope.segment_id, COMMAND_BODY_SEGMENT_ID);
}

#[test]
fn rpg_command_round_trip_is_byte_exact() {
    let command = WorldCommand::rpg(
        CommandStreamId::from_bytes([4; 16]),
        IssuerPrincipal::Player(PlayerPrincipalId::from_bytes([5; 16])),
        8,
        13,
        crate::RpgCommandV1 {
            operations: vec![crate::RpgOperationV1 {
                operation_slot: 0,
                targets: vec![crate::RpgAggregateRefV1 {
                    aggregate_kind: crate::RpgAggregateKindV1::Character,
                    persistent_id: crate::PersistentId::from_bytes([6; 16]),
                    expected_revision: 0,
                }],
                definition_policy_hashes: vec![],
                payload: crate::RpgOperationPayloadV1::SetSkillProficiency {
                    character_id: crate::PersistentId::from_bytes([6; 16]),
                    skill_id: crate::SchemaId::new("rpg.skill.survival")
                        .expect("skill id is valid"),
                    expected_value: 0,
                    new_value: 25,
                },
            }],
        },
    )
    .expect("RPG command is canonical");
    let bytes = command.canonical_bytes().expect("canonical RPG command");

    assert_eq!(
        WorldCommand::from_canonical_bytes(&bytes, CanonicalDecodeLimits::default())
            .expect("RPG command decodes"),
        command
    );
}

#[test]
fn decoder_rejects_unknown_required_field_and_wrong_body_version() {
    let command = command();
    let bytes = command.canonical_bytes().expect("canonical command");
    let decoded =
        decode_canonical_segment(&bytes, CanonicalDecodeLimits::default()).expect("decodes");
    let mut fields = decoded.fields.clone();
    fields.push(CanonicalField::new(13, CANONICAL_TYPE_BYTES, vec![]));
    let unknown_field = encode_canonical_segment(
        &decoded.owner_id,
        &decoded.schema_id,
        &decoded.segment_id,
        fields,
    )
    .expect("generic envelope can encode future field");
    assert!(matches!(
        WorldCommand::from_canonical_bytes(&unknown_field, CanonicalDecodeLimits::default()),
        Err(CommandDecodeError::UnknownField(13))
    ));

    let mut fields = decoded.fields;
    fields
        .iter_mut()
        .find(|field| field.field_id == 1)
        .expect("body version field")
        .payload = 3_u16.to_le_bytes().to_vec();
    let future_bytes = encode_canonical_segment(
        COMMAND_BODY_OWNER_ID,
        COMMAND_BODY_SCHEMA_ID,
        COMMAND_BODY_SEGMENT_ID,
        fields,
    )
    .expect("future body encodes");
    assert!(matches!(
        WorldCommand::from_canonical_bytes(&future_bytes, CanonicalDecodeLimits::default()),
        Err(CommandDecodeError::UnsupportedBodySchemaVersion(3))
    ));
}

#[test]
fn decoder_applies_total_input_bound_before_parsing() {
    let bytes = command().canonical_bytes().expect("canonical command");
    let limits = CanonicalDecodeLimits {
        max_total_bytes: bytes.len() - 1,
        ..CanonicalDecodeLimits::default()
    };
    assert!(matches!(
        WorldCommand::from_canonical_bytes(&bytes, limits),
        Err(CommandDecodeError::Canonical(
            crate::CanonicalDecodeError::InputTooLarge { .. }
        ))
    ));
}
