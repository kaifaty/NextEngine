# NSR3-B4E2D4 step-two strain-refinement research

Date: `2026-08-22`

Status: `RESEARCHED / CONTRACT_READY / NOT_RUN`

## Question

B4E2D3's step-two transaction passes its embedded 40/80 adjacent-state gate,
nonlinear solve and publication, but its maximum compression strain is
`0.0011747197409319732` versus the material cap `0.001`. The next experiment
must distinguish a controller admission defect from compression intrinsic to
the finite-`KAPPA` penalty trajectory.

Changing `KAPPA`, the physical cap or a solver tolerance before that
distinction would conflate temporal and material-model errors. Repeating the
same adaptive pilot would add no information.

## Experiment

Reproduce the exact B4E2D3 step-one commit and failed step-two adaptive
transaction. Reuse its accepted private 80-substep lane, then run independent
fixed 160- and 320-substep lanes from the exact committed step-one decoded
state. All lanes retain the selected SIRDI work path, while each fixed lane
owns a fresh certified topology cache and directed scratch lifetime.

The 160/320 pair is temporally resolved for this discriminator only if:

1. the existing adjacent `smoke_gate(160, 320)` passes; and
2. their maximum-strain difference is no more than `0.00005`, five percent of
   the material cap.

The threshold is frozen before execution. It is not a production accuracy
tolerance.

## Routes

| Condition | Route | Meaning |
|---|---|---|
| private 80 strain is at most `0.001`, but decoded 80 strain exceeds it | `PUBLICATION_STRAIN_ADMISSION_MISSING` | canonical publication alone creates the observed violation |
| resolved and strain at 320 is at most `0.001` | `ADAPTIVE_DENSITY_ADMISSION_MISSING` | existing state-only adaptive gate can commit a physically under-resolved lane |
| resolved and strain at 320 exceeds `0.001` | `FINITE_PENALTY_COMPRESSIBILITY` | temporal refinement does not satisfy the material cap; revisit constraint/penalty formulation |
| otherwise | `TEMPORAL_UNRESOLVED` | extend a separately bounded refinement experiment before redesign |

A diagnostic PASS means the exact experiment completed and emitted one route;
it does not mean that the Dam trajectory or Nonlocal solver is production
ready. Numerical lane failure is a command FAIL, not a route.

## Rejected alternatives

- increasing substeps inside B4E2D3 without a new identity;
- tuning `KAPPA` or relaxing `0.001`;
- interpreting elapsed time on the shared host;
- running steps three/four or comparing to the external step-four output;
- using only terminal state convergence while ignoring the peak strain path;
- merging private peak strain with decoded published-frame strain in the
  diagnostic report.
