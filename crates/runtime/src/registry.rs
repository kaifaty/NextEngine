use std::collections::BTreeMap;

use next_contracts::{
    CapabilityId, CommandPayload, CommandPhase, NOOP_COMMAND_CAPABILITY_ID, NOOP_COMMAND_SCHEMA_ID,
    RPG_COMMAND_CAPABILITY_ID, RPG_COMMAND_SCHEMA_ID, SchemaId,
};

pub const COMMAND_KIND_REGISTRY_VERSION: u32 = 1;
pub const NOOP_PRIORITY_CLASS: u16 = 100;
pub const RPG_PRIORITY_CLASS: u16 = 200;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CommandPayloadKind {
    Noop,
    Rpg,
}

impl CommandPayloadKind {
    #[must_use]
    pub const fn of(payload: &CommandPayload) -> Self {
        match payload {
            CommandPayload::Noop => Self::Noop,
            CommandPayload::Rpg(_) => Self::Rpg,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CommandKindDescriptor {
    schema_id: SchemaId,
    schema_version: u32,
    payload_kind: CommandPayloadKind,
    priority_class: u16,
    required_capabilities: Vec<CapabilityId>,
    ingress_allowed: bool,
    outcome_allowed: bool,
}

impl CommandKindDescriptor {
    #[must_use]
    pub fn schema_id(&self) -> &SchemaId {
        &self.schema_id
    }

    #[must_use]
    pub const fn schema_version(&self) -> u32 {
        self.schema_version
    }

    #[must_use]
    pub const fn priority_class(&self) -> u16 {
        self.priority_class
    }

    #[must_use]
    pub fn required_capabilities(&self) -> &[CapabilityId] {
        &self.required_capabilities
    }

    #[must_use]
    pub const fn accepts_payload(&self, payload: &CommandPayload) -> bool {
        matches!(
            (self.payload_kind, CommandPayloadKind::of(payload)),
            (CommandPayloadKind::Noop, CommandPayloadKind::Noop)
                | (CommandPayloadKind::Rpg, CommandPayloadKind::Rpg)
        )
    }

    #[must_use]
    pub const fn allows_phase(&self, phase: CommandPhase) -> bool {
        match phase {
            CommandPhase::Ingress => self.ingress_allowed,
            CommandPhase::Outcome => self.outcome_allowed,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CommandKindRegistry {
    version: u32,
    descriptors: BTreeMap<(SchemaId, u32), CommandKindDescriptor>,
}

impl CommandKindRegistry {
    #[must_use]
    pub fn core_v1() -> Self {
        let noop_schema_id =
            SchemaId::new(NOOP_COMMAND_SCHEMA_ID).expect("built-in command schema ID is valid");
        let noop_descriptor = CommandKindDescriptor {
            schema_id: noop_schema_id.clone(),
            schema_version: 1,
            payload_kind: CommandPayloadKind::Noop,
            priority_class: NOOP_PRIORITY_CLASS,
            required_capabilities: vec![
                CapabilityId::new(NOOP_COMMAND_CAPABILITY_ID)
                    .expect("built-in command capability ID is valid"),
            ],
            ingress_allowed: true,
            outcome_allowed: true,
        };
        let rpg_schema_id =
            SchemaId::new(RPG_COMMAND_SCHEMA_ID).expect("built-in RPG command schema ID is valid");
        let rpg_descriptor = CommandKindDescriptor {
            schema_id: rpg_schema_id.clone(),
            schema_version: 1,
            payload_kind: CommandPayloadKind::Rpg,
            priority_class: RPG_PRIORITY_CLASS,
            required_capabilities: vec![
                CapabilityId::new(RPG_COMMAND_CAPABILITY_ID)
                    .expect("built-in RPG command capability ID is valid"),
            ],
            ingress_allowed: true,
            outcome_allowed: true,
        };
        Self {
            version: COMMAND_KIND_REGISTRY_VERSION,
            descriptors: BTreeMap::from([
                ((noop_schema_id, 1), noop_descriptor),
                ((rpg_schema_id, 1), rpg_descriptor),
            ]),
        }
    }

    #[must_use]
    pub const fn version(&self) -> u32 {
        self.version
    }

    #[must_use]
    pub fn descriptor(
        &self,
        schema_id: &SchemaId,
        schema_version: u32,
    ) -> Option<&CommandKindDescriptor> {
        self.descriptors.get(&(schema_id.clone(), schema_version))
    }
}

impl Default for CommandKindRegistry {
    fn default() -> Self {
        Self::core_v1()
    }
}

#[cfg(test)]
mod tests {
    use next_contracts::{
        COMMAND_SCHEMA_VERSION, CommandPayload, NOOP_COMMAND_SCHEMA_ID, RPG_COMMAND_SCHEMA_ID,
        RpgCommand, SchemaId,
    };

    use super::{
        COMMAND_KIND_REGISTRY_VERSION, CommandKindRegistry, NOOP_PRIORITY_CLASS, RPG_PRIORITY_CLASS,
    };

    #[test]
    fn built_in_registry_pins_schema_payload_and_validator_priority() {
        let registry = CommandKindRegistry::core_v1();
        let schema =
            SchemaId::new(NOOP_COMMAND_SCHEMA_ID).expect("built-in command schema is valid");
        let descriptor = registry
            .descriptor(&schema, COMMAND_SCHEMA_VERSION)
            .expect("built-in command kind is registered");

        assert_eq!(registry.version(), COMMAND_KIND_REGISTRY_VERSION);
        assert_eq!(descriptor.schema_id(), &schema);
        assert_eq!(descriptor.schema_version(), COMMAND_SCHEMA_VERSION);
        assert_eq!(descriptor.priority_class(), NOOP_PRIORITY_CLASS);
        assert!(descriptor.accepts_payload(&CommandPayload::Noop));

        let rpg_schema =
            SchemaId::new(RPG_COMMAND_SCHEMA_ID).expect("built-in RPG command schema is valid");
        let rpg_descriptor = registry
            .descriptor(&rpg_schema, COMMAND_SCHEMA_VERSION)
            .expect("built-in RPG command kind is registered");
        assert_eq!(rpg_descriptor.priority_class(), RPG_PRIORITY_CLASS);
        assert!(
            rpg_descriptor.accepts_payload(&CommandPayload::Rpg(RpgCommand::TransferItem {
                item_id: next_contracts::PersistentId::from_bytes([1; 16]),
                expected_owner: None,
                new_owner: None,
            }))
        );
        assert!(
            !descriptor.accepts_payload(&CommandPayload::Rpg(RpgCommand::TransferItem {
                item_id: next_contracts::PersistentId::from_bytes([1; 16]),
                expected_owner: None,
                new_owner: None,
            }))
        );
    }
}
