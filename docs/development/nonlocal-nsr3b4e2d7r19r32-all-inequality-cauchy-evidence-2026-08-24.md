# NSR3-B4E2D7R19R32 all-inequality Cauchy normal-step evidence

Date: `2026-08-24`

Status: `PASS / ALL_INEQUALITY_CAUCHY_NORMAL_STEP_CANDIDATE / REPORT ONLY`

## Outcome

R32 proves that the frozen all-row hinge feasibility objective has a finite,
strictly improving matrix-free Cauchy normal step at the inherited
dimensionless global-L2 trust radius `0.25`.

```text
gradient norm                  7.0810463886017454e-9
line minimizer alpha           1.7940547251618376e-7
trust boundary active          false
initial violation norm         8.1139951163476689e-8
final violation norm           7.3077349497269772e-8
violation norm ratio           0.90063339266790055
initial hinge objective        3.2918458374056909e-15
final hinge objective          2.6701495047730572e-15
objective reduction            6.2169633263263364e-16
final directional derivative   5.9047810314627354e-26
```

The violation norm falls by about `9.94%` in one line-exact Cauchy step. The
minimum is interior, and the direct final directional derivative closes at a
relative `8.33885e-18`, below the frozen `1e-10` KKT gate. Direct JVP/VJP
adjoint agreement closes at relative `1.19212e-17`.

The active set changes materially: 208 initially inactive rows enter, 178
initially active rows leave, and 1,450 rows are positive at the minimizer.
This is the expected behavior after R31 rejected scalar damping of a fixed
violated-row solve. A valid normal-step method must account for all frozen
inequalities and allow the active set to change inside the subproblem.

## Controls and authority

All four analytic controls pass: interior optimum, trust-boundary optimum,
inactive-row entry and simultaneous events. Binary128 orders breakpoint
events with stable row-ID ties; all ten route cases, exact R31 parent bytes,
source roots, direct KKT checks, pair-pass ledger and rollback pass.

R32 adds exactly one VJP and one JVP, hence two pair passes. It forms no dense
Jacobian, HVP, nonlinear model evaluation, trial, outer update or correction.
No position or dual state is mutated. No floor, timing, runtime or production
authority is claimed.

## Reproducibility

Research/contract commit: `f962db39`.

Implementation commit: `cbc4b3d9`.

Two clean Release builds:

- `/home/kaifaty/.cache/nextengine/build-nonlocal-b4e2d7r19r32-a.WPbKO1`;
- `/home/kaifaty/.cache/nextengine/build-nonlocal-b4e2d7r19r32-b.wl23fg`.

Both binaries are `7,088,352` bytes, have SHA-256
`5dc15cfd5c9b7376154c484897419fde630ae8803c4f4f7b65aef4a9d4056f0e`
and GNU build ID `b914ce4bc7822529307245ead1eb09046a83fcfa`.

Fresh one-process outputs:

- `/home/kaifaty/.cache/nextengine/run-nonlocal-b4e2d7r19r32-a.WxUiQX`;
- `/home/kaifaty/.cache/nextengine/run-nonlocal-b4e2d7r19r32-b.ZqAquJ`.

Both exit `0`, have empty stderr and reproduce `2,444` stdout bytes exactly:

```text
stdout SHA-256  243a115d66a33667d57f2fec73ca58553c57f00d83a235dcb8640d04f1d701db
semantic        2417db008424792e6b000beebf50999c2e4b71493d0cfb0d7df69ae2e14ba695
route           ALL_INEQUALITY_CAUCHY_NORMAL_STEP_CANDIDATE
```

These are correctness/reproducibility runs, not timing or performance
measurements.

## Next action

Research and freeze a bounded R33 iterative all-inequality normal-step
discriminator. It must reuse the R32 objective and trust geometry, declare a
strict pair-pass budget before execution and show whether repeated
active-set-aware progress materially improves on the one-step baseline. Do
not apply the step, evaluate a moved nonlinear state or choose production
integration yet.
