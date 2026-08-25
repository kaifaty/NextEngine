# NSR3-B4E2D7R20 generalization corpus manifest evidence

Date: `2026-08-25`

Status: `PASS / SUPERSEDED BY V2 BEFORE SOLVER EXECUTION`.

Implementation commit: `e2bbeebb`.

## Result

The manifest-only path freezes six source states before any candidate or
offline oracle execution. Two states are new blind holdouts; four older states
are labeled transfer regressions and are not misrepresented as holdouts.

```text
schema    nextengine.nonlocal.nsr3b4e2d7r20_corpus_manifest.v1
route     GENERALIZATION_CORPUS_MANIFEST_CANDIDATE
semantic  a983588303964ff5483aed5f6dd5939ac37fc9e53b11ef1a442fe2eaac292471
stdout    da427e474f1c8ab2f342f103763d88141434b694b1f69dd918f6fca0375034b5
```

Two independent executions are byte-identical. The report explicitly records
`solver_executed=false`, `oracle_executed=false` and `timing_admitted=false`.

## Frozen sources

| role | state | fluid / boundary | source SHA-256 |
|---|---|---:|---|
| transfer | lower-y face | `75 / 146` | `71e795064bea57f05d1feb5fb133619c36161057e30c8c2de73de1cc8e7345a8` |
| transfer | lower-x/lower-y edge | `45 / 187` | `5110f6acffa487fbcfe723079355f2b36e242ac58a3f1c2f9ed0c98a148a865b` |
| transfer | supported column start | `48 / 544` | `bc150ed57f0249e6a0340646bd2caee3a9bf35e8607dcd0a35846d0b17bd0083` |
| transfer | released block after ten analytic free-flight steps | `27 / 1216` | `4908df6dadbace8072494960951708f211bb8c37509788256ef44245372d94d2` |
| blind | sheared upper-x/lower-y oblique edge | `24 / 1052` | `297e18d9c5e7f3f03c544da3f3df3d6196fcdcf536b177c4ad64e9a07c432f7b` |
| blind | sheared upper-x/upper-y/lower-z opposed corner | `24 / 1060` | `17c5aaec1b8acf04beb9b9ab3cde6759413e18bfeecdd30d349dea8bbc4e957c` |

The complete source root includes positions, velocities, boundary samples,
gravity, contact ownership, box faces, timestep, spacing and trust radius as
binary64 bits. The target map is frozen as the semi-implicit predictor minus
the source position divided by spacing.

## Claim boundary

This evidence proves source identity and provenance only. It says nothing yet
about feasibility, convergence, accuracy, work advantage, nonlinear transfer,
runtime throughput or production readiness. Those observations are forbidden
until the R20 stopping/oracle contract is frozen.

## Supersession

The first input-only operator preflight later showed that five of these six
problems, including both intended blind holdouts, already had zero positive
density rows after projecting the predictor onto contact/trust geometry. They
would test a zero-step exit rather than composed-dual acceleration. No
candidate or oracle iteration had run, so manifest v1 and contract revision 1
were superseded rather than silently retuned. The two quiet states are retained
as explicit negative controls in v2.
