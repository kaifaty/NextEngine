use super::*;

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
#[repr(u8)]
pub enum SpeechActKindV1 {
    Inform = 1,
    Ask = 2,
    Request = 3,
    Offer = 4,
    CounterOffer = 5,
    Accept = 6,
    Reject = 7,
    Promise = 8,
    Warn = 9,
    Threaten = 10,
    Thank = 11,
    Apologize = 12,
    Insult = 13,
    Praise = 14,
    Gossip = 15,
}

impl SpeechActKindV1 {
    fn from_tag(tag: u8) -> Result<Self, CognitionContractError> {
        match tag {
            1 => Ok(Self::Inform),
            2 => Ok(Self::Ask),
            3 => Ok(Self::Request),
            4 => Ok(Self::Offer),
            5 => Ok(Self::CounterOffer),
            6 => Ok(Self::Accept),
            7 => Ok(Self::Reject),
            8 => Ok(Self::Promise),
            9 => Ok(Self::Warn),
            10 => Ok(Self::Threaten),
            11 => Ok(Self::Thank),
            12 => Ok(Self::Apologize),
            13 => Ok(Self::Insult),
            14 => Ok(Self::Praise),
            15 => Ok(Self::Gossip),
            value => Err(CognitionContractError::UnknownTag(value)),
        }
    }

    const fn requires_claim(self) -> bool {
        matches!(
            self,
            Self::Inform
                | Self::Offer
                | Self::CounterOffer
                | Self::Promise
                | Self::Warn
                | Self::Threaten
                | Self::Gossip
        )
    }

    const fn forbids_claim(self) -> bool {
        matches!(
            self,
            Self::Accept | Self::Reject | Self::Thank | Self::Apologize
        )
    }

    const fn requires_reply(self) -> bool {
        matches!(
            self,
            Self::CounterOffer | Self::Accept | Self::Reject | Self::Thank | Self::Apologize
        )
    }
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct SpeechClaimV1 {
    pub claim_id: ContentHash,
    pub subject_id: PersistentId,
    pub predicate_id: SchemaId,
    pub value_id: SchemaId,
    pub confidence_q16: u32,
    pub cited_belief_ids: Vec<ContentHash>,
}

impl SpeechClaimV1 {
    pub fn new(
        subject_id: PersistentId,
        predicate_id: SchemaId,
        value_id: SchemaId,
        confidence_q16: u32,
        mut cited_belief_ids: Vec<ContentHash>,
    ) -> Result<Self, CognitionContractError> {
        cited_belief_ids.sort_unstable();
        cited_belief_ids.dedup();
        let mut value = Self {
            claim_id: ContentHash::default(),
            subject_id,
            predicate_id,
            value_id,
            confidence_q16,
            cited_belief_ids,
        };
        value.claim_id = value.computed_id()?;
        value.validate()?;
        Ok(value)
    }

    pub fn validate(&self) -> Result<(), CognitionContractError> {
        if self.confidence_q16 > COGNITION_Q16_ONE.unsigned_abs()
            || self.cited_belief_ids.is_empty()
            || self.cited_belief_ids.len() > COGNITION_MAX_CLAIM_PROVENANCE
            || self
                .cited_belief_ids
                .windows(2)
                .any(|pair| pair[0] >= pair[1])
            || self.claim_id != self.computed_id()?
        {
            return Err(CognitionContractError::SocialClaimInvalid);
        }
        Ok(())
    }

    fn computed_id(&self) -> Result<ContentHash, CognitionContractError> {
        let mut writer = Writer::with_domain(b"nextengine.speech-claim.v1\0");
        writer.id(self.subject_id);
        writer.text(self.predicate_id.as_str())?;
        writer.text(self.value_id.as_str())?;
        writer.u32(self.confidence_q16);
        writer.count(self.cited_belief_ids.len())?;
        for belief_id in &self.cited_belief_ids {
            writer.hash(*belief_id);
        }
        Ok(writer.finish_hash())
    }
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct StructuredSpeechActV1 {
    pub schema_version: u16,
    pub act_id: ContentHash,
    pub speaker_id: PersistentId,
    pub listener_id: PersistentId,
    pub kind: SpeechActKindV1,
    pub topic_id: SchemaId,
    pub exchange_ordinal: u16,
    pub claim_or_none: Option<SpeechClaimV1>,
    pub requested_response_or_none: Option<SpeechActKindV1>,
    pub in_reply_to_act_id_or_none: Option<ContentHash>,
    pub creation_tick: u64,
    pub expiry_tick: u64,
}

impl StructuredSpeechActV1 {
    #[allow(
        clippy::too_many_arguments,
        reason = "the structured act keeps every semantic boundary explicit"
    )]
    pub fn new(
        speaker_id: PersistentId,
        listener_id: PersistentId,
        kind: SpeechActKindV1,
        topic_id: SchemaId,
        exchange_ordinal: u16,
        claim_or_none: Option<SpeechClaimV1>,
        requested_response_or_none: Option<SpeechActKindV1>,
        in_reply_to_act_id_or_none: Option<ContentHash>,
        creation_tick: u64,
        expiry_tick: u64,
    ) -> Result<Self, CognitionContractError> {
        let mut value = Self {
            schema_version: COGNITION_SCHEMA_VERSION,
            act_id: ContentHash::default(),
            speaker_id,
            listener_id,
            kind,
            topic_id,
            exchange_ordinal,
            claim_or_none,
            requested_response_or_none,
            in_reply_to_act_id_or_none,
            creation_tick,
            expiry_tick,
        };
        value.act_id = value.computed_id()?;
        value.validate()?;
        Ok(value)
    }

    pub fn validate(&self) -> Result<(), CognitionContractError> {
        let claim_present = self.claim_or_none.is_some();
        if self.schema_version != COGNITION_SCHEMA_VERSION
            || self.speaker_id == self.listener_id
            || self.expiry_tick <= self.creation_tick
            || self.kind.requires_claim() != claim_present && self.kind.requires_claim()
            || self.kind.forbids_claim() && claim_present
            || self.kind.requires_reply() != self.in_reply_to_act_id_or_none.is_some()
                && self.kind.requires_reply()
            || self
                .claim_or_none
                .as_ref()
                .is_some_and(|claim| claim.validate().is_err())
            || self.act_id != self.computed_id()?
        {
            return Err(CognitionContractError::SpeechActInvalid);
        }
        Ok(())
    }

    pub fn canonical_payload_bytes(&self) -> Result<Vec<u8>, CognitionContractError> {
        self.validate()?;
        let mut writer = Writer::with_domain(b"nextengine.structured-speech-act.v1\0");
        writer.u16(self.schema_version);
        writer.hash(self.act_id);
        write_speech_act_identity(&mut writer, self)?;
        Ok(writer.into_bytes())
    }

    pub fn from_canonical_payload_bytes(
        bytes: &[u8],
        limits: CanonicalDecodeLimits,
    ) -> Result<Self, CognitionContractError> {
        let mut reader = Reader::new(bytes, limits);
        reader.domain(b"nextengine.structured-speech-act.v1\0")?;
        let schema_version = reader.u16()?;
        let act_id = reader.hash()?;
        let value = Self {
            schema_version,
            act_id,
            speaker_id: reader.id()?,
            listener_id: reader.id()?,
            kind: SpeechActKindV1::from_tag(reader.u8()?)?,
            topic_id: reader.schema_id()?,
            exchange_ordinal: reader.u16()?,
            claim_or_none: read_optional_claim(&mut reader)?,
            requested_response_or_none: read_optional_speech_kind(&mut reader)?,
            in_reply_to_act_id_or_none: reader.optional_hash()?,
            creation_tick: reader.u64()?,
            expiry_tick: reader.u64()?,
        };
        reader.finish()?;
        value.validate()?;
        if value.canonical_payload_bytes()? != bytes {
            return Err(CognitionContractError::NonCanonicalEncoding);
        }
        Ok(value)
    }

    fn computed_id(&self) -> Result<ContentHash, CognitionContractError> {
        let mut writer = Writer::with_domain(b"nextengine.structured-speech-act-id.v1\0");
        writer.u16(self.schema_version);
        write_speech_act_identity(&mut writer, self)?;
        Ok(writer.finish_hash())
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StructuredSpeechExchangeV1 {
    pub acts: Vec<StructuredSpeechActV1>,
}

impl StructuredSpeechExchangeV1 {
    pub fn validate(&self) -> Result<(), CognitionContractError> {
        if self.acts.is_empty()
            || self.acts.len() > COGNITION_MAX_SPEECH_ACTS
            || self.acts.iter().enumerate().any(|(index, act)| {
                usize::from(act.exchange_ordinal) != index
                    || index > 0 && self.acts[index - 1].creation_tick > act.creation_tick
            })
            || self.acts.iter().any(|act| act.validate().is_err())
        {
            return Err(CognitionContractError::SpeechExchangeInvalid);
        }
        for (index, act) in self.acts.iter().enumerate() {
            let Some(reply_id) = act.in_reply_to_act_id_or_none else {
                continue;
            };
            let replied_to = self.acts[..index]
                .iter()
                .find(|candidate| candidate.act_id == reply_id)
                .ok_or(CognitionContractError::SpeechExchangeInvalid)?;
            if act.speaker_id != replied_to.listener_id
                || act.listener_id != replied_to.speaker_id
                || matches!(act.kind, SpeechActKindV1::Accept | SpeechActKindV1::Reject)
                    && !matches!(
                        replied_to.kind,
                        SpeechActKindV1::Offer | SpeechActKindV1::CounterOffer
                    )
            {
                return Err(CognitionContractError::SpeechExchangeInvalid);
            }
        }
        Ok(())
    }

    pub fn validate_work_exchange(&self) -> Result<(), CognitionContractError> {
        self.validate()?;
        if self.acts.len() != 4
            || self.acts[0].kind != SpeechActKindV1::Ask
            || self.acts[1].kind != SpeechActKindV1::Inform
            || self.acts[2].kind != SpeechActKindV1::Offer
            || self.acts[3].kind != SpeechActKindV1::Accept
            || self.acts[1].in_reply_to_act_id_or_none != Some(self.acts[0].act_id)
            || self.acts[3].in_reply_to_act_id_or_none != Some(self.acts[2].act_id)
            || self
                .acts
                .iter()
                .any(|act| act.topic_id != self.acts[0].topic_id)
        {
            return Err(CognitionContractError::SpeechExchangeInvalid);
        }
        Ok(())
    }

    pub fn canonical_bytes(&self) -> Result<Vec<u8>, CognitionContractError> {
        self.validate()?;
        let mut writer = Writer::with_domain(b"nextengine.structured-speech-exchange.v1\0");
        writer.count(self.acts.len())?;
        for act in &self.acts {
            writer.bytes(&act.canonical_payload_bytes()?)?;
        }
        Ok(writer.into_bytes())
    }

    pub fn canonical_hash(&self) -> Result<ContentHash, CognitionContractError> {
        Ok(content_hash_from_bytes(sha256(&self.canonical_bytes()?)))
    }
}

pub fn received_claim_confidence_q16(
    speaker_confidence_q16: u32,
    listener_trust_q16: u32,
) -> Result<u32, CognitionContractError> {
    let one = COGNITION_Q16_ONE.unsigned_abs();
    if speaker_confidence_q16 > one || listener_trust_q16 > one {
        return Err(CognitionContractError::SocialClaimInvalid);
    }
    let product = u64::from(speaker_confidence_q16) * u64::from(listener_trust_q16);
    u32::try_from((product + u64::from(one / 2)) / u64::from(one))
        .map_err(|_| CognitionContractError::SocialClaimInvalid)
}

fn write_speech_act_identity(
    writer: &mut Writer,
    value: &StructuredSpeechActV1,
) -> Result<(), CognitionContractError> {
    writer.id(value.speaker_id);
    writer.id(value.listener_id);
    writer.u8(value.kind as u8);
    writer.text(value.topic_id.as_str())?;
    writer.u16(value.exchange_ordinal);
    write_optional_claim(writer, value.claim_or_none.as_ref())?;
    write_optional_speech_kind(writer, value.requested_response_or_none);
    writer.optional_hash(value.in_reply_to_act_id_or_none);
    writer.u64(value.creation_tick);
    writer.u64(value.expiry_tick);
    Ok(())
}

fn write_optional_claim(
    writer: &mut Writer,
    claim: Option<&SpeechClaimV1>,
) -> Result<(), CognitionContractError> {
    let Some(claim) = claim else {
        writer.u8(0);
        return Ok(());
    };
    claim.validate()?;
    writer.u8(1);
    writer.hash(claim.claim_id);
    writer.id(claim.subject_id);
    writer.text(claim.predicate_id.as_str())?;
    writer.text(claim.value_id.as_str())?;
    writer.u32(claim.confidence_q16);
    writer.count(claim.cited_belief_ids.len())?;
    for belief_id in &claim.cited_belief_ids {
        writer.hash(*belief_id);
    }
    Ok(())
}

fn read_optional_claim(
    reader: &mut Reader<'_>,
) -> Result<Option<SpeechClaimV1>, CognitionContractError> {
    match reader.u8()? {
        0 => Ok(None),
        1 => {
            let claim = SpeechClaimV1 {
                claim_id: reader.hash()?,
                subject_id: reader.id()?,
                predicate_id: reader.schema_id()?,
                value_id: reader.schema_id()?,
                confidence_q16: reader.u32()?,
                cited_belief_ids: (0..reader.count(COGNITION_MAX_CLAIM_PROVENANCE)?)
                    .map(|_| reader.hash())
                    .collect::<Result<Vec<_>, _>>()?,
            };
            claim.validate()?;
            Ok(Some(claim))
        }
        value => Err(CognitionContractError::UnknownTag(value)),
    }
}

fn write_optional_speech_kind(writer: &mut Writer, value: Option<SpeechActKindV1>) {
    match value {
        None => writer.u8(0),
        Some(value) => {
            writer.u8(1);
            writer.u8(value as u8);
        }
    }
}

fn read_optional_speech_kind(
    reader: &mut Reader<'_>,
) -> Result<Option<SpeechActKindV1>, CognitionContractError> {
    match reader.u8()? {
        0 => Ok(None),
        1 => Ok(Some(SpeechActKindV1::from_tag(reader.u8()?)?)),
        value => Err(CognitionContractError::UnknownTag(value)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn id(byte: u8) -> PersistentId {
        PersistentId::from_bytes([byte; 16])
    }

    fn schema(value: &str) -> SchemaId {
        SchemaId::new(value).expect("schema")
    }

    fn claim(subject_id: PersistentId) -> SpeechClaimV1 {
        SpeechClaimV1::new(
            subject_id,
            schema("nextengine.claim.work-available"),
            schema("nextengine.claim-value.relay-shift"),
            COGNITION_Q16_ONE.unsigned_abs(),
            vec![ContentHash::from_bytes([0x41; 32])],
        )
        .expect("claim")
    }

    #[test]
    fn work_exchange_round_trips_without_a_truth_flag() {
        let worker = id(1);
        let broker = id(2);
        let topic = schema("nextengine.topic.relay-work");
        let ask = StructuredSpeechActV1::new(
            worker,
            broker,
            SpeechActKindV1::Ask,
            topic.clone(),
            0,
            None,
            Some(SpeechActKindV1::Inform),
            None,
            1,
            9,
        )
        .expect("ask");
        let inform = StructuredSpeechActV1::new(
            broker,
            worker,
            SpeechActKindV1::Inform,
            topic.clone(),
            1,
            Some(claim(broker)),
            None,
            Some(ask.act_id),
            2,
            9,
        )
        .expect("inform");
        let offer = StructuredSpeechActV1::new(
            broker,
            worker,
            SpeechActKindV1::Offer,
            topic.clone(),
            2,
            Some(claim(broker)),
            Some(SpeechActKindV1::Accept),
            None,
            3,
            9,
        )
        .expect("offer");
        let accept = StructuredSpeechActV1::new(
            worker,
            broker,
            SpeechActKindV1::Accept,
            topic,
            3,
            None,
            None,
            Some(offer.act_id),
            4,
            9,
        )
        .expect("accept");
        let exchange = StructuredSpeechExchangeV1 {
            acts: vec![ask, inform.clone(), offer, accept],
        };
        exchange.validate_work_exchange().expect("valid exchange");

        let bytes = inform.canonical_payload_bytes().expect("encode");
        assert_eq!(
            StructuredSpeechActV1::from_canonical_payload_bytes(
                &bytes,
                CanonicalDecodeLimits::default()
            )
            .expect("decode"),
            inform
        );
        assert_eq!(
            received_claim_confidence_q16(COGNITION_Q16_ONE.unsigned_abs(), 49_152)
                .expect("confidence"),
            49_152
        );
    }

    #[test]
    fn accept_requires_a_real_prior_offer_and_claim_provenance_is_bounded() {
        let invalid_accept = StructuredSpeechActV1::new(
            id(1),
            id(2),
            SpeechActKindV1::Accept,
            schema("nextengine.topic.work"),
            0,
            None,
            None,
            None,
            1,
            2,
        );
        assert_eq!(
            invalid_accept,
            Err(CognitionContractError::SpeechActInvalid)
        );
        assert_eq!(
            SpeechClaimV1::new(
                id(1),
                schema("nextengine.claim.work"),
                schema("nextengine.claim-value.available"),
                COGNITION_Q16_ONE.unsigned_abs(),
                Vec::new(),
            ),
            Err(CognitionContractError::SocialClaimInvalid)
        );
    }
}
