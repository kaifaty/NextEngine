use super::*;

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
#[repr(u8)]
pub enum BodyImpairmentV1 {
    Intact = 1,
    PartialKneeExtensor = 2,
    TendonTransmissionLost = 3,
    NerveControlLost = 4,
}

impl BodyImpairmentV1 {
    pub(super) fn from_tag(tag: u8) -> Result<Self, RpgContractErrorV1> {
        match tag {
            1 => Ok(Self::Intact),
            2 => Ok(Self::PartialKneeExtensor),
            3 => Ok(Self::TendonTransmissionLost),
            4 => Ok(Self::NerveControlLost),
            value => Err(RpgContractErrorV1::InvalidTag(value)),
        }
    }

    #[must_use]
    pub const fn permits_damage_transition_to(self, next: Self) -> bool {
        matches!(
            (self, next),
            (Self::Intact, Self::PartialKneeExtensor)
                | (
                    Self::Intact,
                    Self::TendonTransmissionLost | Self::NerveControlLost
                )
                | (
                    Self::PartialKneeExtensor,
                    Self::TendonTransmissionLost | Self::NerveControlLost
                )
        )
    }
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
#[repr(u8)]
pub enum BodyRecoveryStageV1 {
    Untreated = 1,
    Stabilized = 2,
    Repaired = 3,
    Rehabilitated = 4,
}

impl BodyRecoveryStageV1 {
    pub(super) fn from_tag(tag: u8) -> Result<Self, RpgContractErrorV1> {
        match tag {
            1 => Ok(Self::Untreated),
            2 => Ok(Self::Stabilized),
            3 => Ok(Self::Repaired),
            4 => Ok(Self::Rehabilitated),
            value => Err(RpgContractErrorV1::InvalidTag(value)),
        }
    }

    #[must_use]
    pub const fn permits_transition_to(self, next: Self) -> bool {
        matches!(
            (self, next),
            (Self::Untreated, Self::Stabilized)
                | (Self::Stabilized, Self::Repaired)
                | (Self::Repaired, Self::Rehabilitated)
        )
    }
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
#[repr(u8)]
pub enum BodyTreatmentChannelV1 {
    Medical = 1,
    Magical = 2,
}

impl BodyTreatmentChannelV1 {
    pub(super) fn from_tag(tag: u8) -> Result<Self, RpgContractErrorV1> {
        match tag {
            1 => Ok(Self::Medical),
            2 => Ok(Self::Magical),
            value => Err(RpgContractErrorV1::InvalidTag(value)),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
#[repr(u8)]
pub enum SystemicConditionV1 {
    Stable = 1,
    Impaired = 2,
}

impl SystemicConditionV1 {
    pub(super) fn from_tag(tag: u8) -> Result<Self, RpgContractErrorV1> {
        match tag {
            1 => Ok(Self::Stable),
            2 => Ok(Self::Impaired),
            value => Err(RpgContractErrorV1::InvalidTag(value)),
        }
    }
}
