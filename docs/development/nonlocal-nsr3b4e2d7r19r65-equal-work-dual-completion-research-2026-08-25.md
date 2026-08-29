# NSR3-B4E2D7R19R65 equal-work composed-dual completion research

Date: `2026-08-25`

Status: `RESEARCH COMPLETE / CONTRACT FROZEN / IMPLEMENTATION NEXT`.

## Question

v10 is correct and convergent-looking but misses the FISTA terminal pair at
78.4% of its sparse work. Can the only four additional outers guaranteed by
the same work budget close that accuracy gap?

This is not depth fitting. Twenty is derived before execution as the maximum
outer count whose conservative structural bound remains below the immutable
FISTA budget. No intermediate result may stop or extend the run.

## Fixed extension

- restart from the immutable R63 payload;
- reproduce the complete v10 prefix through outer 16 exactly;
- continue the identical composed-dual policy through outers 17--20;
- add checkpoint 20; retain 1/2/4/8/16;
- use no new tolerance, margin, preconditioner, candidate path or line rule;
- require actual sparse terms below 596,971,680 and report completed-dual local
  vector terms separately;
- compare both terminal KKT residuals strictly with FISTA.

If outer 20 does not dominate, close fixed-depth composed-dual PCG and research
a better preconditioner/direction under a new contract. Do not run outer 21 or
fit a stopping threshold from the trajectory.

Frozen contract:
[equal-work dual completion](../plans/nonlocal-nonlinear-solver-research/03b4e2d7r19r65-equal-work-dual-completion-contract.md).
