#![forbid(unsafe_code)]

use std::error::Error;
use std::fmt::{Display, Formatter};
use std::sync::atomic::{AtomicU8, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

use mlua::chunk::ChunkMode;
use mlua::{Error as LuaError, Lua, LuaString, Value, VmState};
use next_contracts::canonical::sha256;
use next_contracts::extension::{
    ExtensionContractError, ExtensionPackageStateV1, ExtensionViolationCodeV1,
    LuauPackageManifestV1,
};
use next_contracts::ids::{CapabilityId, ContentHash, SchemaId, content_hash_from_bytes};
use next_contracts::mechanics::RpgDefinitionRegistryV2;
use next_contracts::rpg::{RpgPhysicalContactFactV1, RpgSnapshotV2};
use next_mechanics::{
    AbilityInvocationV1, CompiledAbilityEffectV1, MechanicsHostError, compile_contact_ability_v1,
};

pub const LUAU_ADAPTER_VERSION: &str = "mlua-0.12.0+luau-728";
pub const RPG_QUERY_CHARACTER_RESOURCE_CAPABILITY_ID: &str = "rpg.query.character-resource";
pub const REFERENCE_SCRIPTED_MELEE_SOURCE: &str = r#"
local health = nextengine.query_target_health()
if health > 0 then
    nextengine.propose_action("nextengine.action.melee")
    nextengine.set_state("executed")
end
"#;
const HOST_CALL_COST: u64 = 1;
const NO_VIOLATION: u8 = 0;

pub fn reference_scripted_melee_manifest_v1() -> Result<LuauPackageManifestV1, LuauHostError> {
    let mut capabilities = vec![
        CapabilityId::new(RPG_QUERY_CHARACTER_RESOURCE_CAPABILITY_ID)
            .expect("engine-owned query capability is canonical"),
        CapabilityId::new(next_contracts::mechanics::MECHANICS_EFFECT_PROPOSE_CAPABILITY_ID)
            .expect("engine-owned proposal capability is canonical"),
    ];
    capabilities.sort();
    Ok(LuauPackageManifestV1::new(
        next_contracts::ids::MechanicPackageId::new("org.nextengine.reference.scripted-melee")
            .expect("engine-owned package ID is canonical"),
        1,
        content_hash_from_bytes(sha256(REFERENCE_SCRIPTED_MELEE_SOURCE.as_bytes())),
        capabilities,
        SchemaId::new("org.nextengine.reference.scripted-melee.state")
            .expect("engine-owned state schema is canonical"),
        1,
        false,
        next_contracts::extension::ExtensionBudgetPolicyV1::new(
            100_000, 1_048_576, 1_048_576, 64, 4, 4_096,
        )?,
    )?)
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LuauCallbackInputV1 {
    pub gameplay_tick: u64,
    pub target_health: i32,
    pub granted_capabilities: Vec<CapabilityId>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LuauCallbackOutcomeV1 {
    pub proposed_semantic_actions: Vec<SchemaId>,
    pub instruction_units: u64,
    pub transient_allocation_bytes: u64,
    pub host_call_cost_units: u64,
    pub proposed_command_bytes: u64,
    pub package_state_hash: ContentHash,
}

#[derive(Clone, Debug)]
pub struct LuauPackageRuntimeV1 {
    manifest: LuauPackageManifestV1,
    source: Vec<u8>,
    state: ExtensionPackageStateV1,
}

impl LuauPackageRuntimeV1 {
    pub fn new(manifest: LuauPackageManifestV1, source: Vec<u8>) -> Result<Self, LuauHostError> {
        let state = ExtensionPackageStateV1::new(&manifest, Vec::new())?;
        Self::restore(manifest, source, state)
    }

    pub fn restore(
        manifest: LuauPackageManifestV1,
        source: Vec<u8>,
        state: ExtensionPackageStateV1,
    ) -> Result<Self, LuauHostError> {
        manifest.validate()?;
        state.validate()?;
        if content_hash_from_bytes(sha256(&source)) != manifest.source_hash
            || state.package_id != manifest.package_id
            || state.package_manifest_hash != manifest.manifest_hash
            || state.state_schema_id != manifest.state_schema_id
            || state.state_schema_version != manifest.state_schema_version
        {
            return Err(LuauHostError::ManifestBindingMismatch);
        }
        Ok(Self {
            manifest,
            source,
            state,
        })
    }

    #[must_use]
    pub fn manifest(&self) -> &LuauPackageManifestV1 {
        &self.manifest
    }

    #[must_use]
    pub fn state(&self) -> &ExtensionPackageStateV1 {
        &self.state
    }

    pub fn execute(
        &mut self,
        mut input: LuauCallbackInputV1,
    ) -> Result<LuauCallbackOutcomeV1, LuauHostError> {
        if self.state.circuit_opened_at_tick.is_some() {
            return Err(LuauHostError::CircuitOpen);
        }
        input.granted_capabilities.sort();
        input.granted_capabilities.dedup();
        if let Err(error) = validate_capabilities(&self.manifest, &input.granted_capabilities) {
            self.record_violation(input.gameplay_tick)?;
            return Err(error);
        }
        let budget = &self.manifest.budget;
        let initial_transient = u64::try_from(self.source.len())
            .map_err(|_| {
                LuauHostError::Budget(ExtensionViolationCodeV1::TransientAllocationBudget)
            })?
            .checked_add(u64::try_from(self.state.state_bytes.len()).map_err(|_| {
                LuauHostError::Budget(ExtensionViolationCodeV1::TransientAllocationBudget)
            })?)
            .ok_or(LuauHostError::Budget(
                ExtensionViolationCodeV1::TransientAllocationBudget,
            ))?;
        if initial_transient > budget.transient_allocation_limit_bytes {
            self.record_violation(input.gameplay_tick)?;
            return Err(LuauHostError::Budget(
                ExtensionViolationCodeV1::TransientAllocationBudget,
            ));
        }

        let instruction_units = Arc::new(AtomicU64::new(0));
        let transient_bytes = Arc::new(AtomicU64::new(initial_transient));
        let host_call_units = Arc::new(AtomicU64::new(0));
        let command_count = Arc::new(AtomicU64::new(0));
        let command_bytes = Arc::new(AtomicU64::new(0));
        let violation = Arc::new(AtomicU8::new(NO_VIOLATION));
        let proposals = Arc::new(Mutex::new(Vec::<SchemaId>::new()));
        let next_state = Arc::new(Mutex::new(None::<Vec<u8>>));

        let result = execute_luau(
            &self.source,
            &self.state.state_bytes,
            &input,
            ExecutionCounters {
                instruction_units: Arc::clone(&instruction_units),
                transient_bytes: Arc::clone(&transient_bytes),
                host_call_units: Arc::clone(&host_call_units),
                command_count: Arc::clone(&command_count),
                command_bytes: Arc::clone(&command_bytes),
                violation: Arc::clone(&violation),
                proposals: Arc::clone(&proposals),
                next_state: Arc::clone(&next_state),
            },
            budget,
        );
        if let Err(error) = result {
            let code = violation_from_marker(violation.load(Ordering::SeqCst)).unwrap_or({
                if matches!(error, LuaError::MemoryError(_)) {
                    ExtensionViolationCodeV1::LiveMemoryBudget
                } else {
                    ExtensionViolationCodeV1::Trap
                }
            });
            self.record_violation(input.gameplay_tick)?;
            return Err(LuauHostError::Vm {
                code,
                detail: stable_lua_error(&error),
            });
        }

        let proposed_semantic_actions = proposals
            .lock()
            .map_err(|_| LuauHostError::HostInvariant)?
            .clone();
        let state_update = next_state
            .lock()
            .map_err(|_| LuauHostError::HostInvariant)?
            .clone();
        if let Some(state_bytes) = state_update {
            self.state.replace_state_bytes(state_bytes)?;
        }
        Ok(LuauCallbackOutcomeV1 {
            proposed_semantic_actions,
            instruction_units: instruction_units.load(Ordering::SeqCst),
            transient_allocation_bytes: transient_bytes.load(Ordering::SeqCst),
            host_call_cost_units: host_call_units.load(Ordering::SeqCst),
            proposed_command_bytes: command_bytes.load(Ordering::SeqCst),
            package_state_hash: self.state.state_hash,
        })
    }

    fn record_violation(&mut self, gameplay_tick: u64) -> Result<(), LuauHostError> {
        self.state.record_violation(gameplay_tick)?;
        Ok(())
    }
}

pub fn compile_first_scripted_action_v1(
    outcome: &LuauCallbackOutcomeV1,
    registry: &RpgDefinitionRegistryV2,
    snapshot: &RpgSnapshotV2,
    gameplay_tick: u64,
    source_character_id: next_contracts::ids::PersistentId,
    physical_contact_facts: Vec<RpgPhysicalContactFactV1>,
) -> Result<CompiledAbilityEffectV1, LuauHostError> {
    let semantic_action_id = outcome
        .proposed_semantic_actions
        .first()
        .cloned()
        .ok_or(LuauHostError::ProposalMissing)?;
    if outcome.proposed_semantic_actions.len() != 1 {
        return Err(LuauHostError::ProposalCountInvalid);
    }
    Ok(compile_contact_ability_v1(
        registry,
        snapshot,
        AbilityInvocationV1 {
            gameplay_tick,
            source_character_id,
            semantic_action_id,
            prior_cooldown_commit_tick: None,
            physical_contact_facts,
        },
    )?)
}

struct ExecutionCounters {
    instruction_units: Arc<AtomicU64>,
    transient_bytes: Arc<AtomicU64>,
    host_call_units: Arc<AtomicU64>,
    command_count: Arc<AtomicU64>,
    command_bytes: Arc<AtomicU64>,
    violation: Arc<AtomicU8>,
    proposals: Arc<Mutex<Vec<SchemaId>>>,
    next_state: Arc<Mutex<Option<Vec<u8>>>>,
}

fn execute_luau(
    source: &[u8],
    prior_state: &[u8],
    input: &LuauCallbackInputV1,
    counters: ExecutionCounters,
    budget: &next_contracts::extension::ExtensionBudgetPolicyV1,
) -> Result<(), LuaError> {
    let lua = Lua::new();
    let baseline = lua.used_memory();
    let live_limit = baseline
        .checked_add(usize::try_from(budget.live_memory_limit_bytes).unwrap_or(usize::MAX))
        .ok_or_else(|| LuaError::runtime("NEXTENGINE_LIVE_MEMORY_BUDGET"))?;
    lua.set_memory_limit(live_limit)?;

    let instruction_counter = Arc::clone(&counters.instruction_units);
    let instruction_violation = Arc::clone(&counters.violation);
    let instruction_limit = budget.instruction_limit;
    lua.set_interrupt(move |_| {
        charge(
            &instruction_counter,
            1,
            instruction_limit,
            &instruction_violation,
            ExtensionViolationCodeV1::InstructionBudget,
        )?;
        Ok(VmState::Continue)
    });

    let globals = lua.globals();
    for forbidden in [
        "os", "io", "package", "debug", "require", "loadfile", "dofile",
    ] {
        globals.raw_set(forbidden, Value::Nil)?;
    }
    let engine = lua.create_table()?;

    let health_host_calls = Arc::clone(&counters.host_call_units);
    let health_violation = Arc::clone(&counters.violation);
    let host_limit = budget.host_call_cost_limit;
    let target_health = input.target_health;
    engine.set(
        "query_target_health",
        lua.create_function(move |_, ()| {
            charge(
                &health_host_calls,
                HOST_CALL_COST,
                host_limit,
                &health_violation,
                ExtensionViolationCodeV1::HostCallBudget,
            )?;
            Ok(target_health)
        })?,
    )?;

    let state_host_calls = Arc::clone(&counters.host_call_units);
    let state_violation = Arc::clone(&counters.violation);
    let prior_state = prior_state.to_vec();
    engine.set(
        "get_state",
        lua.create_function(move |lua, ()| {
            charge(
                &state_host_calls,
                HOST_CALL_COST,
                host_limit,
                &state_violation,
                ExtensionViolationCodeV1::HostCallBudget,
            )?;
            lua.create_string(&prior_state)
        })?,
    )?;

    let set_state_host_calls = Arc::clone(&counters.host_call_units);
    let set_state_transient = Arc::clone(&counters.transient_bytes);
    let set_state_violation = Arc::clone(&counters.violation);
    let set_state_output = Arc::clone(&counters.next_state);
    let transient_limit = budget.transient_allocation_limit_bytes;
    engine.set(
        "set_state",
        lua.create_function(move |_, value: LuaString| {
            charge(
                &set_state_host_calls,
                HOST_CALL_COST,
                host_limit,
                &set_state_violation,
                ExtensionViolationCodeV1::HostCallBudget,
            )?;
            let bytes = value.as_bytes().to_vec();
            charge(
                &set_state_transient,
                u64::try_from(bytes.len()).unwrap_or(u64::MAX),
                transient_limit,
                &set_state_violation,
                ExtensionViolationCodeV1::TransientAllocationBudget,
            )?;
            *set_state_output
                .lock()
                .map_err(|_| LuaError::runtime("NEXTENGINE_HOST_INVARIANT"))? = Some(bytes);
            Ok(())
        })?,
    )?;

    let proposal_host_calls = Arc::clone(&counters.host_call_units);
    let proposal_count = Arc::clone(&counters.command_count);
    let proposal_bytes = Arc::clone(&counters.command_bytes);
    let proposal_transient = Arc::clone(&counters.transient_bytes);
    let proposal_violation = Arc::clone(&counters.violation);
    let proposals = Arc::clone(&counters.proposals);
    let command_count_limit = u64::from(budget.proposed_command_count_limit);
    let command_bytes_limit = budget.proposed_command_bytes_limit;
    engine.set(
        "propose_action",
        lua.create_function(move |_, action: LuaString| {
            charge(
                &proposal_host_calls,
                HOST_CALL_COST,
                host_limit,
                &proposal_violation,
                ExtensionViolationCodeV1::HostCallBudget,
            )?;
            charge(
                &proposal_count,
                1,
                command_count_limit,
                &proposal_violation,
                ExtensionViolationCodeV1::CommandBudget,
            )?;
            let action = action
                .to_str()
                .map_err(|_| LuaError::runtime("NEXTENGINE_SCHEMA_VIOLATION"))?;
            let action_id = SchemaId::new(action.as_ref()).map_err(|_| {
                proposal_violation.store(
                    ExtensionViolationCodeV1::SchemaViolation as u8,
                    Ordering::SeqCst,
                );
                LuaError::runtime("NEXTENGINE_SCHEMA_VIOLATION")
            })?;
            let bytes = u64::try_from(action_id.as_str().len()).unwrap_or(u64::MAX);
            charge(
                &proposal_bytes,
                bytes,
                command_bytes_limit,
                &proposal_violation,
                ExtensionViolationCodeV1::CommandBudget,
            )?;
            charge(
                &proposal_transient,
                bytes,
                transient_limit,
                &proposal_violation,
                ExtensionViolationCodeV1::TransientAllocationBudget,
            )?;
            proposals
                .lock()
                .map_err(|_| LuaError::runtime("NEXTENGINE_HOST_INVARIANT"))?
                .push(action_id);
            Ok(())
        })?,
    )?;
    engine.set_readonly(true);
    globals.raw_set("nextengine", engine)?;
    lua.sandbox(true)?;
    lua.load(source)
        .set_mode(ChunkMode::Text)
        .set_name("nextengine-package")
        .exec()
}

fn charge(
    counter: &AtomicU64,
    amount: u64,
    limit: u64,
    violation: &AtomicU8,
    code: ExtensionViolationCodeV1,
) -> Result<(), LuaError> {
    let mut current = counter.load(Ordering::SeqCst);
    loop {
        let next = current.saturating_add(amount);
        if next > limit {
            violation.store(code as u8, Ordering::SeqCst);
            return Err(LuaError::runtime(format!(
                "NEXTENGINE_BUDGET_{}",
                code as u8
            )));
        }
        match counter.compare_exchange(current, next, Ordering::SeqCst, Ordering::SeqCst) {
            Ok(_) => return Ok(()),
            Err(actual) => current = actual,
        }
    }
}

fn validate_capabilities(
    manifest: &LuauPackageManifestV1,
    granted: &[CapabilityId],
) -> Result<(), LuauHostError> {
    for required in [
        RPG_QUERY_CHARACTER_RESOURCE_CAPABILITY_ID,
        next_contracts::mechanics::MECHANICS_EFFECT_PROPOSE_CAPABILITY_ID,
    ] {
        let capability =
            CapabilityId::new(required).expect("engine-owned script capability is canonical");
        if manifest
            .requested_capabilities
            .binary_search(&capability)
            .is_err()
            || granted.binary_search(&capability).is_err()
        {
            return Err(LuauHostError::Budget(
                ExtensionViolationCodeV1::CapabilityDenied,
            ));
        }
    }
    Ok(())
}

fn violation_from_marker(marker: u8) -> Option<ExtensionViolationCodeV1> {
    match marker {
        1 => Some(ExtensionViolationCodeV1::InstructionBudget),
        2 => Some(ExtensionViolationCodeV1::TransientAllocationBudget),
        3 => Some(ExtensionViolationCodeV1::LiveMemoryBudget),
        4 => Some(ExtensionViolationCodeV1::HostCallBudget),
        5 => Some(ExtensionViolationCodeV1::CommandBudget),
        6 => Some(ExtensionViolationCodeV1::CapabilityDenied),
        7 => Some(ExtensionViolationCodeV1::Trap),
        8 => Some(ExtensionViolationCodeV1::SchemaViolation),
        _ => None,
    }
}

fn stable_lua_error(error: &LuaError) -> &'static str {
    match error {
        LuaError::SyntaxError { .. } => "EXTENSION_SYNTAX_ERROR",
        LuaError::MemoryError(_) => "EXTENSION_LIVE_MEMORY_LIMIT",
        LuaError::SafetyError(_) => "EXTENSION_SANDBOX_DENIED",
        _ => "EXTENSION_CALLBACK_FAILED",
    }
}

#[derive(Debug)]
#[non_exhaustive]
pub enum LuauHostError {
    Contract(ExtensionContractError),
    Mechanics(MechanicsHostError),
    ManifestBindingMismatch,
    CircuitOpen,
    Budget(ExtensionViolationCodeV1),
    Vm {
        code: ExtensionViolationCodeV1,
        detail: &'static str,
    },
    ProposalMissing,
    ProposalCountInvalid,
    HostInvariant,
}

impl Display for LuauHostError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Contract(error) => write!(formatter, "extension contract: {error}"),
            Self::Mechanics(error) => write!(formatter, "mechanics host: {error}"),
            Self::ManifestBindingMismatch => {
                formatter.write_str("Luau package manifest binding mismatch")
            }
            Self::CircuitOpen => formatter.write_str("Luau package circuit is open"),
            Self::Budget(code) => write!(formatter, "Luau budget violation: {code:?}"),
            Self::Vm { code, detail } => write!(formatter, "Luau VM {code:?}: {detail}"),
            Self::ProposalMissing => formatter.write_str("Luau callback proposed no action"),
            Self::ProposalCountInvalid => {
                formatter.write_str("Luau callback proposed multiple actions")
            }
            Self::HostInvariant => formatter.write_str("Luau host invariant failed"),
        }
    }
}

impl Error for LuauHostError {}

impl From<ExtensionContractError> for LuauHostError {
    fn from(value: ExtensionContractError) -> Self {
        Self::Contract(value)
    }
}

impl From<MechanicsHostError> for LuauHostError {
    fn from(value: MechanicsHostError) -> Self {
        Self::Mechanics(value)
    }
}

#[cfg(test)]
mod tests {
    use std::sync::atomic::{AtomicU8, AtomicU64};

    use next_contracts::canonical::{CanonicalDecodeLimits, sha256};
    use next_contracts::extension::{
        ExtensionBudgetPolicyV1, ExtensionPackageStateV1, ExtensionViolationCodeV1,
        LuauPackageManifestV1,
    };
    use next_contracts::ids::{CapabilityId, MechanicPackageId, SchemaId, content_hash_from_bytes};

    use super::{
        LuauCallbackInputV1, LuauHostError, LuauPackageRuntimeV1,
        RPG_QUERY_CHARACTER_RESOURCE_CAPABILITY_ID,
    };

    const SUCCESS: &str = r#"
        assert(os == nil and io == nil and package == nil and debug == nil)
        assert(require == nil and loadfile == nil and dofile == nil)
        local health = nextengine.query_target_health()
        if health > 0 then
            nextengine.propose_action("nextengine.action.melee")
            nextengine.set_state("executed")
        end
    "#;

    #[test]
    fn sandboxed_callback_proposes_typed_action_and_round_trips_state() {
        let mut runtime = runtime(SUCCESS, budget(100, 10_000, 1_000_000, 8, 1, 100));
        let outcome = runtime.execute(input(7)).expect("callback succeeds");
        assert_eq!(
            outcome.proposed_semantic_actions,
            [SchemaId::new("nextengine.action.melee").expect("action")]
        );
        assert_eq!(runtime.state().state_bytes, b"executed");
        let bytes = runtime.state().canonical_bytes().expect("state bytes");
        let restored_state =
            ExtensionPackageStateV1::from_canonical_bytes(&bytes, CanonicalDecodeLimits::default())
                .expect("state restore");
        let mut restored = LuauPackageRuntimeV1::restore(
            runtime.manifest().clone(),
            SUCCESS.as_bytes().to_vec(),
            restored_state,
        )
        .expect("runtime restore");
        assert_eq!(
            restored
                .execute(input(8))
                .expect("retry after load")
                .proposed_semantic_actions,
            outcome.proposed_semantic_actions
        );
    }

    #[test]
    fn deterministic_limits_accept_exact_boundary_and_reject_plus_one() {
        let counter = AtomicU64::new(0);
        let violation = AtomicU8::new(0);
        super::charge(
            &counter,
            9,
            10,
            &violation,
            ExtensionViolationCodeV1::HostCallBudget,
        )
        .expect("limit-1");
        super::charge(
            &counter,
            1,
            10,
            &violation,
            ExtensionViolationCodeV1::HostCallBudget,
        )
        .expect("limit");
        assert!(
            super::charge(
                &counter,
                1,
                10,
                &violation,
                ExtensionViolationCodeV1::HostCallBudget,
            )
            .is_err()
        );

        let source = "nextengine.query_target_health(); nextengine.query_target_health()";
        let mut host_limited = runtime(source, budget(100, 10_000, 1_000_000, 1, 1, 100));
        assert!(matches!(
            host_limited.execute(input(1)),
            Err(LuauHostError::Vm {
                code: ExtensionViolationCodeV1::HostCallBudget,
                ..
            })
        ));
        let allocation_limit = u64::try_from(SUCCESS.len()).expect("bounded source");
        let mut allocation_limited =
            runtime(SUCCESS, budget(100, allocation_limit, 1_000_000, 8, 1, 100));
        assert!(matches!(
            allocation_limited.execute(input(1)),
            Err(LuauHostError::Vm {
                code: ExtensionViolationCodeV1::TransientAllocationBudget,
                ..
            })
        ));
    }

    #[test]
    fn instruction_loop_trap_capability_denial_and_circuit_fail_closed() {
        let mut looped = runtime("while true do end", budget(1, 10_000, 1_000_000, 8, 1, 100));
        assert!(matches!(
            looped.execute(input(1)),
            Err(LuauHostError::Vm {
                code: ExtensionViolationCodeV1::InstructionBudget,
                ..
            })
        ));
        let mut denied = runtime(SUCCESS, budget(100, 10_000, 1_000_000, 8, 1, 100));
        let denied_input = LuauCallbackInputV1 {
            granted_capabilities: vec![],
            ..input(1)
        };
        assert!(matches!(
            denied.execute(denied_input),
            Err(LuauHostError::Budget(
                ExtensionViolationCodeV1::CapabilityDenied
            ))
        ));

        let mut trapped = runtime("error('trap')", budget(100, 10_000, 1_000_000, 8, 1, 100));
        for tick in [10, 11, 12] {
            assert!(trapped.execute(input(tick)).is_err());
        }
        assert_eq!(trapped.state().circuit_opened_at_tick, Some(12));
        assert!(matches!(
            trapped.execute(input(13)),
            Err(LuauHostError::CircuitOpen)
        ));
    }

    fn input(gameplay_tick: u64) -> LuauCallbackInputV1 {
        LuauCallbackInputV1 {
            gameplay_tick,
            target_health: 100,
            granted_capabilities: capabilities(),
        }
    }

    fn capabilities() -> Vec<CapabilityId> {
        let mut values = vec![
            CapabilityId::new(RPG_QUERY_CHARACTER_RESOURCE_CAPABILITY_ID).expect("query"),
            CapabilityId::new(next_contracts::mechanics::MECHANICS_EFFECT_PROPOSE_CAPABILITY_ID)
                .expect("proposal"),
        ];
        values.sort();
        values
    }

    fn budget(
        instructions: u64,
        transient: u64,
        live: u64,
        host_calls: u64,
        commands: u32,
        command_bytes: u64,
    ) -> ExtensionBudgetPolicyV1 {
        ExtensionBudgetPolicyV1::new(
            instructions,
            transient,
            live,
            host_calls,
            commands,
            command_bytes,
        )
        .expect("budget")
    }

    fn runtime(source: &str, budget: ExtensionBudgetPolicyV1) -> LuauPackageRuntimeV1 {
        let manifest = LuauPackageManifestV1::new(
            MechanicPackageId::new("org.nextengine.test.luau").expect("package"),
            1,
            content_hash_from_bytes(sha256(source.as_bytes())),
            capabilities(),
            SchemaId::new("org.nextengine.test.luau.state").expect("state"),
            1,
            false,
            budget,
        )
        .expect("manifest");
        LuauPackageRuntimeV1::new(manifest, source.as_bytes().to_vec()).expect("runtime")
    }
}
