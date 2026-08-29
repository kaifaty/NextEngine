# NSR3-B4E2D7R19R65 dynamic all-row exploratory probe evidence

Date: `2026-08-25`

Status: `EXPLORATORY PASS / NO SCIENTIFIC CREDIT / ACTIVE-FACE
ACCELERATION REQUIRED`.

Probe implementation commit: `1edf4907` (latest `omega=1.5` variant).

All variants preserve exact R63 parent stdout
`38298214e352a87cfffb5c5432be90ef822c3f8ab00da5a2f7a3b1cb64565b62`,
exact joint reprojection, rollback and the no-runtime/no-timing boundary.

## Stable structural result

The fresh target audit initially owns 1080 row-local candidate-positive rows.
Monotone constraint generation reaches 4680 owned rows before checkpoint 8;
no checkpoint from 8 through 2048 adds an outside-positive row. The working-set
problem is therefore solved: subsequent failure is convergence inside the
owned active face, not missing constraint discovery.

The ascending baseline performs:

```text
cycles                         2048
coordinate visits          9,578,160
coordinate updates         4,657,492
row-entry terms          899,820,896
all-row audits                 2049
dense Gram / Gram updates       0 / 0
```

Stationarity stays between `1.77e-20` and `3.30e-20`, joint reprojection is
bit-exact at every checkpoint and model reduction remains positive. Thus the
dual/primal decomposition is internally consistent.

## Variant comparison

Maximum fresh raw density residual:

| cycles | ascending `omega=1` | fresh-raw descending `omega=1` | ascending `omega=1.5` |
|---:|---:|---:|---:|
| 64 | `1.3949479181178364e-9` | `4.0463403401871395e-9` | `3.9595935263278441e-10` |
| 256 | `4.5668147427388085e-11` | `2.8586006722134624e-10` | `4.868067424294992e-11` |
| 2048 | `9.169217430587303e-12` | `1.6475696463407043e-11` | `2.766668054796938e-12` |

At 2048 cycles, candidate-positive row counts are 1132, 1021 and 1146
respectively. These rows are predominantly the approximately 2270-row
nonzero-dual face approaching equality; zero candidate-positive is therefore
not an appropriate standalone projection stopping rule.

The fresh-raw descending order is rejected: it is worse at all displayed
budgets and adds deterministic sorting. Relaxation `omega=1.5` improves the
late maximum by about 3.3x but is not consistently better at intermediate
budgets and still requires millions of coordinate updates. No further omega
grid is admitted.

```text
baseline stdout SHA-256
bf2902154eef8eaa082328c436998de07be9e3523d8919fab480f8a13488170b
baseline semantic
e923681d6f5300eaa129059096ac0174b62dcf25d656acb87e37da734431ef54

fresh-raw-order stdout SHA-256
cc24a072df52ce8cde79c304ac322e61168dc7d57212efea5aa6ad155d1395ff
fresh-raw-order semantic
9a5e53aa905b777c4bffd604142a7f6e84bbfa62b73316753d90be993d18608f

omega-1.5 stdout SHA-256
f08cd4ebdad478bb47e174172cad064a2a722adf8348f1cd9eb01d469cc6591c
omega-1.5 semantic
8da9362547b28f16714bb979511ca6c263ce790e4f0b740bec3e6e1980c7701e
```

These processes are algorithmic probes, not timing evidence and not frozen
stage reproduction.

## Interpretation and next research

Dykstra/Hildreth is correct but linearly slow on a large, redundant and
ill-conditioned active face. Feasibility improves while inertia approaches a
stable positive-reduction value near `1.796e-14`; stationarity and
complementarity are already much smaller than primal raw residual.

Research an SHQP/active-face block step after monotone ownership stabilizes:

1. identify a conservative face from nonzero dual plus positive fresh upper;
2. keep the exact sparse `A`/`A^T` owner from R64;
3. solve the face dual normal equations matrix-free with a Krylov method and
   nonnegative active-set guards;
4. rebuild `s = target - A^T lambda - p_C`, project the joint set and resume
   Dykstra;
5. compare at equal row-action/operator budgets, not wall time;
6. require full fresh primal/KKT/reprojection and model audits.

The primary SHQP reference explicitly treats Dykstra as dual alternating
minimization and reports accelerated/QP-supported variants:
[Pang 2016](https://doi.org/10.1137/16M106090X). R65 remains unfrozen until
this block-polish design is researched and bounded.
