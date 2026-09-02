//! Presentation-only particle surface boundary (ADR-102).
//!
//! One bounded particle set per desktop run may be replaced per frame by the
//! application. The adapter renders it as a screen-space fluid after the
//! opaque world pass. The set is a reconstructible renderer cache: it never
//! feeds back into simulation and never participates in a gameplay,
//! snapshot, replay or frame-plan root.

use next_contracts::ids::ContentHash;
use next_contracts::project::domain_hash;
use next_contracts::render_content::AabbI64V1;

use crate::DesktopAdapterError;

/// Upper particle bound for the one declared particle surface.
pub const MAX_PARTICLE_SURFACE_PARTICLES: u32 = 65_536;
/// Bytes per packed particle: three binary32 metres plus one u32
/// neighbour count.
pub const PARTICLE_SURFACE_STRIDE: u32 = 16;

const PARTICLE_SURFACE_UPDATE_DOMAIN: &str = "nextengine.desktop.particle-surface-update.v2";
const MICROMETRES_PER_METRE: f64 = 1_000_000.0;

/// Declares the one bounded particle surface for the whole desktop run.
///
/// Every value is a renderer-local presentation constant; nothing here is
/// read by gameplay. Capacity never grows at runtime.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ParticleSurfaceProfileV1 {
    pub particle_capacity: u32,
    /// Sphere impostor radius; also the smoothing band unit.
    pub radius_micrometres: u32,
    /// Every published position must lie inside these bounds.
    pub bounds: AabbI64V1,
    /// Per-metre absorption of the refracted scene colour, linear RGB.
    pub absorption_per_metre: [f32; 3],
    /// Screen-space refraction offset in texture units per metre of
    /// thickness, capped by the shader at one metre.
    pub refraction_strength: f32,
    /// Multiplier on the accumulated sphere thickness.
    pub thickness_scale: f32,
    /// Particles with fewer neighbours than this leave the surface splat
    /// and are drawn as spray; `0` keeps every particle in the surface.
    pub spray_neighbour_threshold: u32,
    /// Disc radius of one spray particle.
    pub spray_radius_micrometres: u32,
    /// Peak opacity of one spray disc, `0..=1`.
    pub spray_alpha: f32,
}

impl ParticleSurfaceProfileV1 {
    pub fn validate(&self) -> Result<(), DesktopAdapterError> {
        if self.particle_capacity == 0 || self.particle_capacity > MAX_PARTICLE_SURFACE_PARTICLES {
            return Err(invalid("particle capacity is outside the declared bound"));
        }
        if self.radius_micrometres == 0 || self.radius_micrometres > 1_000_000 {
            return Err(invalid("particle radius must lie in (0, 1 m]"));
        }
        if self.spray_radius_micrometres > 1_000_000 {
            return Err(invalid("spray radius must not exceed 1 m"));
        }
        if !(0.0..=1.0).contains(&self.spray_alpha) {
            return Err(invalid("spray alpha must lie in 0..=1"));
        }
        let finite = self
            .absorption_per_metre
            .iter()
            .chain([&self.refraction_strength, &self.thickness_scale])
            .all(|value| value.is_finite() && *value >= 0.0);
        if !finite {
            return Err(invalid(
                "particle shading constants must be finite and non-negative",
            ));
        }
        Ok(())
    }
}

/// One immutable replacement of the whole particle set.
#[derive(Clone, Debug, PartialEq)]
pub struct ParticleSurfaceUpdateV1 {
    sequence: u64,
    positions_micrometres: Vec<[i64; 3]>,
    /// Fluid neighbours per particle within the producer's presentation
    /// radius; empty means every particle counts as bulk.
    neighbour_counts: Vec<u8>,
    canonical_hash: ContentHash,
    packed_positions: Vec<u8>,
}

impl ParticleSurfaceUpdateV1 {
    /// Validates the bounded set and binds its content-only canonical hash.
    /// `neighbour_counts` is empty or one count per particle.
    pub fn new(
        sequence: u64,
        positions_micrometres: Vec<[i64; 3]>,
        neighbour_counts: Vec<u8>,
    ) -> Result<Self, DesktopAdapterError> {
        if sequence == 0 {
            return Err(invalid("particle update sequence must be positive"));
        }
        if positions_micrometres.len() > MAX_PARTICLE_SURFACE_PARTICLES as usize {
            return Err(invalid(
                "particle update count exceeds the particle surface bound",
            ));
        }
        if !neighbour_counts.is_empty() && neighbour_counts.len() != positions_micrometres.len() {
            return Err(invalid(
                "particle neighbour counts do not match the particle count",
            ));
        }
        let mut preimage = Vec::with_capacity(positions_micrometres.len() * 25);
        let mut packed_positions = Vec::with_capacity(positions_micrometres.len() * 16);
        for (index, position) in positions_micrometres.iter().enumerate() {
            for component in position {
                preimage.extend_from_slice(&component.to_le_bytes());
                let metres = *component as f64 / MICROMETRES_PER_METRE;
                packed_positions.extend_from_slice(&(metres as f32).to_le_bytes());
            }
            let neighbours = neighbour_counts.get(index).copied().unwrap_or(u8::MAX);
            preimage.push(neighbours);
            packed_positions.extend_from_slice(&u32::from(neighbours).to_le_bytes());
        }
        let canonical_hash = domain_hash(PARTICLE_SURFACE_UPDATE_DOMAIN, &preimage);
        Ok(Self {
            sequence,
            positions_micrometres,
            neighbour_counts,
            canonical_hash,
            packed_positions,
        })
    }

    /// Particles below the threshold (drawn as spray); `0` for no split.
    #[must_use]
    pub fn spray_count(&self, threshold: u32) -> u32 {
        if threshold == 0 {
            return 0;
        }
        let spray = self
            .neighbour_counts
            .iter()
            .filter(|count| u32::from(**count) < threshold)
            .count();
        u32::try_from(spray).unwrap_or(u32::MAX)
    }

    #[must_use]
    pub fn neighbour_counts(&self) -> &[u8] {
        &self.neighbour_counts
    }

    pub fn with_sequence(&self, sequence: u64) -> Result<Self, DesktopAdapterError> {
        if sequence == 0 {
            return Err(invalid("particle update sequence must be positive"));
        }
        Ok(Self {
            sequence,
            ..self.clone()
        })
    }

    #[must_use]
    pub const fn sequence(&self) -> u64 {
        self.sequence
    }

    #[must_use]
    pub fn particle_count(&self) -> u32 {
        u32::try_from(self.positions_micrometres.len()).unwrap_or(u32::MAX)
    }

    #[must_use]
    pub fn positions_micrometres(&self) -> &[[i64; 3]] {
        &self.positions_micrometres
    }

    #[must_use]
    pub const fn canonical_hash(&self) -> ContentHash {
        self.canonical_hash
    }

    #[must_use]
    pub fn packed_positions(&self) -> &[u8] {
        &self.packed_positions
    }
}

const fn invalid(reason: &'static str) -> DesktopAdapterError {
    DesktopAdapterError::ParticleSurfaceInvalid { reason }
}
