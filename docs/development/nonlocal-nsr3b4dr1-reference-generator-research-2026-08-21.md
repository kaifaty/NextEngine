# NSR3-B4DR1 reproducible external-reference generator research -- 2026-08-21

Status: `COMPLETE / R1B_PASS / R1C_DESIGN_AUTHORIZED`

## Problem

The historical W0I curves are physically useful but operationally fragile:
their full payloads, custom adapter source and tracked diff lived outside the
repository and the payloads were kept only under ephemeral `/tmp`. Hashes
prove identity when bytes exist; they do not make the bytes reproducible.

B4DR0 proves that exact restoration is unavailable. Reimplementing the prose
description and assigning the old hashes would destroy provenance. The next
comparator therefore needs a new identity and a reproducible split between
reviewable engine-owned adaptation and external upstream/build/output bytes.

## Selected architecture

Use the official SPlisHSPlasH commit
[`eccce861`](https://github.com/InteractiveComputerGraphics/SPlisHSPlasH/tree/eccce86155776f6ac52d5080b1f720a52bf29450)
as an external dependency. Track only a small NextEngine-owned standalone
adapter, patch/application manifest and build/run recipes in this research
worktree. Do not vendor or link upstream source into the engine and do not add
the external project to the Cargo workspace.

The adapter owns:

- exact scenario/sample initialization and stable sample IDs;
- pre-step position capture and post-DFSPH predictive hard-contact projection;
- outer-face and aperture plane/edge/corner geometry;
- fail-closed convergence, finite, clearance and swept-crossing validation;
- canonical `CWREFV2` serialization and a complete provenance manifest.

The upstream solver remains responsible for DFSPH density/divergence,
neighborhood search and Akinci boundary pseudo-volumes. No NextEngine water
solver code is reused by the comparator.

## Reproducibility boundary

A reference identity binds:

1. upstream commit and recursive submodule closure;
2. adapter source and application/patch hashes;
3. compiler executable/version, target, CMake generator/version, exact flags,
   linked-library closure and local license inventory;
4. binary64, AVX/FMA/fast-math/subnormal/rounding behavior;
5. OMP one-thread execution and deterministic locale/environment;
6. complete scenario/solver/contact/output manifests;
7. adapter binary and every output SHA-256.

Generated upstream source, builds, binaries, logs and trajectories stay under
an explicit external artifact root. The root is supplied by the caller and
contains a content-addressed profile directory; `/tmp` may be a disposable
staging area but is never the only retained location.

## Cost-aware execution ladder

1. **R1A bootstrap:** clone recursively at the exact upstream commit, inventory
   license/dependencies and prove the requested host toolchain/flags configure.
2. **R1B contact algebra:** implement six fixed face/aperture vectors and
   independent geometry validation before linking a trajectory.
3. **R1C 24-step preflight:** hydro, dam and orifice execute twice each; require
   byte equality, finite state, solver completion and exact clearance.
4. **R1D full generation:** run the three independent one-thread scenarios in
   at most three concurrent processes. Concurrency changes wall utilization,
   not any file or reduction order.
5. **R1E attestation:** freeze new source/binary/payload hashes and run a new
   fail-closed reader twice before B4E design.

No full generation starts after any failed earlier stage. This avoids another
42-minute-style monolithic run before cheap causal gates pass.

## Decision

Freeze the [B4DR1 contract](../plans/nonlocal-nonlinear-solver-research/03b4dr1-reference-generator-contract.md).
Only R1A external bootstrap is authorized next. B4D remains historical FAIL;
B4E remains blocked until R1E selects a new exact reference identity.

## R1A outcome

The [R1A evidence](nonlocal-nsr3b4dr1a-external-bootstrap-evidence-2026-08-21.md)
attests the pinned source/dependency/toolchain closure and two byte-identical
fresh full-clone static builds under the strict binary64/no-AVX/no-FMA profile.
It also rejects linked Git worktrees because the upstream revision probe cannot
resolve their HEAD. R1B contract/design is authorized; no adapter execution or
trajectory is authorized yet.

## R1B outcome

The [R1B evidence](nonlocal-nsr3b4dr1b-contact-adapter-evidence-2026-08-21.md)
attests the standalone adapter source/build/binary closure, all contact
branches, fail-closed process mutations and two-process byte equality. The
DFSPH symbol closure is linked but no solver step or trajectory ran. R1C
scenario-manifest design is authorized next; execution still requires a
separately frozen contract.
