use next_contracts::{
    CapabilityId, CommandId, CommandStreamId, IssuerPrincipal, NOOP_COMMAND_CAPABILITY_ID,
    PlayerPrincipalId, WorldCommand,
};
use next_runtime::{AuthorityRegistry, CommandDisposition, RejectionCode, RuntimeState};
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

fn authority(issuers: impl IntoIterator<Item = u8>) -> AuthorityRegistry {
    let capability =
        CapabilityId::new(NOOP_COMMAND_CAPABILITY_ID).expect("built-in capability ID is valid");
    let mut authority = AuthorityRegistry::new();
    for issuer in issuers {
        authority
            .register(
                IssuerPrincipal::Player(PlayerPrincipalId::from_bytes([issuer; 16])),
                [capability.clone()],
            )
            .expect("test principals are unique");
    }
    authority
}

#[test]
fn public_replay_path_is_arrival_independent() {
    let first = command(1, 2, 0, 0);
    let second = command(2, 1, 0, 0);
    let left = ReplayInput {
        authority: authority([1, 2]),
        ticks: vec![ReplayTickInput {
            commands: vec![first.clone(), second.clone()],
        }],
    };
    let right = ReplayInput {
        authority: authority([1, 2]),
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
        authority: authority([1, 2, 3]),
        ticks: vec![ReplayTickInput {
            commands: commands.to_vec(),
        }],
    })
    .expect("reference replay runs");

    for permutation in permutations {
        let candidate = run_replay(&ReplayInput {
            authority: authority([1, 2, 3]),
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
    tampered.claimed_command_id = Some(CommandId::from_bytes([9; 16]));
    let mut runtime = RuntimeState::new(authority([1]));
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
