//! Presentation-only dynamic surface boundary.
//!
//! A dynamic surface is one exact catalog mesh revision whose vertex and index
//! payload may be replaced per frame by the application without rebuilding or
//! re-cooking `RenderContentCatalogV1`. The catalog record keeps the stable
//! identity, material binding and declared presentation bounds; the update is
//! a reconstructible renderer cache that never feeds back into simulation and
//! never participates in a gameplay, snapshot, replay or frame-plan root.

use std::collections::BTreeMap;
use std::sync::Arc;

use next_contracts::ids::ContentHash;
use next_contracts::presentation::PresentationSnapshotV3;
use next_contracts::project::{AssetRevisionRefV1, domain_hash};
use next_contracts::render_content::{AabbI64V1, RenderContentCatalogV1};

use crate::DesktopAdapterError;

/// Upper vertex bound for one declared dynamic surface ring.
pub const MAX_DYNAMIC_SURFACE_VERTICES: u32 = 1 << 20;
/// Upper index bound for one declared dynamic surface ring.
pub const MAX_DYNAMIC_SURFACE_INDICES: u32 = 3 << 20;
/// Upper count of dynamic surfaces one desktop run may declare.
pub const MAX_DYNAMIC_SURFACES: usize = 8;

const DYNAMIC_SURFACE_UPDATE_DOMAIN: &str = "nextengine.desktop.dynamic-surface-update.v1";

/// Where a dynamic surface ring lives on the device.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DynamicSurfaceResidencyV1 {
    /// One host-visible, host-coherent vertex/index pair per frame slot; the
    /// GPU fetches vertices from host memory. Cheapest CPU path.
    HostVisible,
    /// One host-visible staging pair plus one device-local pair per frame
    /// slot; a refresh records a transfer at the start of that frame's
    /// command buffer and the GPU fetches from device memory.
    DeviceLocal,
}

/// Declares one bounded dynamic surface for the whole desktop run.
///
/// The adapter allocates one vertex/index ring per frame slot at this
/// capacity before the first frame. Capacity never grows at runtime; an
/// update that exceeds it fails closed with a typed diagnostic.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DynamicSurfaceProfileV1 {
    pub mesh_revision: AssetRevisionRefV1,
    pub vertex_capacity: u32,
    pub index_capacity: u32,
    pub residency: DynamicSurfaceResidencyV1,
}

/// One immutable vertex/index replacement for a declared dynamic surface.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DynamicSurfaceUpdateV1 {
    mesh_revision: AssetRevisionRefV1,
    sequence: u64,
    positions_micrometres: Vec<[i64; 3]>,
    normals_snorm16: Vec<[i16; 3]>,
    indices: Vec<u32>,
    canonical_hash: ContentHash,
    packed_b0_vertices: Vec<u8>,
    packed_b0_indices: Vec<u8>,
}

impl DynamicSurfaceUpdateV1 {
    /// Validates the closed triangle payload and binds its canonical hash.
    pub fn new(
        mesh_revision: AssetRevisionRefV1,
        sequence: u64,
        positions_micrometres: Vec<[i64; 3]>,
        normals_snorm16: Vec<[i16; 3]>,
        indices: Vec<u32>,
    ) -> Result<Self, DesktopAdapterError> {
        if sequence == 0 {
            return Err(invalid("update sequence must be positive"));
        }
        if positions_micrometres.is_empty() || indices.is_empty() {
            return Err(invalid("update has no renderable triangles"));
        }
        if positions_micrometres.len() > MAX_DYNAMIC_SURFACE_VERTICES as usize {
            return Err(invalid(
                "update vertex count exceeds the dynamic surface bound",
            ));
        }
        if indices.len() > MAX_DYNAMIC_SURFACE_INDICES as usize {
            return Err(invalid(
                "update index count exceeds the dynamic surface bound",
            ));
        }
        if !indices.len().is_multiple_of(3) {
            return Err(invalid("update index count is not a triangle list"));
        }
        if normals_snorm16.len() != positions_micrometres.len() {
            return Err(invalid("update normal count does not match vertices"));
        }
        if normals_snorm16
            .iter()
            .any(|normal| normal.iter().all(|component| *component == 0))
        {
            return Err(invalid("update contains a zero normal"));
        }
        if indices.iter().any(|index| {
            usize::try_from(*index).map_or(true, |index| index >= positions_micrometres.len())
        }) {
            return Err(invalid("update index is outside the vertex array"));
        }
        let canonical_hash = canonical_hash(
            mesh_revision,
            &positions_micrometres,
            &normals_snorm16,
            &indices,
        )?;
        let packed_b0_vertices =
            crate::gpu_content::pack_b0_vertices(&positions_micrometres, &normals_snorm16)
                .ok_or(DesktopAdapterError::CounterOverflow)?;
        let packed_b0_indices = crate::gpu_content::pack_b0_indices(&indices)
            .ok_or(DesktopAdapterError::CounterOverflow)?;
        Ok(Self {
            mesh_revision,
            sequence,
            positions_micrometres,
            normals_snorm16,
            indices,
            canonical_hash,
            packed_b0_vertices,
            packed_b0_indices,
        })
    }

    /// Locked B0 vertex bytes (28 per vertex), packed at construction.
    #[must_use]
    pub(crate) fn packed_b0_vertices(&self) -> &[u8] {
        &self.packed_b0_vertices
    }

    /// Little-endian `u32` triangle indices, packed at construction.
    #[must_use]
    pub(crate) fn packed_b0_indices(&self) -> &[u8] {
        &self.packed_b0_indices
    }

    /// Republishes the same immutable payload under a later sequence, for
    /// example when a bounded keyframe set cycles. The canonical hash binds
    /// content only, so a ring that already holds this payload skips the copy.
    pub fn with_sequence(&self, sequence: u64) -> Result<Self, DesktopAdapterError> {
        if sequence == 0 {
            return Err(invalid("update sequence must be positive"));
        }
        Ok(Self {
            sequence,
            ..self.clone()
        })
    }

    #[must_use]
    pub const fn mesh_revision(&self) -> AssetRevisionRefV1 {
        self.mesh_revision
    }

    #[must_use]
    pub const fn sequence(&self) -> u64 {
        self.sequence
    }

    #[must_use]
    pub fn positions_micrometres(&self) -> &[[i64; 3]] {
        &self.positions_micrometres
    }

    #[must_use]
    pub fn normals_snorm16(&self) -> &[[i16; 3]] {
        &self.normals_snorm16
    }

    #[must_use]
    pub fn indices(&self) -> &[u32] {
        &self.indices
    }

    #[must_use]
    pub const fn canonical_hash(&self) -> ContentHash {
        self.canonical_hash
    }

    #[must_use]
    pub fn vertex_count(&self) -> u32 {
        // Bounded by `MAX_DYNAMIC_SURFACE_VERTICES` in `new`.
        self.positions_micrometres.len() as u32
    }

    #[must_use]
    pub fn index_count(&self) -> u32 {
        // Bounded by `MAX_DYNAMIC_SURFACE_INDICES` in `new`.
        self.indices.len() as u32
    }
}

fn canonical_hash(
    mesh_revision: AssetRevisionRefV1,
    positions: &[[i64; 3]],
    normals: &[[i16; 3]],
    indices: &[u32],
) -> Result<ContentHash, DesktopAdapterError> {
    let vertex_bytes = positions
        .len()
        .checked_mul(24)
        .and_then(|value| value.checked_add(positions.len().checked_mul(6)?))
        .and_then(|value| value.checked_add(indices.len().checked_mul(4)?))
        .and_then(|value| value.checked_add(80))
        .ok_or(DesktopAdapterError::CounterOverflow)?;
    let mut preimage = Vec::new();
    preimage
        .try_reserve_exact(vertex_bytes)
        .map_err(|_| DesktopAdapterError::CounterOverflow)?;
    preimage.extend_from_slice(mesh_revision.asset_id.as_bytes());
    preimage.extend_from_slice(mesh_revision.record_sha256.as_bytes());
    preimage.extend_from_slice(
        &u64::try_from(positions.len())
            .map_err(|_| DesktopAdapterError::CounterOverflow)?
            .to_le_bytes(),
    );
    preimage.extend_from_slice(
        &u64::try_from(indices.len())
            .map_err(|_| DesktopAdapterError::CounterOverflow)?
            .to_le_bytes(),
    );
    for position in positions {
        for component in position {
            preimage.extend_from_slice(&component.to_le_bytes());
        }
    }
    for normal in normals {
        for component in normal {
            preimage.extend_from_slice(&component.to_le_bytes());
        }
    }
    for index in indices {
        preimage.extend_from_slice(&index.to_le_bytes());
    }
    Ok(domain_hash(DYNAMIC_SURFACE_UPDATE_DOMAIN, &preimage))
}

fn invalid(reason: &'static str) -> DesktopAdapterError {
    DesktopAdapterError::DynamicSurfaceInvalid { reason }
}

/// What one frame-source pump publishes to the desktop adapter.
///
/// `snapshot` follows the immutable presentation hand-off and its monotonic
/// transition validation. `dynamic_surface_updates` replace renderer caches
/// for declared surfaces only; they carry no gameplay authority.
#[derive(Clone, Debug, Default)]
pub struct DesktopFramePublicationV1 {
    pub snapshot: Option<Arc<PresentationSnapshotV3>>,
    pub dynamic_surface_updates: Vec<Arc<DynamicSurfaceUpdateV1>>,
}

impl DesktopFramePublicationV1 {
    #[must_use]
    pub fn snapshot_only(snapshot: Option<Arc<PresentationSnapshotV3>>) -> Self {
        Self {
            snapshot,
            dynamic_surface_updates: Vec::new(),
        }
    }
}

#[derive(Clone, Copy, Debug)]
struct DeclaredDynamicSurface {
    profile: DynamicSurfaceProfileV1,
    bounds: AabbI64V1,
}

/// Adapter-owned current dynamic surface state for one desktop run.
#[derive(Clone, Debug)]
pub(crate) struct DynamicSurfaceState {
    declared: BTreeMap<AssetRevisionRefV1, DeclaredDynamicSurface>,
    current: BTreeMap<AssetRevisionRefV1, Arc<DynamicSurfaceUpdateV1>>,
    publications: u64,
}

impl DynamicSurfaceState {
    /// State for a run that declares no dynamic surface.
    #[cfg(test)]
    pub(crate) fn empty() -> Self {
        Self {
            declared: BTreeMap::new(),
            current: BTreeMap::new(),
            publications: 0,
        }
    }

    /// Validates every declared profile against the exact catalog it targets.
    pub(crate) fn new(
        profiles: &[DynamicSurfaceProfileV1],
        catalog: &RenderContentCatalogV1,
    ) -> Result<Self, DesktopAdapterError> {
        validate_profiles(profiles, catalog)?;
        let declared = profiles
            .iter()
            .map(|profile| {
                let mesh = catalog
                    .mesh(profile.mesh_revision)
                    .ok_or(DesktopAdapterError::DynamicSurfaceUndeclared)?;
                Ok((
                    profile.mesh_revision,
                    DeclaredDynamicSurface {
                        profile: *profile,
                        bounds: mesh.bounds(),
                    },
                ))
            })
            .collect::<Result<BTreeMap<_, _>, DesktopAdapterError>>()?;
        Ok(Self {
            declared,
            current: BTreeMap::new(),
            publications: 0,
        })
    }

    /// Returns a staged copy so one publication batch validates completely
    /// before any update becomes visible to the renderer.
    pub(crate) fn stage(&self) -> Self {
        self.clone()
    }

    pub(crate) fn commit(&mut self, staged: Self) {
        *self = staged;
    }

    pub(crate) fn publish(
        &mut self,
        update: Arc<DynamicSurfaceUpdateV1>,
    ) -> Result<(), DesktopAdapterError> {
        let declared = self
            .declared
            .get(&update.mesh_revision())
            .ok_or(DesktopAdapterError::DynamicSurfaceUndeclared)?;
        if update.vertex_count() > declared.profile.vertex_capacity {
            return Err(DesktopAdapterError::DynamicSurfaceCapacityExceeded {
                requested: update.vertex_count(),
                limit: declared.profile.vertex_capacity,
            });
        }
        if update.index_count() > declared.profile.index_capacity {
            return Err(DesktopAdapterError::DynamicSurfaceCapacityExceeded {
                requested: update.index_count(),
                limit: declared.profile.index_capacity,
            });
        }
        if update
            .positions_micrometres()
            .iter()
            .any(|position| !declared.bounds.contains(*position))
        {
            return Err(invalid(
                "update vertex lies outside the declared catalog mesh bounds",
            ));
        }
        if let Some(previous) = self.current.get(&update.mesh_revision())
            && update.sequence() <= previous.sequence()
        {
            return Err(DesktopAdapterError::DynamicSurfaceSequenceRegressed {
                previous: previous.sequence(),
                actual: update.sequence(),
            });
        }
        self.publications = self
            .publications
            .checked_add(1)
            .ok_or(DesktopAdapterError::CounterOverflow)?;
        self.current.insert(update.mesh_revision(), update);
        Ok(())
    }

    pub(crate) const fn current(
        &self,
    ) -> &BTreeMap<AssetRevisionRefV1, Arc<DynamicSurfaceUpdateV1>> {
        &self.current
    }

    pub(crate) const fn publications(&self) -> u64 {
        self.publications
    }

    pub(crate) fn current_hashes(&self) -> Vec<(AssetRevisionRefV1, ContentHash)> {
        self.current
            .iter()
            .map(|(revision, update)| (*revision, update.canonical_hash()))
            .collect()
    }
}

/// Validates declared profiles without allocating device memory.
pub(crate) fn validate_profiles(
    profiles: &[DynamicSurfaceProfileV1],
    catalog: &RenderContentCatalogV1,
) -> Result<(), DesktopAdapterError> {
    if profiles.len() > MAX_DYNAMIC_SURFACES {
        return Err(invalid("too many dynamic surfaces declared"));
    }
    let mut seen = BTreeMap::new();
    for profile in profiles {
        if profile.vertex_capacity == 0 || profile.vertex_capacity > MAX_DYNAMIC_SURFACE_VERTICES {
            return Err(invalid(
                "dynamic surface vertex capacity is outside 1..=MAX",
            ));
        }
        if profile.index_capacity == 0
            || profile.index_capacity > MAX_DYNAMIC_SURFACE_INDICES
            || !profile.index_capacity.is_multiple_of(3)
        {
            return Err(invalid(
                "dynamic surface index capacity is outside 3..=MAX or not a triangle multiple",
            ));
        }
        if seen.insert(profile.mesh_revision, ()).is_some() {
            return Err(invalid("dynamic surface mesh revision declared twice"));
        }
        let mesh = catalog
            .mesh(profile.mesh_revision)
            .ok_or(DesktopAdapterError::DynamicSurfaceUndeclared)?;
        if mesh.primitives().len() != 1 {
            return Err(invalid(
                "dynamic surface catalog mesh must have exactly one primitive",
            ));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use next_contracts::ids::AssetId;

    fn revision(seed: u8) -> AssetRevisionRefV1 {
        AssetRevisionRefV1 {
            asset_id: AssetId::from_bytes([seed; 16]),
            record_sha256: domain_hash("test.dynamic-surface", &[seed]),
        }
    }

    fn triangle(sequence: u64) -> DynamicSurfaceUpdateV1 {
        DynamicSurfaceUpdateV1::new(
            revision(1),
            sequence,
            vec![[0, 0, 0], [1_000_000, 0, 0], [0, 0, 1_000_000]],
            vec![[0, i16::MAX, 0]; 3],
            vec![0, 2, 1],
        )
        .expect("valid triangle")
    }

    #[test]
    fn update_validation_fails_closed() {
        let normals = vec![[0, i16::MAX, 0]; 3];
        let positions = vec![[0, 0, 0], [1, 0, 0], [0, 0, 1]];
        assert!(
            DynamicSurfaceUpdateV1::new(
                revision(1),
                0,
                positions.clone(),
                normals.clone(),
                vec![0, 1, 2]
            )
            .is_err()
        );
        assert!(
            DynamicSurfaceUpdateV1::new(
                revision(1),
                1,
                positions.clone(),
                normals.clone(),
                vec![0, 1]
            )
            .is_err()
        );
        assert!(
            DynamicSurfaceUpdateV1::new(
                revision(1),
                1,
                positions.clone(),
                normals.clone(),
                vec![0, 1, 3]
            )
            .is_err()
        );
        assert!(
            DynamicSurfaceUpdateV1::new(
                revision(1),
                1,
                positions.clone(),
                vec![[0; 3]; 3],
                vec![0, 1, 2]
            )
            .is_err()
        );
        assert!(
            DynamicSurfaceUpdateV1::new(
                revision(1),
                1,
                positions.clone(),
                normals[..2].to_vec(),
                vec![0, 1, 2]
            )
            .is_err()
        );
        assert!(
            DynamicSurfaceUpdateV1::new(revision(1), 1, Vec::new(), Vec::new(), Vec::new())
                .is_err()
        );
        let update = DynamicSurfaceUpdateV1::new(revision(1), 1, positions, normals, vec![0, 1, 2])
            .expect("valid update");
        assert_eq!(update.vertex_count(), 3);
        assert_eq!(update.index_count(), 3);
    }

    #[test]
    fn canonical_hash_binds_payload_but_not_sequence() {
        let first = triangle(1);
        let second = triangle(2);
        assert_eq!(first.canonical_hash(), second.canonical_hash());
        assert_eq!(triangle(1).canonical_hash(), first.canonical_hash());
        let republished = first.with_sequence(9).expect("later sequence");
        assert_eq!(republished.sequence(), 9);
        assert_eq!(republished.canonical_hash(), first.canonical_hash());
        assert!(first.with_sequence(0).is_err());
        let moved = DynamicSurfaceUpdateV1::new(
            revision(1),
            1,
            vec![[0, 1, 0], [1_000_000, 0, 0], [0, 0, 1_000_000]],
            vec![[0, i16::MAX, 0]; 3],
            vec![0, 2, 1],
        )
        .expect("moved triangle");
        assert_ne!(moved.canonical_hash(), first.canonical_hash());
    }

    #[test]
    fn publication_respects_declaration_capacity_bounds_and_sequence() {
        let bounds = AabbI64V1::new([0, 0, 0], [1_000_001, 1, 1_000_001]).expect("bounds");
        let mut state = DynamicSurfaceState {
            declared: BTreeMap::from([(
                revision(1),
                DeclaredDynamicSurface {
                    profile: DynamicSurfaceProfileV1 {
                        mesh_revision: revision(1),
                        vertex_capacity: 3,
                        index_capacity: 3,
                        residency: DynamicSurfaceResidencyV1::DeviceLocal,
                    },
                    bounds,
                },
            )]),
            current: BTreeMap::new(),
            publications: 0,
        };
        state
            .publish(Arc::new(triangle(1)))
            .expect("first publication");
        assert!(matches!(
            state.publish(Arc::new(triangle(1))),
            Err(DesktopAdapterError::DynamicSurfaceSequenceRegressed {
                previous: 1,
                actual: 1
            })
        ));
        state
            .publish(Arc::new(triangle(5)))
            .expect("later sequence");
        assert_eq!(state.publications(), 2);
        assert_eq!(state.current_hashes().len(), 1);

        let undeclared = DynamicSurfaceUpdateV1::new(
            revision(2),
            1,
            vec![[0, 0, 0], [1, 0, 0], [0, 0, 1]],
            vec![[0, i16::MAX, 0]; 3],
            vec![0, 1, 2],
        )
        .expect("undeclared update");
        assert!(matches!(
            state.publish(Arc::new(undeclared)),
            Err(DesktopAdapterError::DynamicSurfaceUndeclared)
        ));

        let too_many = DynamicSurfaceUpdateV1::new(
            revision(1),
            6,
            vec![[0, 0, 0], [1, 0, 0], [0, 0, 1], [1, 0, 1]],
            vec![[0, i16::MAX, 0]; 4],
            vec![0, 1, 2],
        )
        .expect("oversized update");
        assert!(matches!(
            state.publish(Arc::new(too_many)),
            Err(DesktopAdapterError::DynamicSurfaceCapacityExceeded {
                requested: 4,
                limit: 3
            })
        ));

        let outside = DynamicSurfaceUpdateV1::new(
            revision(1),
            7,
            vec![[0, 0, 0], [1, 5, 0], [0, 0, 1]],
            vec![[0, i16::MAX, 0]; 3],
            vec![0, 1, 2],
        )
        .expect("outside update");
        assert!(matches!(
            state.publish(Arc::new(outside)),
            Err(DesktopAdapterError::DynamicSurfaceInvalid { .. })
        ));
        assert_eq!(state.current()[&revision(1)].sequence(), 5);
    }
}
