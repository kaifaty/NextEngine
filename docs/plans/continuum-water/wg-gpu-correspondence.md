# WG — Optional GPU correspondence mirror

## Outcome

Measure an accelerated DFSPH mirror against the passing CPU oracle without
placing GPU output on the authoritative water, reaction, save or gameplay
path. WG is optional and never blocks W2–W6.

## Exact run identity

Every run binds device/vendor ID, driver, OS/target, API, shader/kernel source
and binary hashes, compiler and float-control flags, workgroup sizes, buffer
layout, neighbor/reduction strategy, profile/scenario/corpus hashes and the CPU
oracle commit/root. Fast math and reassociation are explicit; ambient device
defaults are not evidence identity.

## Comparison

Compare at fixed output boundaries:

- sample count/mass and failure class;
- density/divergence residual distributions and iteration counts;
- centre of mass, linear/angular momentum and external-work-aware energy;
- analytical-boundary penetration and reaction impulse;
- dam-break/front/surface landmarks;
- quantized aggregate trajectory summaries and first-divergence step.

Declare absolute/relative metrics, sample populations, maximum/percentile
thresholds and invalid-run rules before execution. CPU/GPU particle bytes and
final quantized states are not expected to be identical, and final
quantization cannot convert a divergent trajectory into canonical evidence.

## Execution boundary

The mirror consumes an immutable CPU-scenario input and produces an immutable
comparison result. It may execute device-asynchronously, but completion timing
cannot select a gameplay tick, CPU state or fallback. A future coupled GPU
authority would have to be logically synchronous inside PhysicalStep and
requires a new Accepted authority/replay decision; WG does not prepare or
imply that promotion.

## Failure and exit

Device loss, nonfinite result, non-convergence, capacity excess, stale input,
result collision or tolerance failure rejects only the mirror result and
leaves CPU evidence intact. `CONTINUUM-MIRROR-P1` records PASS/FAIL per exact
named device/profile and known drift; it grants no public contract or shipped
authority.

## Non-goals

GPU authority, deterministic cross-device claim, gameplay fallback selection,
reaction publication, save format, GPU-first W2 bypass and surface rendering.
