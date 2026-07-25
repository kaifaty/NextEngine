use next_contracts::{
    CommandBodyArchiveV1, CommandFinalResultV1, CommandIdentityIndexV1, CommandPhase,
    CommandReceiptSubjectV1, CommandReceiptV1, CommandStreamId, IssuerPrincipal, PlayerPrincipalId,
    WorldCommand, command_receipt_chain_genesis, command_receipt_chain_next,
    command_receipt_digest, content_hash_from_bytes,
};

fn encode_hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

fn fixed_command() -> WorldCommand {
    WorldCommand::noop(
        CommandStreamId::from_bytes([1; 16]),
        IssuerPrincipal::Player(PlayerPrincipalId::from_bytes([2; 16])),
        7,
        11,
    )
    .expect("fixed command is canonical")
}

fn fixed_receipt(command: &WorldCommand) -> CommandReceiptV1 {
    let body_hash = command.body_hash().expect("body hash");
    CommandReceiptV1 {
        schema_version: 1,
        finalization_ordinal: 0,
        subject: CommandReceiptSubjectV1::Command {
            stream_id: command.stream_id,
            issuer: command.issuer.clone(),
            sequence: command.sequence,
            command_id: command.compute_command_id().expect("command ID"),
            body_hash,
            canonical_body_ref: body_hash,
        },
        phase: CommandPhase::Ingress,
        target_tick: command.target_tick,
        finalized_at_tick: 11,
        priority_class: 10,
        command_kind_registry_hash: content_hash_from_bytes([3; 32]),
        result: CommandFinalResultV1::Committed,
        diagnostic_digest: None,
        event_ids: Vec::new(),
        transaction_result_root: content_hash_from_bytes([4; 32]),
    }
}

#[test]
fn command_ledger_roots_match_neutral_v2_golden_vector() {
    let command = fixed_command();
    let receipt = fixed_receipt(&command);
    receipt.validate().expect("fixed receipt validates");

    let empty_archive = CommandBodyArchiveV1::default();
    let mut single_archive = CommandBodyArchiveV1::default();
    single_archive
        .insert_command(&command)
        .expect("fixed command archives");
    let empty_index = CommandIdentityIndexV1::empty().expect("empty identity index");
    let receipt_bytes = receipt.canonical_bytes().expect("canonical receipt");
    let genesis = command_receipt_chain_genesis();

    let actual = format!(
        concat!(
            "format=nextengine.command-ledger-v2-golden/1\n",
            "archive_empty_root={}\n",
            "archive_single_root={}\n",
            "identity_empty_root={}\n",
            "chain_genesis={}\n",
            "receipt_hex={}\n",
            "receipt_digest={}\n",
            "chain_after_receipt={}\n",
        ),
        empty_archive
            .manifest()
            .expect("empty manifest")
            .archive_root
            .to_hex(),
        single_archive
            .manifest()
            .expect("single manifest")
            .archive_root
            .to_hex(),
        empty_index.index_root.to_hex(),
        genesis.to_hex(),
        encode_hex(&receipt_bytes),
        command_receipt_digest(&receipt)
            .expect("receipt digest")
            .to_hex(),
        command_receipt_chain_next(genesis, &receipt)
            .expect("receipt chain")
            .to_hex(),
    );

    assert_eq!(actual, include_str!("fixtures/command_ledger_v2.vector"));
}
