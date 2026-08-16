use super::*;

pub(super) fn build_agent_cognition_catalog(
    manifest: &ProjectAuthoringManifestV5,
    population: &WorldPopulationCatalogV1,
) -> Result<AgentCognitionCatalogV1, ProjectAuthoringError> {
    let authored = &manifest.agent_cognition_catalog;
    if authored.schema_version != COGNITION_SCHEMA_VERSION {
        return Err(ProjectAuthoringError::InvalidValue);
    }
    let ordinal = usize::try_from(authored.population_subject_ordinal)
        .map_err(|_| ProjectAuthoringError::InvalidValue)?;
    if ordinal >= WORLD_POPULATION_COUNT_V1 {
        return Err(ProjectAuthoringError::InvalidValue);
    }
    let subject_id =
        population_subject_id(&manifest.world_population_catalog.identity_domain, ordinal)?;
    if population.definition(subject_id).is_none() {
        return Err(ProjectAuthoringError::InvalidValue);
    }
    let mut seed_beliefs = authored
        .seed_beliefs
        .iter()
        .map(|belief| {
            SemanticBeliefV1::new(
                subject_id,
                SchemaId::new(&belief.predicate_id)?,
                SchemaId::new(&belief.value_id)?,
                belief.confidence_q16,
                BeliefSourceV1::AuthoredSeed,
                0,
                0,
                BeliefContradictionV1::Consistent,
            )
            .map_err(|_| ProjectAuthoringError::InvalidValue)
        })
        .collect::<Result<Vec<_>, ProjectAuthoringError>>()?;
    seed_beliefs.sort();
    let value = AgentCognitionCatalogV1 {
        schema_version: authored.schema_version,
        catalog_asset_id: asset_id(&authored.catalog_asset_id)?,
        subject_id,
        evaluation_start_tick: authored.evaluation_start_tick,
        evaluation_period_ticks: authored.evaluation_period_ticks,
        retrieval_limit: authored.retrieval_limit,
        goal_switch_threshold_q16: authored.goal_switch_threshold_q16,
        emergency_health_threshold: authored.emergency_health_threshold,
        planner_max_depth: authored.planner_max_depth,
        planner_max_expanded_nodes: authored.planner_max_expanded_nodes,
        ordinary_goal_id: SchemaId::new(&authored.ordinary_goal_id)?,
        emergency_goal_id: SchemaId::new(&authored.emergency_goal_id)?,
        navigate_action_id: SchemaId::new(&authored.navigate_action_id)?,
        hold_action_id: SchemaId::new(&authored.hold_action_id)?,
        route_known_fact_id: SchemaId::new(&authored.route_known_fact_id)?,
        travel_needed_fact_id: SchemaId::new(&authored.travel_needed_fact_id)?,
        emergency_fact_id: SchemaId::new(&authored.emergency_fact_id)?,
        navigate_ready_fact_id: SchemaId::new(&authored.navigate_ready_fact_id)?,
        hold_ready_fact_id: SchemaId::new(&authored.hold_ready_fact_id)?,
        seed_beliefs,
    };
    value
        .validate()
        .map_err(|_| ProjectAuthoringError::InvalidValue)?;
    Ok(value)
}
