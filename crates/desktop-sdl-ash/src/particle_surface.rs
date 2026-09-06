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
/// Bytes per packed particle: three binary32 metres, one u32 of flags
/// (neighbour count in the low byte, cluster size above), three binary32
/// metres per second, one pad, then the symmetric kernel `xx xy xz yy` and
/// `yz zz 0 0` (inverse metres; all zero means isotropic at the profile
/// radius).
pub const PARTICLE_SURFACE_STRIDE: u32 = 64;

const PARTICLE_SURFACE_UPDATE_DOMAIN: &str = "nextengine.desktop.particle-surface-update.v4";
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
    /// Particles in a connected component smaller than this are spray;
    /// `0` disables the cluster criterion.
    pub spray_cluster_threshold: u32,
    /// Sub-droplets drawn per spray particle (`1..=32`).
    pub spray_subdroplets: u32,
    /// Streak length per metre per second of velocity, in seconds.
    pub spray_streak_seconds: f32,
    /// Neighbour count at which the surface splat reaches its full radius.
    pub bulk_neighbour_count: u32,
    /// Surface splat radius scale at the spray threshold, `0..=1`.
    pub edge_radius_scale: f32,
    /// Radius in pixels of the 2D narrow-range cleanup pass after the
    /// separable depth filter; `0` skips it.
    pub cleanup_radius_pixels: u32,
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
        if self.spray_subdroplets == 0 || self.spray_subdroplets > 32 {
            return Err(invalid("spray sub-droplet count must lie in 1..=32"));
        }
        if !self.spray_streak_seconds.is_finite() || self.spray_streak_seconds < 0.0 {
            return Err(invalid(
                "spray streak seconds must be finite and non-negative",
            ));
        }
        if !(0.0..=1.0).contains(&self.edge_radius_scale) {
            return Err(invalid("edge radius scale must lie in 0..=1"));
        }
        if self.cleanup_radius_pixels > 16 {
            return Err(invalid("cleanup radius must not exceed 16 pixels"));
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
    /// Connected-component size per particle; empty means unbounded.
    cluster_sizes: Vec<u16>,
    /// Velocity per particle in micrometres per second; empty means rest.
    velocities_micrometres_per_second: Vec<[i32; 3]>,
    /// Symmetric anisotropic kernel per particle (`xx xy xz yy yz zz`,
    /// inverse metres); empty means isotropic at the profile radius.
    kernels: Vec<[f32; 6]>,
    canonical_hash: ContentHash,
    packed_positions: Vec<u8>,
}

impl ParticleSurfaceUpdateV1 {
    /// Validates the bounded set and binds its content-only canonical hash.
    /// `neighbour_counts`, `cluster_sizes`, `velocities` and `kernels` are
    /// each empty or one entry per particle.
    pub fn new(
        sequence: u64,
        positions_micrometres: Vec<[i64; 3]>,
        neighbour_counts: Vec<u8>,
        cluster_sizes: Vec<u16>,
        velocities_micrometres_per_second: Vec<[i32; 3]>,
        kernels: Vec<[f32; 6]>,
    ) -> Result<Self, DesktopAdapterError> {
        if sequence == 0 {
            return Err(invalid("particle update sequence must be positive"));
        }
        if positions_micrometres.len() > MAX_PARTICLE_SURFACE_PARTICLES as usize {
            return Err(invalid(
                "particle update count exceeds the particle surface bound",
            ));
        }
        let count = positions_micrometres.len();
        if !neighbour_counts.is_empty() && neighbour_counts.len() != count {
            return Err(invalid(
                "particle neighbour counts do not match the particle count",
            ));
        }
        if !cluster_sizes.is_empty() && cluster_sizes.len() != count {
            return Err(invalid(
                "particle cluster sizes do not match the particle count",
            ));
        }
        if !velocities_micrometres_per_second.is_empty()
            && velocities_micrometres_per_second.len() != count
        {
            return Err(invalid(
                "particle velocities do not match the particle count",
            ));
        }
        if !kernels.is_empty() && kernels.len() != count {
            return Err(invalid("particle kernels do not match the particle count"));
        }
        if kernels
            .iter()
            .any(|kernel| kernel.iter().any(|value| !value.is_finite()))
        {
            return Err(invalid("particle kernel is not finite"));
        }
        let mut preimage = Vec::with_capacity(count * 63);
        let mut packed_positions = Vec::with_capacity(count * PARTICLE_SURFACE_STRIDE as usize);
        for (index, position) in positions_micrometres.iter().enumerate() {
            for component in position {
                preimage.extend_from_slice(&component.to_le_bytes());
                let metres = *component as f64 / MICROMETRES_PER_METRE;
                packed_positions.extend_from_slice(&(metres as f32).to_le_bytes());
            }
            let neighbours = neighbour_counts.get(index).copied().unwrap_or(u8::MAX);
            let cluster = cluster_sizes.get(index).copied().unwrap_or(u16::MAX);
            let velocity = velocities_micrometres_per_second
                .get(index)
                .copied()
                .unwrap_or([0; 3]);
            preimage.push(neighbours);
            preimage.extend_from_slice(&cluster.to_le_bytes());
            let flags = u32::from(neighbours) | (u32::from(cluster) << 8);
            packed_positions.extend_from_slice(&flags.to_le_bytes());
            for component in velocity {
                preimage.extend_from_slice(&component.to_le_bytes());
                let metres_per_second = f64::from(component) / MICROMETRES_PER_METRE;
                packed_positions.extend_from_slice(&(metres_per_second as f32).to_le_bytes());
            }
            packed_positions.extend_from_slice(&0_f32.to_le_bytes());
            let kernel = kernels.get(index).copied().unwrap_or([0.0; 6]);
            for value in kernel {
                preimage.extend_from_slice(&value.to_le_bytes());
                packed_positions.extend_from_slice(&value.to_le_bytes());
            }
            packed_positions.extend_from_slice(&[0_u8; 8]);
        }
        let canonical_hash = domain_hash(PARTICLE_SURFACE_UPDATE_DOMAIN, &preimage);
        Ok(Self {
            sequence,
            positions_micrometres,
            neighbour_counts,
            cluster_sizes,
            velocities_micrometres_per_second,
            kernels,
            canonical_hash,
            packed_positions,
        })
    }

    /// Particles meeting the spray criterion: fewer neighbours than
    /// `neighbour_threshold` or a component smaller than
    /// `cluster_threshold` (`0` disables either rule).
    #[must_use]
    pub fn spray_count(&self, neighbour_threshold: u32, cluster_threshold: u32) -> u32 {
        let spray = (0..self.positions_micrometres.len())
            .filter(|index| {
                let neighbours = self
                    .neighbour_counts
                    .get(*index)
                    .copied()
                    .unwrap_or(u8::MAX);
                let cluster = self.cluster_sizes.get(*index).copied().unwrap_or(u16::MAX);
                (neighbour_threshold != 0 && u32::from(neighbours) < neighbour_threshold)
                    || (cluster_threshold != 0 && u32::from(cluster) < cluster_threshold)
            })
            .count();
        u32::try_from(spray).unwrap_or(u32::MAX)
    }

    #[must_use]
    pub fn cluster_sizes(&self) -> &[u16] {
        &self.cluster_sizes
    }

    #[must_use]
    pub fn velocities_micrometres_per_second(&self) -> &[[i32; 3]] {
        &self.velocities_micrometres_per_second
    }

    #[must_use]
    pub fn kernels(&self) -> &[[f32; 6]] {
        &self.kernels
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
