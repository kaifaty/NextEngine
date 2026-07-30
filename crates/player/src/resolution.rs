use std::collections::{BTreeMap, BTreeSet};

use next_contracts::input::{
    ActionBindingTransformV1, ActionConflictPolicyV1, ActionDefinitionV1, ActionMapManifestV1,
    InputContextStackV1, PlayerActionValueKindV1, PlayerActionValueV1,
};
use next_contracts::platform::{NormalizedControlEventV1, PlatformContractError};

use super::{ControlIdentity, PlayerInputError};

pub(super) fn validate_context_compatibility(
    action_map: &ActionMapManifestV1,
    context_stack: &InputContextStackV1,
) -> Result<(), PlayerInputError> {
    context_stack
        .validate_against_action_map(action_map)
        .map_err(|_| PlayerInputError::ContextStale)
}

pub(super) fn action_shapes_match(left: &ActionMapManifestV1, right: &ActionMapManifestV1) -> bool {
    left.actions.len() == right.actions.len()
        && left
            .actions
            .iter()
            .zip(&right.actions)
            .all(|(left, right)| {
                left.action_id == right.action_id && left.value_kind == right.value_kind
            })
}

pub(super) fn matching_bindings<'a>(
    action_map: &'a ActionMapManifestV1,
    control: &NormalizedControlEventV1,
) -> Vec<(
    &'a ActionDefinitionV1,
    &'a next_contracts::input::ActionBindingV1,
)> {
    action_map
        .actions
        .iter()
        .flat_map(|action| {
            action.binding_slots.iter().filter_map(move |binding| {
                (binding.device_class == control.device_class
                    && binding.control_path_id == control.control_path_id
                    && binding
                        .required_modifiers
                        .iter()
                        .all(|modifier| control.modifier_set.binary_search(modifier).is_ok()))
                .then_some((action, binding))
            })
        })
        .collect()
}

pub(super) fn validate_control_shape(
    control: &NormalizedControlEventV1,
    transform: ActionBindingTransformV1,
) -> Result<(), PlayerInputError> {
    let expected_components = match transform {
        ActionBindingTransformV1::Digital { .. }
        | ActionBindingTransformV1::Vector2ContributionQ15 { .. }
        | ActionBindingTransformV1::ScalarPassthroughQ15 { .. } => 1,
        ActionBindingTransformV1::Vector2PassthroughQ15 { .. } => 2,
    };
    if control.quantized_value.len() != expected_components {
        return Err(PlayerInputError::Platform(
            PlatformContractError::KindPayloadMismatch,
        ));
    }
    Ok(())
}

pub(super) fn control_is_active(
    control: &NormalizedControlEventV1,
    bindings: &[(&ActionDefinitionV1, &next_contracts::input::ActionBindingV1)],
) -> bool {
    bindings.iter().any(|(_, binding)| match binding.transform {
        ActionBindingTransformV1::Digital {
            pressed_threshold_q15,
        } => control.quantized_value[0] >= pressed_threshold_q15,
        ActionBindingTransformV1::Vector2ContributionQ15 { .. } => control.quantized_value[0] > 0,
        ActionBindingTransformV1::Vector2PassthroughQ15 { .. } => false,
        ActionBindingTransformV1::ScalarPassthroughQ15 { .. } => control.quantized_value[0] != 0,
    })
}

pub(super) fn resolve_definition(
    definition: &ActionDefinitionV1,
    held_controls: &BTreeMap<ControlIdentity, Vec<i16>>,
    started_controls: &BTreeSet<ControlIdentity>,
    pending_deltas: &BTreeMap<ControlIdentity, [i64; 2]>,
) -> Result<Option<(PlayerActionValueV1, bool, bool)>, PlayerInputError> {
    match definition.value_kind {
        PlayerActionValueKindV1::Digital => {
            let mut active_bindings = 0usize;
            let mut pulsed = false;
            for binding in &definition.binding_slots {
                let active = held_controls
                    .keys()
                    .any(|identity| binding_matches_identity(binding, identity));
                let started = started_controls
                    .iter()
                    .any(|identity| binding_matches_identity(binding, identity));
                active_bindings += usize::from(active);
                pulsed |= started;
            }
            if definition.conflict_policy == ActionConflictPolicyV1::Reject && active_bindings > 1 {
                return Err(PlayerInputError::BindingConflict);
            }
            if active_bindings > 0 {
                Ok(Some((PlayerActionValueV1::Digital(true), true, false)))
            } else if pulsed {
                Ok(Some((PlayerActionValueV1::Digital(true), false, true)))
            } else {
                Ok(None)
            }
        }
        PlayerActionValueKindV1::Vector2Q15 => {
            let mut candidates = Vec::new();
            for binding in &definition.binding_slots {
                match binding.transform {
                    ActionBindingTransformV1::Vector2ContributionQ15 { contribution_q15 } => {
                        if held_controls
                            .keys()
                            .any(|identity| binding_matches_identity(binding, identity))
                        {
                            candidates.push((
                                [
                                    i64::from(contribution_q15[0]),
                                    i64::from(contribution_q15[1]),
                                ],
                                true,
                            ));
                        }
                    }
                    ActionBindingTransformV1::Vector2PassthroughQ15 { scale_q15 } => {
                        let mut binding_total = [0i64; 2];
                        for (identity, delta) in pending_deltas {
                            if binding_matches_identity(binding, identity) {
                                binding_total[0] = binding_total[0]
                                    .checked_add(scale_q15_component(delta[0], scale_q15)?)
                                    .ok_or(PlayerInputError::CounterOverflow)?;
                                binding_total[1] = binding_total[1]
                                    .checked_add(scale_q15_component(delta[1], scale_q15)?)
                                    .ok_or(PlayerInputError::CounterOverflow)?;
                            }
                        }
                        if binding_total != [0, 0] {
                            candidates.push((binding_total, false));
                        }
                    }
                    ActionBindingTransformV1::Digital { .. } => {
                        return Err(PlayerInputError::BindingConflict);
                    }
                    ActionBindingTransformV1::ScalarPassthroughQ15 { .. } => {
                        return Err(PlayerInputError::BindingConflict);
                    }
                }
            }
            if definition.conflict_policy == ActionConflictPolicyV1::Reject && candidates.len() > 1
            {
                return Err(PlayerInputError::BindingConflict);
            }
            let (total, stateful) =
                if definition.conflict_policy == ActionConflictPolicyV1::PreferCanonicalBinding {
                    let Some(candidate) = candidates.into_iter().next() else {
                        return Ok(None);
                    };
                    candidate
                } else {
                    let mut total = [0i64; 2];
                    let mut stateful = false;
                    for (candidate, candidate_stateful) in candidates {
                        total[0] = total[0]
                            .checked_add(candidate[0])
                            .ok_or(PlayerInputError::CounterOverflow)?;
                        total[1] = total[1]
                            .checked_add(candidate[1])
                            .ok_or(PlayerInputError::CounterOverflow)?;
                        stateful |= candidate_stateful;
                    }
                    (total, stateful)
                };
            let value = [clamp_q15(total[0]), clamp_q15(total[1])];
            if value == [0, 0] {
                Ok(None)
            } else {
                Ok(Some((
                    PlayerActionValueV1::Vector2Q15(value),
                    stateful,
                    false,
                )))
            }
        }
        PlayerActionValueKindV1::ScalarQ15 => {
            let mut candidates = Vec::new();
            for binding in &definition.binding_slots {
                let ActionBindingTransformV1::ScalarPassthroughQ15 { scale_q15 } =
                    binding.transform
                else {
                    return Err(PlayerInputError::BindingConflict);
                };
                let mut binding_values = BTreeSet::new();
                for (identity, value) in held_controls {
                    if binding_matches_identity(binding, identity) {
                        binding_values.insert(clamp_q15(scale_q15_component(
                            i64::from(value[0]),
                            scale_q15,
                        )?));
                    }
                }
                binding_values.remove(&0);
                if binding_values.len() > 1 {
                    return Err(PlayerInputError::BindingConflict);
                }
                if let Some(value) = binding_values.into_iter().next() {
                    candidates.push(value);
                }
            }
            if candidates.is_empty() {
                return Ok(None);
            }
            if definition.conflict_policy == ActionConflictPolicyV1::Reject && candidates.len() > 1
            {
                return Err(PlayerInputError::BindingConflict);
            }
            let value = candidates
                .into_iter()
                .next()
                .ok_or(PlayerInputError::BindingConflict)?;
            Ok(Some((PlayerActionValueV1::ScalarQ15(value), true, false)))
        }
    }
}

pub(super) fn binding_matches_identity(
    binding: &next_contracts::input::ActionBindingV1,
    identity: &ControlIdentity,
) -> bool {
    binding.device_class == identity.device_class
        && binding.control_path_id == identity.control_path_id
        && binding
            .required_modifiers
            .iter()
            .all(|modifier| identity.modifier_set.binary_search(modifier).is_ok())
}

fn scale_q15_component(value: i64, scale_q15: i16) -> Result<i64, PlayerInputError> {
    value
        .checked_mul(i64::from(scale_q15))
        .map(|scaled| scaled / i64::from(i16::MAX))
        .ok_or(PlayerInputError::CounterOverflow)
}

fn clamp_q15(value: i64) -> i16 {
    value.clamp(-i64::from(i16::MAX), i64::from(i16::MAX)) as i16
}

pub(super) const fn neutral_value(value: PlayerActionValueV1) -> PlayerActionValueV1 {
    match value {
        PlayerActionValueV1::Digital(_) => PlayerActionValueV1::Digital(false),
        PlayerActionValueV1::ScalarQ15(_) => PlayerActionValueV1::ScalarQ15(0),
        PlayerActionValueV1::Vector2Q15(_) => PlayerActionValueV1::Vector2Q15([0, 0]),
    }
}

pub(super) fn canonical_value_sort_key(value: PlayerActionValueV1) -> [u8; 5] {
    match value {
        PlayerActionValueV1::Digital(value) => [1, u8::from(value), 0, 0, 0],
        PlayerActionValueV1::ScalarQ15(value) => {
            let bytes = value.to_le_bytes();
            [2, bytes[0], bytes[1], 0, 0]
        }
        PlayerActionValueV1::Vector2Q15(value) => {
            let x = value[0].to_le_bytes();
            let y = value[1].to_le_bytes();
            [3, x[0], x[1], y[0], y[1]]
        }
    }
}
