# NCGP4 unpreconditioned solver selection

| Field | Value |
| --- | --- |
| Research ID | `NCGP4` revision 2 |
| Status | `FROZEN / SELECTED_REPAIR / REPORT_ONLY` |
| Parent | NCGP4 revision 1 diagnosis contract |
| Diagnostic source | commit `74d14b4b44c0b78c9f9d53d3d926f8fb5e24c838`, tree `86d9870322775462ae957150c28ceb6b3974ae59` |
| Selected repair | use the retained unpreconditioned Steihaug--Toint profile for the scalable correctness and performance path |

## Frozen evidence

The exact hydrostatic pre-step-39 compensated state has root
`df0fe7a85f061e734e632c7ee3f4bbf10bacf477187afdf6a3d9297215f53824`.
The clean Release diagnostic binary has SHA-256
`52730083c4b26051d7d1eecb8c816020b8bbd6d3101c74a3a6f9c26db838ae77`;
its raw stdout has SHA-256
`78b40392e0981eec77e37c514ce39cae78172c642edfa9ead3de0d737e342f47`
and result root
`2f376da73d4edb8a09d5a223b43870415cfdd3e685601cb5e639b4fff52e4cce`.

On that identical state:

- scalar Jacobi terminates `WorkBudgetExceeded` after 126 HVP, 28 outer
  trials, 20 accepted trials, eight rejected trials and nine radius shrinks;
- the retained unpreconditioned profile succeeds after 69 HVP and 19 outer
  trials;
- the Jacobi path spends three extra HVP per outer trial constructing its
  scalar diagonal, while the resulting scaling increases rather than reduces
  total outer work on this witness;
- the independent same-state long-double comparison gives GPU gradient
  relative L2 `2.1398721376166474e-5`, HVP relative L2
  `1.4262815435566996e-6`, HVP cosine loss
  `9.8629643948550116e-13`, and identical active-pressure signature;
- corrected and ID-permuted trace roots are identical; deliberate event and
  work mutations are rejected.

The repaired physics apparatus also passes: actual forward/reverse free fall
has position error `0`, reconstructed-velocity error
`1.4068186282578665e-7 m/s` and reversible energy drift
`8.9605810875784662e-10`; the relative viscosity gate and complete hi/lo
transaction rollback pass. Physics suite root is
`3b8fc03d8d48f83136365ca6182f00230347fb122a9bcbb5db01428cfa15502b`.

## Hypothesis decision

- H1 as frozen is falsified: unpreconditioned is materially better, so a new
  block-Jacobi implementation is not authorized.
- H3 is falsified for this witness: the operator is within the retained gates,
  so pressure-f64 is not authorized.
- H4 is narrowed to the selected implementation choice rather than a sealed
  counting discrepancy: the scalar diagonal work is real and correctly
  counted, but counterproductive.
- H2 is observed but does not authorize trust-region tuning: the unchanged
  unpreconditioned rules complete within the frozen ceiling.

## Selected path and unchanged gates

Run the repaired NCGP4 correctness sequence with
`NonlocalGpuSolverProfile::Unpreconditioned`. The scalar diagonal probe is not
executed or counted on this path. Pressure, viscosity, surface, compensated
state, graph, contact, trust-region rules, acceptance rules, tolerances,
workload and maximum HVP remain unchanged.

For each total HVP budget in ascending order `{32, 64, 128}`, run the complete
240-step 4k corpus in the frozen order: hydrostatic hold, dam break, orifice.
Select the smallest budget for which all three pass. A failure at one budget
does not authorize a tolerance or physics change. If budget 128 does not pass,
record `REFUTED_BOUNDED` and keep performance `NOT_RUN`.

Only after the retained controls, complete 4k corpus, 16k/50k correctness and
240-step 50k sealed-basin run pass may the unchanged two-process performance
protocol from revision 1 execute. The claim ceiling and CPU DFSPH fallback are
unchanged.
