use super::hashes::*;
use super::*;

const COMMAND_RECEIPT_WINDOW_CHUNK_CAPACITY: usize = 64;

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
#[repr(u8)]
pub enum CommandStreamStateV1 {
    Open = 0,
    CollisionLocked = 1,
    Exhausted = 2,
    Closed = 3,
}

/// The ordered retained suffix of finalized command receipts.
///
/// Its logical value is exactly the canonical receipt sequence. Physical
/// chunks are a reconstructible copy-on-write packing detail, allowing a
/// staged ledger generation to append without cloning all 4,096 receipts.
#[derive(Clone)]
pub struct CommandReceiptWindowV1 {
    chunks: Arc<Vec<Arc<Vec<CommandReceiptV1>>>>,
    head: usize,
    len: usize,
}

impl CommandReceiptWindowV1 {
    #[must_use]
    pub fn new() -> Self {
        Self {
            chunks: Arc::new(Vec::new()),
            head: 0,
            len: 0,
        }
    }

    pub(super) fn from_receipts(
        receipts: Vec<CommandReceiptV1>,
    ) -> Result<Self, CommandLedgerError> {
        if receipts.len() > COMMAND_RECEIPT_WINDOW_CAPACITY {
            return Err(CommandLedgerError::ReceiptWindowLengthMismatch);
        }
        let mut chunks = Vec::with_capacity(
            receipts
                .len()
                .div_ceil(COMMAND_RECEIPT_WINDOW_CHUNK_CAPACITY),
        );
        let mut chunk = Vec::with_capacity(COMMAND_RECEIPT_WINDOW_CHUNK_CAPACITY);
        for receipt in receipts {
            chunk.push(receipt);
            if chunk.len() == COMMAND_RECEIPT_WINDOW_CHUNK_CAPACITY {
                chunks.push(Arc::new(chunk));
                chunk = Vec::with_capacity(COMMAND_RECEIPT_WINDOW_CHUNK_CAPACITY);
            }
        }
        if !chunk.is_empty() {
            chunks.push(Arc::new(chunk));
        }
        let len = chunks.iter().map(|chunk| chunk.len()).sum();
        Ok(Self {
            chunks: Arc::new(chunks),
            head: 0,
            len,
        })
    }

    #[must_use]
    pub const fn len(&self) -> usize {
        self.len
    }

    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub fn iter(&self) -> impl Iterator<Item = &CommandReceiptV1> {
        self.chunks
            .iter()
            .flat_map(|chunk| chunk.iter())
            .skip(self.head)
            .take(self.len)
    }

    #[must_use]
    pub fn first(&self) -> Option<&CommandReceiptV1> {
        self.get(0)
    }

    #[must_use]
    pub fn last(&self) -> Option<&CommandReceiptV1> {
        self.len.checked_sub(1).and_then(|index| self.get(index))
    }

    #[must_use]
    pub fn get(&self, index: usize) -> Option<&CommandReceiptV1> {
        if index >= self.len {
            return None;
        }
        let physical_index = self.head.checked_add(index)?;
        self.chunks
            .get(physical_index / COMMAND_RECEIPT_WINDOW_CHUNK_CAPACITY)?
            .get(physical_index % COMMAND_RECEIPT_WINDOW_CHUNK_CAPACITY)
    }

    fn push(&mut self, receipt: CommandReceiptV1) {
        let chunks = Arc::make_mut(&mut self.chunks);
        match chunks.last_mut() {
            Some(chunk) if chunk.len() < COMMAND_RECEIPT_WINDOW_CHUNK_CAPACITY => {
                if let Some(unique) = Arc::get_mut(chunk) {
                    unique.push(receipt);
                } else {
                    let mut next = Vec::with_capacity(COMMAND_RECEIPT_WINDOW_CHUNK_CAPACITY);
                    next.extend(chunk.iter().cloned());
                    next.push(receipt);
                    *chunk = Arc::new(next);
                }
            }
            _ => {
                let mut chunk = Vec::with_capacity(COMMAND_RECEIPT_WINDOW_CHUNK_CAPACITY);
                chunk.push(receipt);
                chunks.push(Arc::new(chunk));
            }
        }
        self.len += 1;
        if self.len > COMMAND_RECEIPT_WINDOW_CAPACITY {
            self.head += 1;
            self.len -= 1;
            if self.head == chunks[0].len() {
                chunks.remove(0);
                self.head = 0;
            }
        }
    }
}

impl Default for CommandReceiptWindowV1 {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Debug for CommandReceiptWindowV1 {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        formatter.debug_list().entries(self.iter()).finish()
    }
}

impl PartialEq for CommandReceiptWindowV1 {
    fn eq(&self, other: &Self) -> bool {
        self.len == other.len && self.iter().eq(other.iter())
    }
}

impl Eq for CommandReceiptWindowV1 {}

impl std::ops::Index<usize> for CommandReceiptWindowV1 {
    type Output = CommandReceiptV1;

    fn index(&self, index: usize) -> &Self::Output {
        self.get(index).expect("receipt window index out of bounds")
    }
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct CommandReservationV1 {
    pub schema_version: u16,
    pub stream_id: CommandStreamId,
    pub issuer: IssuerPrincipal,
    pub sequence: u64,
    pub command_id: CommandId,
    pub body_hash: CommandBodyHash,
    pub canonical_body_ref: CommandBodyHash,
    pub reserved_at_tick: u64,
    pub target_tick: u64,
    pub phase: CommandPhase,
    pub priority_class: u16,
    pub command_kind_registry_hash: ContentHash,
}

impl CommandReservationV1 {
    pub fn from_command(
        command: &WorldCommand,
        reserved_at_tick: u64,
        priority_class: u16,
        command_kind_registry_hash: ContentHash,
    ) -> Result<Self, CanonicalError> {
        let body_hash = command.body_hash()?;
        Ok(Self {
            schema_version: COMMAND_RESERVATION_SCHEMA_VERSION,
            stream_id: command.stream_id,
            issuer: command.issuer.clone(),
            sequence: command.sequence,
            command_id: command.compute_command_id()?,
            body_hash,
            canonical_body_ref: body_hash,
            reserved_at_tick,
            target_tick: command.target_tick,
            phase: command.phase,
            priority_class,
            command_kind_registry_hash,
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CommandStreamLedgerV2 {
    pub schema_version: u16,
    pub stream_id: CommandStreamId,
    pub issuer: IssuerPrincipal,
    pub stream_slot: u32,
    pub stream_epoch: u32,
    pub state: CommandStreamStateV1,
    pub admission_high_watermark: Option<u64>,
    pub greatest_reserved_target_tick: Option<u64>,
    pub pending: Arc<BTreeMap<u64, CommandReservationV1>>,
    pub receipt_window: CommandReceiptWindowV1,
    pub finalized_receipt_count: u64,
    pub receipt_chain_root: ContentHash,
    pub collision_incident: Option<CommandCollisionIncidentV1>,
}

impl CommandStreamLedgerV2 {
    #[must_use]
    pub fn genesis(
        stream_id: CommandStreamId,
        issuer: IssuerPrincipal,
        stream_slot: u32,
        stream_epoch: u32,
    ) -> Self {
        Self {
            schema_version: COMMAND_STREAM_LEDGER_SCHEMA_VERSION,
            stream_id,
            issuer,
            stream_slot,
            stream_epoch,
            state: CommandStreamStateV1::Open,
            admission_high_watermark: None,
            greatest_reserved_target_tick: None,
            pending: Arc::new(BTreeMap::new()),
            receipt_window: CommandReceiptWindowV1::new(),
            finalized_receipt_count: 0,
            receipt_chain_root: command_receipt_chain_genesis(),
            collision_incident: None,
        }
    }

    pub fn reserve(&mut self, reservation: CommandReservationV1) -> Result<(), CommandLedgerError> {
        self.validate()?;
        self.reserve_incremental(reservation)
    }

    /// Reserves one command against a stream already validated at the current
    /// transaction boundary, without rescanning the retained receipt window.
    pub fn reserve_incremental(
        &mut self,
        reservation: CommandReservationV1,
    ) -> Result<(), CommandLedgerError> {
        self.validate_append_boundary()?;
        if reservation.schema_version != COMMAND_RESERVATION_SCHEMA_VERSION
            || reservation.stream_id != self.stream_id
            || reservation.issuer != self.issuer
            || reservation.body_hash != reservation.canonical_body_ref
        {
            return Err(CommandLedgerError::ReservationMismatch);
        }
        if let Some(existing) = self.pending.get(&reservation.sequence) {
            return if existing == &reservation {
                Ok(())
            } else {
                Err(CommandLedgerError::SequenceNotNew)
            };
        }
        if self.finalized_receipt_count == u64::MAX {
            return Err(CommandLedgerError::FinalizationOrdinalExhausted);
        }
        if self.state != CommandStreamStateV1::Open {
            return Err(CommandLedgerError::StreamNotOpen);
        }
        if self
            .admission_high_watermark
            .is_some_and(|high_watermark| reservation.sequence <= high_watermark)
        {
            return Err(CommandLedgerError::SequenceNotNew);
        }
        if self.pending.len() >= COMMAND_PENDING_CAPACITY {
            return Err(CommandLedgerError::PendingLimit);
        }
        if self
            .greatest_reserved_target_tick
            .is_some_and(|greatest| reservation.target_tick < greatest)
        {
            return Err(CommandLedgerError::TargetTickRegression);
        }
        self.admission_high_watermark = Some(reservation.sequence);
        self.greatest_reserved_target_tick = Some(reservation.target_tick);
        if reservation.sequence == u64::MAX {
            self.state = CommandStreamStateV1::Exhausted;
        }
        Arc::make_mut(&mut self.pending).insert(reservation.sequence, reservation);
        Ok(())
    }

    pub fn lock_for_collision(
        &mut self,
        incident: CommandCollisionIncidentV1,
    ) -> Result<(), CommandLedgerError> {
        self.record_collision(incident, None)
    }

    pub fn append_collision_receipt(
        &mut self,
        incident: CommandCollisionIncidentV1,
        receipt: CommandReceiptV1,
    ) -> Result<(), CommandLedgerError> {
        self.record_collision(incident, Some(receipt))
    }

    fn record_collision(
        &mut self,
        incident: CommandCollisionIncidentV1,
        receipt: Option<CommandReceiptV1>,
    ) -> Result<(), CommandLedgerError> {
        self.validate()?;
        incident.validate()?;
        if incident.stream_id != self.stream_id || incident.issuer != self.issuer {
            return Err(CommandLedgerError::CollisionIncidentMismatch);
        }
        if self.state == CommandStreamStateV1::CollisionLocked {
            return if self.collision_incident.as_ref() == Some(&incident) && receipt.is_none() {
                Ok(())
            } else {
                Err(CommandLedgerError::UnexpectedCollisionReceipt)
            };
        }
        if self.finalized_receipt_count == u64::MAX {
            return Err(CommandLedgerError::FinalizationOrdinalExhausted);
        }
        if self.state != CommandStreamStateV1::Open {
            return Err(CommandLedgerError::StreamNotOpen);
        }
        let sequence_is_retained = self.pending.contains_key(&incident.sequence)
            || self
                .receipt_window
                .iter()
                .any(|receipt| receipt.subject.sequence() == incident.sequence);
        match (sequence_is_retained, receipt.as_ref()) {
            (true, Some(_)) => return Err(CommandLedgerError::UnexpectedCollisionReceipt),
            (false, None) => return Err(CommandLedgerError::CollisionReceiptRequired),
            (false, Some(_))
                if self
                    .admission_high_watermark
                    .is_some_and(|high_watermark| incident.sequence <= high_watermark) =>
            {
                return Err(CommandLedgerError::SequenceNotNew);
            }
            _ => {}
        }

        let mut next = self.clone();
        next.admission_high_watermark = Some(
            next.admission_high_watermark
                .map_or(incident.sequence, |high_watermark| {
                    high_watermark.max(incident.sequence)
                }),
        );
        next.state = CommandStreamStateV1::CollisionLocked;
        next.collision_incident = Some(incident);
        if let Some(receipt) = receipt {
            next.append_receipt_inner(receipt, true)?;
        }
        next.validate()?;
        *self = next;
        Ok(())
    }

    pub fn append_receipt(&mut self, receipt: CommandReceiptV1) -> Result<(), CommandLedgerError> {
        self.append_receipt_inner(receipt, false)
    }

    fn append_receipt_inner(
        &mut self,
        receipt: CommandReceiptV1,
        allow_collision: bool,
    ) -> Result<(), CommandLedgerError> {
        self.validate_append_boundary()?;
        receipt.validate()?;
        if self.finalized_receipt_count == u64::MAX {
            return Err(CommandLedgerError::FinalizationOrdinalExhausted);
        }
        if matches!(
            receipt.subject,
            CommandReceiptSubjectV1::CollisionSet { .. }
        ) && !allow_collision
        {
            return Err(CommandLedgerError::UnexpectedCollisionReceipt);
        }
        if receipt.subject.stream_id() != self.stream_id || receipt.subject.issuer() != &self.issuer
        {
            return Err(CommandLedgerError::ReceiptSubjectMismatch);
        }
        if receipt.finalization_ordinal != self.finalized_receipt_count {
            return Err(CommandLedgerError::FinalizationOrdinalMismatch);
        }
        let sequence = receipt.subject.sequence();
        let pending = self.pending.get(&sequence);
        if let Some(reservation) = pending {
            validate_receipt_against_reservation(&receipt, reservation)?;
        }
        let collision_is_recorded =
            receipt_matches_collision_incident(&receipt, self.collision_incident.as_ref());
        if self
            .admission_high_watermark
            .is_some_and(|high_watermark| sequence <= high_watermark)
            && pending.is_none()
            && !collision_is_recorded
        {
            return Err(CommandLedgerError::SequenceNotNew);
        }
        if self.state != CommandStreamStateV1::Open && pending.is_none() && !collision_is_recorded {
            return Err(CommandLedgerError::StreamNotOpen);
        }
        if allow_collision && !collision_is_recorded {
            return Err(CommandLedgerError::CollisionIncidentMismatch);
        }

        let next_high_watermark = Some(
            self.admission_high_watermark
                .map_or(sequence, |high_watermark| high_watermark.max(sequence)),
        );
        let next_chain_root = command_receipt_chain_next(self.receipt_chain_root, &receipt)?;
        let next_count = self
            .finalized_receipt_count
            .checked_add(1)
            .ok_or(CommandLedgerError::FinalizationOrdinalExhausted)?;

        self.admission_high_watermark = next_high_watermark;
        Arc::make_mut(&mut self.pending).remove(&sequence);
        self.receipt_chain_root = next_chain_root;
        self.receipt_window.push(receipt);
        self.finalized_receipt_count = next_count;
        if self.state == CommandStreamStateV1::Open
            && (sequence == u64::MAX || self.finalized_receipt_count == u64::MAX)
        {
            self.state = CommandStreamStateV1::Exhausted;
        }
        Ok(())
    }

    pub fn validate(&self) -> Result<(), CommandLedgerError> {
        if self.schema_version != COMMAND_STREAM_LEDGER_SCHEMA_VERSION {
            return Err(CommandLedgerError::UnsupportedStreamLedgerVersion(
                self.schema_version,
            ));
        }
        if self.pending.len() > COMMAND_PENDING_CAPACITY {
            return Err(CommandLedgerError::PendingLimit);
        }
        for (sequence, reservation) in self.pending.iter() {
            if sequence != &reservation.sequence
                || reservation.stream_id != self.stream_id
                || reservation.issuer != self.issuer
                || reservation.schema_version != COMMAND_RESERVATION_SCHEMA_VERSION
                || reservation.body_hash != reservation.canonical_body_ref
                || self
                    .admission_high_watermark
                    .is_none_or(|high_watermark| *sequence > high_watermark)
            {
                return Err(CommandLedgerError::ReservationMismatch);
            }
            if self
                .greatest_reserved_target_tick
                .is_none_or(|greatest| reservation.target_tick > greatest)
            {
                return Err(CommandLedgerError::TargetTickRegression);
            }
        }
        let expected_window_len = usize::try_from(
            self.finalized_receipt_count
                .min(COMMAND_RECEIPT_WINDOW_CAPACITY as u64),
        )
        .map_err(|_| CommandLedgerError::CountOverflow)?;
        if self.receipt_window.len() != expected_window_len {
            return Err(CommandLedgerError::ReceiptWindowLengthMismatch);
        }
        let first_ordinal = self
            .finalized_receipt_count
            .checked_sub(
                u64::try_from(self.receipt_window.len())
                    .map_err(|_| CommandLedgerError::CountOverflow)?,
            )
            .ok_or(CommandLedgerError::CountOverflow)?;
        let mut retained_sequences = BTreeSet::new();
        for (index, receipt) in self.receipt_window.iter().enumerate() {
            receipt.validate()?;
            let expected_ordinal = first_ordinal
                .checked_add(u64::try_from(index).map_err(|_| CommandLedgerError::CountOverflow)?)
                .ok_or(CommandLedgerError::CountOverflow)?;
            if receipt.finalization_ordinal != expected_ordinal
                || receipt.subject.stream_id() != self.stream_id
                || receipt.subject.issuer() != &self.issuer
                || self
                    .admission_high_watermark
                    .is_none_or(|high_watermark| receipt.subject.sequence() > high_watermark)
                || !retained_sequences.insert(receipt.subject.sequence())
                || matches!(
                    receipt.subject,
                    CommandReceiptSubjectV1::CollisionSet { .. }
                ) && !receipt_matches_collision_incident(
                    receipt,
                    self.collision_incident.as_ref(),
                )
            {
                return Err(CommandLedgerError::ReceiptWindowInvalid);
            }
        }
        if self.finalized_receipt_count == 0 {
            if self.receipt_chain_root != command_receipt_chain_genesis() {
                return Err(CommandLedgerError::ReceiptChainRootMismatch);
            }
        } else if self.finalized_receipt_count <= COMMAND_RECEIPT_WINDOW_CAPACITY as u64 {
            let mut root = command_receipt_chain_genesis();
            for receipt in self.receipt_window.iter() {
                root = command_receipt_chain_next(root, receipt)?;
            }
            if root != self.receipt_chain_root {
                return Err(CommandLedgerError::ReceiptChainRootMismatch);
            }
        }
        match (&self.state, &self.collision_incident) {
            (CommandStreamStateV1::CollisionLocked, Some(incident)) => {
                incident.validate()?;
                if incident.stream_id != self.stream_id || incident.issuer != self.issuer {
                    return Err(CommandLedgerError::CollisionIncidentMismatch);
                }
            }
            (CommandStreamStateV1::CollisionLocked, None) => {
                return Err(CommandLedgerError::CollisionIncidentMismatch);
            }
            (_, Some(_)) => return Err(CommandLedgerError::CollisionIncidentMismatch),
            _ => {}
        }
        if self.state == CommandStreamStateV1::Open
            && (self.admission_high_watermark == Some(u64::MAX)
                || self.finalized_receipt_count == u64::MAX)
        {
            return Err(CommandLedgerError::OpenStreamExhausted);
        }
        if self.admission_high_watermark.is_none()
            && (!self.pending.is_empty() || !self.receipt_window.is_empty())
        {
            return Err(CommandLedgerError::HighWatermarkMissing);
        }
        Ok(())
    }

    /// Bounded live-checkpoint validation for a stream assembled only through
    /// the checked reserve/append/collision mutation APIs.
    ///
    /// Every retained receipt was fully validated at its own append boundary,
    /// ordinals form a proven contiguous run, the receipt window is privately
    /// owned copy-on-write storage, and the receipt-chain root is advanced
    /// exactly once per append. Re-validating and re-hashing every retained
    /// receipt at each checkpoint would duplicate those proofs, so this
    /// variant checks only the bounded current closure. The complete
    /// validator remains mandatory on decode, restore, migration and any
    /// other untrusted-data boundary.
    pub(crate) fn validate_incremental_checkpoint(&self) -> Result<(), CommandLedgerError> {
        if self.schema_version != COMMAND_STREAM_LEDGER_SCHEMA_VERSION {
            return Err(CommandLedgerError::UnsupportedStreamLedgerVersion(
                self.schema_version,
            ));
        }
        if self.pending.len() > COMMAND_PENDING_CAPACITY {
            return Err(CommandLedgerError::PendingLimit);
        }
        for (sequence, reservation) in self.pending.iter() {
            if sequence != &reservation.sequence
                || reservation.stream_id != self.stream_id
                || reservation.issuer != self.issuer
                || reservation.schema_version != COMMAND_RESERVATION_SCHEMA_VERSION
                || reservation.body_hash != reservation.canonical_body_ref
                || self
                    .admission_high_watermark
                    .is_none_or(|high_watermark| *sequence > high_watermark)
            {
                return Err(CommandLedgerError::ReservationMismatch);
            }
            if self
                .greatest_reserved_target_tick
                .is_none_or(|greatest| reservation.target_tick > greatest)
            {
                return Err(CommandLedgerError::TargetTickRegression);
            }
        }
        let expected_window_len = usize::try_from(
            self.finalized_receipt_count
                .min(COMMAND_RECEIPT_WINDOW_CAPACITY as u64),
        )
        .map_err(|_| CommandLedgerError::CountOverflow)?;
        if self.receipt_window.len() != expected_window_len {
            return Err(CommandLedgerError::ReceiptWindowLengthMismatch);
        }
        if self.finalized_receipt_count == 0 {
            if self.receipt_chain_root != command_receipt_chain_genesis() {
                return Err(CommandLedgerError::ReceiptChainRootMismatch);
            }
        } else {
            let first_ordinal = self
                .finalized_receipt_count
                .checked_sub(
                    u64::try_from(self.receipt_window.len())
                        .map_err(|_| CommandLedgerError::CountOverflow)?,
                )
                .ok_or(CommandLedgerError::CountOverflow)?;
            let first = self
                .receipt_window
                .first()
                .ok_or(CommandLedgerError::ReceiptWindowInvalid)?;
            let last = self
                .receipt_window
                .last()
                .ok_or(CommandLedgerError::ReceiptWindowInvalid)?;
            if first.finalization_ordinal != first_ordinal
                || last.finalization_ordinal.checked_add(1) != Some(self.finalized_receipt_count)
                || first.subject.stream_id() != self.stream_id
                || last.subject.stream_id() != self.stream_id
                || first.subject.issuer() != &self.issuer
                || last.subject.issuer() != &self.issuer
            {
                return Err(CommandLedgerError::ReceiptWindowInvalid);
            }
        }
        match (&self.state, &self.collision_incident) {
            (CommandStreamStateV1::CollisionLocked, Some(incident)) => {
                incident.validate()?;
                if incident.stream_id != self.stream_id || incident.issuer != self.issuer {
                    return Err(CommandLedgerError::CollisionIncidentMismatch);
                }
            }
            (CommandStreamStateV1::CollisionLocked, None) => {
                return Err(CommandLedgerError::CollisionIncidentMismatch);
            }
            (_, Some(_)) => return Err(CommandLedgerError::CollisionIncidentMismatch),
            _ => {}
        }
        if self.state == CommandStreamStateV1::Open
            && (self.admission_high_watermark == Some(u64::MAX)
                || self.finalized_receipt_count == u64::MAX)
        {
            return Err(CommandLedgerError::OpenStreamExhausted);
        }
        if self.admission_high_watermark.is_none()
            && (!self.pending.is_empty() || !self.receipt_window.is_empty())
        {
            return Err(CommandLedgerError::HighWatermarkMissing);
        }
        Ok(())
    }

    fn validate_append_boundary(&self) -> Result<(), CommandLedgerError> {
        if self.schema_version != COMMAND_STREAM_LEDGER_SCHEMA_VERSION {
            return Err(CommandLedgerError::UnsupportedStreamLedgerVersion(
                self.schema_version,
            ));
        }
        let expected_window_len = usize::try_from(
            self.finalized_receipt_count
                .min(COMMAND_RECEIPT_WINDOW_CAPACITY as u64),
        )
        .map_err(|_| CommandLedgerError::CountOverflow)?;
        if self.receipt_window.len() != expected_window_len {
            return Err(CommandLedgerError::ReceiptWindowLengthMismatch);
        }
        if self.finalized_receipt_count == 0 {
            if self.receipt_chain_root != command_receipt_chain_genesis() {
                return Err(CommandLedgerError::ReceiptChainRootMismatch);
            }
            return Ok(());
        }
        let first_ordinal = self
            .finalized_receipt_count
            .checked_sub(
                u64::try_from(self.receipt_window.len())
                    .map_err(|_| CommandLedgerError::CountOverflow)?,
            )
            .ok_or(CommandLedgerError::CountOverflow)?;
        let last_ordinal = self
            .finalized_receipt_count
            .checked_sub(1)
            .ok_or(CommandLedgerError::CountOverflow)?;
        if self
            .receipt_window
            .first()
            .is_none_or(|receipt| receipt.finalization_ordinal != first_ordinal)
            || self
                .receipt_window
                .last()
                .is_none_or(|receipt| receipt.finalization_ordinal != last_ordinal)
        {
            return Err(CommandLedgerError::ReceiptWindowInvalid);
        }
        Ok(())
    }
}
