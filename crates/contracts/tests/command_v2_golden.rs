use next_contracts::{
    CanonicalDecodeLimits, CommandStreamId, IssuerPrincipal, PersistentId, PlayerPrincipalId,
    RpgCommand, SchemaId, WorldCommand,
};

struct GoldenVector<'a> {
    format: &'a str,
    body_schema: &'a str,
    body_hex: &'a str,
    body_hash: &'a str,
    command_id: &'a str,
}

fn parse_vector(input: &str) -> GoldenVector<'_> {
    let mut format = None;
    let mut body_schema = None;
    let mut body_hex = None;
    let mut body_hash = None;
    let mut command_id = None;

    for line in input.lines().filter(|line| !line.is_empty()) {
        let (key, value) = line.split_once('=').expect("golden vector uses key=value");
        match key {
            "format" => format = Some(value),
            "body_schema" => body_schema = Some(value),
            "body_hex" => body_hex = Some(value),
            "body_hash" => body_hash = Some(value),
            "command_id" => command_id = Some(value),
            _ => panic!("unknown golden vector key: {key}"),
        }
    }

    GoldenVector {
        format: format.expect("golden vector format"),
        body_schema: body_schema.expect("golden body schema"),
        body_hex: body_hex.expect("golden body bytes"),
        body_hash: body_hash.expect("golden body hash"),
        command_id: command_id.expect("golden command ID"),
    }
}

fn encode_hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

fn assert_golden(command: &WorldCommand, vector: GoldenVector<'_>) {
    assert_eq!(vector.format, "nextengine.command-v2-golden/1");
    assert_eq!(
        vector.body_schema,
        "nextengine.runtime/nextengine.canonical-command-body/v2"
    );

    let body_bytes = command
        .canonical_bytes()
        .expect("command body is canonical");
    assert_eq!(encode_hex(&body_bytes), vector.body_hex);
    assert_eq!(
        command.body_hash().expect("body hash").to_hex(),
        vector.body_hash
    );
    assert_eq!(
        command.compute_command_id().expect("command ID").to_hex(),
        vector.command_id
    );

    let decoded = WorldCommand::from_canonical_bytes(&body_bytes, CanonicalDecodeLimits::default())
        .expect("golden body decodes");
    assert_eq!(
        decoded.canonical_bytes().expect("body re-encodes"),
        body_bytes
    );
    assert_eq!(
        decoded.compute_command_id().expect("decoded command ID"),
        command.compute_command_id().expect("source command ID")
    );
}

#[test]
fn noop_command_matches_neutral_v2_golden_vector() {
    let command = WorldCommand::noop(
        CommandStreamId::from_bytes([1; 16]),
        IssuerPrincipal::Player(PlayerPrincipalId::from_bytes([2; 16])),
        7,
        11,
    )
    .expect("fixed command is canonical");

    assert_golden(
        &command,
        parse_vector(include_str!("fixtures/command_v2_noop.vector")),
    );
}

#[test]
fn rpg_command_matches_neutral_v2_golden_vector() {
    let command = WorldCommand::rpg(
        CommandStreamId::from_bytes([4; 16]),
        IssuerPrincipal::Player(PlayerPrincipalId::from_bytes([5; 16])),
        8,
        13,
        RpgCommand::LearnSkill {
            character_id: PersistentId::from_bytes([6; 16]),
            skill_id: SchemaId::new("rpg.skill.survival").expect("valid skill ID"),
            delta: 25,
        },
    )
    .expect("fixed RPG command is canonical");

    assert_golden(
        &command,
        parse_vector(include_str!("fixtures/command_v2_rpg.vector")),
    );
}
