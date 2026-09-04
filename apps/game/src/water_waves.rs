//! Plan `continuum-water/38`: the shallow-water presentation grid on every
//! water ring. Presentation-only state of the game's water feed, stepped
//! once per published frame by the wave equation on the ring's own grid,
//! excited by the stage's box and edge records, reflective at the rim,
//! damped, clamped. The stage stays pure and the authority reads nothing.

use next_reference_game::{
    WATER_SURFACE_GRID_COLUMNS, WATER_SURFACE_GRID_ROWS, WaterEdgePresentationV1,
    WaterFloatingBoxV1,
};

/// The presentation clock of the grid (one published frame).
pub(crate) const WAVE_FRAME_SECONDS: f32 = 1.0 / 60.0;
/// Gravity for the wave speed `c = sqrt(g H)`.
pub(crate) const WAVE_GRAVITY_METRES_PER_SECOND_SQUARED: f32 = 9.81;
/// Stability bound of the explicit scheme per axis.
pub(crate) const WAVE_COURANT_LIMIT: f32 = 0.7;
/// Velocity damping per frame.
pub(crate) const WAVE_DAMPING_PER_FRAME: f32 = 0.98;
/// Height relaxation per frame: the displaced volume returns to the level
/// (the ring shows the exact level, the grid only the ripple).
pub(crate) const WAVE_RELAXATION_PER_FRAME: f32 = 0.995;
/// The grid's own height cap (metres); the ring's catalog cap is `20 mm`.
pub(crate) const WAVE_HEIGHT_CAP_METRES: f32 = 0.015;
/// A box's vertical speed pushes the surface under it by this fraction.
pub(crate) const WAVE_BOX_GAIN: f32 = 0.3;
/// The largest push of a stream per frame (metres).
pub(crate) const WAVE_STREAM_CAP_METRES: f32 = 0.005;

/// One ring's grid: the plan it covers and two height fields in metres.
#[derive(Clone, Debug)]
pub(crate) struct WaveGridV1 {
    columns: usize,
    rows: usize,
    minimum_metres: [f32; 2],
    cell_metres: [f32; 2],
    /// `(c dt / dx)^2` per axis.
    courant_squared: [f32; 2],
    heights: Vec<f32>,
    previous: Vec<f32>,
}

impl WaveGridV1 {
    /// A grid over the plan `minimum..maximum` (world micrometres, `x` and
    /// `z`) at the ring's vertex resolution with the authored depth;
    /// `None` when the explicit scheme would be unstable at this size.
    pub(crate) fn new(
        minimum_micrometres: [i64; 2],
        maximum_micrometres: [i64; 2],
        depth_micrometres: i64,
    ) -> Option<Self> {
        let columns = WATER_SURFACE_GRID_COLUMNS as usize;
        let rows = WATER_SURFACE_GRID_ROWS as usize;
        let to_metres = |value: i64| value as f32 / 1_000_000.0;
        let extent = [
            to_metres(maximum_micrometres[0] - minimum_micrometres[0]),
            to_metres(maximum_micrometres[1] - minimum_micrometres[1]),
        ];
        if extent[0] <= 0.0 || extent[1] <= 0.0 || depth_micrometres <= 0 {
            return None;
        }
        let cell = [
            extent[0] / (columns - 1) as f32,
            extent[1] / (rows - 1) as f32,
        ];
        let speed = (WAVE_GRAVITY_METRES_PER_SECOND_SQUARED * to_metres(depth_micrometres)).sqrt();
        let courant = [
            speed * WAVE_FRAME_SECONDS / cell[0],
            speed * WAVE_FRAME_SECONDS / cell[1],
        ];
        if courant[0] > WAVE_COURANT_LIMIT || courant[1] > WAVE_COURANT_LIMIT {
            return None;
        }
        Some(Self {
            columns,
            rows,
            minimum_metres: [
                to_metres(minimum_micrometres[0]),
                to_metres(minimum_micrometres[1]),
            ],
            cell_metres: cell,
            courant_squared: [courant[0] * courant[0], courant[1] * courant[1]],
            heights: vec![0.0; columns * rows],
            previous: vec![0.0; columns * rows],
        })
    }

    /// The cell under a world point, if inside the plan.
    fn cell_of(&self, x_micrometres: i64, z_micrometres: i64) -> Option<(usize, usize)> {
        let x = x_micrometres as f32 / 1_000_000.0 - self.minimum_metres[0];
        let z = z_micrometres as f32 / 1_000_000.0 - self.minimum_metres[1];
        if x < 0.0 || z < 0.0 {
            return None;
        }
        let column = (x / self.cell_metres[0]).round() as usize;
        let row = (z / self.cell_metres[1]).round() as usize;
        (column < self.columns && row < self.rows).then_some((column, row))
    }

    /// Adds `delta_metres` to the height of every cell inside the plan
    /// rectangle `minimum..maximum` (world micrometres, `x` and `z`).
    pub(crate) fn push_rectangle(
        &mut self,
        minimum_micrometres: [i64; 2],
        maximum_micrometres: [i64; 2],
        delta_metres: f32,
    ) {
        let to_index = |value: i64, axis: usize, count: usize| -> i64 {
            let offset = value as f32 / 1_000_000.0 - self.minimum_metres[axis];
            (offset / self.cell_metres[axis])
                .round()
                .clamp(-1.0, count as f32) as i64
        };
        let (c0, c1) = (
            to_index(minimum_micrometres[0], 0, self.columns),
            to_index(maximum_micrometres[0], 0, self.columns),
        );
        let (r0, r1) = (
            to_index(minimum_micrometres[1], 1, self.rows),
            to_index(maximum_micrometres[1], 1, self.rows),
        );
        for row in r0.max(0)..=r1.min(self.rows as i64 - 1) {
            for column in c0.max(0)..=c1.min(self.columns as i64 - 1) {
                let index = row as usize * self.columns + column as usize;
                // A displacement, not a velocity kick: both fields move.
                self.heights[index] += delta_metres;
                self.previous[index] += delta_metres;
            }
        }
    }

    /// Adds `delta_metres` to the cell under a world point.
    pub(crate) fn push_point(&mut self, x_micrometres: i64, z_micrometres: i64, delta_metres: f32) {
        if let Some((column, row)) = self.cell_of(x_micrometres, z_micrometres) {
            self.heights[row * self.columns + column] += delta_metres;
            self.previous[row * self.columns + column] += delta_metres;
        }
    }

    /// One frame of the wave equation with reflective edges, damping and
    /// the cap.
    pub(crate) fn step(&mut self) {
        let (columns, rows) = (self.columns, self.rows);
        let mut next = vec![0.0_f32; columns * rows];
        for row in 0..rows {
            for column in 0..columns {
                let at = |c: usize, r: usize| self.heights[r * columns + c];
                let here = at(column, row);
                let left = if column > 0 {
                    at(column - 1, row)
                } else {
                    here
                };
                let right = if column + 1 < columns {
                    at(column + 1, row)
                } else {
                    here
                };
                let back = if row > 0 { at(column, row - 1) } else { here };
                let front = if row + 1 < rows {
                    at(column, row + 1)
                } else {
                    here
                };
                let laplacian = self.courant_squared[0] * (left + right - 2.0 * here)
                    + self.courant_squared[1] * (back + front - 2.0 * here);
                let index = row * columns + column;
                let undamped = 2.0 * here - self.previous[index] + laplacian;
                let value =
                    (here + WAVE_DAMPING_PER_FRAME * (undamped - here)) * WAVE_RELAXATION_PER_FRAME;
                next[index] = value.clamp(-WAVE_HEIGHT_CAP_METRES, WAVE_HEIGHT_CAP_METRES);
            }
        }
        self.previous = std::mem::replace(&mut self.heights, next);
    }

    /// The grid's height at a vertex, micrometres.
    pub(crate) fn height_micrometres(&self, column: usize, row: usize) -> i64 {
        (self.heights[row * self.columns + column] * 1_000_000.0).round() as i64
    }

    /// `Σ h²`, the grid's energy proxy.
    #[cfg(test)]
    pub(crate) fn energy(&self) -> f32 {
        self.heights.iter().map(|h| h * h).sum()
    }

    /// The largest radius (in cells, Chebyshev, from the centre) of any
    /// cell with `|h|` above `threshold`.
    #[cfg(test)]
    pub(crate) fn disturbed_radius_cells(&self, threshold: f32) -> usize {
        let (cc, cr) = (self.columns / 2, self.rows / 2);
        let mut radius = 0;
        for row in 0..self.rows {
            for column in 0..self.columns {
                if self.heights[row * self.columns + column].abs() > threshold {
                    radius = radius.max(cc.abs_diff(column).max(cr.abs_diff(row)));
                }
            }
        }
        radius
    }

    /// Excites the grid from one frame's records: the boxes over the plan
    /// and the edges whose crest lies in it.
    pub(crate) fn excite(
        &mut self,
        boxes: &[WaterFloatingBoxV1],
        edges: &[WaterEdgePresentationV1],
        ticks_per_second: u32,
    ) {
        for floating in boxes {
            // A falling box (negative speed) pushes the surface down.
            let delta = (floating.vertical_velocity_micrometres_per_second as f32 / 1_000_000.0)
                * WAVE_FRAME_SECONDS
                * WAVE_BOX_GAIN;
            if delta != 0.0 {
                self.push_rectangle(
                    [
                        floating.minimum_micrometres[0],
                        floating.minimum_micrometres[2],
                    ],
                    [
                        floating.maximum_micrometres[0],
                        floating.maximum_micrometres[2],
                    ],
                    delta,
                );
            }
        }
        let cell_area_square_millimetres = self.cell_metres[0] * self.cell_metres[1] * 1_000_000.0;
        for edge in edges {
            let flux_per_second =
                edge.flux_cubic_millimetres.unsigned_abs() as f32 * ticks_per_second as f32;
            let push = (flux_per_second / cell_area_square_millimetres / 1_000.0
                * WAVE_FRAME_SECONDS)
                .min(WAVE_STREAM_CAP_METRES);
            if push > 0.0 {
                self.push_point(edge.crest_micrometres[0], edge.crest_micrometres[2], -push);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn grid() -> WaveGridV1 {
        // The reference basin: 4 x 2 m, 0.5 m deep (c = 2.2 m/s, cells
        // 0.129 x 0.133 m, courant 0.28 / 0.28).
        WaveGridV1::new([4_500_000, 1_000_000], [8_500_000, 3_000_000], 500_000).expect("stable")
    }

    #[test]
    fn a_point_impulse_spreads_at_the_wave_speed_and_never_gains_energy() {
        let mut grid = grid();
        grid.push_point(6_500_000, 2_000_000, 0.01);
        let speed = (9.81_f32 * 0.5).sqrt();
        let initial_energy = grid.energy();
        let mut peak_energy = 0.0_f32;
        for frame in 1..=24 {
            grid.step();
            peak_energy = peak_energy.max(grid.energy());
            if frame % 8 == 0 {
                // The front (2 % of the impulse) travels at c, one cell per
                // 3.5 frames here, within two cells of the stencil's reach.
                let expected = speed * frame as f32 * WAVE_FRAME_SECONDS / grid.cell_metres[0];
                let radius = grid.disturbed_radius_cells(2e-4) as f32;
                assert!(
                    (radius - expected).abs() <= 2.0,
                    "frame {frame}: front {radius} cells, expected {expected}"
                );
            }
        }
        assert!(
            peak_energy <= initial_energy,
            "the impulse's energy never grows: {peak_energy} vs {initial_energy}"
        );
    }

    #[test]
    fn a_wave_reaches_the_rim_reflects_and_the_grid_settles() {
        let mut grid = grid();
        grid.push_point(6_500_000, 2_000_000, 0.01);
        let edge_sum = |grid: &WaveGridV1| -> f32 {
            (0..grid.rows)
                .map(|row| grid.heights[row * grid.columns].abs())
                .sum()
        };
        let edge_before = edge_sum(&grid);
        let mut edge_peak = 0.0_f32;
        for _ in 0..600 {
            grid.step();
            edge_peak = edge_peak.max(edge_sum(&grid));
            assert!(
                grid.heights
                    .iter()
                    .all(|h| h.abs() <= WAVE_HEIGHT_CAP_METRES && h.is_finite())
            );
        }
        assert!(
            edge_peak > edge_before + 1e-4,
            "the wave reached the rim: {edge_peak}"
        );
        assert!(
            grid.energy() < 1e-8,
            "the grid settles: {} from 1e-4",
            grid.energy()
        );
    }

    #[test]
    fn the_clamp_holds_and_an_unstable_ring_gets_no_grid() {
        let mut grid = grid();
        grid.push_point(6_500_000, 2_000_000, 1.0);
        grid.step();
        assert!(
            grid.heights
                .iter()
                .all(|h| h.abs() <= WAVE_HEIGHT_CAP_METRES)
        );
        // 0.4 m wide, 3 m deep: cells 13 mm, c 5.4 m/s, courant 7.
        assert!(WaveGridV1::new([0, 0], [400_000, 200_000], 3_000_000).is_none());
    }

    #[test]
    fn records_excite_the_cells_they_cover() {
        let mut grid = grid();
        let falling = WaterFloatingBoxV1 {
            minimum_micrometres: [6_250_000, 0, 1_750_000],
            maximum_micrometres: [6_750_000, 500_000, 2_250_000],
            vertical_velocity_micrometres_per_second: -1_000_000,
        };
        grid.excite(&[falling], &[], 30);
        assert!(
            grid.height_micrometres(16, 8) < 0,
            "a falling box pushes the surface down"
        );
        assert_eq!(grid.height_micrometres(0, 0), 0);
    }

    /// Plan 38 G3 timing probe: the four reference rings' excitation and
    /// step per frame; run with `--ignored --nocapture` in release.
    #[test]
    #[ignore = "timing probe"]
    fn bench_four_rings_per_frame() {
        let mut grids = vec![
            grid(),
            WaveGridV1::new([12_000_000, 1_000_000], [14_000_000, 2_500_000], 500_000).expect("a"),
            WaveGridV1::new([15_000_000, 1_000_000], [17_800_000, 2_500_000], 500_000).expect("b"),
            WaveGridV1::new([-9_000_000, 5_000_000], [1_000_000, 9_500_000], 1_400_000)
                .expect("pond"),
        ];
        let falling = WaterFloatingBoxV1 {
            minimum_micrometres: [6_250_000, 0, 1_750_000],
            maximum_micrometres: [6_750_000, 500_000, 2_250_000],
            vertical_velocity_micrometres_per_second: -300_000,
        };
        let started = std::time::Instant::now();
        let frames = 2_000;
        for _ in 0..frames {
            for grid in &mut grids {
                grid.excite(&[falling], &[], 30);
                grid.step();
            }
        }
        eprintln!(
            "four rings: {} us per frame",
            started.elapsed().as_micros() / frames
        );
    }
}
