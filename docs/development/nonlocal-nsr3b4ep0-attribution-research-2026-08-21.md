# NSR3-B4EP0 nominal cost-attribution research -- 2026-08-21

Status: `COMPLETE / GPROF_ATTRIBUTION_SELECTED / NO_OPTIMIZATION_YET`

## Question

Which serial stage actually dominates the 48.8-second B4E1M macro, and which
single optimization should be tested first without changing solver decisions?

## Evidence before profiling

B4E1M executes 42 attempted substeps, 221 outer trials, 226 joint evaluations,
459 HVPs and 227 complete flat workspaces. Those workspaces materialize
151,461,068 directed adjacency records. These counts establish possible cost
centres but do not establish their wall-time shares.

Code inspection adds four competing hypotheses:

1. `joint_workspace_hash` serializes and SHA-hashes every fluid/support state,
   pair list and pressure tape unconditionally. `record_queries=false` only
   suppresses the metric vector; it does not suppress these hashes.
2. Every accepted/current/trial state rebuilds cell ranges, the exact pair
   list, flat CSR, density/gradient evaluation, radius/compression tape and
   their allocations. Only the immutable support index is reused.
3. Each HVP allocates joint/result vectors and traverses active directed pairs
   twice per pressure-active centre. The scatter form prevents naive parallel
   writes without a deterministic reduction design.
4. The nonlinear policy requires 221 serial outer trials for 42 attempted
   substeps. Even individually efficient kernels can remain too expensive at
   this count.

The first remedy depends on attribution. Hash-policy separation is appropriate
only if evidence hashing dominates; a Verlet/superset topology is appropriate
only if rebuilds dominate; a deterministic pair-contribution kernel is
appropriate only if HVP dominates; and predictor/globalization research is
appropriate only if serial outer count dominates after kernel costs.

## Profiler selection

Linux `perf` 7.0.12 is installed but rejects even process-local events because
`kernel.perf_event_paranoid=4`. Changing a machine-wide sysctl is unnecessary
for this research and is explicitly excluded.

GNU gprof 2.46 is available without privilege changes. Build a separate
external Release executable with GCC 15.2, `-O3 -DNDEBUG -g -pg` and `-pg` at
link. Run the exact B4E1M command once under its 900-second watchdog, require
stdout SHA equality with the uninstrumented oracle, then generate flat/call-
graph reports from a nonempty `gmon.out`.

gprof is statistical and its instrumentation perturbs absolute runtime. Its
self-time shares select the next controlled Release ablation; they are not a
production benchmark and do not replace B4E1M external timing.

## Routing rule

- hashing/stream formatting dominant: B4EP1 separates full research evidence
  from the physics hot path and measures a no-inner-hash Release candidate;
- neighborhood/evaluation/tape dominant: B4EP1 designs exact filtered Verlet
  topology plus in-place tape refresh;
- HVP dominant: B4EP1 designs per-pair contribution buffers and deterministic
  ordered reduction before parallel execution;
- no dominant leaf or outer-policy overhead dominant: add scoped internal
  timing next, then consider warm-start/globalization changes only under a new
  mathematical solver identity.

Any profiler run whose stdout differs from B4E1M fails correspondence and
cannot select an optimization.

## Decision

Freeze B4EP0 as one external gprof attribution run. Make no solver/source
change, do not run B4E2 and do not infer GPU speedup from CPU samples. A PASS
may authorize only one measured B4EP1 ablation contract.
