# NSR3-B4E1S Hydro spectrum research -- 2026-08-21

Status: `COMPLETE / SPECTRUM_PROBE_SELECTED / NO_KKT_OR_TRAJECTORY`

## Observation

B4E0 proves that nominal Hydro begins with nine positive compression centres,
but their maximum strain is only `6.6613381477509392e-16`. The current macro
controller branches on `active_centers > 0`, so these machine-floor centres
select the same 48-HVP spectral path as a materially compressed state.

Running a whole macro step next would mix three costs and three failure
classes: the 48-HVP spectrum, adaptive refinement selection and nonlinear KKT
solves. It would also hide a hard policy discriminator. The controller accepts
at most 192 fine substeps and requires an adjacent coarse/fine pair, so the
initial spectral count must be at most 96 before its current four-level policy
can possibly select an admissible fine level.

## Selected discriminator

Add a Hydro-only B4E1S command which reconstructs the exact B4E0 workspace,
runs the existing deterministic 48-step Lanczos estimate twice and reports:

- exact B4E0 scenario/frame/static-index/pair roots and 9-centre parent fact;
- maximum eigenvalue/eigenfrequency and derived initial substeps;
- exactly 48 HVP calls per estimate, flat-CSR work and zero all-pairs audit;
- bit-exact repeat in-process and byte-exact repeat across fresh processes;
- wall/RSS only outside the deterministic report.

The gate requires a finite nonnegative eigenvalue and
`initial_substeps <= 96`. This is not a fitted performance threshold: 96 is
derived from the already selected `fine <= 192` adjacent-level policy.

## Decision

Freeze B4E1S before any one-macro execution. A PASS may authorize only B4E1M
one-macro research/contract design. A capacity failure routes to temporal
controller redesign; a high external wall time routes to performance work.
Neither is a DFSPH comparison or a physical-water failure.
