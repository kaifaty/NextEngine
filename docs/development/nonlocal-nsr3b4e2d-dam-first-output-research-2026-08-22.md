# NSR3-B4E2D Dam first-output research

Date: `2026-08-22`

Status: `COMPLETE / CONTRACT_ROUTE_SELECTED / NO_TRAJECTORY_YET`

## Question

Can the selected Nonlocal continuum research solver advance the nominal
6,000-sample Dam state through four canonical macro steps, preserve its own
transactional and physical invariants, and reach the independently extracted
DFSPH first-output envelope without starting the full corpus?

## Selected bounded experiment

B4E2D is a four-macro pilot, not a production or performance run. It uses the
already selected SIRDI research path: eight deterministic owner-computes
workers, work-only transient evidence, flat CSR, invariant coefficient tape,
fused evaluation/tape construction, split incoming gathers and transaction-
local directed scratch reuse. The 900-second bound is only a watchdog.

The initial Dam state is the exact B4E0-aligned `80 x 20 x 20` box with the
same 6,000 fluid samples, 16,384 support samples, mass, density, gravity and
`1/240 s` macro time step as the external reference. The first output is step
four, so no temporal interpolation is needed.

## Cache lifetime correction

The static support index and the dynamic fluid topology cache have different
validity domains:

- the support set is immutable, so one static index is built before step one
  and retained through step four;
- the Verlet-style fluid superset is certified against an anchor state with
  a maximum admitted displacement of about `2.7 mm` (`0.04h` skin plus the
  conservative contraction certificate);
- the independent Dam first output has mean x displacement of about
  `4.87 mm`, already beyond that certificate;
- a certificate failure is deliberately terminal and has no fallback build.

Therefore one dynamic topology cache is created and destroyed inside each
macro transaction. It may be reused by all queries of that transaction, but
it never crosses a committed canonical state boundary. This is an ownership
and correctness rule, not a new optimization.

## State and failure ownership

Step `k` starts from the exact decode of committed canonical frame `k-1`.
Only the selected fine member can append one frame and one ledger entry. The
global state, frame prefix and ledger prefix advance only after the complete
transaction passes. A failure stops at its first exact stage, preserves the
already committed prefix and permits neither a retry nor coefficient/tolerance
tuning.

The pilot recomputes the global four-frame trajectory root and both global
ledger roots. It also proves the final in-memory state equals the decode of
frame four.

## Physical gates

Every step keeps the existing adaptive/KKT gates. Across all four committed
steps the pilot additionally requires:

- maximum positive density strain at most `1e-3`;
- maximum private or decoded penetration at most `2.5 mm`;
- KKT-scale publication-ledger residual at most `1e-9`, with the stricter
  normalizer retained as a finite diagnostic;
- support reaction closure at most `1e-10`;
- cumulative absolute pressure and mechanical publication deltas each at
  most `1%` of the initial energy scale;
- no energy creation beyond `1%` of that scale plus the explicit cumulative
  mechanical publication delta;
- aggregate-balanced publication impulse within its exact four-frame
  micrometre quantization bound;
- no candidate all-pairs evaluation/HVP, balanced retained/scratch ownership,
  and zero live workspaces at every transaction boundary.

## Independent-reference comparison

The reference side is frozen B4E2R integer evidence, not an in-process DFSPH
library call. Per-particle errors, pressure/divergence iteration counts,
density extrema and visual judgment are excluded because the methods do not
share those internal variables.

At step four compare only bulk observables:

- centre from the three checked position sums, normalized by the Dam box
  extents `(4, 1, 1) m`;
- q99 front `(q99_x + 25,000 um) / 4 m` and q99 height
  `(q99_y + 25,000 um) / 1 m`.

For both the three-component centre vector and the two-component q99 vector,
require normalized RMSE at most `0.05` and maximum absolute component error at
most `0.10`. These are the already selected B4E aggregate tolerances. They
are deliberately broad enough for a first-output plumbing/early-physics gate
and are not a convergence claim.

## Execution route

Build Release twice from the same frozen source. Run build A in a fresh
process under an external 900-second watchdog and without a timing wrapper.
If A fails identity, capacity, trajectory, cumulative physics or reference
comparison, stop and do not run B. If A passes, run build B and require exit
zero, empty stderr and byte-identical LF-terminated stdout.

PASS authorizes only Hydro first-output B4E2H research and contract design.
It does not authorize the full Dam/Hydro schedules, Orifice, runtime/schema,
GPU, PhysX coupling, production use or any speed claim.

