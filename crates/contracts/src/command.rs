mod body;
mod codec;
mod envelope;
mod error;
mod event;
mod principal;

pub use body::{
    COMMAND_BODY_OWNER_ID, COMMAND_BODY_SCHEMA_ID, COMMAND_BODY_SCHEMA_VERSION,
    COMMAND_BODY_SEGMENT_ID, COMMAND_SCHEMA_VERSION, CanonicalCommandBodyV2, CapabilityRefV1,
    CommandPayload, CommandPhase, CommandPreconditionV1, NOOP_COMMAND_CAPABILITY_ID,
    NOOP_COMMAND_SCHEMA_ID,
};
pub use envelope::{
    COMMAND_ENVELOPE_SCHEMA_VERSION, WorldCommand, WorldCommandEnvelopeV2,
    compute_command_id_from_body_bytes,
};
pub use error::CommandDecodeError;
pub use event::{DomainEvent, DomainEventEnvelopeV2, EventPayload};
pub use principal::{IssuerPrincipal, IssuerPrincipalV2, PrincipalDecodeError};

#[cfg(test)]
mod tests;
