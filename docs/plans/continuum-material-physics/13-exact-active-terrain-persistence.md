# Package 13T — Exact active terrain persistence

## Outcome

Persist the modified dry-sand patch in its exact active representation before
adding saturation or any lossy sleep/streaming model.

## Owner segment

The future composite physical checkpoint stores patch/profile/content
identity, completed tick/substep, exact conservation totals and the complete
sorted sample state: stable ID, mass where non-uniform, position, velocity,
affine velocity, deformation gradient and plastic/internal fields. Exact field
formats are those frozen in Package 10T.

The MPM grid, stencil contributions, SVD/stress scratch, contact patch surface
and render mesh are reconstructed and never persisted. The terrain segment
commits atomically with the rigid physical checkpoint; no sidecar can succeed
or fail separately.

## Evidence

- deform → save at N → process restart → resume to M versus uninterrupted
  exact material/rigid roots;
- repeated-pass and post-unloading checkpoints;
- `game`/deterministic `headless` root parity;
- insertion/decode order canonicalization;
- `N-1/N/N+1` sample/byte/matrix/segment bounds;
- corrupt matrix/state, duplicate/missing ID, stale profile/content/wheel
  mapping, root mismatch and publication fault before mutation.

The patch stays pinned active. `owner-root parity` means identical active
representation and continuation; it does not compare active state to a
compact/sleep approximation.

## Exit

Exact active terrain persistence passes before Package 14T may add saturation.
Failure retains the prior world/save and keeps wet terrain blocked.

## Non-goals

Sleep/wake, unloaded modified patch, lossy compression, region transfer,
saturation, pore pressure, water exchange and migration of unshipped formats.
