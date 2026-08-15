use crate::canonical::CanonicalDecodeLimits;
use crate::command::{IssuerPrincipal, WorldCommand};
use crate::ids::{CommandId, CommandStreamId, PlayerPrincipalId};
use crate::persistence::ReplayCommandRecord;

#[test]
fn replay_record_preserves_invalid_envelope_claim_for_deterministic_rejection() {
    let principal = IssuerPrincipal::Player(PlayerPrincipalId::from_bytes([1; 16]));
    let mut command = WorldCommand::noop(CommandStreamId::from_bytes([2; 16]), principal, 0, 0)
        .expect("canonical command");
    command.envelope_schema_version = 1;
    command.claimed_command_id = Some(CommandId::from_bytes([9; 16]));
    let record = ReplayCommandRecord::from_command(&command).expect("record");
    let decoded = record
        .decode_command(CanonicalDecodeLimits::default())
        .expect("record decodes");
    assert_eq!(decoded, command);
}
