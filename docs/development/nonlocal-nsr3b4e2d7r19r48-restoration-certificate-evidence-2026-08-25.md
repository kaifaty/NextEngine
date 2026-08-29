# NSR3-B4E2D7R19R48 restoration-certificate evidence

Date: `2026-08-25`

Status: `PASS / RESTORATION_CERTIFICATE_UNRESOLVED / ROLLBACK ONLY`.

Implementation commit: `c20a7da3`.

Frozen v2 identity SHA-256:
`b20c6c6b530bd3078de7707e87c8deacf8eae0dab581584cf30421e5a1d52332`.

## Result

R48 reproduces exact R47 and rebuilds the next-TRQP linearized inequality
operator at the exact filter-acceptable R43 moved point. The frozen
Malitsky--Pock PDAL generator makes substantial progress but neither
authoritative certificate closes:

```text
source psi                     6.8540208964482797e-16
source h                       3.7024372773750754e-08
generated psi                  1.3258397316434649e-17
generated h                    5.1494460510689202e-09
h reduction                    7.1899719710754608x
psi reduction                  51.695696944850745x
directed maximum row upper     4.6927711453206623e-10
certified positive rows        1042
dual lower bound              -8.1696003125403582e-12
route                           RESTORATION_CERTIFICATE_UNRESOLVED
```

The negative dual lower bound does not prove local infeasibility, while the
positive directed row upper bounds do not prove primal compatibility. The
route therefore owns only an unresolved bounded reference result. It is not a
physics failure and does not establish that no compatible normal witness
exists.

No normal witness, R43 state, restoration point, filter entry or following
outer was committed. Restoration exit, runtime and production authority
remain false.

## Geometry and generator

V1 had stopped before solver work because its dimensionless half-skin
assumption was invalid; that diagnostic retains no solver credit. V2 uses the
independently reclosed dyadic geometry:

```text
next trust radius              0.0625
normal radius                  0.03125
R43 displacement norm          1.0290544256239871e-06
exact inherited half-skin      0.059999999999940004
active contact components      1290
```

The generated witness norm is only `3.7060636244142894e-7`, or
`1.1859403598125726e-5` of the normal radius. Trust/contact geometry is
therefore not the observed limiting mechanism. The frozen generator completes
128 main iterations, 193 line attempts and 65 extra backtracks, ending at
`tau=5.6579614875862019`. Its checkpoint root is
`ba1b8163e377c983583b3b69978b2249264b6b368aecc02068143afcde2ee8dc`.

## Certificates, work and controls

The primal authority is a rowwise directed-JVP forward enclosure, independent
of the generator's own residual. The dual authority is the outward-rounded
Fenchel lower bound using a separately audited support upper bound. Neither
sign is unresolved in arithmetic, so binary128 is correctly not executed.

Exact new work is 128 JVPs, 194 VJPs, 128 projection scans and one support
audit. Twelve precedence routes close at route root
`4a2cfcaeabacc06dd026d08d6439c809feb8458afd48ece68f11da3a4bc16d96`.
Projection, support, dense feasible/infeasible, weak-duality, rounding,
parent, source, workspace, geometry, work and rollback controls pass.

Semantic result SHA-256:
`7d83b855aeda6c32202e197c65c94f88235e6e997563f7847ddb5972d88a7e91`.

## Clean Release reproducibility

```text
/home/kaifaty/.cache/nextengine/build-nonlocal-b4e2d7r19r48-a.KIlDpK
/home/kaifaty/.cache/nextengine/build-nonlocal-b4e2d7r19r48-b.0fGFXI
binary SHA-256 acfc341aecaa8564ce2bb53d455f1158ba4ddcd6080ff8cd3a193a3bde88d4bf
size           7819280
ELF build-id   0b5d7c83b478f9188e4a683cee8119421decfd2e

/home/kaifaty/.cache/nextengine/run-nonlocal-b4e2d7r19r48-a.DhKy9E
/home/kaifaty/.cache/nextengine/run-nonlocal-b4e2d7r19r48-b.0KpCgc
stdout bytes   1834
stdout SHA-256 70a66443d0a278697e3e4d8d4456f80f5f1ccb1269596d3e5ce569e87d9b90b6
```

Both binaries and stdout payloads are byte-exact. Each run reproduces the
exact R47 parent stdout SHA
`dac0ffd871c927cb62253f64a078866d9515faefda59e48a54d923089dbc371d`.
These are correctness/reproducibility runs, not timing evidence.

## Consequence

R48 validates the specialized matrix-free problem, contact/trust geometry,
independent primal/dual certificate layer and exact rollback. It also rejects
the frozen first-order PDAL recurrence as a sufficient witness generator under
its bounded work budget. Raising the cap would repeat an established slow
first-order regime rather than test a new mechanism.

The next research stage should preserve the R48 problem and certificate
authorities but replace only the primal candidate generator. The leading
hypothesis is a contact-box/trust-feasible guarded Hager--Zhang recurrence with
exact hinge line minimization, because R38--R39 already established large
equal-work dominance on the related all-inequality linearized problem. No
restoration transaction or runtime integration is authorized until the
independent primal certificate closes.
