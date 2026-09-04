//! Plan 25 (ADR-106): the PhysX water presentation lane. A GPU fluid over
//! the basin whose floor is the exact water level and whose walls are the
//! rim's inner faces; particles are born where the exact model says water
//! leaves it (a floating box's waterline while it moves, the crest of a jet
//! or fall inside the box) and die when they settle on the level. Stepped
//! at `60 Hz` from the frame clock and published to the ADR-102 particle
//! pass every frame. Presentation only: no command, query, save, replay or
//! root reads it; without the GPU library the lane reports itself
//! unavailable and the run continues with the stage's droplets.

use std::sync::Arc;
use std::time::{Duration, Instant};

use next_contracts::physics::WATER_FLOW_GRAVITY_MICROMETRES_PER_SECOND_SQUARED;
use next_contracts::render_content::AabbI64V1;
use next_desktop_sdl_ash::{
    DesktopAdapterError, ParticleSurfaceProfileV1, ParticleSurfaceUpdateV1,
};
use next_physics_physx_ffi::{FluidDesc, FluidSample, NativeFluid, gpu_library_path};
use next_reference_game::{
    REFERENCE_WATER_BASIN_INITIAL_LEVEL_MICROMETRES, REFERENCE_WATER_BASIN_RIM_BOXES_MICROMETRES,
    WATER_JET_DROPLET_VOLUME_CUBIC_MILLIMETRES, WATER_JET_MAX_SPAWN_PER_FRAME,
    WATER_SPLASH_LAUNCH_SPEED_MICROMETRES_PER_SECOND, WATER_SPLASH_MAX_PER_BOX,
    WATER_SPLASH_SPEED_PER_DROPLET_MICROMETRES_PER_SECOND,
    WATER_SPLASH_SPEED_THRESHOLD_MICROMETRES_PER_SECOND, WaterEdgePresentationKindV1,
    WaterPresentationFrameV1, reference_water_basin_definition,
};

const LANE_SPACING_METRES: f32 = 0.05;
const LANE_TIMESTEP: Duration = Duration::from_micros(16_667);
const LANE_MAX_STEPS_PER_FRAME: u32 = 4;
const LANE_MAX_PARTICLES: u32 = 16_384;
const LANE_RADIUS_MICROMETRES: u32 = 45_000;
/// Plan 27: neighbours within two spacings drive spray and bulk.
const NEIGHBOUR_RADIUS_METRES: f32 = 2.0 * LANE_SPACING_METRES;
const SPRAY_NEIGHBOUR_THRESHOLD: u32 = 6;
const SPRAY_CLUSTER_THRESHOLD: u32 = 8;
const SPRAY_RADIUS_MICROMETRES: u32 = 4_000;
const SPRAY_ALPHA: f32 = 0.5;
const SPRAY_SUBDROPLETS: u32 = 12;
const SPRAY_STREAK_SECONDS: f32 = 1.0 / 60.0;
const BULK_NEIGHBOUR_COUNT: u32 = 20;
const EDGE_RADIUS_SCALE: f32 = 0.5;
/// Plan 30: anisotropic kernels (Yu and Turk 2010) from the weighted
/// covariance of the neighbourhood; semi-axes `KS·√λ`, the smallest axis
/// raised to `r_max / KR`, every axis clamped to the profile radius band.
const KERNEL_MIN_NEIGHBOURS: u32 = 8;
const KERNEL_SCALE: f32 = 1.0;
const KERNEL_MAX_RATIO: f32 = 4.0;
const KERNEL_MIN_AXIS_SCALE: f32 = 0.25;
const KERNEL_MAX_AXIS_SCALE: f32 = 2.0;
const KERNEL_PROFILE_RADIUS_METRES: f32 = LANE_RADIUS_MICROMETRES as f32 / 1_000_000.0;
/// Plan 40: the kernel's weight on its own previous value, the pause before
/// the fluid is recreated after a failure, and the attempts allowed.
const KERNEL_BLEND_PREVIOUS: f32 = 0.7;
const RECREATE_AFTER_FRAMES: u64 = 300;
const RECREATE_MAX_ATTEMPTS: u32 = 3;
/// Plan 25: a particle this close above the level at rest has returned to
/// the exact water (two spacings).
const ABSORB_BAND_METRES: f32 = 2.0 * LANE_SPACING_METRES;
const ABSORB_SPEED_METRES_PER_SECOND: f32 = 0.3;
/// Plan 24's block for the look (`--physx-water-pour`).
const POUR_BLOCK_HALF_METRES: f32 = 0.5;
const POUR_BLOCK_BOTTOM_METRES: f32 = 1.0;
const POUR_BLOCK_HEIGHT_METRES: f32 = 0.6;

/// The lane's fluid box in metres: floor at the level, walls at the rim.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct FluidBox {
    pub(crate) min: [f32; 3],
    pub(crate) max: [f32; 3],
}

impl FluidBox {
    pub(crate) fn contains_plan(&self, x: f32, z: f32) -> bool {
        x >= self.min[0] && x <= self.max[0] && z >= self.min[2] && z <= self.max[2]
    }
}

/// Plan 25: the basin's fluid box from the reference scene.
pub(crate) fn basin_fluid_box() -> FluidBox {
    let basin = reference_water_basin_definition();
    let metres = |value: i64| value as f32 / 1_000_000.0;
    let mut min = basin.minimum_micrometres.map(metres);
    let mut max = basin.maximum_micrometres.map(metres);
    for (centre, half) in REFERENCE_WATER_BASIN_RIM_BOXES_MICROMETRES {
        let (centre, half) = (centre.map(metres), half.map(metres));
        if half[0] < half[2] {
            if centre[0] < min[0] + 1.0 {
                min[0] = min[0].max(centre[0] + half[0]);
            } else {
                max[0] = max[0].min(centre[0] - half[0]);
            }
        } else if centre[2] > max[2] - 1.0 {
            max[2] = max[2].min(centre[2] - half[2]);
        } else {
            min[2] = min[2].max(centre[2] + half[2]);
        }
    }
    min[1] = metres(REFERENCE_WATER_BASIN_INITIAL_LEVEL_MICROMETRES);
    max[1] += 10.0;
    FluidBox { min, max }
}

/// Plan 25 emission: the particles a stage frame brings into the fluid,
/// positions in metres and velocities in metres per second. Pure.
pub(crate) fn emission_for_frame(
    frame: &WaterPresentationFrameV1,
    fluid_box: &FluidBox,
    level_metres: f32,
    room: usize,
) -> (Vec<[f32; 3]>, Vec<[f32; 3]>) {
    let mut positions = Vec::new();
    let mut velocities = Vec::new();
    let metres = |value: i64| value as f32 / 1_000_000.0;
    let launch =
        metres(WATER_SPLASH_LAUNCH_SPEED_MICROMETRES_PER_SECOND) * std::f32::consts::FRAC_1_SQRT_2;
    // Floating boxes: plan 17's rule on the waterline perimeter.
    for floating in &frame.boxes {
        let speed = floating
            .vertical_velocity_micrometres_per_second
            .abs()
            .saturating_sub(WATER_SPLASH_SPEED_THRESHOLD_MICROMETRES_PER_SECOND);
        if speed <= 0 {
            continue;
        }
        let count = (speed / WATER_SPLASH_SPEED_PER_DROPLET_MICROMETRES_PER_SECOND)
            .clamp(0, i64::from(WATER_SPLASH_MAX_PER_BOX)) as usize;
        let [x0, _, z0] = floating.minimum_micrometres.map(metres);
        let [x1, _, z1] = floating.maximum_micrometres.map(metres);
        if !fluid_box.contains_plan((x0 + x1) * 0.5, (z0 + z1) * 0.5) {
            continue;
        }
        let width = (x1 - x0).max(0.001);
        let depth = (z1 - z0).max(0.001);
        let perimeter = 2.0 * (width + depth);
        for k in 0..count {
            if positions.len() >= room {
                return (positions, velocities);
            }
            let along = perimeter * k as f32 / count as f32;
            let (x, z, out_x, out_z) = if along < width {
                (x0 + along, z0, 0.0, -1.0)
            } else if along < width + depth {
                (x1, z0 + (along - width), 1.0, 0.0)
            } else if along < 2.0 * width + depth {
                (x1 - (along - width - depth), z1, 0.0, 1.0)
            } else {
                (x0, z1 - (along - 2.0 * width - depth), -1.0, 0.0)
            };
            positions.push([
                x + out_x * LANE_SPACING_METRES,
                level_metres + LANE_SPACING_METRES,
                z + out_z * LANE_SPACING_METRES,
            ]);
            velocities.push([out_x * launch, launch, out_z * launch]);
        }
    }
    // Jets and falls: plan 21's rule at the crest, inside the box only.
    let gravity = metres(WATER_FLOW_GRAVITY_MICROMETRES_PER_SECOND_SQUARED);
    for record in &frame.edges {
        if !matches!(
            record.kind,
            WaterEdgePresentationKindV1::Jet | WaterEdgePresentationKindV1::Fall
        ) {
            continue;
        }
        let crest = record.crest_micrometres.map(metres);
        if !fluid_box.contains_plan(crest[0], crest[2]) {
            continue;
        }
        let head = metres(
            record
                .source_level_micrometres
                .saturating_sub(record.crest_micrometres[1]),
        )
        .max(0.0);
        if head <= 0.0 {
            continue;
        }
        let speed = (2.0 * gravity * head).sqrt();
        let count = (record.flux_cubic_millimetres.abs()
            / WATER_JET_DROPLET_VOLUME_CUBIC_MILLIMETRES)
            .clamp(0, i64::from(WATER_JET_MAX_SPAWN_PER_FRAME)) as usize;
        let direction = [
            record.direction_q15[0] as f32 / 32_767.0,
            record.direction_q15[1] as f32 / 32_767.0,
        ];
        for k in 0..count {
            if positions.len() >= room {
                return (positions, velocities);
            }
            let lateral = ((k as i64 * 7_919) % 41 - 20) as f32 * 0.002;
            positions.push([
                crest[0] - direction[1] * lateral,
                crest[1],
                crest[2] + direction[0].abs() * lateral,
            ]);
            velocities.push([direction[0] * speed, 0.0, direction[1] * speed]);
        }
    }
    (positions, velocities)
}

/// Plan 25 absorption: keeps the particles still in flight; a particle
/// within the band above the level moving slower than the threshold, or
/// below the floor, has returned to the exact water. Pure. Returns the kept
/// set and the number absorbed.
#[allow(
    clippy::type_complexity,
    reason = "the absorption's four outputs are one frame's sample, kept together"
)]
pub(crate) fn absorb(
    sample: &FluidSample,
    level_metres: f32,
) -> (Vec<[f32; 3]>, Vec<[f32; 3]>, usize, Vec<usize>) {
    let mut positions = Vec::with_capacity(sample.positions.len());
    let mut velocities = Vec::with_capacity(sample.positions.len());
    let mut kept = Vec::with_capacity(sample.positions.len());
    let mut absorbed = 0;
    for (index, (position, velocity)) in sample.positions.iter().zip(&sample.velocities).enumerate()
    {
        if !position.iter().all(|value| value.is_finite())
            || !velocity.iter().all(|value| value.is_finite())
        {
            absorbed += 1;
            continue;
        }
        let speed =
            (velocity[0] * velocity[0] + velocity[1] * velocity[1] + velocity[2] * velocity[2])
                .sqrt();
        let height = position[1] - level_metres;
        if height < 0.0 || (height < ABSORB_BAND_METRES && speed < ABSORB_SPEED_METRES_PER_SECOND) {
            absorbed += 1;
            continue;
        }
        positions.push(*position);
        velocities.push(*velocity);
        kept.push(index);
    }
    (positions, velocities, absorbed, kept)
}

/// Plan 40: one particle's kernel blended with its own previous one.
#[must_use]
pub(crate) fn blend_kernel(previous: Option<[f32; 6]>, new: [f32; 6]) -> [f32; 6] {
    let is_zero = |kernel: &[f32; 6]| kernel.iter().all(|value| *value == 0.0);
    match previous {
        Some(previous) if !is_zero(&previous) && !is_zero(&new) => {
            let mut blended = [0.0; 6];
            for (axis, value) in blended.iter_mut().enumerate() {
                *value = KERNEL_BLEND_PREVIOUS * previous[axis]
                    + (1.0 - KERNEL_BLEND_PREVIOUS) * new[axis];
            }
            blended
        }
        _ => new,
    }
}

/// Plan 40: the flicker measure `sum |k - k_previous| / sum |k|` in permille
/// over the particles with a previous kernel.
#[must_use]
pub(crate) fn kernel_change_permille(pairs: &[([f32; 6], [f32; 6])]) -> u32 {
    let mut change = 0.0_f32;
    let mut magnitude = 0.0_f32;
    for (kernel, previous) in pairs {
        for (value, before) in kernel.iter().zip(previous) {
            change += (value - before).abs();
            magnitude += value.abs();
        }
    }
    if magnitude <= 0.0 {
        0
    } else {
        (change / magnitude * 1_000.0)
            .round()
            .clamp(0.0, 1_000_000.0) as u32
    }
}

/// Plan 26: the colliders a stage frame places in the fluid: one per
/// floating box whose plan centre lies inside the fluid box, as `(centre,
/// half extents)` in metres. Pure.
pub(crate) fn colliders_for_frame(
    frame: &WaterPresentationFrameV1,
    fluid_box: &FluidBox,
) -> Vec<([f32; 3], [f32; 3])> {
    let metres = |value: i64| value as f32 / 1_000_000.0;
    frame
        .boxes
        .iter()
        .filter_map(|floating| {
            let minimum = floating.minimum_micrometres.map(metres);
            let maximum = floating.maximum_micrometres.map(metres);
            let centre = [
                (minimum[0] + maximum[0]) * 0.5,
                (minimum[1] + maximum[1]) * 0.5,
                (minimum[2] + maximum[2]) * 0.5,
            ];
            let half = [
                (maximum[0] - minimum[0]) * 0.5,
                (maximum[1] - minimum[1]) * 0.5,
                (maximum[2] - minimum[2]) * 0.5,
            ];
            (fluid_box.contains_plan(centre[0], centre[2]) && half.iter().all(|value| *value > 0.0))
                .then_some((centre, half))
        })
        .take(16)
        .collect()
}

/// Plan 26: particles strictly inside a collider shrunk by one spacing.
pub(crate) fn particles_inside(
    positions: &[[f32; 3]],
    colliders: &[([f32; 3], [f32; 3])],
) -> usize {
    positions
        .iter()
        .filter(|position| {
            colliders.iter().any(|(centre, half)| {
                (0..3).all(|axis| {
                    (position[axis] - centre[axis]).abs() < half[axis] - LANE_SPACING_METRES
                })
            })
        })
        .count()
}

/// Plan 27: neighbour counts within `radius` (capped at `255`) and the
/// sizes of the connected components over the same links (capped at
/// `65,535`), through a dense uniform grid of cell `radius` over the
/// particles' bounds (linked cells, the forward half of the `27`
/// neighbourhood so every pair is visited once). Pure.
#[cfg(test)]
pub(crate) fn neighbour_counts_and_clusters(
    positions: &[[f32; 3]],
    radius: f32,
) -> (Vec<u8>, Vec<u16>) {
    let neighbourhood = neighbourhood_of(positions, radius);
    (neighbourhood.counts, neighbourhood.clusters)
}

/// Plan 27's counts and clusters plus plan 30's kernels, from one sweep.
pub(crate) struct NeighbourhoodV1 {
    pub(crate) counts: Vec<u8>,
    pub(crate) clusters: Vec<u16>,
    /// Symmetric kernel `xx xy xz yy yz zz` in inverse metres; zero for a
    /// particle with too few neighbours (the pass's isotropic fallback).
    pub(crate) kernels: Vec<[f32; 6]>,
}

/// Weighted moment sums of one particle's neighbourhood (self excluded).
#[derive(Clone, Copy, Default)]
struct Moments {
    weight: f32,
    first: [f32; 3],
    second: [f32; 6],
}

impl Moments {
    #[inline(always)]
    fn add(&mut self, delta: [f32; 3], weight: f32) {
        self.weight += weight;
        for (first, delta) in self.first.iter_mut().zip(delta) {
            *first += weight * delta;
        }
        self.second[0] += weight * delta[0] * delta[0];
        self.second[1] += weight * delta[0] * delta[1];
        self.second[2] += weight * delta[0] * delta[2];
        self.second[3] += weight * delta[1] * delta[1];
        self.second[4] += weight * delta[1] * delta[2];
        self.second[5] += weight * delta[2] * delta[2];
    }

    /// The weighted covariance with the particle itself at weight one.
    fn covariance(&self) -> [f32; 6] {
        let total = self.weight + 1.0;
        let mean = self.first.map(|value| value / total);
        [
            self.second[0] / total - mean[0] * mean[0],
            self.second[1] / total - mean[0] * mean[1],
            self.second[2] / total - mean[0] * mean[2],
            self.second[3] / total - mean[1] * mean[1],
            self.second[4] / total - mean[1] * mean[2],
            self.second[5] / total - mean[2] * mean[2],
        ]
    }
}

/// Eigenvalues (ascending) and unit eigenvectors (columns) of a symmetric
/// `xx xy xz yy yz zz` matrix: the closed trigonometric form for the
/// values, cross products of rows of `A − λI` for the vectors (a repeated
/// value falls back to an orthonormal completion). Plan 30 revision 2:
/// the Jacobi solver kept below as the test oracle cost 300 ns per
/// particle; this form costs a third of it.
pub(crate) fn symmetric_eigen(matrix: [f32; 6]) -> ([f32; 3], [[f32; 3]; 3]) {
    let [xx, xy, xz, yy, yz, zz] = matrix;
    let off = xy * xy + xz * xz + yz * yz;
    let q = (xx + yy + zz) / 3.0;
    let p2 = (xx - q) * (xx - q) + (yy - q) * (yy - q) + (zz - q) * (zz - q) + 2.0 * off;
    let p = (p2 / 6.0).sqrt();
    if p <= 1e-6 * q.abs().max(f32::MIN_POSITIVE) {
        // Every direction is an eigenvector: an isotropic matrix.
        return (
            [q, q, q],
            [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]],
        );
    }
    let b = [
        (xx - q) / p,
        xy / p,
        xz / p,
        (yy - q) / p,
        yz / p,
        (zz - q) / p,
    ];
    let determinant = b[0] * (b[3] * b[5] - b[4] * b[4]) - b[1] * (b[1] * b[5] - b[4] * b[2])
        + b[2] * (b[1] * b[4] - b[3] * b[2]);
    let r = (determinant / 2.0).clamp(-1.0, 1.0);
    let phi = r.acos() / 3.0;
    let largest = q + 2.0 * p * phi.cos();
    let smallest = q + 2.0 * p * (phi + 2.0 * std::f32::consts::FRAC_PI_3).cos();
    let middle = 3.0 * q - largest - smallest;
    let values = [smallest, middle, largest];
    let eigenvector = |value: f32| -> Option<[f32; 3]> {
        let rows = [
            [xx - value, xy, xz],
            [xy, yy - value, yz],
            [xz, yz, zz - value],
        ];
        let cross = |a: [f32; 3], b: [f32; 3]| {
            [
                a[1] * b[2] - a[2] * b[1],
                a[2] * b[0] - a[0] * b[2],
                a[0] * b[1] - a[1] * b[0],
            ]
        };
        let candidates = [
            cross(rows[0], rows[1]),
            cross(rows[0], rows[2]),
            cross(rows[1], rows[2]),
        ];
        let mut best = candidates[0];
        let mut best_length = 0.0;
        for candidate in candidates {
            let length = candidate[0] * candidate[0]
                + candidate[1] * candidate[1]
                + candidate[2] * candidate[2];
            if length > best_length {
                best_length = length;
                best = candidate;
            }
        }
        let floor = 1e-10 * p2 * p2;
        (best_length > floor).then(|| best.map(|component| component / best_length.sqrt()))
    };
    let unit_orthogonal = |v: [f32; 3]| -> [f32; 3] {
        let helper = if v[0].abs() < 0.9 {
            [1.0, 0.0, 0.0]
        } else {
            [0.0, 1.0, 0.0]
        };
        let w = [
            v[1] * helper[2] - v[2] * helper[1],
            v[2] * helper[0] - v[0] * helper[2],
            v[0] * helper[1] - v[1] * helper[0],
        ];
        let length = (w[0] * w[0] + w[1] * w[1] + w[2] * w[2]).sqrt();
        w.map(|component| component / length)
    };
    // The vector of the value farthest from the other two is the best
    // conditioned; the second comes from the other extreme or a completion.
    let (first, second) = if largest - middle >= middle - smallest {
        let v_largest = eigenvector(largest).unwrap_or([0.0, 0.0, 1.0]);
        let v_smallest = eigenvector(smallest).unwrap_or_else(|| unit_orthogonal(v_largest));
        (v_smallest, v_largest)
    } else {
        let v_smallest = eigenvector(smallest).unwrap_or([1.0, 0.0, 0.0]);
        let v_largest = eigenvector(largest).unwrap_or_else(|| unit_orthogonal(v_smallest));
        (v_smallest, v_largest)
    };
    // Near-repeated values leave the two extremes ill conditioned; the
    // second is re-orthogonalised against the first (Gram–Schmidt) and a
    // collapsed remainder falls back to a completion.
    let projection = first[0] * second[0] + first[1] * second[1] + first[2] * second[2];
    let mut second = [
        second[0] - projection * first[0],
        second[1] - projection * first[1],
        second[2] - projection * first[2],
    ];
    let second_length =
        (second[0] * second[0] + second[1] * second[1] + second[2] * second[2]).sqrt();
    if second_length > 1e-3 {
        second = second.map(|component| component / second_length);
    } else {
        second = unit_orthogonal(first);
    }
    let v_middle = [
        second[1] * first[2] - second[2] * first[1],
        second[2] * first[0] - second[0] * first[2],
        second[0] * first[1] - second[1] * first[0],
    ];
    let mut vectors = [[0.0; 3]; 3];
    for row in 0..3 {
        vectors[row][0] = first[row];
        vectors[row][1] = v_middle[row];
        vectors[row][2] = second[row];
    }
    (values, vectors)
}

/// The cyclic Jacobi solver, the test oracle for `symmetric_eigen`.
#[cfg(test)]
pub(crate) fn symmetric_eigen_jacobi(matrix: [f32; 6]) -> ([f32; 3], [[f32; 3]; 3]) {
    let mut a = [
        [matrix[0], matrix[1], matrix[2]],
        [matrix[1], matrix[3], matrix[4]],
        [matrix[2], matrix[4], matrix[5]],
    ];
    let mut v = [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]];
    // Jacobi converges quadratically: a relative off-diagonal floor stops
    // the sweeps after a few rotations instead of running them all.
    let scale = (matrix[0] * matrix[0] + matrix[3] * matrix[3] + matrix[5] * matrix[5])
        .max(f32::MIN_POSITIVE);
    for _ in 0..16 {
        let off = a[0][1] * a[0][1] + a[0][2] * a[0][2] + a[1][2] * a[1][2];
        if off <= 1e-12 * scale {
            break;
        }
        for (p, q) in [(0, 1), (0, 2), (1, 2)] {
            if a[p][q].abs() < 1e-20 {
                continue;
            }
            let theta = (a[q][q] - a[p][p]) / (2.0 * a[p][q]);
            let t = theta.signum() / (theta.abs() + (theta * theta + 1.0).sqrt());
            let c = 1.0 / (t * t + 1.0).sqrt();
            let s = t * c;
            for row in &mut a {
                let akp = row[p];
                let akq = row[q];
                row[p] = c * akp - s * akq;
                row[q] = s * akp + c * akq;
            }
            let (row_p, row_q) = (a[p], a[q]);
            a[p] = std::array::from_fn(|k| c * row_p[k] - s * row_q[k]);
            a[q] = std::array::from_fn(|k| s * row_p[k] + c * row_q[k]);
            for row in &mut v {
                let vp = row[p];
                let vq = row[q];
                row[p] = c * vp - s * vq;
                row[q] = s * vp + c * vq;
            }
        }
    }
    let mut order = [0, 1, 2];
    order.sort_by(|x, y| a[*x][*x].total_cmp(&a[*y][*y]));
    let values = order.map(|index| a[index][index]);
    let mut vectors = [[0.0; 3]; 3];
    for (column, index) in order.iter().enumerate() {
        for row in 0..3 {
            vectors[row][column] = v[row][*index];
        }
    }
    (values, vectors)
}

/// Plan 30: the kernel `G = R·diag(1/r)·Rᵀ` from a neighbourhood's
/// covariance, semi-axes clamped as frozen in the plan.
fn kernel_of(covariance: [f32; 6], profile_radius: f32) -> [f32; 6] {
    let (values, vectors) = symmetric_eigen(covariance);
    let mut radii = values.map(|value| KERNEL_SCALE * value.max(0.0).sqrt());
    let largest = radii[2].max(1e-6);
    for radius in &mut radii {
        *radius = (*radius).max(largest / KERNEL_MAX_RATIO).clamp(
            KERNEL_MIN_AXIS_SCALE * profile_radius,
            KERNEL_MAX_AXIS_SCALE * profile_radius,
        );
    }
    let mut kernel = [0.0; 6];
    for axis in 0..3 {
        let inverse = 1.0 / radii[axis];
        let v = [vectors[0][axis], vectors[1][axis], vectors[2][axis]];
        kernel[0] += inverse * v[0] * v[0];
        kernel[1] += inverse * v[0] * v[1];
        kernel[2] += inverse * v[0] * v[2];
        kernel[3] += inverse * v[1] * v[1];
        kernel[4] += inverse * v[1] * v[2];
        kernel[5] += inverse * v[2] * v[2];
    }
    kernel
}

/// Threads of the neighbourhood sweep (plan 30 revision 2), bounded so
/// the frame's other work keeps its cores.
const NEIGHBOURHOOD_THREADS: usize = 4;

pub(crate) fn neighbourhood_of(positions: &[[f32; 3]], radius: f32) -> NeighbourhoodV1 {
    let count = positions.len();
    if count == 0 {
        return NeighbourhoodV1 {
            counts: Vec::new(),
            clusters: Vec::new(),
            kernels: Vec::new(),
        };
    }
    // Plan 30 revision 2: particles are sorted by cell (counting sort) and
    // every worker sweeps its own contiguous range of particles over the
    // full 27-cell neighbourhood, writing only its own particle's count,
    // moments and kernel; the pairs `a < b` it finds are edges for the
    // sequential union-find of the clusters afterwards. Results are
    // scattered back to the input order at the end.
    let mut minimum = [f32::INFINITY; 3];
    for position in positions {
        for axis in 0..3 {
            minimum[axis] = minimum[axis].min(position[axis]);
        }
    }
    let mut dims = [1_usize; 3];
    for position in positions {
        for axis in 0..3 {
            let cell = ((position[axis] - minimum[axis]) / radius) as usize + 1;
            dims[axis] = dims[axis].max(cell + 1);
        }
    }
    let cell_of = |position: &[f32; 3]| -> [usize; 3] {
        [
            ((position[0] - minimum[0]) / radius) as usize,
            ((position[1] - minimum[1]) / radius) as usize,
            ((position[2] - minimum[2]) / radius) as usize,
        ]
    };
    let index_of = |cell: [usize; 3]| (cell[2] * dims[1] + cell[1]) * dims[0] + cell[0];
    let cell_count = dims[0] * dims[1] * dims[2];
    let mut cell_index = vec![0_usize; count];
    let mut starts = vec![0_usize; cell_count + 1];
    for (index, position) in positions.iter().enumerate() {
        let slot = index_of(cell_of(position));
        cell_index[index] = slot;
        starts[slot + 1] += 1;
    }
    for slot in 0..cell_count {
        starts[slot + 1] += starts[slot];
    }
    let mut fill = starts.clone();
    let mut order = vec![0_u32; count];
    let mut sorted_cell = vec![0_usize; count];
    let mut xs = vec![0.0_f32; count];
    let mut ys = vec![0.0_f32; count];
    let mut zs = vec![0.0_f32; count];
    for (index, position) in positions.iter().enumerate() {
        let slot = cell_index[index];
        let sorted = fill[slot];
        order[sorted] = index as u32;
        sorted_cell[sorted] = slot;
        xs[sorted] = position[0];
        ys[sorted] = position[1];
        zs[sorted] = position[2];
        fill[slot] += 1;
    }
    let radius_squared = radius * radius;
    let inverse_radius = 1.0 / radius;
    let (xs, ys, zs, starts, sorted_cell) = (&xs, &ys, &zs, &starts, &sorted_cell);
    let dims = &dims;
    // One worker's sweep over `range` of sorted particles.
    let sweep = |range: std::ops::Range<usize>,
                 counts: &mut [u32],
                 kernels: &mut [[f32; 6]],
                 edges: &mut Vec<(u32, u32)>| {
        const BLOCK: usize = 64;
        for (local, a) in range.enumerate() {
            let (ax, ay, az) = (xs[a], ys[a], zs[a]);
            let slot = sorted_cell[a];
            let cell = [
                slot % dims[0],
                (slot / dims[0]) % dims[1],
                slot / (dims[0] * dims[1]),
            ];
            let mut a_count = 0_u32;
            let mut a_moments = Moments::default();
            for dz in -1_isize..=1 {
                for dy in -1_isize..=1 {
                    for dx in -1_isize..=1 {
                        let neighbour = [
                            cell[0] as isize + dx,
                            cell[1] as isize + dy,
                            cell[2] as isize + dz,
                        ];
                        if neighbour[0] < 0
                            || neighbour[1] < 0
                            || neighbour[2] < 0
                            || neighbour[0] as usize >= dims[0]
                            || neighbour[1] as usize >= dims[1]
                            || neighbour[2] as usize >= dims[2]
                        {
                            continue;
                        }
                        let other = index_of([
                            neighbour[0] as usize,
                            neighbour[1] as usize,
                            neighbour[2] as usize,
                        ]);
                        let (mut b_begin, run_end) = (starts[other], starts[other + 1]);
                        while b_begin < run_end {
                            let length = (run_end - b_begin).min(BLOCK);
                            let mut distances = [0.0_f32; BLOCK];
                            let (bx, by, bz) = (
                                &xs[b_begin..b_begin + length],
                                &ys[b_begin..b_begin + length],
                                &zs[b_begin..b_begin + length],
                            );
                            for (distance, ((x, y), z)) in distances[..length]
                                .iter_mut()
                                .zip(bx.iter().zip(by).zip(bz))
                            {
                                let dx = x - ax;
                                let dy = y - ay;
                                let dz = z - az;
                                *distance = dx * dx + dy * dy + dz * dz;
                            }
                            for (k, distance_squared) in distances[..length].iter().enumerate() {
                                let b = b_begin + k;
                                if b == a || *distance_squared > radius_squared {
                                    continue;
                                }
                                a_count += 1;
                                let ratio = distance_squared.sqrt() * inverse_radius;
                                let weight = 1.0 - ratio * ratio * ratio;
                                a_moments.add([bx[k] - ax, by[k] - ay, bz[k] - az], weight);
                                if b > a {
                                    edges.push((a as u32, b as u32));
                                }
                            }
                            b_begin += length;
                        }
                    }
                }
            }
            counts[local] = a_count;
            if a_count >= KERNEL_MIN_NEIGHBOURS {
                kernels[local] = kernel_of(a_moments.covariance(), KERNEL_PROFILE_RADIUS_METRES);
            }
        }
    };
    let mut counts = vec![0_u32; count];
    let mut kernels = vec![[0.0_f32; 6]; count];
    let threads = std::thread::available_parallelism()
        .map_or(1, std::num::NonZero::get)
        .clamp(1, NEIGHBOURHOOD_THREADS);
    let chunk = count.div_ceil(threads).max(1);
    let mut edge_lists: Vec<Vec<(u32, u32)>> = Vec::new();
    if threads == 1 || count < 2 * chunk {
        let mut edges = Vec::new();
        sweep(0..count, &mut counts, &mut kernels, &mut edges);
        edge_lists.push(edges);
    } else {
        std::thread::scope(|scope| {
            let handles: Vec<_> = counts
                .chunks_mut(chunk)
                .zip(kernels.chunks_mut(chunk))
                .enumerate()
                .map(|(worker, (counts, kernels))| {
                    let sweep = &sweep;
                    scope.spawn(move || {
                        let begin = worker * chunk;
                        let mut edges = Vec::new();
                        sweep(begin..begin + counts.len(), counts, kernels, &mut edges);
                        edges
                    })
                })
                .collect();
            for handle in handles {
                edge_lists.push(handle.join().expect("neighbourhood worker panicked"));
            }
        });
    }
    let mut parent: Vec<u32> = (0..count as u32).collect();
    fn find(parent: &mut [u32], mut index: u32) -> u32 {
        while parent[index as usize] != index {
            parent[index as usize] = parent[parent[index as usize] as usize];
            index = parent[index as usize];
        }
        index
    }
    for (a, b) in edge_lists.iter().flatten() {
        let root_a = find(&mut parent, *a);
        let root_b = find(&mut parent, *b);
        if root_a != root_b {
            parent[root_a as usize] = root_b;
        }
    }
    let mut sizes = vec![0_u32; count];
    let roots: Vec<u32> = (0..count as u32)
        .map(|index| find(&mut parent, index))
        .collect();
    for root in &roots {
        sizes[*root as usize] += 1;
    }
    let mut out_counts = vec![0_u8; count];
    let mut out_clusters = vec![0_u16; count];
    let mut out_kernels = vec![[0.0_f32; 6]; count];
    for (sorted, original) in order.iter().enumerate() {
        let original = *original as usize;
        out_counts[original] = u8::try_from(counts[sorted]).unwrap_or(u8::MAX);
        out_clusters[original] = u16::try_from(sizes[roots[sorted] as usize]).unwrap_or(u16::MAX);
        out_kernels[original] = kernels[sorted];
    }
    NeighbourhoodV1 {
        counts: out_counts,
        clusters: out_clusters,
        kernels: out_kernels,
    }
}

/// Plan 25 statistics, printed at session end.
#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct LaneStats {
    pub(crate) frames: u64,
    pub(crate) peak_particles: usize,
    pub(crate) emitted: u64,
    pub(crate) absorbed: u64,
    pub(crate) last_particles: usize,
    pub(crate) cost_total_microseconds: u128,
    pub(crate) cost_max_microseconds: u128,
    /// Plan 26: the largest count of particles inside a collider.
    pub(crate) inside_colliders_max: usize,
    /// Plan 27: the host analysis per frame and the spray fraction.
    pub(crate) analysis_total_microseconds: u128,
    pub(crate) analysis_max_microseconds: u128,
    pub(crate) spray_fraction_max_permille: u32,
    pub(crate) spray_fraction_last_permille: u32,
    /// Plan 30: particles sent with a non-zero kernel in the last frame.
    pub(crate) kernels_last_permille: u32,
    /// Plan 40: particles blended with a previous kernel, the flicker
    /// measure, and the fluid's recreations after failures.
    pub(crate) kernels_blended_permille: u32,
    pub(crate) kernel_change_permille: u32,
    pub(crate) recoveries: u32,
}

/// One frame's fluid state after the step, absorption and emission.
struct FluidAdvance {
    positions: Vec<[f32; 3]>,
    velocities: Vec<[f32; 3]>,
    absorbed: usize,
    emitted: usize,
}

/// Plan 40: the fluid's creation inputs, kept for a recreation.
#[derive(Clone)]
struct FluidRecipe {
    fluid_box: FluidBox,
    gpu_library_path: String,
}

pub(crate) struct PhysxWaterLane {
    /// `None` after a failure (plan 29): the lane is demoted until plan 40's
    /// recreation succeeds, and the stage's droplets show meanwhile.
    fluid: Option<NativeFluid>,
    failure: Option<String>,
    inject_failure_after: Option<u64>,
    /// Plan 40: identity per particle slot and the previous kernels by id.
    ids: Vec<u32>,
    next_id: u32,
    previous_kernels: std::collections::HashMap<u32, [f32; 6]>,
    recipe: FluidRecipe,
    frames_demoted: u64,
    recreation_attempts: u32,
    fluid_box: FluidBox,
    level_metres: f32,
    accumulator: Duration,
    bounds: AabbI64V1,
    last_frame_index: Option<u64>,
    pending_positions: Vec<[f32; 3]>,
    pending_velocities: Vec<[f32; 3]>,
    colliders: Vec<([f32; 3], [f32; 3])>,
    collider_slots: u32,
    stats: LaneStats,
}

impl PhysxWaterLane {
    /// Creates the empty fluid over the basin (plus plan 24's block when
    /// `pour` is set); `Err` carries the reason the lane is unavailable.
    pub(crate) fn new(
        bounds: AabbI64V1,
        pour: bool,
        inject_failure_after: Option<u64>,
    ) -> Result<Self, String> {
        let fluid_box = basin_fluid_box();
        let level_metres = fluid_box.min[1];
        let centre_x = (fluid_box.min[0] + fluid_box.max[0]) * 0.5;
        let centre_z = (fluid_box.min[2] + fluid_box.max[2]) * 0.5;
        let (seed_min, seed_max) = if pour {
            (
                [
                    centre_x - POUR_BLOCK_HALF_METRES,
                    level_metres + POUR_BLOCK_BOTTOM_METRES,
                    centre_z - POUR_BLOCK_HALF_METRES,
                ],
                [
                    centre_x + POUR_BLOCK_HALF_METRES,
                    level_metres + POUR_BLOCK_BOTTOM_METRES + POUR_BLOCK_HEIGHT_METRES,
                    centre_z + POUR_BLOCK_HALF_METRES,
                ],
            )
        } else {
            (
                [centre_x, level_metres, centre_z],
                [centre_x, level_metres, centre_z],
            )
        };
        let path = gpu_library_path().ok_or_else(|| "PhysX GPU library not found".to_owned())?;
        let recipe = FluidRecipe {
            fluid_box,
            gpu_library_path: path.display().to_string(),
        };
        let mut fluid = Self::create_fluid(&recipe, seed_min, seed_max)?;
        let seeded = fluid
            .read()
            .map(|sample| sample.positions.len())
            .unwrap_or(0);
        Ok(Self {
            fluid: Some(fluid),
            failure: None,
            inject_failure_after,
            ids: (0..seeded as u32).collect(),
            next_id: seeded as u32,
            previous_kernels: std::collections::HashMap::new(),
            recipe,
            frames_demoted: 0,
            recreation_attempts: 0,
            fluid_box,
            level_metres,
            accumulator: Duration::ZERO,
            bounds,
            last_frame_index: None,
            pending_positions: Vec::new(),
            pending_velocities: Vec::new(),
            colliders: Vec::new(),
            collider_slots: 0,
            stats: LaneStats::default(),
        })
    }

    /// Creates a fluid from the recipe, seeded inside `seed_min..seed_max`
    /// (an empty seed for an empty fluid).
    fn create_fluid(
        recipe: &FluidRecipe,
        seed_min: [f32; 3],
        seed_max: [f32; 3],
    ) -> Result<NativeFluid, String> {
        NativeFluid::create_reporting(&FluidDesc {
            spacing_metres: LANE_SPACING_METRES,
            box_min_metres: recipe.fluid_box.min,
            box_max_metres: recipe.fluid_box.max,
            seed_min_metres: seed_min,
            seed_max_metres: seed_max,
            timestep_seconds: LANE_TIMESTEP.as_secs_f32(),
            max_particles: LANE_MAX_PARTICLES,
            gpu_library_path: Some(recipe.gpu_library_path.clone()),
        })
        .map_err(|(error, reason)| format!("PhysX fluid unavailable: {error} ({reason})"))
    }

    /// Plan 40: after the pause, recreates the fluid empty; a failure keeps
    /// the lane demoted until the next attempt or for good.
    fn try_recreate(&mut self) {
        self.frames_demoted += 1;
        if self.frames_demoted < RECREATE_AFTER_FRAMES
            || self.recreation_attempts >= RECREATE_MAX_ATTEMPTS
        {
            return;
        }
        self.frames_demoted = 0;
        self.recreation_attempts += 1;
        let centre = [
            (self.recipe.fluid_box.min[0] + self.recipe.fluid_box.max[0]) * 0.5,
            self.recipe.fluid_box.min[1],
            (self.recipe.fluid_box.min[2] + self.recipe.fluid_box.max[2]) * 0.5,
        ];
        match Self::create_fluid(&self.recipe, centre, centre) {
            Ok(fluid) => {
                eprintln!(
                    "next_game: PHYSX_WATER_RECREATED after attempt {}",
                    self.recreation_attempts
                );
                self.fluid = Some(fluid);
                self.failure = None;
                self.ids.clear();
                self.previous_kernels.clear();
                self.accumulator = Duration::ZERO;
                self.collider_slots = 0;
                self.stats.recoveries += 1;
            }
            Err(reason) => {
                eprintln!(
                    "next_game: PHYSX_WATER_RECREATION_FAILED attempt {}: {reason}",
                    self.recreation_attempts
                );
                self.failure = Some(format!("recreation failed: {reason}"));
            }
        }
    }

    pub(crate) fn particle_surface_profile(&self) -> ParticleSurfaceProfileV1 {
        ParticleSurfaceProfileV1 {
            particle_capacity: LANE_MAX_PARTICLES,
            radius_micrometres: LANE_RADIUS_MICROMETRES,
            bounds: self.bounds,
            absorption_per_metre: [1.2, 0.5, 0.25],
            refraction_strength: 0.08,
            thickness_scale: 1.0,
            spray_neighbour_threshold: SPRAY_NEIGHBOUR_THRESHOLD,
            spray_radius_micrometres: SPRAY_RADIUS_MICROMETRES,
            spray_alpha: SPRAY_ALPHA,
            spray_cluster_threshold: SPRAY_CLUSTER_THRESHOLD,
            spray_subdroplets: SPRAY_SUBDROPLETS,
            spray_streak_seconds: SPRAY_STREAK_SECONDS,
            bulk_neighbour_count: BULK_NEIGHBOUR_COUNT,
            edge_radius_scale: EDGE_RADIUS_SCALE,
            cleanup_radius_pixels: 4,
        }
    }

    /// Queues the emission of a newly published stage frame.
    pub(crate) fn observe_frame(&mut self, frame: &WaterPresentationFrameV1) {
        if self.last_frame_index == Some(frame.frame_index) {
            return;
        }
        self.last_frame_index = Some(frame.frame_index);
        let room = LANE_MAX_PARTICLES as usize;
        let (positions, velocities) =
            emission_for_frame(frame, &self.fluid_box, self.level_metres, room);
        self.pending_positions.extend(positions);
        self.pending_velocities.extend(velocities);
        // Plan 26: the frame's boxes as kinematic colliders; a vanished box
        // clears its slot. A failed collider update is reported once by the
        // adapter's client error at the next step.
        self.colliders = colliders_for_frame(frame, &self.fluid_box);
        let Some(fluid) = self.fluid.as_mut() else {
            return;
        };
        let used = u32::try_from(self.colliders.len()).unwrap_or(0);
        for (slot, (centre, half)) in self.colliders.iter().enumerate() {
            let slot = u32::try_from(slot).unwrap_or(u32::MAX);
            if fluid.set_box(slot, *centre, *half).is_err() {
                break;
            }
        }
        for slot in used..self.collider_slots {
            let _ = fluid.clear_box(slot);
        }
        self.collider_slots = used;
    }

    /// Steps the fluid by the frame's elapsed time, absorbs settled
    /// particles, injects the pending emission and returns the particle
    /// update for this frame.
    pub(crate) fn advance(
        &mut self,
        elapsed: Duration,
        sequence: u64,
    ) -> Result<Option<Arc<ParticleSurfaceUpdateV1>>, DesktopAdapterError> {
        // Plan 29: a demoted lane answers with no update; a failure of the
        // fluid demotes it and clears the picture once.
        if self.fluid.is_none() {
            self.try_recreate();
            return Ok(None);
        }
        let started = Instant::now();
        if self.inject_failure_after == Some(self.stats.frames) {
            self.inject_failure_after = None;
            return self.demote(
                "PHYSX_WATER_INJECTED_FAILURE",
                &format!("injected after {} lane frames", self.stats.frames),
                sequence,
            );
        }
        let FluidAdvance {
            positions,
            velocities,
            absorbed,
            emitted,
        } = match self.advance_fluid(elapsed) {
            Ok(result) => result,
            Err((code, message)) => return self.demote(code, &message, sequence),
        };
        self.stats.frames += 1;
        self.stats.emitted += emitted as u64;
        self.stats.absorbed += absorbed as u64;
        self.stats.peak_particles = self.stats.peak_particles.max(positions.len());
        self.stats.last_particles = positions.len();
        self.stats.inside_colliders_max = self
            .stats
            .inside_colliders_max
            .max(particles_inside(&positions, &self.colliders));
        let cost = started.elapsed().as_micros();
        self.stats.cost_total_microseconds += cost;
        self.stats.cost_max_microseconds = self.stats.cost_max_microseconds.max(cost);
        // Droplets that left the declared bounds (the pass rejects any
        // position outside them, exclusive at the maximum) are dropped for
        // this frame's picture.
        // Plan 27: neighbours and clusters from the fluid's own density.
        let analysis_started = Instant::now();
        let NeighbourhoodV1 {
            counts: neighbours,
            clusters,
            kernels: raw_kernels,
        } = neighbourhood_of(&positions, NEIGHBOUR_RADIUS_METRES);
        // Plan 40: every kernel blended with the same particle's previous
        // one; the flicker measure over the blended pairs.
        let mut kernels = Vec::with_capacity(raw_kernels.len());
        let mut pairs = Vec::new();
        let mut next_previous = std::collections::HashMap::with_capacity(raw_kernels.len());
        for (index, raw) in raw_kernels.iter().enumerate() {
            let id = self.ids.get(index).copied().unwrap_or(u32::MAX);
            let previous = self.previous_kernels.get(&id).copied();
            let blended = blend_kernel(previous, *raw);
            if let Some(previous) = previous
                && previous.iter().any(|value| *value != 0.0)
                && blended.iter().any(|value| *value != 0.0)
            {
                pairs.push((blended, previous));
            }
            if blended.iter().any(|value| *value != 0.0) {
                next_previous.insert(id, blended);
            }
            kernels.push(blended);
        }
        self.previous_kernels = next_previous;
        self.stats.kernels_blended_permille = if positions.is_empty() {
            0
        } else {
            u32::try_from(pairs.len() * 1_000 / positions.len()).unwrap_or(u32::MAX)
        };
        self.stats.kernel_change_permille = kernel_change_permille(&pairs);
        let analysis_cost = analysis_started.elapsed().as_micros();
        self.stats.analysis_total_microseconds += analysis_cost;
        self.stats.analysis_max_microseconds =
            self.stats.analysis_max_microseconds.max(analysis_cost);
        if !positions.is_empty() {
            let spray = neighbours
                .iter()
                .zip(&clusters)
                .filter(|(count, cluster)| {
                    u32::from(**count) < SPRAY_NEIGHBOUR_THRESHOLD
                        || u32::from(**cluster) < SPRAY_CLUSTER_THRESHOLD
                })
                .count();
            let permille = u32::try_from(spray * 1_000 / positions.len()).unwrap_or(u32::MAX);
            self.stats.spray_fraction_last_permille = permille;
            self.stats.spray_fraction_max_permille =
                self.stats.spray_fraction_max_permille.max(permille);
        }
        let micrometres = |value: f32| (value * 1_000_000.0).round() as i64;
        let mut update_positions = Vec::with_capacity(positions.len());
        let mut update_velocities = Vec::with_capacity(positions.len());
        let mut update_neighbours = Vec::with_capacity(positions.len());
        let mut update_clusters = Vec::with_capacity(positions.len());
        let mut update_kernels = Vec::with_capacity(positions.len());
        for (index, (position, velocity)) in positions.iter().zip(&velocities).enumerate() {
            let point = position.map(micrometres);
            if !self.bounds.contains(point) {
                continue;
            }
            update_positions.push(point);
            update_velocities
                .push(velocity.map(|value| (value * 1_000_000.0).clamp(-2.0e9, 2.0e9) as i32));
            update_neighbours.push(neighbours[index]);
            update_clusters.push(clusters[index]);
            update_kernels.push(kernels[index]);
        }
        if update_positions.is_empty() {
            return Ok(None);
        }
        let with_kernel = update_kernels
            .iter()
            .filter(|kernel| kernel.iter().any(|value| *value != 0.0))
            .count();
        self.stats.kernels_last_permille =
            u32::try_from(with_kernel * 1_000 / update_positions.len()).unwrap_or(u32::MAX);
        Ok(Some(Arc::new(ParticleSurfaceUpdateV1::new(
            sequence,
            update_positions,
            update_neighbours,
            update_clusters,
            update_velocities,
            update_kernels,
        )?)))
    }

    /// Steps, reads and refills the fluid; `Err` carries the failed
    /// operation's code and message.
    fn advance_fluid(&mut self, elapsed: Duration) -> Result<FluidAdvance, (&'static str, String)> {
        let fluid = self
            .fluid
            .as_mut()
            .ok_or(("PHYSX_WATER_DEMOTED", String::new()))?;
        self.accumulator = self.accumulator.saturating_add(elapsed);
        let mut steps = 0;
        while self.accumulator >= LANE_TIMESTEP && steps < LANE_MAX_STEPS_PER_FRAME {
            self.accumulator -= LANE_TIMESTEP;
            fluid
                .step()
                .map_err(|error| ("PHYSX_WATER_STEP_FAILED", error.to_string()))?;
            steps += 1;
        }
        if steps == LANE_MAX_STEPS_PER_FRAME {
            self.accumulator = Duration::ZERO;
        }
        let sample = fluid
            .read()
            .map_err(|error| ("PHYSX_WATER_READ_FAILED", error.to_string()))?;
        let (mut positions, mut velocities, absorbed, kept) = absorb(&sample, self.level_metres);
        // Plan 40: the kept slots keep their ids; the emitted get new ones.
        if self.ids.len() != sample.positions.len() {
            self.ids = (0..sample.positions.len())
                .map(|_| {
                    self.next_id = self.next_id.wrapping_add(1);
                    self.next_id
                })
                .collect();
        }
        let mut ids: Vec<u32> = kept.iter().map(|index| self.ids[*index]).collect();
        let room = (fluid.max_particles() as usize).saturating_sub(positions.len());
        let emitted = self.pending_positions.len().min(room);
        positions.extend(self.pending_positions.drain(..emitted));
        velocities.extend(self.pending_velocities.drain(..emitted));
        for _ in 0..emitted {
            self.next_id = self.next_id.wrapping_add(1);
            ids.push(self.next_id);
        }
        self.ids = ids;
        self.pending_positions.clear();
        self.pending_velocities.clear();
        if absorbed > 0 || emitted > 0 {
            fluid
                .set(&positions, &velocities)
                .map_err(|error| ("PHYSX_WATER_SET_FAILED", error.to_string()))?;
        }
        Ok(FluidAdvance {
            positions,
            velocities,
            absorbed,
            emitted,
        })
    }

    /// Plan 29: records the failure, frees the fluid and publishes one
    /// empty update so the last picture does not freeze on the screen.
    fn demote(
        &mut self,
        code: &str,
        message: &str,
        sequence: u64,
    ) -> Result<Option<Arc<ParticleSurfaceUpdateV1>>, DesktopAdapterError> {
        let reason = format!("{code}: {message}");
        eprintln!(
            "next_game: PHYSX_WATER_DEMOTED after {} frames: {reason}",
            self.stats.frames
        );
        self.failure = Some(reason);
        self.fluid = None;
        self.pending_positions.clear();
        self.pending_velocities.clear();
        Ok(Some(Arc::new(ParticleSurfaceUpdateV1::new(
            sequence,
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
        )?)))
    }

    pub(crate) fn stats(&self) -> &LaneStats {
        &self.stats
    }

    /// Plan 29: the reason the lane was demoted, if it was.
    pub(crate) fn failure(&self) -> Option<&str> {
        self.failure.as_deref()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use next_reference_game::{WaterEdgePresentationV1, WaterFloatingBoxV1, WaterJetParticlesV1};

    /// Semi-axes of a packed kernel, ascending (metres).
    fn kernel_axes(kernel: [f32; 6]) -> [f32; 3] {
        let (values, _) = symmetric_eigen(kernel);
        let mut axes = values.map(|value| 1.0 / value);
        axes.sort_by(f32::total_cmp);
        axes
    }

    #[test]
    fn jacobi_solver_recovers_diagonal_and_rotated_eigenvalues() {
        let (values, vectors) = symmetric_eigen([3.0, 0.0, 0.0, 1.0, 0.0, 2.0]);
        assert!((values[0] - 1.0).abs() < 1e-5 && (values[2] - 3.0).abs() < 1e-5);
        assert!(
            (vectors[1][0].abs() - 1.0).abs() < 1e-5,
            "smallest axis is y"
        );
        // diag(1, 2, 3) rotated by 30 degrees about z.
        let (c, s) = (30_f32.to_radians().cos(), 30_f32.to_radians().sin());
        let xx = c * c * 1.0 + s * s * 2.0;
        let yy = s * s * 1.0 + c * c * 2.0;
        let xy = c * s * (1.0 - 2.0);
        let (values, _) = symmetric_eigen([xx, xy, 0.0, yy, 0.0, 3.0]);
        for (value, expected) in values.iter().zip([1.0, 2.0, 3.0]) {
            assert!((value - expected).abs() < 1e-5, "{values:?}");
        }
    }

    /// Plan 30 timing probe at the lane's density (5,000 particles on a
    /// jittered 0.05 m lattice); run with `--ignored --nocapture`.
    #[test]
    #[ignore = "timing probe"]
    fn bench_neighbourhood() {
        let mut state = 0x9E37_79B9_u32;
        let mut next = move || {
            state ^= state << 13;
            state ^= state >> 17;
            state ^= state << 5;
            (state % 10_000) as f32 / 10_000.0
        };
        let mut positions = Vec::new();
        for x in 0..17 {
            for y in 0..17 {
                for z in 0..17 {
                    if positions.len() < 5_000 {
                        positions.push([
                            x as f32 * 0.05 + next() * 0.01,
                            y as f32 * 0.05 + next() * 0.01,
                            z as f32 * 0.05 + next() * 0.01,
                        ]);
                    }
                }
            }
        }
        for _ in 0..3 {
            let started = Instant::now();
            let neighbourhood = neighbourhood_of(&positions, NEIGHBOUR_RADIUS_METRES);
            let sweep = started.elapsed();
            let mean = neighbourhood
                .counts
                .iter()
                .map(|c| f32::from(*c))
                .sum::<f32>()
                / positions.len() as f32;
            let started = Instant::now();
            let mut sum = 0.0_f32;
            for m in 0..5_000 {
                let (values, _) =
                    symmetric_eigen([1e-3 + m as f32 * 1e-7, 1e-4, 2e-4, 2e-3, 1e-4, 1.5e-3]);
                sum += values[0];
            }
            let eigen = started.elapsed();
            eprintln!("sweep {sweep:?} mean_neighbours {mean} eigen5000 {eigen:?} ({sum})");
        }
    }

    #[test]
    fn closed_form_eigen_matches_the_jacobi_oracle() {
        let mut state = 0x2545_F491_u32;
        let mut next = move || {
            state ^= state << 13;
            state ^= state >> 17;
            state ^= state << 5;
            (state % 10_000) as f32 / 10_000.0 - 0.5
        };
        for case in 0..2_000 {
            // Random covariance-like matrices at the lane's scale, plus a
            // few degenerate ones.
            let m = match case % 4 {
                0 => [2e-3, 0.0, 0.0, 2e-3, 0.0, 2e-3],
                1 => [2e-3, 0.0, 0.0, 2e-3, 0.0, 5e-4],
                _ => {
                    let a = [next() * 1e-3, next() * 1e-3, next() * 1e-3];
                    let b = [next() * 1e-3, next() * 1e-3, next() * 1e-3];
                    let c = [next() * 1e-3, next() * 1e-3, next() * 1e-3];
                    let dot = |u: [f32; 3], v: [f32; 3]| u[0] * v[0] + u[1] * v[1] + u[2] * v[2];
                    let row = |i: usize| [a[i], b[i], c[i]];
                    [
                        dot(row(0), row(0)),
                        dot(row(0), row(1)),
                        dot(row(0), row(2)),
                        dot(row(1), row(1)),
                        dot(row(1), row(2)),
                        dot(row(2), row(2)),
                    ]
                }
            };
            let (values, vectors) = symmetric_eigen(m);
            let (oracle, _) = symmetric_eigen_jacobi(m);
            let scale = oracle[2].abs().max(1e-9);
            for axis in 0..3 {
                assert!(
                    (values[axis] - oracle[axis]).abs() <= 1e-3 * scale,
                    "case {case}: {values:?} vs {oracle:?}"
                );
                // A·v = λ·v and the columns are orthonormal.
                let v = [vectors[0][axis], vectors[1][axis], vectors[2][axis]];
                let av = [
                    m[0] * v[0] + m[1] * v[1] + m[2] * v[2],
                    m[1] * v[0] + m[3] * v[1] + m[4] * v[2],
                    m[2] * v[0] + m[4] * v[1] + m[5] * v[2],
                ];
                for k in 0..3 {
                    assert!(
                        (av[k] - values[axis] * v[k]).abs() <= 1e-3 * scale,
                        "case {case} axis {axis}: {av:?} vs {values:?} {v:?}"
                    );
                }
                let length = v[0] * v[0] + v[1] * v[1] + v[2] * v[2];
                assert!((length - 1.0).abs() < 1e-4, "case {case}: {vectors:?}");
                let other = (axis + 1) % 3;
                let w = [vectors[0][other], vectors[1][other], vectors[2][other]];
                assert!(
                    (v[0] * w[0] + v[1] * w[1] + v[2] * w[2]).abs() < 1e-3,
                    "case {case}: {vectors:?}"
                );
            }
        }
    }

    #[test]
    fn kernels_blend_with_their_previous_and_the_flicker_halves() {
        let a = [10.0, 0.0, 0.0, 12.0, 0.0, 11.0];
        let b = [12.0, 0.0, 0.0, 10.0, 0.0, 13.0];
        assert_eq!(blend_kernel(None, a), a);
        assert_eq!(blend_kernel(Some(a), [0.0; 6]), [0.0; 6]);
        let mixed = blend_kernel(Some(a), b);
        assert!((mixed[0] - 10.6).abs() < 1e-5 && (mixed[3] - 11.4).abs() < 1e-5);
        let mut raw_pairs = Vec::new();
        let mut blended_pairs = Vec::new();
        let mut previous = a;
        let mut blended_previous = a;
        for frame in 0..20 {
            let raw = if frame % 2 == 0 { b } else { a };
            raw_pairs.push((raw, previous));
            let blended = blend_kernel(Some(blended_previous), raw);
            blended_pairs.push((blended, blended_previous));
            previous = raw;
            blended_previous = blended;
        }
        let raw_change = kernel_change_permille(&raw_pairs);
        let blended_change = kernel_change_permille(&blended_pairs);
        assert!(
            blended_change * 2 <= raw_change,
            "raw {raw_change} vs blended {blended_change}"
        );
    }

    #[test]
    fn absorb_returns_the_kept_indices_in_order() {
        let sample = FluidSample {
            positions: vec![
                [0.0, 1.0, 0.0],
                [0.0, 0.2, 0.0],
                [0.0, 0.05, 0.0],
                [0.0, 2.0, 0.0],
            ],
            velocities: vec![[0.0; 3], [0.0; 3], [0.0; 3], [0.0, -1.0, 0.0]],
        };
        // 0.2 m over the level is above the absorption band (0.1 m); 0.05 m
        // at rest is absorbed.
        let (positions, _, absorbed, kept) = absorb(&sample, 0.0);
        assert_eq!(kept, vec![0, 1, 3]);
        assert_eq!(positions.len(), 3);
        assert_eq!(absorbed, 1);
    }

    #[test]
    fn isolated_particle_sends_the_zero_kernel() {
        let neighbourhood = neighbourhood_of(&[[0.0, 0.0, 0.0], [5.0, 0.0, 0.0]], 0.1);
        assert_eq!(neighbourhood.kernels, vec![[0.0; 6]; 2]);
    }

    #[test]
    fn flat_sheet_flattens_the_kernel_vertically() {
        let mut positions = Vec::new();
        for x in 0..21 {
            for z in 0..21 {
                positions.push([
                    x as f32 * LANE_SPACING_METRES,
                    0.0,
                    z as f32 * LANE_SPACING_METRES,
                ]);
            }
        }
        let centre = 10 * 21 + 10;
        let neighbourhood = neighbourhood_of(&positions, NEIGHBOUR_RADIUS_METRES);
        let kernel = neighbourhood.kernels[centre];
        assert!(kernel[3] > kernel[0] && kernel[3] > kernel[5], "{kernel:?}");
        assert!(
            kernel[1].abs() < 1e-6 && kernel[4].abs() < 1e-6,
            "{kernel:?}"
        );
        let axes = kernel_axes(kernel);
        assert!(axes[2] / axes[0] <= KERNEL_MAX_RATIO + 1e-3, "{axes:?}");
        assert!(axes[0] >= KERNEL_MIN_AXIS_SCALE * KERNEL_PROFILE_RADIUS_METRES - 1e-6);
    }

    #[test]
    fn filled_ball_keeps_the_kernel_near_isotropic() {
        let mut positions = vec![[0.0, 0.0, 0.0]];
        for x in -4..=4 {
            for y in -4..=4 {
                for z in -4..=4 {
                    let point = [x, y, z].map(|value| value as f32 * LANE_SPACING_METRES);
                    if (x, y, z) != (0, 0, 0)
                        && point[0] * point[0] + point[1] * point[1] + point[2] * point[2]
                            <= 0.2 * 0.2
                    {
                        positions.push(point);
                    }
                }
            }
        }
        let neighbourhood = neighbourhood_of(&positions, NEIGHBOUR_RADIUS_METRES);
        let axes = kernel_axes(neighbourhood.kernels[0]);
        assert!(axes[2] / axes[0] < 1.1, "{axes:?}");
        assert!(
            (axes[1] - KERNEL_PROFILE_RADIUS_METRES).abs() < 0.3 * KERNEL_PROFILE_RADIUS_METRES,
            "{axes:?}"
        );
    }

    fn frame(
        boxes: Vec<WaterFloatingBoxV1>,
        edges: Vec<WaterEdgePresentationV1>,
    ) -> WaterPresentationFrameV1 {
        WaterPresentationFrameV1 {
            tick: 1,
            frame_index: 1,
            surfaces: Vec::new(),
            jet: WaterJetParticlesV1::default(),
            edges,
            boxes,
        }
    }

    fn crate_box(vertical_velocity: i64) -> WaterFloatingBoxV1 {
        WaterFloatingBoxV1 {
            minimum_micrometres: [6_250_000, 300_000, 1_750_000],
            maximum_micrometres: [6_750_000, 800_000, 2_250_000],
            vertical_velocity_micrometres_per_second: vertical_velocity,
        }
    }

    #[test]
    fn emission_follows_the_stage_rules_inside_the_box() {
        let fluid_box = basin_fluid_box();
        let level = fluid_box.min[1];
        // Plan 17: 1 m/s above the 0.3 m/s threshold, one droplet per 0.02 m/s.
        let (positions, velocities) = emission_for_frame(
            &frame(vec![crate_box(1_000_000)], Vec::new()),
            &fluid_box,
            level,
            16_384,
        );
        assert_eq!(positions.len(), 35);
        assert!(velocities.iter().all(|velocity| velocity[1] > 0.0));
        assert!(
            positions
                .iter()
                .all(|position| fluid_box.contains_plan(position[0], position[2]))
        );
        assert!(
            emission_for_frame(
                &frame(vec![crate_box(0)], Vec::new()),
                &fluid_box,
                level,
                16_384
            )
            .0
            .is_empty()
        );
        assert!(
            emission_for_frame(&frame(Vec::new(), Vec::new()), &fluid_box, level, 16_384)
                .0
                .is_empty()
        );
        let fall = |crest_x: i64| WaterEdgePresentationV1 {
            edge_id: next_contracts::ids::PersistentId::from_bytes([0x11; 16]),
            kind: WaterEdgePresentationKindV1::Fall,
            crest_micrometres: [crest_x, 500_000, 2_000_000],
            direction_q15: [32_767, 0],
            source_level_micrometres: 700_000,
            sink_level_micrometres: 300_000,
            flux_cubic_millimetres: 5_000_000,
        };
        let inside = emission_for_frame(
            &frame(Vec::new(), vec![fall(6_500_000)]),
            &fluid_box,
            level,
            16_384,
        );
        assert_eq!(inside.0.len(), 10);
        assert!(inside.1.iter().all(|velocity| velocity[0] > 0.0));
        let outside = emission_for_frame(
            &frame(Vec::new(), vec![fall(16_000_000)]),
            &fluid_box,
            level,
            16_384,
        );
        assert!(outside.0.is_empty());
        let capped = emission_for_frame(
            &frame(vec![crate_box(1_000_000)], Vec::new()),
            &fluid_box,
            level,
            10,
        );
        assert_eq!(capped.0.len(), 10);
    }

    #[test]
    fn colliders_follow_the_frame_boxes_inside_the_fluid_box() {
        let fluid_box = basin_fluid_box();
        let inside = colliders_for_frame(&frame(vec![crate_box(0)], Vec::new()), &fluid_box);
        assert_eq!(inside.len(), 1);
        assert_eq!(inside[0].0, [6.5, 0.55, 2.0]);
        assert_eq!(inside[0].1, [0.25, 0.25, 0.25]);
        let mut far = crate_box(0);
        far.minimum_micrometres[0] += 10_000_000;
        far.maximum_micrometres[0] += 10_000_000;
        assert!(colliders_for_frame(&frame(vec![far], Vec::new()), &fluid_box).is_empty());
        assert!(colliders_for_frame(&frame(Vec::new(), Vec::new()), &fluid_box).is_empty());
        // Inside means strictly inside the box shrunk by one spacing.
        let positions = [[6.5, 0.55, 2.0], [6.5, 0.77, 2.0], [7.0, 0.55, 2.0]];
        assert_eq!(particles_inside(&positions, &inside), 1);
    }

    #[test]
    fn neighbour_grid_matches_brute_force_and_clusters_are_consistent() {
        // A deterministic pseudo-random cloud in a 1 m cube plus one
        // isolated particle far away.
        let mut state = 0x9e37_79b9_u32;
        let mut next = || {
            state ^= state << 13;
            state ^= state >> 17;
            state ^= state << 5;
            (state % 10_000) as f32 / 10_000.0
        };
        let mut positions: Vec<[f32; 3]> = (0..2_000).map(|_| [next(), next(), next()]).collect();
        positions.push([10.0, 10.0, 10.0]);
        let radius = NEIGHBOUR_RADIUS_METRES;
        let (counts, clusters) = neighbour_counts_and_clusters(&positions, radius);
        let count = positions.len();
        let mut brute = vec![0_u32; count];
        let mut adjacency: Vec<Vec<usize>> = vec![Vec::new(); count];
        for a in 0..count {
            for b in (a + 1)..count {
                let d: f32 = (0..3)
                    .map(|axis| (positions[a][axis] - positions[b][axis]).powi(2))
                    .sum();
                if d <= radius * radius {
                    brute[a] += 1;
                    brute[b] += 1;
                    adjacency[a].push(b);
                    adjacency[b].push(a);
                }
            }
        }
        for index in 0..count {
            assert_eq!(
                u32::from(counts[index]),
                brute[index].min(255),
                "particle {index}"
            );
        }
        let mut component = vec![usize::MAX; count];
        let mut sizes = Vec::new();
        for start in 0..count {
            if component[start] != usize::MAX {
                continue;
            }
            let id = sizes.len();
            let mut stack = vec![start];
            let mut size = 0;
            component[start] = id;
            while let Some(index) = stack.pop() {
                size += 1;
                for &other in &adjacency[index] {
                    if component[other] == usize::MAX {
                        component[other] = id;
                        stack.push(other);
                    }
                }
            }
            sizes.push(size);
        }
        for index in 0..count {
            assert_eq!(
                usize::from(clusters[index]),
                sizes[component[index]].min(65_535)
            );
        }
        assert_eq!(counts[count - 1], 0);
        assert_eq!(clusters[count - 1], 1);
    }

    #[test]
    fn absorption_keeps_flying_particles_and_removes_settled_ones() {
        let level = 0.5;
        let sample = FluidSample {
            positions: vec![
                [6.5, 0.55, 2.0],
                [6.5, 1.0, 2.0],
                [6.5, 0.4, 2.0],
                [6.5, 0.55, 2.0],
            ],
            velocities: vec![
                [0.0, -0.1, 0.0],
                [0.0, 0.0, 0.0],
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
            ],
        };
        let (positions, velocities, absorbed, _kept) = absorb(&sample, level);
        assert_eq!(absorbed, 2, "the settled one and the one below the floor");
        assert_eq!(positions.len(), 2);
        assert_eq!(velocities.len(), 2);
        assert_eq!(positions[0], [6.5, 1.0, 2.0], "still falling, kept");
        assert_eq!(positions[1], [6.5, 0.55, 2.0], "fast at the level, kept");
    }
}
