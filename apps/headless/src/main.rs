#![forbid(unsafe_code)]

use next_contracts::{
    CapabilityId, CommandStreamId, IssuerPrincipal, NOOP_COMMAND_CAPABILITY_ID, PlayerPrincipalId,
    WorldCommand,
};
use next_runtime::AuthorityRegistry;
use next_verification::{ReplayInput, ReplayTickInput, compare_replay_outputs, run_replay};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let first = command(1, 2, 0, 0)?;
    let second = command(2, 1, 0, 0)?;
    let third = command(1, 2, 1, 1)?;
    let authority = authority()?;
    let reference_input = ReplayInput {
        authority: authority.clone(),
        ticks: vec![
            ReplayTickInput {
                commands: vec![first.clone(), second.clone()],
            },
            ReplayTickInput {
                commands: vec![third.clone()],
            },
        ],
    };
    let permuted_input = ReplayInput {
        authority,
        ticks: vec![
            ReplayTickInput {
                commands: vec![second, first],
            },
            ReplayTickInput {
                commands: vec![third],
            },
        ],
    };

    let reference = run_replay(&reference_input)?;
    let permuted = run_replay(&permuted_input)?;
    compare_replay_outputs(&reference, &permuted)?;
    let final_root = reference
        .final_state_root()
        .ok_or("headless scenario did not execute any ticks")?;
    println!(
        "{{\"status\":\"PASS\",\"ticks\":{},\"state_root\":\"{}\"}}",
        reference.ticks.len(),
        final_root.to_hex()
    );
    Ok(())
}

fn authority() -> Result<AuthorityRegistry, Box<dyn std::error::Error>> {
    let capability = CapabilityId::new(NOOP_COMMAND_CAPABILITY_ID)?;
    let mut authority = AuthorityRegistry::new();
    for issuer in [1, 2] {
        authority.register(
            IssuerPrincipal::Player(PlayerPrincipalId::from_bytes([issuer; 16])),
            [capability.clone()],
        )?;
    }
    Ok(authority)
}

fn command(
    stream: u8,
    issuer: u8,
    sequence: u64,
    tick: u64,
) -> Result<WorldCommand, next_contracts::CanonicalError> {
    WorldCommand::noop(
        CommandStreamId::from_bytes([stream; 16]),
        IssuerPrincipal::Player(PlayerPrincipalId::from_bytes([issuer; 16])),
        sequence,
        tick,
    )
}
