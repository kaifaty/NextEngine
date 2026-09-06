//! Plan `continuum-water/36`: the gate lever. The player closes and reopens
//! the flow gate between the vessels through the accepted flow command
//! (`WaterFlowCommandV1::SetGate`), issued by the reference game's water-gate
//! system principal on its own stream when the interact action fires within
//! reach of the lever. Everything the runtime needs (the revision, the
//! ordering, the log) stays the runtime's; this module only decides the
//! payload and the prompt from committed state.

use next_contracts::command::{IssuerPrincipal, WorldCommand};
use next_contracts::ids::CommandStreamId;
use next_contracts::physics::{
    PhysicsBodyIdV1, PhysicsCanonicalSnapshotV2, WaterFlowCommandV1, WaterFlowEdgeStateV1,
    WaterFlowNetworkV1,
};

use crate::ReferenceGameError;

/// The reach of the lever from the avatar's body centre, horizontally.
pub const WATER_GATE_LEVER_REACH_MICROMETRES: i64 = 1_000_000;
/// A closed gate; the reopened gate returns to the authored full opening.
pub const WATER_GATE_CLOSED_PERMILLE: u32 = 0;
pub const WATER_GATE_OPEN_PERMILLE: u32 = 1_000;

/// What the lever would do if pulled now.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WaterGatePromptV1 {
    /// The gate is open; pulling closes it.
    Close,
    /// The gate is closed; pulling opens it.
    Open,
}

/// The gate's current prompt from its committed state.
#[must_use]
pub const fn gate_prompt(state: &WaterFlowEdgeStateV1) -> WaterGatePromptV1 {
    if state.opening_permille > WATER_GATE_CLOSED_PERMILLE {
        WaterGatePromptV1::Close
    } else {
        WaterGatePromptV1::Open
    }
}

/// The toggle payload: the opposite opening at the state's own revision.
#[must_use]
pub const fn toggle_payload(state: &WaterFlowEdgeStateV1) -> WaterFlowCommandV1 {
    WaterFlowCommandV1::SetGate {
        edge_id: state.edge_id,
        expected_record_revision: state.record_revision,
        opening_permille: match gate_prompt(state) {
            WaterGatePromptV1::Close => WATER_GATE_CLOSED_PERMILLE,
            WaterGatePromptV1::Open => WATER_GATE_OPEN_PERMILLE,
        },
    }
}

/// Whether the avatar's body centre lies within the lever's reach,
/// horizontally.
pub fn lever_in_reach(
    snapshot: &PhysicsCanonicalSnapshotV2,
    avatar: PhysicsBodyIdV1,
    lever: PhysicsBodyIdV1,
) -> Result<bool, ReferenceGameError> {
    let translation = |body: PhysicsBodyIdV1| {
        snapshot
            .sorted_body_states
            .get(&body)
            .map(|state| state.pose.translation_micrometres)
            .ok_or(ReferenceGameError::BodyMissing)
    };
    let (a, b) = (translation(avatar)?, translation(lever)?);
    let dx = i128::from(a[0] - b[0]);
    let dz = i128::from(a[2] - b[2]);
    let reach = i128::from(WATER_GATE_LEVER_REACH_MICROMETRES);
    Ok(dx * dx + dz * dz <= reach * reach)
}

/// The gate's committed state, when the network carries the gate.
#[must_use]
pub fn gate_state(
    network: &WaterFlowNetworkV1,
    gate: next_contracts::ids::PersistentId,
) -> Option<WaterFlowEdgeStateV1> {
    network.edge_states.get(&gate).copied()
}

/// The toggle command for the target tick on the water-gate stream; the
/// sequence is the tick itself.
pub fn toggle_command(
    state: &WaterFlowEdgeStateV1,
    stream_id: CommandStreamId,
    principal: &IssuerPrincipal,
    target_tick: u64,
) -> Result<WorldCommand, ReferenceGameError> {
    WorldCommand::water_flow(
        stream_id,
        principal.clone(),
        target_tick,
        target_tick,
        toggle_payload(state),
    )
    .map_err(ReferenceGameError::Canonical)
}

#[cfg(test)]
mod tests {
    use super::*;
    use next_contracts::ids::PersistentId;

    fn state(opening: u32, revision: u64) -> WaterFlowEdgeStateV1 {
        WaterFlowEdgeStateV1 {
            edge_id: PersistentId::from_bytes([0x80; 16]),
            record_revision: revision,
            opening_permille: opening,
            enabled: true,
            rate_cubic_millimetres_per_second: 0,
            last_flux_cubic_millimetres: 0,
        }
    }

    #[test]
    fn the_toggle_closes_an_open_gate_and_opens_a_closed_one_at_its_revision() {
        assert_eq!(gate_prompt(&state(1_000, 0)), WaterGatePromptV1::Close);
        assert_eq!(gate_prompt(&state(0, 3)), WaterGatePromptV1::Open);
        assert_eq!(
            toggle_payload(&state(1_000, 2)),
            WaterFlowCommandV1::SetGate {
                edge_id: PersistentId::from_bytes([0x80; 16]),
                expected_record_revision: 2,
                opening_permille: 0,
            }
        );
        assert_eq!(
            toggle_payload(&state(0, 3)),
            WaterFlowCommandV1::SetGate {
                edge_id: PersistentId::from_bytes([0x80; 16]),
                expected_record_revision: 3,
                opening_permille: 1_000,
            }
        );
    }
}
