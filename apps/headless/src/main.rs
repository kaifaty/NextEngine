#![forbid(unsafe_code)]

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let options = HeadlessOptions::parse(std::env::args().skip(1))?;
    let (report, project_lock) = match options.project {
        Some(project_path) => {
            let store = next_assets::ContentStore::new(project_path);
            let activated = next_project::activate_project(&store)?;
            let actual_lock = activated.composition_lock.composition_lock_sha256.to_hex();
            if let Some(expected_lock) = options.expected_lock
                && expected_lock != actual_lock
            {
                return Err(format!(
                    "project lock mismatch: expected {expected_lock}, activated {actual_lock}"
                )
                .into());
            }
            (
                next_verification::run_play_check_with_activated_project(activated)?,
                Some(actual_lock),
            )
        }
        None => (next_verification::run_play_check()?, None),
    };
    let translation = report.final_pose.translation_micrometres;
    let project_lock_json =
        project_lock.map_or_else(|| "null".to_owned(), |lock| format!("\"{lock}\""));
    println!(
        "{{\"status\":\"PASS\",\"project_lock\":{},\"ticks\":{},\"final_pose_um\":[{},{},{}],\"events\":{},\"rpg_events\":{},\"interactive_object_state\":\"{}\",\"dialogue_node\":\"{}\",\"quest_state\":\"{}\",\"npc_player_trust\":{},\"npc_health\":{},\"player_health\":{},\"agent_intent\":\"{}\",\"agent_projection\":\"{}\",\"ledger_hash\":\"{}\",\"state_root\":\"{}\"}}",
        project_lock_json,
        report.ticks,
        translation[0],
        translation[1],
        translation[2],
        report.events,
        report.rpg_events,
        report.interactive_object_state.as_str(),
        report.dialogue_node_id.as_str(),
        report.quest_state_id.as_str(),
        report.npc_player_trust,
        report.npc_health,
        report.player_health,
        report.agent_intent_id.to_hex(),
        report.agent_projection_hash.to_hex(),
        report.final_command_ledger_hash.to_hex(),
        report.final_state_root.to_hex()
    );
    Ok(())
}

#[derive(Debug, Default)]
struct HeadlessOptions {
    project: Option<std::path::PathBuf>,
    expected_lock: Option<String>,
}

impl HeadlessOptions {
    fn parse(
        mut arguments: impl Iterator<Item = String>,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let mut options = Self::default();
        while let Some(argument) = arguments.next() {
            match argument.as_str() {
                "--project" => {
                    let value = arguments
                        .next()
                        .ok_or("--project requires a cooked content-store path")?;
                    if options.project.replace(value.into()).is_some() {
                        return Err("--project may only be specified once".into());
                    }
                }
                "--lock" => {
                    let value = arguments
                        .next()
                        .ok_or("--lock requires a 64-character lowercase hex digest")?;
                    if value.len() != 64
                        || !value
                            .bytes()
                            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
                    {
                        return Err("--lock must be a 64-character lowercase hex digest".into());
                    }
                    if options.expected_lock.replace(value).is_some() {
                        return Err("--lock may only be specified once".into());
                    }
                }
                "--help" | "-h" => {
                    println!("usage: next_headless [--project <cooked-store>] [--lock <sha256>]");
                    std::process::exit(0);
                }
                _ => return Err(format!("unsupported argument: {argument}").into()),
            }
        }
        if options.project.is_none() && options.expected_lock.is_some() {
            return Err("--lock requires --project".into());
        }
        Ok(options)
    }
}
