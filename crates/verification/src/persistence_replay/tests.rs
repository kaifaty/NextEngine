use next_contracts::rpg::CORE_INTERACTIVE_OBJECT_ACTIVATED_STATE_ID;

use super::run_persistence_replay_check;

#[test]
fn product_check_covers_npc_transition_replay_and_structural_fallbacks() {
    let report = run_persistence_replay_check().expect("product check passes");
    assert_eq!(report.ticks, 19);
    assert_eq!(report.generations, 2);
    assert_eq!(report.rpg_events, 8);
    assert_eq!(
        report.interactive_object_state.as_str(),
        CORE_INTERACTIVE_OBJECT_ACTIVATED_STATE_ID
    );
    assert_eq!(
        report.dialogue_node_id.as_str(),
        "nextengine.reference-alpha.dialogue.offer"
    );
    assert_eq!(
        report.quest_state_id.as_str(),
        "nextengine.reference-alpha.quest.available"
    );
    assert_eq!(report.npc_player_trust, 0);
    assert_eq!(report.npc_health, 50);
    assert_eq!(report.player_health, 50);
    assert_eq!(
        report.final_pose.translation_micrometres,
        [200_000, 900_000, 200_000]
    );
}
