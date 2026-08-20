# NSR3-B3D2 finite-precision merit evidence -- 2026-08-21

Status: `PASS / FLOOR_STATIONARITY_MERIT_CANDIDATE / NO_B3_RETRY`

The frozen
[D2 discriminator](../plans/nonlocal-nonlinear-solver-research/03b3d2-finite-precision-merit-contract.md)
replayed all six D1 first-floor states twice byte-identically. PASS means the
diagnosis and classification are valid; it does not repair D1 or authorize a
boundary trajectory retry.

## Factored objective result

The direct endpoint expression agrees with its `long double` evaluation
inside the derived binary64 bound in all six states. It does not, however,
recover a common descent decision:

| State | model decrease | factored endpoint decrease | error bound | sign |
|---|---:|---:|---:|---|
| face / 96 | `5.51e-21` | `+2.32e-16` | `5.21e-19` | positive |
| face / 192 | `1.21e-23` | `-1.80e-16` | `1.90e-19` | negative |
| face / 384 | `5.61e-24` | `-2.97e-16` | `7.16e-19` | negative |
| corner / 96 | `1.25e-20` | `+2.76e-16` | `6.69e-19` | positive |
| corner / 192 | `1.76e-23` | `-1.93e-16` | `1.99e-19` | negative |
| corner / 384 | `4.55e-24` | `+1.57e-16` | `6.25e-19` | positive |

The endpoint energy signal is dominated by materialized-coordinate/density
quantization: maximum compression changes are only one or two binary64 ULPs
(`2.22e-16--4.44e-16`). Between 6 and 88 nonzero correction components do not
change their materialized position, depending on the state. Direct factoring
removes total-sum cancellation but cannot turn these quantized endpoints into
a reliable common descent oracle. `FACTORED_OBJECTIVE_DIFFERENCE_CANDIDATE` is
therefore rejected.

## Stationarity result

All endpoints retain exactly one active pressure center and exact pair counts
(`2798` face, `2308` corner). The proposed trust step reaches the unchanged
reaction gate in every state:

| State | current defect | trial defect | limit | trial/current |
|---|---:|---:|---:|---:|
| face / 96 | `5.22e-11` | `7.56e-13` | `4.01e-12` | `0.0145` |
| face / 192 | `2.39e-12` | `9.38e-13` | `2.01e-12` | `0.3921` |
| face / 384 | `1.12e-12` | `6.54e-13` | `1.02e-12` | `0.5843` |
| corner / 96 | `5.66e-11` | `1.71e-13` | `2.40e-12` | `0.0030` |
| corner / 192 | `2.10e-12` | `6.22e-13` | `1.20e-12` | `0.2964` |
| corner / 384 | `8.25e-13` | `5.18e-13` | `6.06e-13` | `0.6279` |

This selects `FLOOR_STATIONARITY_MERIT_CANDIDATE`. The merit is
`0.5*||h*grad F||^2`, so it targets the first-order equation of the unchanged
objective. It is not a substitute energy and may be used only after the old
energy-floor predicate fires, with unchanged topology and strict residual
improvement.

## Next experiment

Freeze a D3 full-trajectory candidate that retains D1 displacement ownership
and the ordinary trust-energy path. At an active energy-floor exit only, it may
accept the already proposed trial when:

1. active-center and pair topology are unchanged;
2. trial reaction residual is finite and strictly smaller;
3. trial residual is already at or below the existing mixed limit.

Any other floor state preserves the D1 failure. D3 must charge the trial
evaluation, retain all trajectory/correspondence gates and cannot silently
iterate under a residual-only globalizer.

## Repeatability and regression

```text
D2 raw SHA-256 (two identical runs):
0b9581489e133039c7562f938fdb79d73a8810222341f74aa09f61edf668a1ae

D2 JSON-without-newline SHA-256:
573bf5a943d338bbbed4c05919bea2a0255b6d71e98d85c195a3ee38adf82f6e

D2 semantic SHA-256:
da1da7a0fcd4adf29c01a60a5d72f97c4d7a4c9794484bc90f575b9d271d4601

D1 raw preserved:
f500187b0fb02c36d4383e2e9fe1003677afc5b2744c02b587fe9f9d86b8587a

B3D raw preserved:
66474768533403f41e7ba7becf3ca03ddcab0cd57a99edf16ed18017735aec35
```

No B3 retry, physical corpus, CUDA, performance, runtime or production
authority is granted.
