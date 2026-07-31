use super::*;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
/// Report-only counters for one reusable B0 frame planner.
///
/// These counters are presentation diagnostics and never participate in a
/// gameplay, snapshot, replay, or frame-plan hash.
pub struct B0FramePlannerMetricsV1 {
    pub cache_hits: u64,
    pub cache_misses: u64,
    pub build_failures: u64,
    pub explicit_invalidations: u64,
}

#[derive(Clone, Debug)]
struct CachedB0FramePlanV1 {
    snapshot: PresentationSnapshotV2,
    catalog_hash: ContentHash,
    target: RenderTargetV1,
    plan: B0FramePlanV1,
}

impl CachedB0FramePlanV1 {
    fn matches(
        &self,
        snapshot: &PresentationSnapshotV2,
        catalog_hash: ContentHash,
        target: RenderTargetV1,
    ) -> bool {
        &self.snapshot == snapshot && self.catalog_hash == catalog_hash && self.target == target
    }
}

#[derive(Clone, Debug)]
/// Reuses one validated B0 frame plan and the allocations needed to replace it.
///
/// Cache hits require exact snapshot value equality rather than trusting the
/// public `canonical_hash` field alone. A caller that mutates a snapshot while
/// leaving a stale hash therefore takes the normal validating miss path and
/// cannot expose the prior plan as a false hit.
pub struct B0FramePlannerV1 {
    cached: Option<CachedB0FramePlanV1>,
    staged_draws: Vec<B0IndexedDrawV1>,
    hash_preimage: Vec<u8>,
    metrics: B0FramePlannerMetricsV1,
}

impl B0FramePlannerV1 {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            cached: None,
            staged_draws: Vec::new(),
            hash_preimage: Vec::new(),
            metrics: B0FramePlannerMetricsV1 {
                cache_hits: 0,
                cache_misses: 0,
                build_failures: 0,
                explicit_invalidations: 0,
            },
        }
    }

    #[must_use]
    pub const fn metrics(&self) -> B0FramePlannerMetricsV1 {
        self.metrics
    }

    #[must_use]
    pub fn cached_plan(&self) -> Option<&B0FramePlanV1> {
        self.cached.as_ref().map(|cached| &cached.plan)
    }

    pub fn build_or_reuse(
        &mut self,
        snapshot: &PresentationSnapshotV2,
        catalog: &RenderContentCatalogV1,
        target: RenderTargetV1,
    ) -> Result<&B0FramePlanV1, RenderDeviceError> {
        let catalog_hash = catalog.catalog_sha256();
        let cache_hit = self
            .cached
            .as_ref()
            .is_some_and(|cached| cached.matches(snapshot, catalog_hash, target));
        if cache_hit {
            self.metrics.cache_hits = self.metrics.cache_hits.saturating_add(1);
            return Ok(&self.cached.as_ref().expect("cache hit was checked").plan);
        }

        self.metrics.cache_misses = self.metrics.cache_misses.saturating_add(1);
        let parts = match build_b0_frame_plan_parts(
            snapshot,
            catalog,
            target,
            &mut self.staged_draws,
            &mut self.hash_preimage,
        ) {
            Ok(parts) => parts,
            Err(error) => {
                self.metrics.build_failures = self.metrics.build_failures.saturating_add(1);
                return Err(error);
            }
        };
        let candidate = parts.finish(std::mem::take(&mut self.staged_draws));

        let cached_snapshot = if let Some(CachedB0FramePlanV1 {
            snapshot: mut prior_snapshot,
            plan: prior_plan,
            ..
        }) = self.cached.take()
        {
            prior_snapshot.clone_from(snapshot);
            self.staged_draws = prior_plan.draws;
            prior_snapshot
        } else {
            snapshot.clone()
        };
        self.cached = Some(CachedB0FramePlanV1 {
            snapshot: cached_snapshot,
            catalog_hash,
            target,
            plan: candidate,
        });
        Ok(&self.cached.as_ref().expect("candidate was cached").plan)
    }

    pub fn invalidate(&mut self) {
        let Some(cached) = self.cached.take() else {
            return;
        };
        self.staged_draws = cached.plan.draws;
        self.metrics.explicit_invalidations = self.metrics.explicit_invalidations.saturating_add(1);
    }

    pub fn reset_metrics(&mut self) {
        self.metrics = B0FramePlannerMetricsV1::default();
    }
}

impl Default for B0FramePlannerV1 {
    fn default() -> Self {
        Self::new()
    }
}
