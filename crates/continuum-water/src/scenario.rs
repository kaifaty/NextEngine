#![forbid(unsafe_code)]

use crate::error::{
    DUPLICATE_SAMPLE_ID, SAMPLE_CAPACITY_EXCEEDED, SCENARIO_INVALID, STEP_CAPACITY_EXCEEDED,
    WaterError,
};
use crate::hash::{FrozenRoots, smoke_scenario_root};
use crate::model::{Aperture, Box3i, CanonicalSample, Geometry, Scenario, StorageOrder, Vec3i};
use crate::profile::{
    LATTICE_SPACING_UM, MAXIMUM_SAMPLES, MAXIMUM_STEPS, POSITION_LIMIT_UM, VELOCITY_LIMIT_UM_S,
};

const SMOKE_MANIFEST: &str = include_str!("../fixtures/smoke-manifest-v1.txt");

pub(crate) fn find(id: &str) -> Result<Scenario, WaterError> {
    let scenario = match id {
        "CW-HYDRO-001" => scenario(
            "CW-HYDRO-001",
            "hydrostatic-column",
            bounds([0, 0, 0], [1_000_000, 1_000_000, 1_000_000]),
            None,
            [20, 15, 20],
            [25_000, 25_000, 25_000],
            [0, 0, 0],
            1_200,
            24,
            false,
        ),
        "CW-FREEFALL-001" => scenario(
            "CW-FREEFALL-001",
            "pre-impact-free-fall",
            bounds([0, 0, 0], [1_000_000, 2_000_000, 1_000_000]),
            None,
            [10, 10, 10],
            [275_000, 1_275_000, 275_000],
            [0, 0, 0],
            96,
            1,
            false,
        ),
        "CW-DAMBREAK-001" => scenario(
            "CW-DAMBREAK-001",
            "three-dimensional-dam-break",
            bounds([0, 0, 0], [4_000_000, 1_000_000, 1_000_000]),
            None,
            [20, 15, 20],
            [25_000, 25_000, 25_000],
            [0, 0, 0],
            720,
            4,
            false,
        ),
        "CW-STILL-001" => scenario(
            "CW-STILL-001",
            "long-horizon-still-tank",
            bounds([0, 0, 0], [1_000_000, 1_000_000, 1_000_000]),
            None,
            [20, 10, 20],
            [25_000, 25_000, 25_000],
            [0, 0, 0],
            7_200,
            240,
            false,
        ),
        "CW-ORIFICE-001" => scenario(
            "CW-ORIFICE-001",
            "sealed-two-chamber-orifice-transfer",
            bounds([0, 0, 0], [2_000_000, 1_000_000, 1_000_000]),
            Some(Aperture {
                wall_x_um: 1_000_000,
                y_min_um: 200_000,
                y_max_um: 400_000,
                z_min_um: 400_000,
                z_max_um: 600_000,
            }),
            [20, 15, 20],
            [25_000, 25_000, 25_000],
            [0, 0, 0],
            720,
            4,
            false,
        ),
        "CW-SEALED-001" => scenario(
            "CW-SEALED-001",
            "nominal-product-boundary-stress",
            bounds(
                [-2_000_000, 0, -1_000_000],
                [2_000_000, 1_000_000, 1_000_000],
            ),
            None,
            [80, 15, 40],
            [-1_975_000, 25_000, -975_000],
            [500_000, 0, 0],
            480,
            8,
            false,
        ),
        "CW-ORDER-001" => scenario(
            "CW-ORDER-001",
            "input-storage-order-invariance",
            bounds([0, 0, 0], [600_000, 600_000, 600_000]),
            None,
            [12, 8, 12],
            [25_000, 25_000, 25_000],
            [0, 0, 0],
            240,
            24,
            false,
        ),
        "SMOKE-CW-HYDRO-001" => smoke_hydro(),
        "SMOKE-CW-FREEFALL-001" => smoke_freefall(),
        "SMOKE-CW-DAMBREAK-001" => smoke_dambreak(),
        "SMOKE-CW-STILL-001" => smoke_still(),
        "SMOKE-CW-ORIFICE-001" => smoke_orifice(),
        "SMOKE-CW-SEALED-001" => smoke_sealed(),
        "SMOKE-CW-ORDER-001" => smoke_order(),
        _ => {
            return Err(WaterError::new(
                SCENARIO_INVALID,
                format!("unknown frozen scenario {id:?}"),
            ));
        }
    };
    validate_definition(&scenario)?;
    Ok(scenario)
}

pub(crate) fn root_for(scenario: &Scenario, roots: &FrozenRoots) -> Result<[u8; 32], WaterError> {
    let source_projection = if scenario.smoke_only {
        let prefix = format!("scenario.{}.", scenario.id);
        let mut lines = Vec::new();
        for line in SMOKE_MANIFEST.split_inclusive('\n') {
            if line.starts_with(&prefix) {
                lines.extend_from_slice(line.as_bytes());
            }
        }
        if lines.is_empty() {
            return Err(WaterError::new(
                SCENARIO_INVALID,
                format!("smoke scenario {} has no manifest projection", scenario.id),
            ));
        }
        lines
    } else {
        roots.scenario_projection(scenario.id)?
    };
    let implementation_projection = implementation_projection(scenario)?;
    if source_projection != implementation_projection.as_bytes() {
        return Err(WaterError::new(
            SCENARIO_INVALID,
            format!(
                "scenario {} implementation differs from its hash-bound manifest projection",
                scenario.id
            ),
        ));
    }
    if scenario.smoke_only {
        Ok(smoke_scenario_root(&source_projection))
    } else {
        roots.scenario_root(scenario.id)
    }
}

pub(crate) fn successor_projection(scenario: &Scenario) -> Result<String, WaterError> {
    use std::fmt::Write as _;

    if scenario.smoke_only {
        return Err(WaterError::new(
            SCENARIO_INVALID,
            "smoke scenarios are not part of the successor corpus",
        ));
    }
    let mut output = String::new();
    writeln!(output, "scenario.{}.kind={}", scenario.id, scenario.kind)
        .map_err(|_| WaterError::new(SCENARIO_INVALID, "successor kind formatting failed"))?;
    writeln!(
        output,
        "scenario.{}.box_um=({},{},{})..({},{},{})",
        scenario.id,
        scenario.geometry.bounds.min.x,
        scenario.geometry.bounds.min.y,
        scenario.geometry.bounds.min.z,
        scenario.geometry.bounds.max.x,
        scenario.geometry.bounds.max.y,
        scenario.geometry.bounds.max.z
    )
    .map_err(|_| WaterError::new(SCENARIO_INVALID, "successor box formatting failed"))?;
    if let Some(aperture) = scenario.geometry.aperture {
        writeln!(
            output,
            "scenario.{}.boundary=successor-outer-box-plus-patch-16",
            scenario.id
        )
        .map_err(|_| WaterError::new(SCENARIO_INVALID, "successor wall formatting failed"))?;
        writeln!(
            output,
            "scenario.{}.aperture_um=x={};y={}..{};z={}..{};opening-set-closed",
            scenario.id,
            aperture.wall_x_um,
            aperture.y_min_um,
            aperture.y_max_um,
            aperture.z_min_um,
            aperture.z_max_um
        )
        .map_err(|_| WaterError::new(SCENARIO_INVALID, "successor opening formatting failed"))?;
    } else if scenario.id == "CW-SEALED-001" {
        writeln!(
            output,
            "scenario.{}.boundary=successor-closed-outer-box;static-support-count=24704",
            scenario.id
        )
        .map_err(|_| {
            WaterError::new(
                SCENARIO_INVALID,
                "successor product boundary formatting failed",
            )
        })?;
    } else {
        writeln!(
            output,
            "scenario.{}.boundary=successor-closed-outer-box",
            scenario.id
        )
        .map_err(|_| WaterError::new(SCENARIO_INVALID, "successor boundary formatting failed"))?;
    }
    writeln!(
        output,
        "scenario.{}.fluid={},{},{};({},{},{});({},{},{})",
        scenario.id,
        scenario.dimensions[0],
        scenario.dimensions[1],
        scenario.dimensions[2],
        scenario.first_position_um.x,
        scenario.first_position_um.y,
        scenario.first_position_um.z,
        scenario.initial_velocity_um_s.x,
        scenario.initial_velocity_um_s.y,
        scenario.initial_velocity_um_s.z
    )
    .map_err(|_| WaterError::new(SCENARIO_INVALID, "successor fluid formatting failed"))?;
    writeln!(output, "scenario.{}.steps={}", scenario.id, scenario.steps)
        .map_err(|_| WaterError::new(SCENARIO_INVALID, "successor steps formatting failed"))?;
    writeln!(
        output,
        "scenario.{}.outputs=0..{}/every={}",
        scenario.id, scenario.steps, scenario.output_every
    )
    .map_err(|_| WaterError::new(SCENARIO_INVALID, "successor outputs formatting failed"))?;
    if scenario.id == "CW-FREEFALL-001" {
        writeln!(
            output,
            "scenario.CW-FREEFALL-001.recurrence=publish-vy-plus-gravity-then-publish-y-plus-published-vy-over-240"
        )
        .map_err(|_| {
            WaterError::new(SCENARIO_INVALID, "successor recurrence formatting failed")
        })?;
    }
    if scenario.id == "CW-ORDER-001" {
        writeln!(
            output,
            "scenario.CW-ORDER-001.storage-orders=identity,reverse,affine-257k-plus-17-mod-1152"
        )
        .map_err(|_| WaterError::new(SCENARIO_INVALID, "successor order formatting failed"))?;
    }
    writeln!(
        output,
        "scenario.{}.reference={}",
        scenario.id,
        reference_label(scenario.id)?
    )
    .map_err(|_| WaterError::new(SCENARIO_INVALID, "successor reference formatting failed"))?;
    Ok(output)
}

fn implementation_projection(scenario: &Scenario) -> Result<String, WaterError> {
    use std::fmt::Write as _;

    let mut output = String::new();
    writeln!(output, "scenario.{}.kind={}", scenario.id, scenario.kind)
        .map_err(|_| WaterError::new(SCENARIO_INVALID, "scenario projection formatting failed"))?;
    writeln!(
        output,
        "scenario.{}.box_um=({},{},{})..({},{},{})",
        scenario.id,
        scenario.geometry.bounds.min.x,
        scenario.geometry.bounds.min.y,
        scenario.geometry.bounds.min.z,
        scenario.geometry.bounds.max.x,
        scenario.geometry.bounds.max.y,
        scenario.geometry.bounds.max.z
    )
    .map_err(|_| WaterError::new(SCENARIO_INVALID, "scenario box formatting failed"))?;
    if let Some(aperture) = scenario.geometry.aperture {
        writeln!(
            output,
            "scenario.{}.boundary=closed-outer-shell-plus-wall-x-{}",
            scenario.id, aperture.wall_x_um
        )
        .map_err(|_| WaterError::new(SCENARIO_INVALID, "scenario boundary formatting failed"))?;
        writeln!(
            output,
            "scenario.{}.aperture_um=x={};y={}..{};z={}..{};opening-set-closed",
            scenario.id,
            aperture.wall_x_um,
            aperture.y_min_um,
            aperture.y_max_um,
            aperture.z_min_um,
            aperture.z_max_um
        )
        .map_err(|_| WaterError::new(SCENARIO_INVALID, "scenario aperture formatting failed"))?;
    } else {
        writeln!(
            output,
            "scenario.{}.boundary=closed-outer-shell",
            scenario.id
        )
        .map_err(|_| WaterError::new(SCENARIO_INVALID, "scenario boundary formatting failed"))?;
    }
    writeln!(
        output,
        "scenario.{}.fluid={},{},{};({},{},{});({},{},{})",
        scenario.id,
        scenario.dimensions[0],
        scenario.dimensions[1],
        scenario.dimensions[2],
        scenario.first_position_um.x,
        scenario.first_position_um.y,
        scenario.first_position_um.z,
        scenario.initial_velocity_um_s.x,
        scenario.initial_velocity_um_s.y,
        scenario.initial_velocity_um_s.z
    )
    .map_err(|_| WaterError::new(SCENARIO_INVALID, "scenario fluid formatting failed"))?;
    writeln!(output, "scenario.{}.steps={}", scenario.id, scenario.steps)
        .map_err(|_| WaterError::new(SCENARIO_INVALID, "scenario steps formatting failed"))?;
    writeln!(
        output,
        "scenario.{}.outputs=0..{}/every={}",
        scenario.id, scenario.steps, scenario.output_every
    )
    .map_err(|_| WaterError::new(SCENARIO_INVALID, "scenario outputs formatting failed"))?;
    if scenario.id.ends_with("ORDER-001") {
        let affine = if scenario.smoke_only {
            "affine-257k-plus-17-mod-N"
        } else {
            "affine-257k-plus-17-mod-1152"
        };
        writeln!(
            output,
            "scenario.{}.storage_orders=identity,reverse,{affine}",
            scenario.id
        )
        .map_err(|_| {
            WaterError::new(SCENARIO_INVALID, "scenario storage order formatting failed")
        })?;
    }
    writeln!(
        output,
        "scenario.{}.reference={}",
        scenario.id,
        reference_label(scenario.id)?
    )
    .map_err(|_| WaterError::new(SCENARIO_INVALID, "scenario reference formatting failed"))?;
    Ok(output)
}

fn reference_label(id: &str) -> Result<&'static str, WaterError> {
    match id {
        "CW-HYDRO-001" => Ok("analytical-mass-symmetry-work-and-splishsplash-aggregate"),
        "CW-FREEFALL-001" => Ok("exact-canonical-semi-implicit-gravity-recurrence"),
        "CW-DAMBREAK-001" => Ok("splishsplash-aggregate"),
        "CW-STILL-001" => Ok("analytical-mass-symmetry-work-and-repeat-root"),
        "CW-ORIFICE-001" => Ok("splishsplash-aggregate-and-exact-partition-accounting"),
        "CW-SEALED-001" => Ok("exact-count-mass-boundary-and-repeat-root"),
        "CW-ORDER-001" => Ok("exact-trajectory-root-equality"),
        id if id.starts_with("SMOKE-CW-") => Ok("smoke-only-no-corpus-credit"),
        _ => Err(WaterError::new(
            SCENARIO_INVALID,
            format!("scenario {id} has no frozen reference label"),
        )),
    }
}

pub(crate) fn initial_samples(
    scenario: &Scenario,
    order: StorageOrder,
) -> Result<Vec<CanonicalSample>, WaterError> {
    let count = sample_count(scenario.dimensions)?;
    validate_capacity(
        count,
        MAXIMUM_SAMPLES,
        SAMPLE_CAPACITY_EXCEEDED,
        "fluid samples",
    )?;
    let mut samples = Vec::new();
    samples.try_reserve_exact(count).map_err(|error| {
        WaterError::new(
            SAMPLE_CAPACITY_EXCEEDED,
            format!("cannot reserve {count} fluid samples: {error}"),
        )
    })?;
    let nx = usize::try_from(scenario.dimensions[0])
        .map_err(|_| WaterError::new(SCENARIO_INVALID, "nx conversion overflow"))?;
    let ny = usize::try_from(scenario.dimensions[1])
        .map_err(|_| WaterError::new(SCENARIO_INVALID, "ny conversion overflow"))?;
    let nz = usize::try_from(scenario.dimensions[2])
        .map_err(|_| WaterError::new(SCENARIO_INVALID, "nz conversion overflow"))?;
    for iy in 0..ny {
        for iz in 0..nz {
            for ix in 0..nx {
                let id_usize = ((iy * nz + iz) * nx) + ix;
                let id = u32::try_from(id_usize).map_err(|_| {
                    WaterError::new(SCENARIO_INVALID, "SampleId conversion overflow")
                })?;
                let x = lattice_coordinate(scenario.first_position_um.x, ix)?;
                let y = lattice_coordinate(scenario.first_position_um.y, iy)?;
                let z = lattice_coordinate(scenario.first_position_um.z, iz)?;
                samples.push(CanonicalSample {
                    id,
                    position_um: Vec3i::new(x, y, z),
                    velocity_um_s: scenario.initial_velocity_um_s,
                });
            }
        }
    }
    reorder(&mut samples, order)?;
    validate_sample_identity(&samples)?;
    Ok(samples)
}

fn reorder(samples: &mut Vec<CanonicalSample>, order: StorageOrder) -> Result<(), WaterError> {
    match order {
        StorageOrder::Identity => {}
        StorageOrder::Reverse => samples.reverse(),
        StorageOrder::Affine => {
            let count = samples.len();
            if count == 0 {
                return Ok(());
            }
            let mut reordered = Vec::new();
            reordered.try_reserve_exact(count).map_err(|error| {
                WaterError::new(
                    SAMPLE_CAPACITY_EXCEEDED,
                    format!("cannot reserve affine storage order: {error}"),
                )
            })?;
            for k in 0..count {
                let index = 257_usize
                    .checked_mul(k)
                    .and_then(|value| value.checked_add(17))
                    .ok_or_else(|| {
                        WaterError::new(SCENARIO_INVALID, "affine storage index overflow")
                    })?
                    % count;
                reordered.push(samples[index].clone());
            }
            *samples = reordered;
        }
    }
    Ok(())
}

pub(crate) fn validate_sample_identity(samples: &[CanonicalSample]) -> Result<(), WaterError> {
    let mut ids = Vec::new();
    ids.try_reserve_exact(samples.len()).map_err(|error| {
        WaterError::new(
            SAMPLE_CAPACITY_EXCEEDED,
            format!("cannot reserve SampleId validation buffer: {error}"),
        )
    })?;
    for sample in samples {
        ids.push(sample.id);
    }
    ids.sort_unstable();
    for pair in ids.windows(2) {
        if pair[0] == pair[1] {
            return Err(WaterError::new(
                DUPLICATE_SAMPLE_ID,
                format!("duplicate SampleId {}", pair[0]),
            ));
        }
    }
    Ok(())
}

pub(crate) fn validate_capacity(
    value: usize,
    maximum: usize,
    code: &'static str,
    label: &str,
) -> Result<(), WaterError> {
    if value > maximum {
        Err(WaterError::new(
            code,
            format!("{label} count {value} exceeds {maximum}"),
        ))
    } else {
        Ok(())
    }
}

fn validate_definition(scenario: &Scenario) -> Result<(), WaterError> {
    if scenario.steps > MAXIMUM_STEPS {
        return Err(WaterError::new(
            STEP_CAPACITY_EXCEEDED,
            format!("{} steps exceed {MAXIMUM_STEPS}", scenario.steps),
        ));
    }
    if scenario.output_every == 0 {
        return Err(WaterError::new(
            SCENARIO_INVALID,
            "output cadence must be nonzero",
        ));
    }
    if scenario.dimensions.contains(&0) {
        return Err(WaterError::new(
            SCENARIO_INVALID,
            "frozen scenario dimensions must be nonzero",
        ));
    }
    let bounds = scenario.geometry.bounds;
    if bounds.min.x >= bounds.max.x || bounds.min.y >= bounds.max.y || bounds.min.z >= bounds.max.z
    {
        return Err(WaterError::new(
            SCENARIO_INVALID,
            "scenario box has an empty axis",
        ));
    }
    for coordinate in [
        bounds.min.x,
        bounds.min.y,
        bounds.min.z,
        bounds.max.x,
        bounds.max.y,
        bounds.max.z,
    ] {
        if coordinate.unsigned_abs() > POSITION_LIMIT_UM as u64 {
            return Err(WaterError::new(
                SCENARIO_INVALID,
                "scenario box exceeds the W0B laboratory bound",
            ));
        }
    }
    for velocity in [
        scenario.initial_velocity_um_s.x,
        scenario.initial_velocity_um_s.y,
        scenario.initial_velocity_um_s.z,
    ] {
        if velocity.unsigned_abs() > VELOCITY_LIMIT_UM_S as u64 {
            return Err(WaterError::new(
                SCENARIO_INVALID,
                "scenario initial velocity exceeds the W0B laboratory bound",
            ));
        }
    }
    let last = Vec3i::new(
        lattice_coordinate(
            scenario.first_position_um.x,
            scenario.dimensions[0] as usize - 1,
        )?,
        lattice_coordinate(
            scenario.first_position_um.y,
            scenario.dimensions[1] as usize - 1,
        )?,
        lattice_coordinate(
            scenario.first_position_um.z,
            scenario.dimensions[2] as usize - 1,
        )?,
    );
    let first = scenario.first_position_um;
    if first.x < bounds.min.x
        || first.y < bounds.min.y
        || first.z < bounds.min.z
        || last.x > bounds.max.x
        || last.y > bounds.max.y
        || last.z > bounds.max.z
    {
        return Err(WaterError::new(
            SCENARIO_INVALID,
            "scenario fluid lattice is outside its box",
        ));
    }
    if let Some(aperture) = scenario.geometry.aperture
        && (aperture.wall_x_um <= bounds.min.x
            || aperture.wall_x_um >= bounds.max.x
            || aperture.y_min_um > aperture.y_max_um
            || aperture.z_min_um > aperture.z_max_um
            || aperture.y_min_um < bounds.min.y
            || aperture.y_max_um > bounds.max.y
            || aperture.z_min_um < bounds.min.z
            || aperture.z_max_um > bounds.max.z)
    {
        return Err(WaterError::new(
            SCENARIO_INVALID,
            "scenario aperture is outside or degenerates its internal wall",
        ));
    }
    let count = sample_count(scenario.dimensions)?;
    validate_capacity(
        count,
        MAXIMUM_SAMPLES,
        SAMPLE_CAPACITY_EXCEEDED,
        "fluid samples",
    )
}

fn sample_count(dimensions: [u32; 3]) -> Result<usize, WaterError> {
    dimensions.into_iter().try_fold(1_usize, |product, value| {
        product.checked_mul(value as usize).ok_or_else(|| {
            WaterError::new(SAMPLE_CAPACITY_EXCEEDED, "sample count product overflow")
        })
    })
}

fn lattice_coordinate(first: i64, index: usize) -> Result<i64, WaterError> {
    let index = i64::try_from(index)
        .map_err(|_| WaterError::new(SCENARIO_INVALID, "lattice index overflow"))?;
    LATTICE_SPACING_UM
        .checked_mul(index)
        .and_then(|offset| first.checked_add(offset))
        .ok_or_else(|| WaterError::new(SCENARIO_INVALID, "lattice coordinate overflow"))
}

fn bounds(min: [i64; 3], max: [i64; 3]) -> Box3i {
    Box3i {
        min: Vec3i::new(min[0], min[1], min[2]),
        max: Vec3i::new(max[0], max[1], max[2]),
    }
}

#[allow(clippy::too_many_arguments)]
fn scenario(
    id: &'static str,
    kind: &'static str,
    bounds: Box3i,
    aperture: Option<Aperture>,
    dimensions: [u32; 3],
    first: [i64; 3],
    velocity: [i64; 3],
    steps: u32,
    output_every: u32,
    smoke_only: bool,
) -> Scenario {
    Scenario {
        id,
        kind,
        geometry: Geometry { bounds, aperture },
        dimensions,
        first_position_um: Vec3i::new(first[0], first[1], first[2]),
        initial_velocity_um_s: Vec3i::new(velocity[0], velocity[1], velocity[2]),
        steps,
        output_every,
        smoke_only,
    }
}

fn smoke_hydro() -> Scenario {
    scenario(
        "SMOKE-CW-HYDRO-001",
        "hydrostatic-column-smoke-only",
        bounds([0, 0, 0], [600_000, 600_000, 600_000]),
        None,
        [4, 3, 4],
        [225_000, 225_000, 225_000],
        [0, 0, 0],
        8,
        1,
        true,
    )
}

fn smoke_freefall() -> Scenario {
    scenario(
        "SMOKE-CW-FREEFALL-001",
        "pre-impact-free-fall-smoke-only",
        bounds([0, 0, 0], [600_000, 1_000_000, 600_000]),
        None,
        [4, 4, 4],
        [225_000, 675_000, 225_000],
        [0, 0, 0],
        8,
        1,
        true,
    )
}

fn smoke_dambreak() -> Scenario {
    scenario(
        "SMOKE-CW-DAMBREAK-001",
        "three-dimensional-dam-break-smoke-only",
        bounds([0, 0, 0], [800_000, 600_000, 600_000]),
        None,
        [4, 3, 4],
        [225_000, 225_000, 225_000],
        [0, 0, 0],
        8,
        1,
        true,
    )
}

fn smoke_still() -> Scenario {
    scenario(
        "SMOKE-CW-STILL-001",
        "long-horizon-still-tank-smoke-only",
        bounds([0, 0, 0], [600_000, 600_000, 600_000]),
        None,
        [4, 2, 4],
        [225_000, 225_000, 225_000],
        [0, 0, 0],
        8,
        1,
        true,
    )
}

fn smoke_orifice() -> Scenario {
    scenario(
        "SMOKE-CW-ORIFICE-001",
        "sealed-two-chamber-orifice-transfer-smoke-only",
        bounds([0, 0, 0], [1_000_000, 600_000, 600_000]),
        Some(Aperture {
            wall_x_um: 500_000,
            y_min_um: 200_000,
            y_max_um: 400_000,
            z_min_um: 200_000,
            z_max_um: 400_000,
        }),
        [4, 3, 4],
        [225_000, 225_000, 225_000],
        [0, 0, 0],
        8,
        1,
        true,
    )
}

fn smoke_sealed() -> Scenario {
    scenario(
        "SMOKE-CW-SEALED-001",
        "nominal-product-boundary-stress-smoke-only",
        bounds([-400_000, 0, -400_000], [400_000, 600_000, 400_000]),
        None,
        [8, 3, 4],
        [-175_000, 225_000, -75_000],
        [50_000, 0, 0],
        8,
        1,
        true,
    )
}

fn smoke_order() -> Scenario {
    scenario(
        "SMOKE-CW-ORDER-001",
        "input-storage-order-invariance-smoke-only",
        bounds([0, 0, 0], [600_000, 600_000, 600_000]),
        None,
        [4, 3, 4],
        [225_000, 225_000, 225_000],
        [0, 0, 0],
        8,
        1,
        true,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exact_corpus_counts_are_frozen() {
        for (id, expected) in [
            ("CW-HYDRO-001", 6_000),
            ("CW-FREEFALL-001", 1_000),
            ("CW-DAMBREAK-001", 6_000),
            ("CW-STILL-001", 4_000),
            ("CW-ORIFICE-001", 6_000),
            ("CW-SEALED-001", 48_000),
            ("CW-ORDER-001", 1_152),
        ] {
            assert_eq!(
                initial_samples(&find(id).unwrap(), StorageOrder::Reverse)
                    .unwrap()
                    .len(),
                expected
            );
        }
    }

    #[test]
    fn successor_scenario_definitions_match_every_rooted_projection() {
        let repository_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .and_then(std::path::Path::parent)
            .unwrap();
        let roots = crate::hash::SuccessorRoots::verify(repository_root).unwrap();
        for id in [
            "CW-HYDRO-001",
            "CW-FREEFALL-001",
            "CW-DAMBREAK-001",
            "CW-STILL-001",
            "CW-ORIFICE-001",
            "CW-SEALED-001",
            "CW-ORDER-001",
        ] {
            let scenario = find(id).unwrap();
            assert_eq!(
                successor_projection(&scenario).unwrap().as_bytes(),
                roots.scenario_projection(id).unwrap()
            );
        }
    }

    #[test]
    fn duplicate_ids_are_rejected_and_empty_or_singleton_are_valid() {
        assert!(validate_sample_identity(&[]).is_ok());
        let one = CanonicalSample {
            id: 7,
            position_um: Vec3i::new(0, 0, 0),
            velocity_um_s: Vec3i::new(0, 0, 0),
        };
        assert!(validate_sample_identity(std::slice::from_ref(&one)).is_ok());
        assert_eq!(
            validate_sample_identity(&[one.clone(), one])
                .unwrap_err()
                .code(),
            DUPLICATE_SAMPLE_ID
        );
    }

    #[test]
    fn sample_and_step_capacities_accept_n_minus_one_and_n() {
        for value in [MAXIMUM_SAMPLES - 1, MAXIMUM_SAMPLES] {
            assert!(
                validate_capacity(value, MAXIMUM_SAMPLES, SAMPLE_CAPACITY_EXCEEDED, "samples")
                    .is_ok()
            );
        }
        assert_eq!(
            validate_capacity(
                MAXIMUM_SAMPLES + 1,
                MAXIMUM_SAMPLES,
                SAMPLE_CAPACITY_EXCEEDED,
                "samples"
            )
            .unwrap_err()
            .code(),
            SAMPLE_CAPACITY_EXCEEDED
        );
        for value in [MAXIMUM_STEPS - 1, MAXIMUM_STEPS] {
            assert!(
                validate_capacity(
                    value as usize,
                    MAXIMUM_STEPS as usize,
                    STEP_CAPACITY_EXCEEDED,
                    "steps"
                )
                .is_ok()
            );
        }
        assert_eq!(
            validate_capacity(
                (MAXIMUM_STEPS + 1) as usize,
                MAXIMUM_STEPS as usize,
                STEP_CAPACITY_EXCEEDED,
                "steps"
            )
            .unwrap_err()
            .code(),
            STEP_CAPACITY_EXCEEDED
        );
    }

    #[test]
    fn all_seven_smoke_analogues_are_checked_in() {
        for id in [
            "SMOKE-CW-HYDRO-001",
            "SMOKE-CW-FREEFALL-001",
            "SMOKE-CW-DAMBREAK-001",
            "SMOKE-CW-STILL-001",
            "SMOKE-CW-ORIFICE-001",
            "SMOKE-CW-SEALED-001",
            "SMOKE-CW-ORDER-001",
        ] {
            let scenario = find(id).unwrap();
            assert!(scenario.smoke_only);
            assert!(SMOKE_MANIFEST.contains(&format!("scenario.{id}.kind=")));
        }
    }

    #[test]
    fn executed_scenario_definitions_match_every_hash_bound_projection() {
        let repository_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .and_then(std::path::Path::parent)
            .unwrap();
        let roots = FrozenRoots::verify(repository_root).unwrap();
        for id in [
            "CW-HYDRO-001",
            "CW-FREEFALL-001",
            "CW-DAMBREAK-001",
            "CW-STILL-001",
            "CW-ORIFICE-001",
            "CW-SEALED-001",
            "CW-ORDER-001",
            "SMOKE-CW-HYDRO-001",
            "SMOKE-CW-FREEFALL-001",
            "SMOKE-CW-DAMBREAK-001",
            "SMOKE-CW-STILL-001",
            "SMOKE-CW-ORIFICE-001",
            "SMOKE-CW-SEALED-001",
            "SMOKE-CW-ORDER-001",
        ] {
            let scenario = find(id).unwrap();
            root_for(&scenario, &roots)
                .unwrap_or_else(|error| panic!("{id} projection mismatch: {error}"));
        }
    }
}
