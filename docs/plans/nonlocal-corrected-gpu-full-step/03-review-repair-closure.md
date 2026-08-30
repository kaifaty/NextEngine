# NCGP1 independent-review repair closure — revision 4

| Field | Value |
| --- | --- |
| Research ID | `NCGP1` revision 4 |
| Status | `FROZEN / ONE REPAIR BATCH / REPORT_ONLY` |
| Parent contracts | revisions 1–3 in this directory |
| Trigger | Initial independent review found load-bearing apparatus defects before the first negative route could be accepted |

This revision changes no physical coefficient, tolerance, workload or success
criterion. It freezes the only permitted repair batch before the repaired
candidate is rebuilt and rerun.

## Oracle and input identity

- Retained pair/tetra CPU graphs are direct all-pairs traversals. The 4k CPU
  implementation retains its independently written CSR traversal.
- Dynamic and ghost positions, references and velocities are explicitly
  round-tripped through IEEE binary32 before the initial CPU/GPU comparison.
  The canonical ID-sorted bytes have one common input root.
- Pressure `J^T J v` and its Jacobi diagonal include only pressure-active
  centers. Inactive and mixed-active controls compare against an analytic
  inertia result and an independent energy finite difference.

## Work and transaction identity

- A Jacobi diagonal probe that executes the full directional plus HVP kernels
  consumes one unit of the selected `{32,64,128}` total-HVP budget. Its
  `diagonal_probes` count remains visible as a subset of `hvp_applications`.
- The step work identity includes profile root, solver profile, variant and HVP
  budget. The result identity additionally seals the input root, route,
  numerical terminal, state and boundary summary.
- A failed step restores reference, current, predicted and velocity arrays.
  A post-finalization failure injection must leave the next evaluation and
  accepted step identical to a fresh workspace.

## Admission, boundary and result precedence

- Profile, dynamic state and ghosts fail closed before mutation on nonfinite,
  sign, capacity, ID-order or finite-double-to-nonfinite-binary32 failure.
- Analytical box contact uses the first swept-segment plane intersection.
  Face tests, hits, an ID-bound face-mask checksum and accepted contact impulse
  are included in the work/result receipts.
- The advected admission corpus is clamped to the exact binary32 inset bounds
  before its input root is computed; it may not begin outside the basin.
- CPU-oracle failure, input/identity failure, or corrected/permuted route/work
  disagreement is `INCONCLUSIVE`. `PHYSICS_REFUTED` is allowed only when the
  healthy CPU oracle succeeds and corrected/permuted GPU executions produce
  the same admitted `PhysicsGateFailed` result and work identities.

The required failure JSON schema is
`nextengine.nonlocal.ncgp1.result.v1`. It records contract, profile, input,
source, commit, tree, binary, work and result roots; compiler flags; exact
command; GPU/CUDA environment; and corrected, permuted and CPU receipts.
