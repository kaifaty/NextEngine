use next_contracts::{
    CommandId, CommandStreamId, IssuerPrincipal, PlayerPrincipalId, WorldCommand,
};
use next_runtime::{CommandDisposition, RejectionCode, RuntimeState};
use next_verification::{ReplayInput, ReplayTickInput, compare_replay_outputs, run_replay};

fn command(stream: u8, issuer: u8, sequence: u64, tick: u64) -> WorldCommand {
    WorldCommand::noop(
        CommandStreamId::from_bytes([stream; 16]),
        IssuerPrincipal::Player(PlayerPrincipalId::from_bytes([issuer; 16])),
        sequence,
        tick,
    )
    .expect("test command is canonical")
}

#[test]
fn public_replay_path_is_arrival_independent() {
    let first = command(1, 2, 0, 0);
    let second = command(2, 1, 0, 0);
    let left = ReplayInput {
        ticks: vec![ReplayTickInput {
            commands: vec![first.clone(), second.clone()],
        }],
    };
    let right = ReplayInput {
        ticks: vec![ReplayTickInput {
            commands: vec![second, first],
        }],
    };

    let left = run_replay(&left).expect("left replay runs");
    let right = run_replay(&right).expect("right replay runs");
    compare_replay_outputs(&left, &right).expect("arrival permutation must be exact");
}

#[test]
fn all_three_command_arrival_permutations_match() {
    let commands = [
        command(1, 3, 0, 0),
        command(2, 2, 0, 0),
        command(3, 1, 0, 0),
    ];
    let permutations = [
        [0, 1, 2],
        [0, 2, 1],
        [1, 0, 2],
        [1, 2, 0],
        [2, 0, 1],
        [2, 1, 0],
    ];
    let reference = run_replay(&ReplayInput {
        ticks: vec![ReplayTickInput {
            commands: commands.to_vec(),
        }],
    })
    .expect("reference replay runs");

    for permutation in permutations {
        let candidate = run_replay(&ReplayInput {
            ticks: vec![ReplayTickInput {
                commands: permutation
                    .into_iter()
                    .map(|index| commands[index].clone())
                    .collect(),
            }],
        })
        .expect("permuted replay runs");
        compare_replay_outputs(&reference, &candidate)
            .expect("every arrival permutation must be exact");
    }
}

#[test]
fn public_command_tamper_is_rejected_without_authoritative_ledger_mutation() {
    let mut tampered = command(1, 1, 0, 0);
    tampered.command_id = CommandId::from_bytes([9; 16]);
    let mut runtime = RuntimeState::default();
    let report = runtime
        .run_tick([tampered])
        .expect("tamper is a stable rejection");

    assert_eq!(
        report.results[0].disposition,
        CommandDisposition::Rejected(RejectionCode::CommandIdMismatch)
    );
    assert!(report.events.is_empty());
    assert!(report.snapshot.command_ledgers.is_empty());
}
