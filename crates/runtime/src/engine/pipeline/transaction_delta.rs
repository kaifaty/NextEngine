use std::collections::BTreeMap;
use std::sync::Arc;

use next_contracts::command::{DomainEvent, WorldCommand};
use next_contracts::ids::{CommandBodyHash, CommandId, ContentHash};
use next_contracts::ledger::{
    CausalIdentityKey, CausalIdentityKind, CommandBodyArchiveV1, CommandIdentityBindingState,
    CommandIdentityBindingV1, CommandIdentityIndexV1, CommandIdentityOccurrenceV1,
    CommandLedgerError, CommandLedgerV2, IdentityInsertResult, PreparedCommandBodyArchiveUpdate,
    PreparedCommandIdentityIndexUpdate, causal_provenance_hash,
};

use crate::engine::error::RuntimeFatalError;

#[derive(Debug, Default)]
pub(crate) struct CommandLedgerTransactionDelta {
    archive_additions: BTreeMap<CommandBodyHash, Arc<[u8]>>,
    identity_replacements: BTreeMap<CommandId, CommandIdentityBindingV1>,
    causal_additions: BTreeMap<CausalIdentityKey, ContentHash>,
}

impl CommandLedgerTransactionDelta {
    pub(super) fn archive_bytes<'a>(
        &'a self,
        base: &'a CommandBodyArchiveV1,
        body_hash: &CommandBodyHash,
    ) -> Option<&'a [u8]> {
        self.archive_additions
            .get(body_hash)
            .or_else(|| base.entries().get(body_hash))
            .map(AsRef::as_ref)
    }

    pub(crate) fn archive_additions(&self) -> &BTreeMap<CommandBodyHash, Arc<[u8]>> {
        &self.archive_additions
    }

    pub(super) fn identity_binding<'a>(
        &'a self,
        base: &'a CommandIdentityIndexV1,
        command_id: &CommandId,
    ) -> Option<&'a CommandIdentityBindingV1> {
        self.identity_replacements
            .get(command_id)
            .or_else(|| base.body.bindings.get(command_id))
    }

    pub(super) fn stage_command_identity(
        &mut self,
        base_ledger: &CommandLedgerV2,
        base_archive: &CommandBodyArchiveV1,
        command: &WorldCommand,
        command_id: CommandId,
        body_hash: CommandBodyHash,
        canonical_bytes: &[u8],
    ) -> Result<IdentityInsertResult, RuntimeFatalError> {
        match self.archive_bytes(base_archive, &body_hash) {
            Some(existing) if existing == canonical_bytes => {}
            Some(_) => return Err(CommandLedgerError::CommandBodyHashCollision.into()),
            None => {
                self.archive_additions
                    .insert(body_hash, Arc::from(canonical_bytes));
            }
        }

        let occurrence = CommandIdentityOccurrenceV1 {
            body_hash,
            first_stream_id: command.stream_id,
            first_sequence: command.sequence,
        };
        let Some(existing) = self.identity_binding(&base_ledger.identity_index, &command_id) else {
            self.identity_replacements.insert(
                command_id,
                CommandIdentityBindingV1 {
                    command_id,
                    occurrences: vec![occurrence],
                    state: CommandIdentityBindingState::Unique,
                },
            );
            return Ok(IdentityInsertResult::Inserted);
        };
        if existing
            .occurrences
            .iter()
            .any(|retained| retained.body_hash == body_hash)
        {
            return Ok(IdentityInsertResult::Existing);
        }
        let mut replacement = existing.clone();
        replacement.occurrences.push(occurrence);
        replacement
            .occurrences
            .sort_by_key(|occurrence| occurrence.body_hash);
        replacement.state = CommandIdentityBindingState::Collision;
        self.identity_replacements.insert(command_id, replacement);
        Ok(IdentityInsertResult::Collision)
    }

    pub(super) fn stage_event_identity(
        &mut self,
        base_ledger: &CommandLedgerV2,
        event: &DomainEvent,
    ) -> Result<IdentityInsertResult, RuntimeFatalError> {
        let key = CausalIdentityKey {
            identity_kind: CausalIdentityKind::DomainEvent,
            identity_bytes: *event.event_id.as_bytes(),
        };
        let provenance = event.canonical_bytes()?;
        let provenance_hash = causal_provenance_hash(key.identity_kind, &provenance)?;
        match self
            .causal_additions
            .get(&key)
            .or_else(|| base_ledger.causal_identity_registry.bindings.get(&key))
        {
            Some(existing) if *existing == provenance_hash => Ok(IdentityInsertResult::Existing),
            Some(_) => Ok(IdentityInsertResult::Collision),
            None => {
                self.causal_additions.insert(key, provenance_hash);
                Ok(IdentityInsertResult::Inserted)
            }
        }
    }

    pub(crate) fn prepare(
        self,
        base_ledger: &CommandLedgerV2,
        base_archive: &CommandBodyArchiveV1,
    ) -> Result<PreparedCommandLedgerTransaction, RuntimeFatalError> {
        let archive = base_archive.prepare_additions(self.archive_additions)?;
        let identity = base_ledger
            .identity_index
            .prepare_replacements(self.identity_replacements)?;
        if archive.entry_count() != identity.occurrence_count() {
            return Err(CommandLedgerError::CommandBodyArchiveCorrupt.into());
        }
        Ok(PreparedCommandLedgerTransaction {
            archive,
            identity,
            causal_additions: self.causal_additions,
        })
    }
}

#[derive(Clone, Debug)]
pub(crate) struct PreparedCommandLedgerTransaction {
    archive: PreparedCommandBodyArchiveUpdate,
    identity: PreparedCommandIdentityIndexUpdate,
    causal_additions: BTreeMap<CausalIdentityKey, ContentHash>,
}

impl PreparedCommandLedgerTransaction {
    pub(crate) fn materialize(
        &self,
        staged_ledger: &CommandLedgerV2,
        base_archive: &CommandBodyArchiveV1,
    ) -> (CommandLedgerV2, CommandBodyArchiveV1) {
        let mut archive = base_archive.clone();
        archive.commit_prepared_additions(self.archive.clone());
        let mut ledger = staged_ledger.clone();
        ledger
            .identity_index
            .commit_prepared_replacements(self.identity.clone());
        let bindings = Arc::make_mut(&mut ledger.causal_identity_registry.bindings);
        bindings.extend(
            self.causal_additions
                .iter()
                .map(|(key, hash)| (*key, *hash)),
        );
        ledger.body_archive = self.archive.manifest();
        (ledger, archive)
    }

    pub(crate) fn commit_deferred_roots(
        self,
        live_ledger: &mut CommandLedgerV2,
        live_archive: &mut CommandBodyArchiveV1,
        staged_ledger: CommandLedgerV2,
    ) {
        let CommandLedgerV2 { streams, .. } = staged_ledger;
        let archive_entry_count = self.archive.entry_count();
        live_archive.commit_prepared_additions(self.archive);
        live_ledger
            .identity_index
            .commit_prepared_replacements_deferred(self.identity);
        let bindings = Arc::make_mut(&mut live_ledger.causal_identity_registry.bindings);
        bindings.extend(self.causal_additions);
        live_ledger.streams = streams;
        live_ledger.body_archive.entry_count = archive_entry_count;
    }
}
