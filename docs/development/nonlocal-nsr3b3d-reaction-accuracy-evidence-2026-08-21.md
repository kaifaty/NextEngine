# NSR3-B3D reaction-accuracy diagnostic evidence -- 2026-08-21

Status: `PASS / INACTIVE_FORWARD_ERROR_CERTIFICATE_REJECTED / NO_B3_RETRY`

The frozen
[B3D diagnostic](../plans/nonlocal-nonlinear-solver-research/03b3d-reaction-accuracy-contract.md)
completed both non-aborting trajectories and all causal replays. PASS means
the diagnostic decision is valid; `candidate_selected=false` means reaction
authority and B3 retry remain blocked.

## Causal decomposition

The exact B3 failed report replays byte-identically. Across every completed
fixed trace:

- virtual translation defect is at most `1.92e-17 kg m/s`;
- contact impulse defect is bit-zero;
- every inactive reconstruction defect is below its computed per-step
  binary64 forward bound (`max defect/bound = 0.01421`);
- active stationarity, not translation or contact algebra, dominates the
  uncorrected ledger.

Non-aborting adaptive face/corner trajectories complete, but retain maximum
ordinary ledger residuals `1.64e-4` and `1.77e-7`. Completion therefore cannot
be relabelled as reaction correctness.

## Active reaction-aware replay

The isolated counterfactual succeeds within its frozen cost and state-change
limits:

| Replay | ordinary defect | reaction-aware defect | HVP ordinary/aware | position change | velocity change |
|---|---:|---:|---:|---:|---:|
| face pre-contact `/192` | `3.14e-13` | `3.13e-13` | `2 / 2` | `0 dx` | `0 c` |
| corner contact `/192` | `1.3282e-5` | `7.12e-13` | `0 / 2` | `4.01e-9 dx` | `9.33e-8 c` |

The corner stationarity defect falls by about `1.86e7x`. Two HVP calls are
inside `2.5x+2`; no coefficient, contact order or physical threshold changes.
This validates the reaction-aware stop mechanism for the captured active
states, but it cannot be selected alone because the inactive certificate
fails its cumulative-bound gate.

## Why the inactive certificate is rejected

The corrected forward bound follows the actual arithmetic graph: local
`gamma_6` per particle/component plus `gamma_(N-1)` for deterministic
summation. It tightly covers observed error, but position subtraction still
contains the term `|x|/h`. Consequently its cumulative bound grows under time
refinement:

| Fixture | substeps/frame | cumulative `B_fp` | certificate |
|---|---:|---:|---|
| face | 96 | `2.52e-8` | PASS |
| face | 192 | `1.01e-7` | FAIL |
| face | 384 | `4.03e-7` | FAIL |
| corner | 96 | `1.67e-8` | PASS |
| corner | 192 | `6.68e-8` | FAIL |
| corner | 384 | `2.67e-7` | FAIL |

The cap is `1e-10*M*c`; the finer rows exceed it even though actual inactive
defects are only `0.8%--1.4%` of the bound. This is not evidence for relaxing
the cap. It shows that reconstructing a tiny velocity increment by subtracting
two world positions is an increasingly ill-conditioned representation.

## Architectural conclusion

The next numerical state boundary should be tested as:

```text
substep start x
prediction displacement delta*=h(v+h*g)
accepted trust corrections p_k
owned displacement delta=delta*+sum p_k
contact edits delta on hit axes

position = x + delta
velocity = delta / h
```

This avoids re-deriving `delta` from `position-x`. It changes transient
numerical ownership, not the continuum objective, and must receive a separate
report identity. Public/runtime state remains out of scope.

## Repeatability and regression

```text
B3D raw SHA-256:
66474768533403f41e7ba7becf3ca03ddcab0cd57a99edf16ed18017735aec35

B3D semantic SHA-256:
e99cda02f945e8402e6f4ca30742853c1fa29f1c845ed5c8a24db180877fa3b1

B3 raw preserved:
c64ad0b8d73f7ada62364a3daa2d3bed66a8bfc5a1fe6c0148b7c1f8f2e5fb2f

B2 raw preserved:
d6ba5f8e802966c25283d0c8384ed01beec20b347acb343cf5c7c2bf360d69d9
```

Two B3D reports are byte-identical. B3 remains FAIL and the physical corpus
remains blocked.
