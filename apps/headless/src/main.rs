#![forbid(unsafe_code)]

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let report = next_verification::run_play_check()?;
    let translation = report.final_pose.translation_micrometres;
    println!(
        "{{\"status\":\"PASS\",\"ticks\":{},\"final_pose_um\":[{},{},{}],\"events\":{},\"rpg_events\":{},\"interactive_object_state\":\"{}\",\"dialogue_node\":\"{}\",\"quest_state\":\"{}\",\"npc_player_trust\":{},\"ledger_hash\":\"{}\",\"state_root\":\"{}\"}}",
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
        report.final_command_ledger_hash.to_hex(),
        report.final_state_root.to_hex()
    );
    Ok(())
}
