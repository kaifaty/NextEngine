# NSR3-B4C3Q aggregate-preserving canonical quantization research -- 2026-08-21

Status: `COMPLETE / AGGREGATE_BALANCED_SELECTED / B4C3A1_REQUIRED`

## Newly exposed defect

B4C3A proves local nearest-even publication and transaction ownership for one
macro frame. It does not prove the conservation ledger of a continued
canonical trajectory.

For one velocity component, independent publication gives

```text
v_hat_i = round(v_i * S) / S,  S = 1,000,000
|v_hat_i - v_i| <= 0.5 / S
```

but the publication impulse is

```text
I_q = m * sum_i(v_hat_i - v_i)
|I_q| <= m * N * 0.5 / S  per component and substep.
```

The selected KKT ledger is evaluated before publication, so it does not include
`I_q`. Repeating publish/decode can therefore accumulate unreported momentum,
center-of-mass and energy changes even though every local B4C3A bound passes.
The same issue applies to independent position rounding.

This is not a failure of the Nonlocal pressure formula. It is a representation
and continuation-policy gap between a binary64 variational solve and an
integer-authoritative state.

## Alternatives

1. **Independent nearest-even.** Keeps the optimal `0.5`-unit local bound but
   allows an `N/2`-unit aggregate error per component.
2. **Snapshot-only canonical state.** Keeps binary64 continuation and publishes
   integers only for observation/checkpoints. This preserves solver physics but
   does not provide exact replay from the published state.
3. **Hidden temporal error feedback.** Reduces long-horizon bias but the carry
   becomes authoritative state that must itself be published and rooted.
4. **Aggregate-balanced apportionment.** First round each value nearest-even,
   then deterministically redistribute the aggregate integer discrepancy with
   minimum incremental squared error. It keeps local error below one unit and
   reduces aggregate error from `N/2` units to `1/2` unit without hidden state.

The fourth option is the first research candidate. Snapshot-only publication
remains a valid architecture alternative if integer continuation proves too
physically invasive.

The first implementation report rejected only a test that added decimal
`7e-6` directly to arbitrary unquantized binary64 values and expected an exact
seven-unit output shift. That operation is not a canonical-lattice
translation: neither the decimal increment nor the binary addition is exact.
The repaired control translates published integers by an integer unit count,
decodes them, and republishes. This changes no candidate arithmetic or bound;
the original rejection remains negative test-design evidence.

## Candidate derivation

For scaled exact binary64 values `x_i`, let `r_i = round_even(x_i)` and
`T = round_even(sum_i x_i)`. Define `D = T - sum_i r_i`.

- if `D > 0`, increment the `D` samples with smallest exact residual
  `e_i = r_i - x_i`;
- if `D < 0`, decrement the `-D` samples with largest exact residual;
- break equal-cost ties by ascending `SampleId` after canonical sorting.

This is the minimum-incremental-squared-error one-unit correction. Under sign
inversion, nearest-even and `D` both change sign and increment/decrement costs
map to each other, so the same IDs receive opposite corrections. Integer-unit
translation leaves residuals unchanged. Exact binary64 rational arithmetic is
required for the aggregate target and residual ordering; a binary64 or
unordered sum would merely move the determinism defect.

The policy guarantees per component:

```text
abs(sum(q_i) - sum(x_i)) <= 0.5 scaled unit
abs(q_i - x_i) < 1 scaled unit
```

It does not make publication physically free. The remaining quantization
impulse must be included explicitly in the canonical ledger.

## Roadmap change

Insert two gates before the full controller:

```text
B4C3A nearest-even transaction mechanics PASS
  -> B4C3Q independent-vs-balanced quantization discriminator
  -> B4C3A1 selected-policy one-frame transaction revalidation
  -> B4C3T complete canonical controller and quantization-aware ledger
```

B4C3Q must include adversarial biased residuals, exact small exhaustive
optimality, sign/translation/order covariance, long free-flight drift and P1/P2
one-frame physical controls. A balanced-policy PASS changes the canonical
profile identity; B4C3A's nearest-even roots must not be reused.

## Decision

Freeze and execute B4C3Q before designing B4C3T. B4C4 packaging, B4D nominal,
runtime and production remain blocked.

B4C3Q subsequently passes twice byte-identically and selects aggregate-balanced
apportionment. The observed tradeoff is explicit: biased aggregate and temporal
center drift improve `49x`, while the local physical publication error may rise
from the nearest-even half-unit bound to below one unit. Rebind the profile and
revalidate the transaction as B4C3A1 before any full trajectory.
