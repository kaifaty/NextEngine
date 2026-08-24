# NSR3-B4E2D7R19R36 equal-work hybrid evidence -- 2026-08-24

Status: `PASS / EQUAL_WORK_CURVATURE_POLISH_HYBRID_CANDIDATE`.

## Outcome

The frozen hybrid reproduces exact R35 curvature outers 1--3, then executes
six unchanged R33 projected exact-line polish steps. At the same `51` new
pair-pass budget as R34 and R35 it strictly improves all three frozen terminal
metrics over the R34 first-order reference:

| Metric | R34 | R36 | R36 / R34 |
|---|---:|---:|---:|
| hinge objective | `1.6477454905319591e-16` | `3.5527997925597766e-17` | `0.21561581038906613` |
| violation norm | `1.8153487216135412e-8` | `8.4294718607511539e-9` | `0.46434449537931005` |
| projected mapping norm | `1.4988794937300478e-9` | `7.4709090069224081e-10` | `0.49843293194509064` |

The hybrid therefore passes the predeclared strict-dominance rule. This does
not erase R35's negative standalone-curvature result: the selected mechanism
is specifically a curvature prefix followed by first-order stationarity
polish.

## Hard-gate evidence

- identity SHA-256:
  `2e5cd1cb1fc8722c7d75fdf81c59e0d999809d8a9c00886b5260de62b80e3054`;
- exact R35 parent stdout SHA-256:
  `8f4161e6d4cf9ac24d1ee828dec7019ae0179bd5979c97ea8fec3f2910446fe9`;
- exact curvature roots:
  `b35a0e69...b1fa2`, `6457aeb4...9391`, `95200174...8c53`;
- curvature work: three blocks, five HVPs each, zero nonpositive-curvature
  stops;
- polish work: six accepted steps with strict reduction, trust feasibility
  and direct line KKT in every step;
- fresh terminal response defect: `6.504277497005991e-16`;
- total candidate work: `51` pair passes and exactly `15` HVPs;
- one workspace build/release, zero nonlinear models/trials/outers;
- all 12 route cases, direct terminal checks and rollback pass.

The endpoint is not claimed projected-stationary: its nonzero mapping norm is
reported above. No correction is applied and no nonlinear moved-state
evaluation occurs.

## Clean reproducibility

Implementation commit: `ad968f55`.

Clean Release build directories:

- `/home/kaifaty/.cache/nextengine/build-nonlocal-b4e2d7r19r36-a.pRwbbH`;
- `/home/kaifaty/.cache/nextengine/build-nonlocal-b4e2d7r19r36-b.GbtRha`.

Both binaries have:

- SHA-256
  `c071a0892a9eaf5b78449f1ce5f8c310aafbda180fbae1a5964748a53582f86d`;
- size `7,329,728` bytes;
- ELF build-id `b6383c38db59099a873390726d29f1ce2cfa011a`.

Fresh proof-run directories:

- `/home/kaifaty/.cache/nextengine/run-nonlocal-b4e2d7r19r36-a.a6DJYJ`;
- `/home/kaifaty/.cache/nextengine/run-nonlocal-b4e2d7r19r36-b.XvqPqp`.

The two `4,795`-byte stdout files are byte-exact. Their SHA-256 is
`13566a2dc1ed03336e906ba68a6a2c80af84cb98a38981e61e22595b50c07550`;
semantic result SHA-256 is
`6efda6f07993f4e4cf067c894bd2ffee7b0bf3f10585ad043aba6054008bae99`.

No timing was taken or interpreted. R36 supplies no nonlinear-correction,
runtime, GPU, trajectory or production authority.

## Decision

Retain the hybrid composition as the current equal-work linearized normal-step
candidate. Before any moved nonlinear evaluation, research a separately
frozen continuation that tests whether the selected endpoint continues toward
projected stationarity under the unchanged polish recurrence and establishes
a termination curve without fitting a tolerance to this result.
