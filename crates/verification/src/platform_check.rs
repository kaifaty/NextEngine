use std::error::Error;
use std::fmt::{Display, Formatter};

use next_contracts::{
    NormalizedControlEventV1, NormalizedControlPhaseV1, PersistentId, PlatformEventKindV1,
    PlatformEventPayloadV1, PlatformEventV1, PresentationTargetKindV1, SchemaId,
};
use next_platform::{PlatformHost, PlatformHostError, ReferencePlatformHost};

use crate::{GameCheckReport, PlayCheckError, run_game_check, run_play_check};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PlatformCheckReport {
    pub normalized_events: usize,
    pub rendered_objects: u32,
    pub authoritative_state_root: next_contracts::StateRoot,
    pub authoritative_ledger_hash: next_contracts::CommandLedgerHash,
    pub presentation_snapshot_hash: next_contracts::ContentHash,
    pub candidate_status: PlatformCandidateStatus,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PlatformCandidateStatus {
    NotRunOnDeveloperHost,
}

pub fn run_platform_check() -> Result<PlatformCheckReport, PlatformCheckError> {
    let headless = run_play_check()?;
    let game = run_game_check()?;
    verify_authoritative_parity(&headless, &game)?;

    let mut host = ReferencePlatformHost::interactive()?;
    if host.presentation_target_kind() != PresentationTargetKindV1::Interactive
        || host.drawable_extent() != [960, 540]
    {
        return Err(PlatformCheckError::HostContractMismatch);
    }
    let host_instance = PersistentId::from_bytes([0x71; 16]);
    let window_source = SchemaId::new("nextengine.platform.source.window")?;
    let keyboard_source = SchemaId::new("nextengine.platform.source.keyboard")?;
    let capability_hash = host.capability_set().canonical_hash;
    let events = [
        PlatformEventV1::new(
            host_instance,
            window_source.clone(),
            0,
            0,
            PlatformEventKindV1::FocusChanged,
            PlatformEventPayloadV1::FocusChanged { focused: false },
            capability_hash,
        )?,
        PlatformEventV1::new(
            host_instance,
            window_source.clone(),
            1,
            1,
            PlatformEventKindV1::CapabilityChanged,
            PlatformEventPayloadV1::WindowExtentChanged {
                width: 1_280,
                height: 720,
            },
            capability_hash,
        )?,
        PlatformEventV1::new(
            host_instance,
            window_source,
            2,
            2,
            PlatformEventKindV1::CloseRequested,
            PlatformEventPayloadV1::Reason {
                reason: SchemaId::new("nextengine.platform.reason.user-close")?,
            },
            capability_hash,
        )?,
        PlatformEventV1::new(
            host_instance,
            keyboard_source.clone(),
            0,
            2,
            PlatformEventKindV1::Control,
            PlatformEventPayloadV1::Control(NormalizedControlEventV1::new(
                SchemaId::new("nextengine.input.keyboard")?,
                PersistentId::from_bytes([0x72; 16]),
                SchemaId::new("nextengine.input.key.forward")?,
                NormalizedControlPhaseV1::Started,
                vec![i16::MAX],
                Vec::new(),
                2,
                0,
            )?),
            capability_hash,
        )?,
    ];
    for event in events.into_iter().rev() {
        host.inject_event(event)?;
    }
    let normalized = host.poll_events()?;
    if normalized.len() != 4
        || !normalized
            .iter()
            .any(|event| event.kind == PlatformEventKindV1::Control)
        || !normalized
            .iter()
            .any(|event| event.kind == PlatformEventKindV1::FocusChanged)
        || !normalized
            .iter()
            .any(|event| event.kind == PlatformEventKindV1::CloseRequested)
    {
        return Err(PlatformCheckError::HostContractMismatch);
    }
    let headless_host = ReferencePlatformHost::headless()?;
    if headless_host.presentation_target_kind() != PresentationTargetKindV1::None
        || headless_host.drawable_extent() != [0, 0]
    {
        return Err(PlatformCheckError::HeadlessCreatedPresentationTarget);
    }

    Ok(PlatformCheckReport {
        normalized_events: normalized.len(),
        rendered_objects: game.rendered_object_count,
        authoritative_state_root: game.play.final_state_root,
        authoritative_ledger_hash: game.play.final_command_ledger_hash,
        presentation_snapshot_hash: game.presentation_snapshot_hash,
        candidate_status: PlatformCandidateStatus::NotRunOnDeveloperHost,
    })
}

fn verify_authoritative_parity(
    headless: &crate::PlayCheckReport,
    game: &GameCheckReport,
) -> Result<(), PlatformCheckError> {
    if headless.final_state_root != game.play.final_state_root
        || headless.final_command_ledger_hash != game.play.final_command_ledger_hash
        || headless.ticks != game.play.ticks
        || headless.rpg_events != game.play.rpg_events
    {
        return Err(PlatformCheckError::AuthoritativeParityMismatch);
    }
    Ok(())
}

#[derive(Debug)]
#[non_exhaustive]
pub enum PlatformCheckError {
    Play(PlayCheckError),
    Platform(PlatformHostError),
    Contract(next_contracts::PlatformContractError),
    Identifier(next_contracts::IdentifierError),
    AuthoritativeParityMismatch,
    HostContractMismatch,
    HeadlessCreatedPresentationTarget,
}

impl Display for PlatformCheckError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Play(error) => write!(formatter, "{error}"),
            Self::Platform(error) => write!(formatter, "{error}"),
            Self::Contract(error) => write!(formatter, "{error}"),
            Self::Identifier(error) => write!(formatter, "{error}"),
            Self::AuthoritativeParityMismatch => {
                formatter.write_str("game/headless authoritative hashes diverged")
            }
            Self::HostContractMismatch => {
                formatter.write_str("platform host lifecycle normalization mismatch")
            }
            Self::HeadlessCreatedPresentationTarget => {
                formatter.write_str("headless created a presentation target")
            }
        }
    }
}

impl Error for PlatformCheckError {}

impl From<PlayCheckError> for PlatformCheckError {
    fn from(error: PlayCheckError) -> Self {
        Self::Play(error)
    }
}

impl From<PlatformHostError> for PlatformCheckError {
    fn from(error: PlatformHostError) -> Self {
        Self::Platform(error)
    }
}

impl From<next_contracts::PlatformContractError> for PlatformCheckError {
    fn from(error: next_contracts::PlatformContractError) -> Self {
        Self::Contract(error)
    }
}

impl From<next_contracts::IdentifierError> for PlatformCheckError {
    fn from(error: next_contracts::IdentifierError) -> Self {
        Self::Identifier(error)
    }
}

#[cfg(test)]
mod tests {
    use super::run_platform_check;

    #[test]
    fn game_headless_platform_and_presentation_contracts_match() {
        let report = run_platform_check().expect("platform check");
        assert_eq!(report.normalized_events, 4);
        assert_eq!(report.rendered_objects, 5);
    }
}
