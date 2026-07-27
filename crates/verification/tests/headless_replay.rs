use next_contracts::{
    CapabilityId, CommandId, CommandStreamId, IssuerPrincipal, NOOP_COMMAND_CAPABILITY_ID,
    PlayerPrincipalId, WorldCommand,
};
use next_runtime::{CommandDisposition, RejectionCode, RuntimeState};
use next_verification::{
    NeutralRuntimeFixture, ReplayInput, ReplayTickInput, build_neutral_runtime_fixture,
    compare_replay_outputs, run_replay,
};

fn principal(value: u8) -> IssuerPrincipal {
    IssuerPrincipal::Player(PlayerPrincipalId::from_bytes([value; 16]))
}

fn command(fixture: &NeutralRuntimeFixture, issuer: u8, sequence: u64, tick: u64) -> WorldCommand {
    let principal = principal(issuer);
    WorldCommand::noop(
        fixture.stream_for(&principal).expect("fixture stream"),
        principal,
        sequence,
        tick,
    )
    .expect("test command is canonical")
}

fn fixture(project_id: &str, issuers: impl IntoIterator<Item = u8>) -> NeutralRuntimeFixture {
    let capability =
        CapabilityId::new(NOOP_COMMAND_CAPABILITY_ID).expect("built-in capability ID is valid");
    build_neutral_runtime_fixture(
        project_id,
        issuers
            .into_iter()
            .map(|issuer| (principal(issuer), vec![capability.clone()])),
    )
    .expect("neutral fixture")
}

#[test]
fn public_replay_path_is_arrival_independent() {
    let fixture = fixture("nextengine.headless-replay.two", [1, 2]);
    let first = command(&fixture, 2, 0, 0);
    let second = command(&fixture, 1, 0, 0);
    let left = ReplayInput {
        bootstrap: fixture.bootstrap.clone(),
        authority: fixture.authority.clone(),
        ticks: vec![ReplayTickInput {
            commands: vec![first.clone(), second.clone()],
        }],
    };
    let right = ReplayInput {
        bootstrap: fixture.bootstrap,
        authority: fixture.authority,
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
    let fixture = fixture("nextengine.headless-replay.three", [1, 2, 3]);
    let commands = [
        command(&fixture, 3, 0, 0),
        command(&fixture, 2, 0, 0),
        command(&fixture, 1, 0, 0),
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
        bootstrap: fixture.bootstrap.clone(),
        authority: fixture.authority.clone(),
        ticks: vec![ReplayTickInput {
            commands: commands.to_vec(),
        }],
    })
    .expect("reference replay runs");

    for permutation in permutations {
        let candidate = run_replay(&ReplayInput {
            bootstrap: fixture.bootstrap.clone(),
            authority: fixture.authority.clone(),
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
    let fixture = fixture("nextengine.headless-replay.tamper", [1]);
    let mut tampered = command(&fixture, 1, 0, 0);
    tampered.claimed_command_id = Some(CommandId::from_bytes([9; 16]));
    let stream_id: CommandStreamId = tampered.stream_id;
    let mut runtime = RuntimeState::new(fixture.bootstrap, fixture.authority)
        .expect("fixture bootstrap is valid");
    let report = runtime
        .run_tick([tampered])
        .expect("tamper is a stable rejection");

    assert_eq!(
        report.results[0].disposition,
        CommandDisposition::Rejected(RejectionCode::CommandIdMismatch)
    );
    assert!(report.events.is_empty());
    let stream = report
        .snapshot
        .command_ledger
        .streams
        .get(&stream_id)
        .expect("predeclared stream exists");
    assert!(stream.receipt_window.is_empty());
    assert!(stream.pending.is_empty());
}
