# 04 — GPU water correspondence

## Outcome

Implement an accelerated DFSPH mirror and measure correspondence with the CPU
oracle. This package cannot promote GPU output to authoritative state.

## Exact run identity

Every run binds device/vendor ID, driver, OS/target, API and shader/kernel
versions, compiler flags, float controls, workgroup sizes, buffer layout,
neighbor construction, reduction strategy, time-step/profile hash and scenario
corpus hash. Fast-math/reassociation settings are explicit.

## Comparison

Compare CPU/GPU per-frame mass, center of mass, momentum, density/divergence
residuals, boundary impulse, free-surface landmarks and quantized trajectory
summaries. Report maximum, percentile and first-divergence values. Tolerances
are declared before the run and cannot be widened after inspecting failures.

Run on at least one supported Windows and one supported Linux profile before
any cross-target claim. Vary dispatch/workgroup shape only as a named profile;
completion order cannot select an authoritative tick.

## Failure and exit

NaN, non-convergence, capacity excess, device loss, stale input or tolerance
failure rejects the mirror result and leaves CPU evidence intact. Exit is
`CONTINUUM-MIRROR-P1 = PASS` for the named devices and budgets plus a report of
known drift. The next decision must explicitly choose one of:

- mirror/quality-only GPU forever;
- recorded authoritative result/state;
- device/compiler-closed authority with compatibility rejection;
- deterministic reduced gameplay model with GPU presentation detail.

Quantizing the final GPU state by itself is not an admissible decision.
