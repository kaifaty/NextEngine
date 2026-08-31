# Physical Sound R3A V12-C4a — runner and repeated preflight result

| Field | Value |
| --- | --- |
| Date | `2026-08-31` |
| Decision | `C4A_PREFLIGHT_REPEAT_PASS` |
| Runner revision | `object41-force-certified-one-shot-common-pole-v2` |
| Implementation commit | `c416cb4c` |
| Signal access | `0` source bytes, `0` decoded samples |
| Product effect | None; authored clips remain authoritative |

## Exact evidence

The runner freezes only the sixteen C3 `estimator_fit` contacts. It binds the
C3 manifest `a6dd159e…66b3`, inventory report `a300199f…20d7`, exact `4 GiB`
prefix `93ad4484…55eb` and role-order root `235e418f…0620`.

External artifacts:

| Artifact | SHA-256 |
| --- | --- |
| `r3a-v12-c4a-object41-freeze/manifest.json` | `7b00cf1f405939cd494487ddd20b5eb7f1fd928b669cc29af1e1fab734288e8e` |
| `r3a-v12-c4a-object41-preflight-a/report.json` | `a236ffba82ecf6dd3d792227db188b08742eb05efc642f6548032fac45a89400` |
| `r3a-v12-c4a-object41-preflight-b/report.json` | `a236ffba82ecf6dd3d792227db188b08742eb05efc642f6548032fac45a89400` |

Both preflight reports are byte-identical and decide
`C4A_REAL_FRF_FIT_FROZEN`. Network requests, source bytes, force samples,
microphone samples and protected-role samples are all exactly zero.

## Implemented boundary

- Force PCM is decoded and certified before microphone PCM.
- Force coverage and all four leave-quarter-out masks use the frozen noise,
  SNR, relative-power and width gates.
- Single-record coherence is always `NOT_APPLICABLE_SINGLE_RECORD`.
- Stable poles require three of four refit matches, local certified coverage
  and residues in at least eight contacts.
- Raw H1, unit-impulse response, input-ignorant output modal, cyclic force
  permutation and baseline-only zero-force controls are explicit.
- Every artifact is external; protected roles have no decode path in C4a.

## Verification

- `26/26` focused C1+C3+C4 Python tests passed.
- Runner and test `py_compile` passed.
- `git diff --check` passed before the implementation commit.

## Consequence

The repeated zero-read prerequisite is complete. The next and only authorized
signal action is two independent C4a runs over the exact sixteen-contact fit
budget: at most `4,608,000` force and `4,608,000` microphone PCM16 values.
Development, representation holdout, validator calibration, validator method
holdout and admission shadow remain sealed. A fit pass opens only a separately
committed development protocol; reject/OOD keeps all later roles closed.
