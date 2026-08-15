use std::error::Error;
use std::fmt::{Display, Formatter};
use std::path::Path;
use std::time::Instant;

use next_contracts::canonical::sha256;
use next_contracts::ids::{CommandLedgerHash, ContentHash, StateRoot, content_hash_from_bytes};
use next_contracts::snapshot::world_checkpoint_with_world_services_v1_state_root;
use next_contracts::world_population::{
    PopulationCadenceClassV1, WORLD_POPULATION_ACTIVE_COUNT_V1,
    WORLD_POPULATION_BACKGROUND_COUNT_V1, WORLD_POPULATION_COUNT_V1,
    WORLD_POPULATION_NEAR_COUNT_V1, WorldPopulationCatalogV1,
};
use next_runtime::RuntimeState;
use next_world::{
    PopulationNavigationServiceReportV1, WorldPopulationOwnerV1, WorldRoutineOwnerV1,
    WorldStreamerV1,
};

use crate::player_fixture::prepare_fixture_project_package_with_scratch;
use crate::scratch::ScratchContext;

pub const R4_100NPC_WARMUP_TICKS: u64 = 1_000;
pub const R4_100NPC_MEASURED_TICKS: u64 = 10_000;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct PopulationDueCountsV1 {
    pub active: u64,
    pub near: u64,
    pub background: u64,
    pub queries: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PopulationPerformanceReportV1 {
    pub npc_count: u32,
    pub active_count: u32,
    pub near_count: u32,
    pub background_count: u32,
    pub warmup_ticks: u64,
    pub measured_ticks: u64,
    pub due_counts: PopulationDueCountsV1,
    pub maximum_queue_depth: u32,
    pub deferred_work: u64,
    pub dropped_work: u64,
    pub maximum_starvation_age_ticks: u64,
    pub navigation_tick_microseconds: Vec<u64>,
    pub world_services_tick_microseconds: Vec<u64>,
    pub elapsed_microseconds: u128,
    pub command_body_count: u64,
    pub final_population_snapshot_bytes: u64,
    pub due_trace_root: ContentHash,
    pub final_population_state_hash: ContentHash,
    pub final_application_state_root: StateRoot,
    pub final_command_ledger_hash: CommandLedgerHash,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PopulationPerformanceErrorV1 {
    context: &'static str,
    detail: String,
}

impl PopulationPerformanceErrorV1 {
    fn new(context: &'static str, detail: impl Into<String>) -> Self {
        Self {
            context,
            detail: detail.into(),
        }
    }
}

impl Display for PopulationPerformanceErrorV1 {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "{}: {}", self.context, self.detail)
    }
}

impl Error for PopulationPerformanceErrorV1 {}

pub fn run_population_performance_check()
-> Result<PopulationPerformanceReportV1, PopulationPerformanceErrorV1> {
    run_population_performance_check_in(&std::env::temp_dir())
}

pub fn run_population_performance_check_in(
    scratch_root: &Path,
) -> Result<PopulationPerformanceReportV1, PopulationPerformanceErrorV1> {
    run_population_performance_check_for_ticks_in(
        scratch_root,
        R4_100NPC_WARMUP_TICKS,
        R4_100NPC_MEASURED_TICKS,
    )
}

fn run_population_performance_check_for_ticks_in(
    scratch_root: &Path,
    warmup_ticks: u64,
    measured_ticks: u64,
) -> Result<PopulationPerformanceReportV1, PopulationPerformanceErrorV1> {
    if measured_ticks == 0 {
        return Err(PopulationPerformanceErrorV1::new(
            "workload shape",
            "measured tick count must be non-zero",
        ));
    }
    let scratch = ScratchContext::new(scratch_root)
        .map_err(|error| PopulationPerformanceErrorV1::new("scratch root", error.to_string()))?;
    let prepared = prepare_fixture_project_package_with_scratch(
        &scratch,
        "nextengine.performance.r4-100npc.v1",
    )
    .map_err(|error| {
        PopulationPerformanceErrorV1::new("prepare reference package", error.to_string())
    })?;
    let result = run_prepared_population_workload(&prepared.package, warmup_ticks, measured_ticks);
    prepared.finish(result, |error| {
        PopulationPerformanceErrorV1::new("cleanup reference package", error.to_string())
    })
}

fn run_prepared_population_workload(
    package: &next_project::ActivatedProjectPackage,
    warmup_ticks: u64,
    measured_ticks: u64,
) -> Result<PopulationPerformanceReportV1, PopulationPerformanceErrorV1> {
    let fixture = next_reference_game::build_reference_game_session(package.project.clone())
        .map_err(|error| {
            PopulationPerformanceErrorV1::new("activate reference session", error.to_string())
        })?;
    let mut runtime = RuntimeState::new(fixture.bootstrap.clone(), fixture.authority.clone())
        .map_err(|error| {
            PopulationPerformanceErrorV1::new("activate runtime", error.to_string())
        })?;
    let mut world = WorldStreamerV1::activate(
        package.project.clone(),
        package.content_generation.clone(),
        fixture.world_topology().initial_chunk_id().clone(),
    )
    .map_err(|error| {
        PopulationPerformanceErrorV1::new("activate world streaming", error.to_string())
    })?;
    let mut routine = WorldRoutineOwnerV1::activate(
        package.project.world_routine_catalog_or_none,
        runtime.next_tick(),
    )
    .map_err(|error| {
        PopulationPerformanceErrorV1::new("activate world routine", error.to_string())
    })?;
    let mut population = WorldPopulationOwnerV1::activate(
        package.project.world_population_catalog.clone(),
        package.project.world_navigation_catalog.clone(),
        runtime.next_tick(),
    )
    .map_err(|error| {
        PopulationPerformanceErrorV1::new("activate world population", error.to_string())
    })?;

    for _ in 0..warmup_ticks {
        advance_joint_tick(&mut runtime, &mut routine, &mut population, &mut world)?;
    }
    let measured_start_tick = runtime.next_tick();
    let measured_capacity = usize::try_from(measured_ticks).map_err(|error| {
        PopulationPerformanceErrorV1::new("measured capacity", error.to_string())
    })?;
    let mut service_reports = Vec::with_capacity(measured_capacity);
    let mut navigation_tick_microseconds = Vec::with_capacity(measured_capacity);
    let mut world_services_tick_microseconds = Vec::with_capacity(measured_capacity);
    let measured_started = Instant::now();
    for _ in 0..measured_ticks {
        let tick = runtime.next_tick();
        let navigation_started = Instant::now();
        let service_report = population.service_tick(tick).map_err(|error| {
            PopulationPerformanceErrorV1::new("profile population navigation", error.to_string())
        })?;
        navigation_tick_microseconds.push(duration_microseconds(
            navigation_started.elapsed().as_micros(),
            "navigation duration",
        )?);

        let world_services_started = Instant::now();
        advance_joint_tick(&mut runtime, &mut routine, &mut population, &mut world)?;
        world_services_tick_microseconds.push(duration_microseconds(
            world_services_started.elapsed().as_micros(),
            "world services duration",
        )?);
        service_reports.push(service_report);
    }
    let elapsed_microseconds = measured_started.elapsed().as_micros();

    let population_catalog = population.population_catalog_or_none().ok_or_else(|| {
        PopulationPerformanceErrorV1::new("population closure", "catalog is missing")
    })?;
    let expected_due =
        expected_due_counts(population_catalog, measured_start_tick, measured_ticks)?;
    let (due_counts, maximum_queue_depth) = observed_due_counts(&service_reports)?;
    if due_counts != expected_due {
        return Err(PopulationPerformanceErrorV1::new(
            "due-work closure",
            format!("expected {expected_due:?}, observed {due_counts:?}"),
        ));
    }
    runtime
        .validate_world_routine_ledger_closure(&routine)
        .map_err(|error| {
            PopulationPerformanceErrorV1::new("routine ledger closure", error.to_string())
        })?;
    runtime
        .validate_world_population_ledger_closure(&population)
        .map_err(|error| {
            PopulationPerformanceErrorV1::new("population ledger closure", error.to_string())
        })?;

    let checkpoint = runtime.world_checkpoint().map_err(|error| {
        PopulationPerformanceErrorV1::new("final checkpoint", error.to_string())
    })?;
    let routine_snapshot = routine.snapshot_or_none().copied();
    let population_snapshot = population.snapshot_or_none().cloned().ok_or_else(|| {
        PopulationPerformanceErrorV1::new("population closure", "snapshot is missing")
    })?;
    let population_bytes = population_snapshot.canonical_bytes().map_err(|error| {
        PopulationPerformanceErrorV1::new("population snapshot", error.to_string())
    })?;
    let final_application_state_root = world_checkpoint_with_world_services_v1_state_root(
        &checkpoint.runtime_snapshot,
        &checkpoint.rpg_snapshot,
        &checkpoint.physics_checkpoint,
        world.snapshot(),
        routine_snapshot.as_ref(),
        Some(&population_snapshot),
    )
    .map_err(|error| {
        PopulationPerformanceErrorV1::new("application state root", error.to_string())
    })?;
    let final_command_ledger_hash =
        checkpoint
            .runtime_snapshot
            .command_ledger_hash()
            .map_err(|error| {
                PopulationPerformanceErrorV1::new("command ledger hash", error.to_string())
            })?;

    Ok(PopulationPerformanceReportV1 {
        npc_count: count_u32(WORLD_POPULATION_COUNT_V1, "NPC count")?,
        active_count: count_u32(WORLD_POPULATION_ACTIVE_COUNT_V1, "active count")?,
        near_count: count_u32(WORLD_POPULATION_NEAR_COUNT_V1, "near count")?,
        background_count: count_u32(WORLD_POPULATION_BACKGROUND_COUNT_V1, "background count")?,
        warmup_ticks,
        measured_ticks,
        due_counts,
        maximum_queue_depth,
        deferred_work: 0,
        dropped_work: 0,
        maximum_starvation_age_ticks: 0,
        navigation_tick_microseconds,
        world_services_tick_microseconds,
        elapsed_microseconds,
        command_body_count: u64::try_from(runtime.body_archive().entries().len()).map_err(
            |error| PopulationPerformanceErrorV1::new("command body count", error.to_string()),
        )?,
        final_population_snapshot_bytes: u64::try_from(population_bytes.len()).map_err(
            |error| {
                PopulationPerformanceErrorV1::new("population snapshot size", error.to_string())
            },
        )?,
        due_trace_root: population_due_trace_root(&service_reports, measured_start_tick)?,
        final_population_state_hash: content_hash_from_bytes(sha256(&population_bytes)),
        final_application_state_root,
        final_command_ledger_hash,
    })
}

fn advance_joint_tick(
    runtime: &mut RuntimeState,
    routine: &mut WorldRoutineOwnerV1,
    population: &mut WorldPopulationOwnerV1,
    world: &mut WorldStreamerV1,
) -> Result<(), PopulationPerformanceErrorV1> {
    let expected_tick = runtime.next_tick();
    let prepared = runtime
        .tick_preparation()
        .prepare_with_world_services([], routine, population, world)
        .map_err(|error| {
            PopulationPerformanceErrorV1::new("prepare world services tick", error.to_string())
        })?;
    let validated = runtime
        .validate_prepared_world_services_tick_without_application_evidence(
            routine, population, world, prepared,
        )
        .map_err(|error| {
            PopulationPerformanceErrorV1::new("validate world services tick", error.to_string())
        })?;
    let report = runtime
        .commit_validated_world_services_tick_without_application_evidence(
            routine, population, world, validated,
        )
        .map_err(|error| {
            PopulationPerformanceErrorV1::new("commit world services tick", error.to_string())
        })?;
    if report.tick != expected_tick {
        return Err(PopulationPerformanceErrorV1::new(
            "world services tick",
            "committed tick differs from requested tick",
        ));
    }
    Ok(())
}

fn expected_due_counts(
    catalog: &WorldPopulationCatalogV1,
    start_tick: u64,
    tick_count: u64,
) -> Result<PopulationDueCountsV1, PopulationPerformanceErrorV1> {
    let end_tick = start_tick.checked_add(tick_count).ok_or_else(|| {
        PopulationPerformanceErrorV1::new("due-work range", "tick range overflow")
    })?;
    let mut counts = PopulationDueCountsV1::default();
    for tick in start_tick..end_tick {
        for definition in &catalog.records {
            if tick % definition.cadence_class.period_ticks() != definition.cadence_phase {
                continue;
            }
            match definition.cadence_class {
                PopulationCadenceClassV1::Active => increment(&mut counts.active, "active due")?,
                PopulationCadenceClassV1::Near => increment(&mut counts.near, "near due")?,
                PopulationCadenceClassV1::Background => {
                    increment(&mut counts.background, "background due")?;
                }
            }
            increment(&mut counts.queries, "expected queries")?;
        }
    }
    Ok(counts)
}

fn observed_due_counts(
    reports: &[PopulationNavigationServiceReportV1],
) -> Result<(PopulationDueCountsV1, u32), PopulationPerformanceErrorV1> {
    let mut counts = PopulationDueCountsV1::default();
    let mut maximum_queue_depth = 0_u32;
    for report in reports {
        counts.active = counts
            .active
            .checked_add(u64::from(report.active_due))
            .ok_or_else(|| {
                PopulationPerformanceErrorV1::new("observed due work", "active overflow")
            })?;
        counts.near = counts
            .near
            .checked_add(u64::from(report.near_due))
            .ok_or_else(|| {
                PopulationPerformanceErrorV1::new("observed due work", "near overflow")
            })?;
        counts.background = counts
            .background
            .checked_add(u64::from(report.background_due))
            .ok_or_else(|| {
                PopulationPerformanceErrorV1::new("observed due work", "background overflow")
            })?;
        let due = report
            .active_due
            .checked_add(report.near_due)
            .and_then(|value| value.checked_add(report.background_due))
            .ok_or_else(|| {
                PopulationPerformanceErrorV1::new("observed due work", "queue depth overflow")
            })?;
        if report.query_count != due {
            return Err(PopulationPerformanceErrorV1::new(
                "observed due work",
                "a due navigation query was dropped or duplicated",
            ));
        }
        counts.queries = counts
            .queries
            .checked_add(u64::from(report.query_count))
            .ok_or_else(|| {
                PopulationPerformanceErrorV1::new("observed due work", "query overflow")
            })?;
        maximum_queue_depth = maximum_queue_depth.max(due);
    }
    Ok((counts, maximum_queue_depth))
}

fn population_due_trace_root(
    reports: &[PopulationNavigationServiceReportV1],
    expected_start_tick: u64,
) -> Result<ContentHash, PopulationPerformanceErrorV1> {
    let mut ordered = reports.iter().collect::<Vec<_>>();
    ordered.sort_by_key(|report| report.simulation_tick);
    let mut preimage = b"nextengine.performance.r4-100npc.due-trace.v1\0".to_vec();
    for (offset, report) in ordered.into_iter().enumerate() {
        let offset = u64::try_from(offset)
            .map_err(|error| PopulationPerformanceErrorV1::new("due trace", error.to_string()))?;
        let expected_tick = expected_start_tick
            .checked_add(offset)
            .ok_or_else(|| PopulationPerformanceErrorV1::new("due trace", "tick overflow"))?;
        if report.simulation_tick != expected_tick {
            return Err(PopulationPerformanceErrorV1::new(
                "due trace",
                "ticks are duplicated or non-contiguous",
            ));
        }
        preimage.extend_from_slice(&report.simulation_tick.to_le_bytes());
        preimage.extend_from_slice(&report.active_due.to_le_bytes());
        preimage.extend_from_slice(&report.near_due.to_le_bytes());
        preimage.extend_from_slice(&report.background_due.to_le_bytes());
        preimage.extend_from_slice(&report.query_count.to_le_bytes());
        preimage.extend_from_slice(report.route_plan_root.as_bytes());
    }
    Ok(content_hash_from_bytes(sha256(&preimage)))
}

fn duration_microseconds(
    value: u128,
    context: &'static str,
) -> Result<u64, PopulationPerformanceErrorV1> {
    u64::try_from(value)
        .map_err(|error| PopulationPerformanceErrorV1::new(context, error.to_string()))
}

fn count_u32(value: usize, context: &'static str) -> Result<u32, PopulationPerformanceErrorV1> {
    u32::try_from(value)
        .map_err(|error| PopulationPerformanceErrorV1::new(context, error.to_string()))
}

fn increment(value: &mut u64, context: &'static str) -> Result<(), PopulationPerformanceErrorV1> {
    *value = value
        .checked_add(1)
        .ok_or_else(|| PopulationPerformanceErrorV1::new(context, "count overflow"))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn short_production_runs_preserve_exact_due_and_authoritative_roots() {
        let first = run_population_performance_check_for_ticks_in(&std::env::temp_dir(), 8, 30)
            .expect("first short R4 workload");
        let second = run_population_performance_check_for_ticks_in(&std::env::temp_dir(), 8, 30)
            .expect("second short R4 workload");

        assert_eq!(first.npc_count, 100);
        assert_eq!(
            (first.active_count, first.near_count, first.background_count),
            (16, 32, 52)
        );
        assert_eq!(first.due_counts, second.due_counts);
        assert_eq!(first.due_trace_root, second.due_trace_root);
        assert_eq!(
            first.final_population_state_hash,
            second.final_population_state_hash
        );
        assert_eq!(
            first.final_application_state_root,
            second.final_application_state_root
        );
        assert_eq!(
            first.final_command_ledger_hash,
            second.final_command_ledger_hash
        );
        assert_eq!(first.deferred_work, 0);
        assert_eq!(first.dropped_work, 0);
        assert_eq!(first.maximum_starvation_age_ticks, 0);
    }

    #[test]
    fn due_trace_root_is_independent_of_completion_arrival_order() {
        let reports = (20_u64..24)
            .map(|tick| PopulationNavigationServiceReportV1 {
                simulation_tick: tick,
                active_due: u32::try_from(tick % 3).expect("bounded"),
                near_due: u32::try_from(tick % 5).expect("bounded"),
                background_due: 0,
                query_count: u32::try_from(tick % 3 + tick % 5).expect("bounded"),
                route_plan_root: ContentHash::from_bytes(
                    [u8::try_from(tick).expect("bounded"); 32],
                ),
            })
            .collect::<Vec<_>>();
        let expected = population_due_trace_root(&reports, 20).expect("ordered trace");
        let mut reversed = reports;
        reversed.reverse();
        assert_eq!(
            population_due_trace_root(&reversed, 20).expect("permuted trace"),
            expected
        );
    }
}
