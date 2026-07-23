#![forbid(unsafe_code)]

use next_contracts::{CommandStreamId, IssuerPrincipal, PlayerPrincipalId, WorldCommand};
use next_verification::{ReplayInput, ReplayTickInput, compare_replay_outputs, run_replay};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let first = command(1, 2, 0, 0)?;
    let second = command(2, 1, 0, 0)?;
    let third = command(1, 2, 1, 1)?;
    let reference_input = ReplayInput {
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
