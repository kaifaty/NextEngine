# NSR3-B4C3Q balanced canonical quantization evidence -- 2026-08-21

Status: `PASS / CANONICAL_AGGREGATE_BALANCED_CANDIDATE / B4C3A1_DESIGN_AUTHORIZED`

## Reproduction

```text
nonlocal-formula-reclosure --canonical-conservation-self-test
```

Two reports are byte-identical:

```text
raw JSON plus LF  ae44e39fb137ffbcca17be40672471cc8b35d1eb3c26b7131c3b658db3640731
JSON without LF   35def7a51dcce516d3f65d763595a4e3dc52fe771b16368773d3b4493c30182c
semantic result   ca5f49f3179e05f01fed58a690919b1b78877cc7d6ad2d20611aeb836dd6468f
```

Whole-chain wall times were `63.29/63.65 s`; maximum resident sets were
`8,316/8,428 KiB`. Each execution includes B4C3A and its complete parent
controller chain. These are validation-harness costs, not production solver
timings.

## Exact algebra

The fixed 1,280-bit superaccumulator forms exact scaled binary64 rationals,
their aggregate nearest-even target and exact residual ordering. It has no
external/runtime dependency.

On 48 samples with equal `+0.49`-unit residual:

| Policy | Aggregate error | Local bound |
|---|---:|---:|
| independent nearest-even | `23.52` units | `<=0.5` unit |
| aggregate balanced | `0.48` unit | `<1` unit |

Balanced improvement is `49x`, with 24 deterministic one-unit corrections.
Across the algebra corpus its observed maximum local error is `0.51` unit and
maximum aggregate error is `0.48` unit. Exact aggregate/local proofs,
known positive/negative half-tie targets, small exhaustive
minimum-squared-error oracle, zero-correction equality,
reverse/affine order, sign covariance and canonical-lattice translation all
pass. The biased frame root is:

```text
d6fddccfa3116700e4dd9316bab2a49bf990292bc8d1856c78f5e2d3ae9a2f36
```

## Physical controls

| Case | Binary position RMS | Binary velocity RMS | Nearest position RMS | Nearest velocity RMS | Max aggregate position/velocity error |
|---|---:|---:|---:|---:|---:|
| P1 `21/42` | `5.29549e-6 m` | `7.65510e-4 m/s` | `4.13572e-6 m` | `6.88826e-4 m/s` | `0.480915 / 0.498122` units |
| P2 `1/2` | `4.38274e-7 m` | `1.00000e-6 m/s` | `5.09175e-7 m` | `1.38778e-6 m/s` | `0.390625 / 0.5` units |

Both embedded gates, decoded continuation, contact sets/times and
reverse/affine roots pass. P1's maximum local position error rises to
`0.947078 µm`, exposing the selected tradeoff rather than hiding it. Maximum
one-publication aggregate momentum impulses are only `8.64e-8 kg m/s` for P1
and `6.25e-8 kg m/s` for P2; the kinetic perturbation inequality passes.

Selected one-frame trajectory roots are:

```text
P1  ece583962cb07f7a15bb1b84a83719895ec5af2c2735b0904ea76f833180939d
P2  8138d5202b4a754c43b8ee7306200b29d6d1c75b959717f9a0f3c54c1f543ddd
```

## Temporal residual stress

For 48 samples over 1,024 deterministic `+0.49`-unit publications:

```text
independent center position/velocity drift  5.0176e-4
balanced center position/velocity drift     1.0240e-5
frozen summed balanced bound                1.066666668e-5
improvement                                 49x
trajectory root  aafc245b19704c6a9b1de0c2a1978dd9bad1f053fb77e6576f0c73b8ca438342
```

This is a quantization-bias stress test, not a fluid-accuracy result.

## Failure and negative evidence

Forced solver failure, nonfinite input, range overflow, duplicate ID, sample
capacity and infeasible correction capacity all return the exact typed error,
commit zero frames and preserve pre-transaction state.

The first implementation report failed only because its test added
non-representable decimal `7e-6` to arbitrary binary64 inputs instead of
translating the canonical lattice. All other gates already passed. Preserved
hashes:

```text
raw JSON plus LF  a09e25cb78da42099b1947d2eacb2a657e636411c3c5c52d753a1e5dac42601e
JSON without LF   1270061055c7ee0bc1c1d65333b726de8a859fb78b932f68f643db77a0abba63
semantic result   e889dc2301042fe97eceb03501e19d2bf0726924faeaeb62d9db9ed5becdc63e
```

The repaired control applies an integer translation to published units,
decodes and republishes. Candidate arithmetic and thresholds did not change.

Historical raw reports remain exact:

```text
NPR1-A  464a55bc33741f18ddc4b3c1cda6b46b5248bb653789a5e31585482f871ee207
B4C3A   80920420f90893e0fad756ef169469fb41770e77b68b0c6ce31ede21b2338ee3
```

B4C3A's embedded B4C2T no-final-LF parent check also remains exact.

## Decision

Select `CANONICAL_AGGREGATE_BALANCED_CANDIDATE`. It reduces the admitted
aggregate publication error from `N/2` to `1/2` unit per component without
hidden state, in exchange for a local bound below one unit rather than half a
unit.

This authorizes only B4C3A1 one-frame transaction revalidation under the new
profile and quantization-aware ledger. B4C3T, B4C4, B4D, CUDA, runtime, schema
and production remain blocked.
