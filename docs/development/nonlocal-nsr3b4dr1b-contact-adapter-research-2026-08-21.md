# NSR3-B4DR1B standalone contact-adapter research -- 2026-08-21

Status: `COMPLETE / CONTRACT_FROZEN / IMPLEMENTATION_AUTHORIZED`

## Problem

R1A proves that the independent DFSPH library can be rebuilt exactly, but the
lost W0I adapter also owned hard geometry. Running even a 24-step trajectory
before re-closing that algebra would mix solver behavior with an unaudited
collision implementation and could recreate the old wall-penetration error.

The parent six-vector list also leaves two subtle branches implicit: collision
with the solid part of the internal plane and the fractional-ulp already-
touching restart that was found during the historical 24-step run.

## Selected design

Build one standalone NextEngine-owned C++17 research tool. Its own vector,
quadratic, feature-selection, response and validation code is separate from
both the Rust water oracle and SPlisHSPlasH. The pinned external library is
linked only to expose the actual `SPH::Real` ABI and a DFSPH method anchor at
this stage.

The tool has three ordered layers:

```text
process/ABI preflight
        -> independent analytical contact
        -> separate clearance/chord validator
```

No layer can initialize a particle world. The preflight proves the environment
rather than repairing it. Contact uses the historical eight-hit frictionless
tail projection and feature-time tie order. The validator sees only input and
accepted endpoints and independently checks geometry.

## Fixture decision

Retain the six parent fixtures for outer clamp, tunnelling, opening pass,
tangent edge, edge deflection and exact corner-sphere ownership. Add two
sentinels inside the same executable rather than expanding to a trajectory:

- an internal solid-face crossing selects feature `16`;
- a one-ulp embedded restart selects the edge at `t=0`.

Canonical micrometre outputs are frozen where binary64 square roots may differ
in their insignificant tail. Feature counts, non-finite rejection, schedule
completion and chord legality remain exact.

## Reproducibility decision

The executable binds the R1A static archive, compiler and flags, plus its own
tracked source root and final link command. Default output is canonical text
with only identity/ABI/environment/vector results; no wall time or host path is
printed. Two fresh processes must match byte-for-byte.

The canonical profile has two structural mutation tests: swap edge order
`17/18`, and increment the radius bit pattern by one. Rounding and FTZ negative
modes prove the process preflight fails before `contact_started`.

## Rejected alternatives

- Calling the Rust W0F contact implementation: not independent.
- Patching SPlisHSPlasH collision behavior directly: mixes upstream solver
  identity with engine-owned reference geometry and makes review harder.
- Validating only final points: a straight accepted chord could still cross
  the wall outside the clearance-safe opening.
- Using epsilons for hit order or tangency: changes the closed-opening and
  stable-feature semantics.
- Beginning the 24-step orifice run now: R1C manifests and serialization are
  not frozen.

## Pre-implementation geometry reclosure

The first frozen draft accidentally used `[0,1]^3` both as the outer-clamp
fixture and as the orifice outer box, placing internal wall `x=1` on the outer
face. That would classify a legal aperture endpoint at `x=1.025` as an outer
escape. No code or test ran under that draft. Contract v2 keeps `[0,1]^3` for
outer controls and uses the real `[0,2] x [0,1] x [0,1]` orifice box for
internal vectors. The rejected v1 root remains recorded as negative evidence.

## Decision

Freeze the [R1B contract](../plans/nonlocal-nonlinear-solver-research/03b4dr1b-contact-adapter-contract.md)
and implement only this no-trajectory self-test next. R1C and B4E remain
blocked.
