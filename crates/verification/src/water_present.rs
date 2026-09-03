//! `CONTINUUM-WATER-PRESENT-P1` (ADR-100, plan `continuum-water/09`): the
//! water presentation stage is a pure, bounded function of the committed
//! physics checkpoint and a frame index; computing it never moves a
//! gameplay root, every update stays inside the declared capacities and
//! bounds, and the stage fits the frozen cost budget.

use std::error::Error;
use std::fmt::{Display, Formatter};
use std::time::Instant;

use next_contracts::canonical::sha256;
use next_contracts::command::WorldCommand;
use next_contracts::ids::{ContentHash, StateRoot, content_hash_from_bytes};
use next_contracts::physics::PhysicsWorldCheckpointV1;
use next_contracts::render_content::AabbI64V1;
use next_reference_game::{
    WATER_JET_MAX_PARTICLES, WATER_RIPPLE_CAP_MICROMETRES, WATER_SURFACE_INDEX_CAPACITY,
    WATER_SURFACE_VERTEX_CAPACITY, WaterPresentationFrameV1, WaterSurfaceBindingV1,
    compute_water_presentation_frame, cooked_project_rpg_snapshot,
    reference_water_surface_bindings,
};
use next_runtime::RuntimeState;

use crate::player_fixture::prepare_fixture_project_package_with_scratch;
use crate::scratch::ScratchContext;

/// Plan `continuum-water/09`: `600` table ticks (`20 s` at the reference
/// tick rate) cover the first drain of vessel A through the open gate.
const RUN_TICKS: u64 = 600;
/// The presentation runs at `60` frames per second over a `30 Hz` table:
/// two frame indices per tick.
const FRAMES_PER_TICK: u64 = 2;
/// The jet particle bounds: the union of every water volume extent plus
/// this margin (the same rule as the desktop feed in `apps/game`).
const JET_BOUNDS_MARGIN_MICROMETRES: i64 = 1_000_000;
/// G4: stage CPU time per frame in a release build.
const COST_LIMIT_MICROSECONDS: u128 = 1_000;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WaterPresentCheckReportV1 {
    pub run_ticks: u64,
    pub frames: u64,
    pub surfaces_per_frame: usize,
    pub vertex_capacity: u32,
    pub index_capacity: u32,
    pub particle_capacity: u32,
    pub roots_identical: bool,
    pub capacity_violations: u64,
    pub bounds_violations: u64,
    pub purity_identical: bool,
    pub max_jet_particles: u32,
    pub frames_with_jet: u64,
    pub max_ripple_micrometres: i64,
    pub stage_cost_max_microseconds: u128,
    pub stage_cost_mean_microseconds: u128,
    pub stage_cost_debug_build: bool,
    pub repeated_run_identical: bool,
    pub final_state_root: StateRoot,
    pub final_physics_checkpoint_hash: ContentHash,
    pub frames_digest: ContentHash,
    pub matrix_digest: ContentHash,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WaterPresentCheckErrorV1 {
    context: String,
    detail: String,
}

impl WaterPresentCheckErrorV1 {
    fn new(context: impl Into<String>, detail: impl Into<String>) -> Self {
        Self {
            context: context.into(),
            detail: detail.into(),
        }
    }

    fn condition(context: impl Into<String>) -> Self {
        Self::new(context, "condition not met")
    }
}

impl Display for WaterPresentCheckErrorV1 {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "{}: {}", self.context, self.detail)
    }
}

impl Error for WaterPresentCheckErrorV1 {}

/// Everything one generation produces that must repeat exactly; the cost
/// samples are excluded because wall time never repeats.
#[derive(Clone, Debug, Eq, PartialEq)]
struct GenerationEvidenceV1 {
    frames: u64,
    surfaces_per_frame: usize,
    roots_identical: bool,
    capacity_violations: u64,
    bounds_violations: u64,
    purity_identical: bool,
    max_jet_particles: u32,
    frames_with_jet: u64,
    max_ripple_micrometres: i64,
    final_state_root: StateRoot,
    final_physics_checkpoint_hash: ContentHash,
    frames_digest: ContentHash,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct GenerationOutcomeV1 {
    evidence: GenerationEvidenceV1,
    stage_cost_max_microseconds: u128,
    stage_cost_mean_microseconds: u128,
}

pub fn run_water_present_check() -> Result<WaterPresentCheckReportV1, WaterPresentCheckErrorV1> {
    let scratch = ScratchContext::new(&std::env::temp_dir()).map_err(|error| {
        WaterPresentCheckErrorV1::new("create scratch context", error.to_string())
    })?;
    let prepared =
        prepare_fixture_project_package_with_scratch(&scratch, "nextengine.water-present-check")
            .map_err(|error| {
                WaterPresentCheckErrorV1::new("activate reference package", error.to_string())
            })?;
    let result = (|| {
        let first = run_generation(&prepared.package)?;
        let repeated = run_generation(&prepared.package)?;
        if repeated.evidence != first.evidence {
            return Err(WaterPresentCheckErrorV1::condition(
                "repeated water presentation generation is identical",
            ));
        }
        // G4 binds the release build only; a debug run reports the cost.
        if !cfg!(debug_assertions) && first.stage_cost_max_microseconds > COST_LIMIT_MICROSECONDS {
            return Err(WaterPresentCheckErrorV1::new(
                "stage cost per frame within the budget",
                format!(
                    "{} us > {} us",
                    first.stage_cost_max_microseconds, COST_LIMIT_MICROSECONDS
                ),
            ));
        }
        let matrix_digest = evidence_digest(&first.evidence);
        let evidence = first.evidence;
        Ok(WaterPresentCheckReportV1 {
            run_ticks: RUN_TICKS,
            frames: evidence.frames,
            surfaces_per_frame: evidence.surfaces_per_frame,
            vertex_capacity: WATER_SURFACE_VERTEX_CAPACITY,
            index_capacity: WATER_SURFACE_INDEX_CAPACITY,
            particle_capacity: WATER_JET_MAX_PARTICLES,
            roots_identical: evidence.roots_identical,
            capacity_violations: evidence.capacity_violations,
            bounds_violations: evidence.bounds_violations,
            purity_identical: evidence.purity_identical,
            max_jet_particles: evidence.max_jet_particles,
            frames_with_jet: evidence.frames_with_jet,
            max_ripple_micrometres: evidence.max_ripple_micrometres,
            stage_cost_max_microseconds: first.stage_cost_max_microseconds,
            stage_cost_mean_microseconds: first.stage_cost_mean_microseconds,
            stage_cost_debug_build: cfg!(debug_assertions),
            repeated_run_identical: true,
            final_state_root: evidence.final_state_root,
            final_physics_checkpoint_hash: evidence.final_physics_checkpoint_hash,
            frames_digest: evidence.frames_digest,
            matrix_digest,
        })
    })();
    prepared.finish(result, |error| {
        WaterPresentCheckErrorV1::new("remove reference package", error.to_string())
    })
}

fn run_generation(
    package: &next_project::ActivatedProjectPackage,
) -> Result<GenerationOutcomeV1, WaterPresentCheckErrorV1> {
    let fixture = next_reference_game::build_reference_game_session(package.project.clone())
        .map_err(|error| {
            WaterPresentCheckErrorV1::new("build reference session", error.to_string())
        })?;
    let bindings = reference_water_surface_bindings();
    let surface_bounds = surface_bounds(&fixture, &bindings)?;
    let jet_bounds = jet_bounds(&fixture.bootstrap.physics_checkpoint)?;

    // Two runtimes from the same bootstrap: the stage is computed for the
    // first one only; the second never sees presentation state.
    let mut staged = activate_runtime(&fixture)?;
    let mut plain = activate_runtime(&fixture)?;

    let mut roots_identical = true;
    let mut capacity_violations = 0_u64;
    let mut bounds_violations = 0_u64;
    let mut purity_identical = true;
    let mut max_jet_particles = 0_u32;
    let mut frames_with_jet = 0_u64;
    let mut max_ripple_micrometres = 0_i64;
    let mut frames = 0_u64;
    let mut cost_max: u128 = 0;
    let mut cost_total: u128 = 0;
    let mut frames_preimage = b"nextengine.water-present-check.frames.v1\0".to_vec();
    let mut final_physics_checkpoint_hash = ContentHash::default();

    for tick in 0..RUN_TICKS {
        let staged_report = staged
            .run_tick(Vec::<WorldCommand>::new())
            .map_err(|error| WaterPresentCheckErrorV1::new("run staged tick", error.to_string()))?;
        let plain_report = plain
            .run_tick(Vec::<WorldCommand>::new())
            .map_err(|error| WaterPresentCheckErrorV1::new("run plain tick", error.to_string()))?;
        // G1: the stage never moves a gameplay root.
        let staged_root = state_root(&staged)?;
        let plain_root = state_root(&plain)?;
        if staged_report.physics_checkpoint_hash != plain_report.physics_checkpoint_hash
            || staged_report.events != plain_report.events
            || staged_root != plain_root
        {
            roots_identical = false;
        }

        final_physics_checkpoint_hash = staged_report.physics_checkpoint_hash;
        let checkpoint = staged.physics_checkpoint();
        let published_tick = staged_report.tick;
        for frame_offset in 0..FRAMES_PER_TICK {
            let frame_index = tick * FRAMES_PER_TICK + frame_offset;
            let started = Instant::now();
            let frame = compute_water_presentation_frame(
                &checkpoint.water_volumes,
                &checkpoint.water_flow,
                &bindings,
                published_tick,
                frame_index,
            );
            let elapsed = started.elapsed().as_micros();
            cost_max = cost_max.max(elapsed);
            cost_total += elapsed;
            frames += 1;

            // G3: the same checkpoint and frame index yield the same frame.
            let again = compute_water_presentation_frame(
                &checkpoint.water_volumes,
                &checkpoint.water_flow,
                &bindings,
                published_tick,
                frame_index,
            );
            if again != frame {
                purity_identical = false;
            }

            // G2: capacities and bounds.
            if frame.surfaces.len() != bindings.len() {
                capacity_violations += 1;
            }
            for surface in &frame.surfaces {
                let vertex_count = surface.positions_micrometres.len();
                if vertex_count > WATER_SURFACE_VERTEX_CAPACITY as usize
                    || surface.normals_snorm16.len() != vertex_count
                    || surface.indices.len() > WATER_SURFACE_INDEX_CAPACITY as usize
                    || surface.indices.len() % 3 != 0
                    || surface
                        .indices
                        .iter()
                        .any(|index| *index as usize >= vertex_count)
                {
                    capacity_violations += 1;
                }
                let Some(bounds) = surface_bounds
                    .iter()
                    .find(|(volume_id, _)| *volume_id == surface.volume_id)
                    .map(|(_, bounds)| *bounds)
                else {
                    bounds_violations += 1;
                    continue;
                };
                for position in &surface.positions_micrometres {
                    if !bounds.contains(*position) {
                        bounds_violations += 1;
                    }
                    let ripple = position[1].abs();
                    if ripple > WATER_RIPPLE_CAP_MICROMETRES {
                        bounds_violations += 1;
                    }
                    max_ripple_micrometres = max_ripple_micrometres.max(ripple);
                }
            }
            let jet_count = u32::try_from(frame.jet.positions_micrometres.len())
                .map_err(|_| WaterPresentCheckErrorV1::condition("jet count fits u32"))?;
            if jet_count > WATER_JET_MAX_PARTICLES
                || frame.jet.velocities_micrometres_per_second.len()
                    != frame.jet.positions_micrometres.len()
            {
                capacity_violations += 1;
            }
            for position in &frame.jet.positions_micrometres {
                if !jet_bounds.contains(*position) {
                    bounds_violations += 1;
                }
            }
            if jet_count > 0 {
                frames_with_jet += 1;
            }
            max_jet_particles = max_jet_particles.max(jet_count);
            absorb_frame(&mut frames_preimage, &frame);
        }
    }

    let staged_final = staged.world_checkpoint().map_err(|error| {
        WaterPresentCheckErrorV1::new("build staged final checkpoint", error.to_string())
    })?;
    let plain_final = plain.world_checkpoint().map_err(|error| {
        WaterPresentCheckErrorV1::new("build plain final checkpoint", error.to_string())
    })?;
    roots_identical = roots_identical && staged_final.state_root == plain_final.state_root;
    require(
        roots_identical,
        "roots are identical every tick with and without the stage",
    )?;
    require(capacity_violations == 0, "every update fits its capacity")?;
    require(
        bounds_violations == 0,
        "every update stays inside its bounds",
    )?;
    require(
        purity_identical,
        "the stage is a pure function of its inputs",
    )?;
    require(
        frames_with_jet > 0 && max_jet_particles > 0,
        "the open gate produces a jet",
    )?;
    require(
        max_ripple_micrometres > 0,
        "the incoming flux produces a ripple",
    )?;
    Ok(GenerationOutcomeV1 {
        evidence: GenerationEvidenceV1 {
            frames,
            surfaces_per_frame: bindings.len(),
            roots_identical,
            capacity_violations,
            bounds_violations,
            purity_identical,
            max_jet_particles,
            frames_with_jet,
            max_ripple_micrometres,
            final_state_root: staged_final.state_root,
            final_physics_checkpoint_hash,
            frames_digest: content_hash_from_bytes(sha256(&frames_preimage)),
        },
        stage_cost_max_microseconds: cost_max,
        stage_cost_mean_microseconds: cost_total / u128::from(frames.max(1)),
    })
}

fn activate_runtime(
    fixture: &next_reference_game::ReferenceGameSession,
) -> Result<RuntimeState, WaterPresentCheckErrorV1> {
    RuntimeState::with_rpg_snapshot(
        fixture.bootstrap.clone(),
        fixture.authority.clone(),
        cooked_project_rpg_snapshot(fixture),
    )
    .map_err(|error| WaterPresentCheckErrorV1::new("activate runtime", error.to_string()))
}

fn state_root(runtime: &RuntimeState) -> Result<StateRoot, WaterPresentCheckErrorV1> {
    runtime
        .world_checkpoint()
        .map(|checkpoint| checkpoint.state_root)
        .map_err(|error| WaterPresentCheckErrorV1::new("build world checkpoint", error.to_string()))
}

/// The authored bounds of every surface quad mesh from the cooked render
/// catalog: the adapter rejects a dynamic update outside them.
fn surface_bounds(
    fixture: &next_reference_game::ReferenceGameSession,
    bindings: &[WaterSurfaceBindingV1],
) -> Result<Vec<(next_contracts::ids::PersistentId, AabbI64V1)>, WaterPresentCheckErrorV1> {
    let catalog = &fixture.activated_project.render_content_catalog;
    bindings
        .iter()
        .map(|binding| {
            catalog
                .meshes()
                .iter()
                .find(|mesh| mesh.asset_id() == binding.mesh_asset_id)
                .map(|mesh| (binding.volume_id, mesh.bounds()))
                .ok_or_else(|| {
                    WaterPresentCheckErrorV1::condition(
                        "every surface quad mesh exists in the render catalog",
                    )
                })
        })
        .collect()
}

/// The jet bounds: the union of every water volume extent plus the feed
/// margin.
fn jet_bounds(
    checkpoint: &PhysicsWorldCheckpointV1,
) -> Result<AabbI64V1, WaterPresentCheckErrorV1> {
    let mut minimum = [i64::MAX; 3];
    let mut maximum = [i64::MIN; 3];
    for definition in checkpoint.water_volumes.definitions.values() {
        for axis in 0..3 {
            minimum[axis] = minimum[axis].min(definition.minimum_micrometres[axis]);
            maximum[axis] = maximum[axis].max(definition.maximum_micrometres[axis]);
        }
    }
    if minimum.contains(&i64::MAX) {
        return Err(WaterPresentCheckErrorV1::condition(
            "the reference scene declares at least one water volume",
        ));
    }
    for axis in 0..3 {
        minimum[axis] -= JET_BOUNDS_MARGIN_MICROMETRES;
        maximum[axis] += JET_BOUNDS_MARGIN_MICROMETRES;
    }
    AabbI64V1::new(minimum, maximum)
        .map_err(|error| WaterPresentCheckErrorV1::new("jet bounds", error.to_string()))
}

fn absorb_frame(preimage: &mut Vec<u8>, frame: &WaterPresentationFrameV1) {
    preimage.extend_from_slice(&frame.tick.to_le_bytes());
    preimage.extend_from_slice(&frame.frame_index.to_le_bytes());
    for surface in &frame.surfaces {
        preimage.extend_from_slice(surface.volume_id.as_bytes());
        preimage.extend_from_slice(surface.mesh_asset_id.as_bytes());
        for position in &surface.positions_micrometres {
            for component in position {
                preimage.extend_from_slice(&component.to_le_bytes());
            }
        }
        for normal in &surface.normals_snorm16 {
            for component in normal {
                preimage.extend_from_slice(&component.to_le_bytes());
            }
        }
        for index in &surface.indices {
            preimage.extend_from_slice(&index.to_le_bytes());
        }
    }
    for position in &frame.jet.positions_micrometres {
        for component in position {
            preimage.extend_from_slice(&component.to_le_bytes());
        }
    }
    for velocity in &frame.jet.velocities_micrometres_per_second {
        for component in velocity {
            preimage.extend_from_slice(&component.to_le_bytes());
        }
    }
}

fn evidence_digest(evidence: &GenerationEvidenceV1) -> ContentHash {
    let mut preimage = b"nextengine.water-present-check.v1\0".to_vec();
    preimage.extend_from_slice(&evidence.frames.to_le_bytes());
    preimage.extend_from_slice(&(evidence.surfaces_per_frame as u64).to_le_bytes());
    for flag in [evidence.roots_identical, evidence.purity_identical] {
        preimage.push(u8::from(flag));
    }
    preimage.extend_from_slice(&evidence.capacity_violations.to_le_bytes());
    preimage.extend_from_slice(&evidence.bounds_violations.to_le_bytes());
    preimage.extend_from_slice(&evidence.max_jet_particles.to_le_bytes());
    preimage.extend_from_slice(&evidence.frames_with_jet.to_le_bytes());
    preimage.extend_from_slice(&evidence.max_ripple_micrometres.to_le_bytes());
    preimage.extend_from_slice(evidence.final_state_root.as_bytes());
    preimage.extend_from_slice(evidence.final_physics_checkpoint_hash.as_bytes());
    preimage.extend_from_slice(evidence.frames_digest.as_bytes());
    content_hash_from_bytes(sha256(&preimage))
}

fn require(condition: bool, context: &str) -> Result<(), WaterPresentCheckErrorV1> {
    if condition {
        Ok(())
    } else {
        Err(WaterPresentCheckErrorV1::condition(context))
    }
}
