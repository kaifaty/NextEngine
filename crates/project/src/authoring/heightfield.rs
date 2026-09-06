//! Scene look L6a (plan `look/06a`): a height field to a neutral mesh.
//! Rows along `z`, columns along `x`; heights map the sample range linearly
//! onto `[low, high]`; normals from central differences; `uv0` in `0..1`
//! over the field; cells fully inside a hole are dropped (their vertices
//! stay, unreferenced vertices are removed).

/// The mesh a height field becomes.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct HeightfieldMesh {
    pub(crate) positions_micrometres: Vec<[i64; 3]>,
    pub(crate) normals_snorm16: Vec<[i16; 3]>,
    pub(crate) uv0_q16: Vec<[i32; 2]>,
    pub(crate) indices: Vec<u32>,
    pub(crate) bounds_min: [i64; 3],
    pub(crate) bounds_max: [i64; 3],
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum HeightfieldError {
    /// Fewer than two samples along a side, or a sample count mismatch.
    Degenerate,
    /// A cell size or range that is not positive.
    InvalidParameter,
    /// Every cell fell into a hole.
    Empty,
}

#[allow(
    clippy::too_many_arguments,
    reason = "the record's fields are explicit at the authoring boundary"
)]
pub(crate) fn build_heightfield(
    samples: &[u16],
    columns: u32,
    rows: u32,
    origin_micrometres: [i64; 2],
    cell_micrometres: i64,
    height_range_micrometres: [i64; 2],
    holes: &[[i64; 4]],
) -> Result<HeightfieldMesh, HeightfieldError> {
    if columns < 2 || rows < 2 || samples.len() != (columns as usize) * (rows as usize) {
        return Err(HeightfieldError::Degenerate);
    }
    if cell_micrometres <= 0 || height_range_micrometres[1] <= height_range_micrometres[0] {
        return Err(HeightfieldError::InvalidParameter);
    }
    let range = (height_range_micrometres[1] - height_range_micrometres[0]) as f64;
    let height_at = |column: usize, row: usize| -> f64 {
        height_range_micrometres[0] as f64
            + f64::from(samples[row * columns as usize + column]) / 65_535.0 * range
    };
    let cell = cell_micrometres as f64;
    let mut positions = Vec::with_capacity(samples.len());
    let mut normals = Vec::with_capacity(samples.len());
    let mut uvs = Vec::with_capacity(samples.len());
    for row in 0..rows as usize {
        for column in 0..columns as usize {
            let x = origin_micrometres[0] + column as i64 * cell_micrometres;
            let z = origin_micrometres[1] + row as i64 * cell_micrometres;
            let y = height_at(column, row).round() as i64;
            positions.push([x, y, z]);
            // Central differences (one-sided at the border).
            let (left, right) = (
                height_at(column.saturating_sub(1), row),
                height_at((column + 1).min(columns as usize - 1), row),
            );
            let (back, front) = (
                height_at(column, row.saturating_sub(1)),
                height_at(column, (row + 1).min(rows as usize - 1)),
            );
            let dx = (column.min(columns as usize - 1) - column.saturating_sub(1)) as f64
                + if column + 1 < columns as usize {
                    1.0
                } else {
                    0.0
                };
            let dz = (row.min(rows as usize - 1) - row.saturating_sub(1)) as f64
                + if row + 1 < rows as usize { 1.0 } else { 0.0 };
            let slope_x = (right - left) / (dx.max(1.0) * cell);
            let slope_z = (front - back) / (dz.max(1.0) * cell);
            let normal = [-slope_x, 1.0, -slope_z];
            let length = (normal[0] * normal[0] + 1.0 + normal[2] * normal[2]).sqrt();
            normals.push([
                (normal[0] / length * 32_767.0).round() as i16,
                (normal[1] / length * 32_767.0).round() as i16,
                (normal[2] / length * 32_767.0).round() as i16,
            ]);
            uvs.push([
                ((column as f64 / (columns - 1) as f64) * 65_536.0).round() as i32,
                ((row as f64 / (rows - 1) as f64) * 65_536.0).round() as i32,
            ]);
        }
    }
    let mut indices = Vec::with_capacity(((columns - 1) * (rows - 1) * 6) as usize);
    for row in 0..rows as usize - 1 {
        for column in 0..columns as usize - 1 {
            let x0 = origin_micrometres[0] + column as i64 * cell_micrometres;
            let z0 = origin_micrometres[1] + row as i64 * cell_micrometres;
            let (x1, z1) = (x0 + cell_micrometres, z0 + cell_micrometres);
            if holes
                .iter()
                .any(|hole| x0 >= hole[0] && z0 >= hole[1] && x1 <= hole[2] && z1 <= hole[3])
            {
                continue;
            }
            let a = (row * columns as usize + column) as u32;
            let b = a + 1;
            let c = a + columns;
            let d = c + 1;
            // Counter-clockwise seen from above (+y): a (x0,z0), c (x0,z1),
            // b (x1,z0) and b, c, d.
            indices.extend_from_slice(&[a, c, b, b, c, d]);
        }
    }
    if indices.is_empty() {
        return Err(HeightfieldError::Empty);
    }
    // Drop unreferenced vertices, keeping the grid order.
    let mut referenced = vec![false; positions.len()];
    for index in &indices {
        referenced[*index as usize] = true;
    }
    let mut remap = vec![u32::MAX; positions.len()];
    let mut kept_positions = Vec::with_capacity(positions.len());
    let mut kept_normals = Vec::with_capacity(positions.len());
    let mut kept_uvs = Vec::with_capacity(positions.len());
    for (old, used) in referenced.iter().enumerate() {
        if *used {
            remap[old] =
                u32::try_from(kept_positions.len()).map_err(|_| HeightfieldError::Degenerate)?;
            kept_positions.push(positions[old]);
            kept_normals.push(normals[old]);
            kept_uvs.push(uvs[old]);
        }
    }
    let indices: Vec<u32> = indices.iter().map(|index| remap[*index as usize]).collect();
    let mut bounds_min = [i64::MAX; 3];
    let mut bounds_max = [i64::MIN; 3];
    for position in &kept_positions {
        for axis in 0..3 {
            bounds_min[axis] = bounds_min[axis].min(position[axis]);
            bounds_max[axis] = bounds_max[axis].max(position[axis]);
        }
    }
    Ok(HeightfieldMesh {
        positions_micrometres: kept_positions,
        normals_snorm16: kept_normals,
        uv0_q16: kept_uvs,
        indices,
        bounds_min,
        bounds_max: bounds_max.map(|value| value + 1),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Plan look/06a G3: a raised centre gives the expected positions,
    /// normals leaning away from it and the uv corners; a hole over the
    /// centre cell drops its triangles.
    #[test]
    fn raised_centre_and_hole() {
        let samples = [0, 0, 0, 0, 65_535, 0, 0, 0, 0];
        let mesh = build_heightfield(
            &samples,
            3,
            3,
            [-1_000_000, -1_000_000],
            1_000_000,
            [0, 500_000],
            &[],
        )
        .expect("mesh");
        assert_eq!(mesh.positions_micrometres.len(), 9);
        assert_eq!(mesh.positions_micrometres[4], [0, 500_000, 0]);
        assert_eq!(mesh.positions_micrometres[0], [-1_000_000, 0, -1_000_000]);
        assert_eq!(mesh.indices.len(), 24);
        // The left neighbour of the centre slopes up toward +x, so its
        // normal leans toward -x.
        assert!(
            mesh.normals_snorm16[3][0] < -5_000,
            "{:?}",
            mesh.normals_snorm16[3]
        );
        assert!(mesh.normals_snorm16[5][0] > 5_000);
        assert!(mesh.normals_snorm16[1][2] < -5_000);
        assert_eq!(mesh.normals_snorm16[4], [0, 32_767, 0]);
        assert_eq!(mesh.uv0_q16[0], [0, 0]);
        assert_eq!(mesh.uv0_q16[8], [65_536, 65_536]);
        assert_eq!(mesh.bounds_min, [-1_000_000, 0, -1_000_000]);
        assert_eq!(mesh.bounds_max, [1_000_001, 500_001, 1_000_001]);
        let holed = build_heightfield(
            &samples,
            3,
            3,
            [-1_000_000, -1_000_000],
            1_000_000,
            [0, 500_000],
            &[[0, 0, 1_000_000, 1_000_000]],
        )
        .expect("mesh");
        assert_eq!(holed.indices.len(), 18);
        // The centre vertex is unreferenced and dropped.
        assert_eq!(holed.positions_micrometres.len(), 8);
        let all = build_heightfield(
            &samples,
            3,
            3,
            [0, 0],
            1_000_000,
            [0, 1],
            &[[-1, -1, 9_000_000, 9_000_000]],
        );
        assert_eq!(all, Err(HeightfieldError::Empty));
    }
}
