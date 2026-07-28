use super::hashes::*;
use super::*;

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct CommandCollisionCandidateV1 {
    pub command_id: CommandId,
    pub body_hash: CommandBodyHash,
    pub canonical_body_ref: CommandBodyHash,
}

impl CommandCollisionCandidateV1 {
    pub(super) fn canonical_record(&self) -> Result<Vec<u8>, CanonicalError> {
        struct_record([
            CanonicalField::new(1, CANONICAL_TYPE_ID128, self.command_id.as_bytes().to_vec()),
            CanonicalField::new(
                2,
                CANONICAL_TYPE_HASH256,
                self.body_hash.as_bytes().to_vec(),
            ),
            CanonicalField::new(
                3,
                CANONICAL_TYPE_HASH256,
                self.canonical_body_ref.as_bytes().to_vec(),
            ),
        ])
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CommandCollisionIncidentV1 {
    pub stream_id: CommandStreamId,
    pub issuer: IssuerPrincipal,
    pub sequence: u64,
    pub candidates: Vec<CommandCollisionCandidateV1>,
    pub candidates_root: ContentHash,
    pub incident_digest: ContentHash,
}

impl CommandCollisionIncidentV1 {
    pub fn new(
        stream_id: CommandStreamId,
        issuer: IssuerPrincipal,
        sequence: u64,
        mut candidates: Vec<CommandCollisionCandidateV1>,
    ) -> Result<Self, CommandLedgerError> {
        candidates.sort_by(|left, right| {
            (&left.body_hash, &left.command_id).cmp(&(&right.body_hash, &right.command_id))
        });
        if !collision_candidates_are_canonical(&candidates) {
            return Err(CommandLedgerError::CollisionCandidatesInvalid);
        }
        let candidates_root = command_collision_candidates_root(&candidates)?;
        let incident_digest =
            command_collision_incident_digest(stream_id, sequence, candidates_root);
        Ok(Self {
            stream_id,
            issuer,
            sequence,
            candidates,
            candidates_root,
            incident_digest,
        })
    }

    pub fn validate(&self) -> Result<(), CommandLedgerError> {
        if !collision_candidates_are_canonical(&self.candidates) {
            return Err(CommandLedgerError::CollisionCandidatesInvalid);
        }
        if command_collision_candidates_root(&self.candidates)? != self.candidates_root {
            return Err(CommandLedgerError::CollisionCandidatesRootMismatch);
        }
        if command_collision_incident_digest(self.stream_id, self.sequence, self.candidates_root)
            != self.incident_digest
        {
            return Err(CommandLedgerError::CollisionIncidentDigestMismatch);
        }
        Ok(())
    }
}
