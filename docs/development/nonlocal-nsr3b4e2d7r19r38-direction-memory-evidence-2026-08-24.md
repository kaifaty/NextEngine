# NSR3-B4E2D7R19R38 direction-memory replay evidence -- 2026-08-24

Status: `PASS / HAGER_ZHANG_DIRECTION_CANDIDATE`.

## Outcome

The frozen replay compares four one-step directions at exact R37 continuation
states `6/12/24` without accepting any update. Hager--Zhang and DY-HS+ pass
raw descent, projected descent and exact piecewise-line KKT at all three
states and strictly dominate the equal-work steepest reference in objective,
violation norm and active projected mapping. Frozen precedence therefore
selects Hager--Zhang for a separately contracted recurrence experiment.

| R37 state | Lane | Final objective | Final violation | Active | Objective ratio to steepest |
|---:|---|---:|---:|---:|---:|
| `6` | steepest | `1.9068240934105646e-17` | `6.1754742221315520e-9` | `852` | `1` |
| `6` | Hager--Zhang | `7.3516086212267260e-19` | `1.2125682348822045e-9` | `410` | `0.0385543` |
| `12` | steepest | `1.1456276699215390e-17` | `4.7867059026464931e-9` | `848` | `1` |
| `12` | Hager--Zhang | `9.8200795755424273e-20` | `4.4317219171654775e-10` | `294` | `0.00857170` |
| `24` | steepest | `4.2118709613243394e-18` | `2.9023683299417181e-9` | `840` | `1` |
| `24` | Hager--Zhang | `1.3684753550918572e-20` | `1.6543732076480548e-10` | `260` | `0.00324909` |

Equivalently, the selected one-step direction lowers the objective about
`25.94x`, `116.66x` and `307.78x` relative to one steepest step at the three
checkpoints. This is a direction-quality discriminator, not a timing or
throughput result.

PRP+ produces a valid scalar beta but loses projected descent at all three
states. The frozen guard restarts those lanes to steepest exactly. This is a
constructive counterexample to unguarded nonlinear-CG recurrence under the
current projection and changing active set.

## Controls and work

- R38 identity:
  `8ebdf87b7d9635b15d276f0b05e11cb5d81e5909ea96c12f80458c64cc1f6aa1`;
- exact R37 parent stdout:
  `f475c20302c444be1ca355a1c9f0ec39309bbe7cd33f4db73fad676bef1c3571`;
- source checkpoint root:
  `e404d5997578df09bab907e16f37e2344f4c754cb7951fad2aa01a0c43f31f4e`;
- replay record root:
  `958fd00810a03ae4e98be366b04b0a61a8af550af38f6a19ba0f5541bc4244b6`;
- dense-control root:
  `597f83607e042b7b5d11891c66390396e8428db4cbdd017ef7a628c1d69e512c`;
- work: exactly `15` new pair/JVP passes, zero VJP, HVP, model,
  trial or outer work;
- one parent replay and one workspace lifecycle; all 12 route cases and exact
  rollback pass.

The first diagnostic rejected the replay state because it incorrectly
required a newly accumulated response to be bit-identical to the captured
maintained response. The exact failure and narrow repair are retained in the
[diagnostic record](nonlocal-nsr3b4e2d7r19r38-first-diagnostic-2026-08-24.md).
The repair uses the captured response as checkpoint state and the fresh JVP
only as the already-contracted bounded source defect; it changes neither
direction formulas nor nominal selection.

## Clean reproducibility

Contract/research commit: `b26a3371`. Implementation commit: `06eacf81`.

Clean Release build directories:

- `/home/kaifaty/.cache/nextengine/build-nonlocal-b4e2d7r19r38-a.WFDAMe`;
- `/home/kaifaty/.cache/nextengine/build-nonlocal-b4e2d7r19r38-b.FNYKjX`.

Both binaries have SHA-256
`392bab4fdfcfa3b45a872c765a6268d4c267dba700eaf473a40251bac904e0e6`,
size `7,436,384` bytes and ELF build-id
`9d19df9d4943f0b8b0815ec9dfd9953d31f63971`.

Fresh run directories:

- `/home/kaifaty/.cache/nextengine/run-nonlocal-b4e2d7r19r38-a.Rnx0Zf`;
- `/home/kaifaty/.cache/nextengine/run-nonlocal-b4e2d7r19r38-b.K7sao8`.

The two `7,027`-byte stdout files are byte-exact. Their SHA-256 is
`7f574289496e70fb76886381c2b1460227f15a3f6d07214b0772dabf7247abb1`;
semantic result SHA-256 is
`cc030c6559643952faf70dd3545f7f62a01133527ad5fdf8427159ed994ab7d3`.
Both stderr files are empty. No timing was taken or interpreted.

## Decision

Promote only the guarded Hager--Zhang direction formula to a new, bounded,
rollback-only recurrence discriminator. Start from the exact R37 terminal
state, give Hager--Zhang and steepest equal operator work, retain exact
piecewise-line globalization, and restart to steepest on invalid beta,
non-finite state, raw non-descent or projected non-descent. The recurrence,
history ownership, horizon and selection gates must be frozen before code.

R38 proves neither recurrent convergence nor runtime value. It accepts no
step, applies no correction, selects no tolerance, evaluates no nonlinear
moved state and provides no runtime or production authority.
