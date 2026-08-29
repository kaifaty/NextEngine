# NSR3-B4E2D7R19R28 shadow outer-11 execution evidence

Date: `2026-08-24`

Status: `PASS / SHADOW_OUTER11_EXECUTION_CANDIDATE / NOT ADMISSIBLE`

## Outcome

The slice candidate and independent unsliced oracle execute outer 11 with
complete bit/work equivalence. Candidate remains within epoch-1 capacity.
The result is finite and accepted but still non-admissible.

```text
update root       1c094e09452339d1baa33ce6fbee576d8cfd83c2696d074737e90cf2002899eb
work root         228e67eb941173d122f168170f199d753695c8ff1714d81d55a3a8000bd52b87
trials            2 accepted / 0 rejected
HVP/workspace/precision delta   51 / 4 / 2
candidate/oracle exact          true
```

## Physical and mechanism result

```text
primal          1.8403617518814031e-08   bits 0x3e53c2bf74000000
stationarity    5.4380393614070454e-14   bits 0x3d2e9d0a6841b669
admissible      false
position root   e0f7ba79637171012b11750b752ab862b7b24174f4d79e2ec115e06bc8b93828
dual root       cd0100c30eb55ad7f91f1b94c00ee4281ff3c75e90ca287be038e87ab7a5215d
```

Primal improves about `4.4290%` from outer 10, after outer 10 improved only
`1.0032%`. Both trials are accepted, use 51 HVP total, and the successor
ledger records the last trial at `26/34`. Stationarity remains small.

This falsifies the premature interpretation that outer 10 alone established
a stable primal plateau. Reject churn, immediate trust-cap pressure,
budget-offset sensitivity and monotonic inner-solve degradation do not explain
the current residual. The wider sequence is still slow and non-monotone in
progress rate, so neither convergence nor a discretization floor is proven.

The next useful discriminator is causal rather than another post hoc trend
threshold: decompose the current constraint residual into the component inside
the linearized feasible image of `J` and the orthogonal/unreachable component,
then distinguish representation rank from active-boundary and outer-policy
limits. That diagnostic must be separately researched and frozen.

## Canonical successor

```text
history         fd838a046b2629f5d5e4d5aa0ab87160078b2e4b748b98101ac2da00bfa3a37d
consumed owner  13feea1f5a87fb577ed5f9ccf89e0f46d6acdb03554ea1c48b5e2c81081c5264
receipt         b145ce1af6d3ffca167f2264b55618b0bfccc55c340ff1eee17206569a239af6
state           851b4eb8d1387a7347bb4dd8adba3be016d8a39dbd04133dcaf306fb46b690d7
used            12,2,26,1,333,856,58,34,824,32,2,34,0
```

Epoch remains `1`; slice/cumulative HVP become `333/856`, leaving 179 slice
HVP. Recurrence/direct accounting closes at `824 + 32 = 856`.

All `13/13` routes pass at corpus root
`1c78743d1c3789549a7d57f99aff674abe976d7a3af48ff78a4c4670537da513`.
Rollback and duplicate replay are exact. No following outer, substep, macro,
trajectory, timing or public/world commit occurs.

## Reproducibility

Contract/research commit: `c4435fde`.

Implementation commit: `7b424ff7`.

Two clean Release builds:

- `/home/kaifaty/.cache/nextengine/build-nonlocal-b4e2d7r19r28-a.vGc6QS`;
- `/home/kaifaty/.cache/nextengine/build-nonlocal-b4e2d7r19r28-b.Z2sVq4`.

Both binaries are `6,845,400` bytes, have SHA-256
`f7a8b08840981188c0a67e8e22a2b9d4de9569e86dc069ab85db30b574228364`
and GNU build ID `889664926eeeee139b38733d7b699f5483cc6011`.

Fresh sequential one-process outputs:

- `/home/kaifaty/.cache/nextengine/run-nonlocal-b4e2d7r19r28-a.uU80Jz`;
- `/home/kaifaty/.cache/nextengine/run-nonlocal-b4e2d7r19r28-b.lXyCSu`.

Both exit `0`, emit empty stderr and reproduce `1,887` stdout bytes:

```text
stdout SHA-256  9c953a118d6babb192be6be23e94374576f8a48619754567d341576c33a37237
semantic        006620643aae830c4bfe5d07e3d0a0610d34ac0fb2ada57d452597ddaa4f12d0
route           SHADOW_OUTER11_EXECUTION_CANDIDATE
```

These are correctness/reproducibility runs, not timing or performance
measurements.

## Next action

Research/freeze a zero-state-mutation linearized-feasibility range diagnostic
over the exact R28 successor. It must separate unconstrained representation
rank, active-boundary/tangent restrictions and AL outer-policy response before
any refinement, penalty, cap or following-outer experiment is authorized.

