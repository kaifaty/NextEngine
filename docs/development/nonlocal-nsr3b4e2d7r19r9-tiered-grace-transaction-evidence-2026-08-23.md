# NSR3-B4E2D7R19R9 tiered-grace transaction evidence

Date: `2026-08-23`

Status: `PASS / NORMALIZED_NOMINAL_STRUCTURAL_WATCHDOG_EXHAUSTED / PRIVATE FIRST SUBSTEP`

## Outcome

The exact R8 tiered completion envelope integrates correctly into one bounded
private normalized first-substep transaction. The previously denied later
solve enters tier 2, passes the HVP-33 continuation gates, converges on HVP 34
and forms an accepted residual-model trial with zero model HVPs.

The transaction advances from R6's `9` accepted trials and outer update `2`
to `20` accepted trials and outer update `6`. It then stops exactly at the
existing total budget of `512` HVPs. No grace predicate, precision, lifecycle
or physics-control failure occurs before that boundary.

This removes the known guarded-completion blocker. It does not establish
substep confirmation or justify raising the total budget.

## Parent and anchors

```text
R9 identity SHA-256  2d7bba5ca68fb92aa154d7de10e950769ae8d722e1a91e5ae7a58ac6f1421a65
R8 stdout SHA-256    def805d3ff7b9b596dd00ec0ff07cd6490dc0f483baa37efc9b8bff511a503d5
R8 semantic          65b9a51e231f51dd9d693ba55b63b0bfd3c4b16ccd5e0c9c298f7f1df7578f63
R7/R6/R5 retained    true/true/true
R2 stdout SHA-256    3dad88903f5f619d540587e805b35d63e2ef8c848e53e1ab87786c9e90587ba0
```

R2 trials `0..4` remain binary64-exact. Trial `5` retains the complete R5
anchor:

```text
current root    54bafbf48d0798438fd9baad9fb91e67c7b5cf6b12e37c4bb1d49694384ddf8a
step root       74a9b58726d5d0279699498c41a70fb0e199f45e531d11befd2bc2dfe692d2bd
trial root      932ce178a6238025c8de6ea907d9966638fa0f5abef7780a15fb9c8171f67ea2
predicted bits  0x3bc27dd9b2871ea7
divided bits    0x3bc27dd8dc16d400
ratio bits      0x3feffffe8ce92225
precision root  a58caa2c1cb83c23dbcc15b8d2daf243de7749e6e692a92369141bbfd741f8db
```

## First integrated tier-2 trial

The former R6 denial at outer `1`, trial `3` now produces:

```text
current root    96294a6434bab090158a1fd5dda13f2e17482b39e1abfeedbb40b81f1645d7e9
step root       35054f8939760fe7baaa4d03250b7d14ec82a284a4f01d44a15cacff6a6b0cef
trial root      0721c28d73a736419b2d4b3ccaf723d631a071df7da8a670425a21763975024f
predicted       3.2892171491745966e-22
divided         3.2892147770498944e-22
ratio           0.9999992788178479
accepted        true
```

It consumes 34 recurrence HVPs, zero model HVPs and uses `r_final-g`. Both
tier-2 entry and continuation predicates are recorded as eligible.

## Work ownership and terminal boundary

```text
transaction root       c6a03d43a04ba00cef80b618bb983ab23f59e27d79ad073f327f119b145660cf
outer updates           6
accepted/rejected       20 / 0
recorded trials         20
total HVP               512
recurrence HVP          494
direct model HVP        18
residual model uses     2
workspace build/release 31 / 31
precision audits        20
failure                 INNER:STRUCTURAL_BUDGET_TOTAL_HVP
```

The two residual models are exactly one tier-1 and one tier-2 completion.
Eighteen ordinary trials retain one direct model HVP each. The work identity
is exact: `494 + 18 = 512`.

Tier provenance is:

```text
tier 1 admitted/HVP/converged       1 / 1 / 1
tier 2 admitted/HVP33/continuation  1 / 1 / 1 eligible
tier 2 HVP34/converged              1 / 1
guard denials                       0
nonconverged grace paths            0
```

All 20 accepted trials receive finite resolved-positive long-double audits.
There are no rejections, binary128 audits, sign contradictions or all-pair
candidate calls. Static binding, finite/mass controls, divided repeats,
lifecycle and rollback pass.

## Reproducibility

Implementation commit:
`1c2c769a` (`research: integrate tiered grace transaction`).

Two clean Release builds:

- `/home/kaifaty/.cache/nextengine/build-nonlocal-b4e2d7r19r9-a.tiStB1`;
- `/home/kaifaty/.cache/nextengine/build-nonlocal-b4e2d7r19r9-b.hR1CPO`.

Both binaries are `6,006,480` bytes, have SHA-256
`3808871408eff8ded734f526bcc10166292975374edece321212ccf4cf175c75`
and GNU build ID `0f6b7418c5cf5fc67946c5da88972a5730ca1b6e`.

Fresh process outputs:

- `/home/kaifaty/.cache/nextengine/run-nonlocal-b4e2d7r19r9-a.sCakbv`;
- `/home/kaifaty/.cache/nextengine/run-nonlocal-b4e2d7r19r9-b.K3YdRV`.

Both exit `0`, emit empty stderr and reproduce:

```text
stdout-with-LF bytes  3,567
stdout SHA-256        f1cb461d270e1642e9c0220c60fc59c64cb5bb69039f293ed8e9eade7f6ed1f0
semantic result       40e152b8f45f2d2b7b654d0bc629aa76ed160f6c9a9467db1097a5fef877934d
route                 NORMALIZED_NOMINAL_STRUCTURAL_WATCHDOG_EXHAUSTED
```

## Decision

Retain the tiered policy as a successful private integration candidate. The
next research stage must diagnose the total-budget boundary before changing
`512`:

- expose per-outer and per-trial stationarity/primal/HVP progress;
- identify the exact solve and recurrence prefix interrupted by total budget;
- distinguish continued convergence from stagnation or repeated expensive
  ordinary solves;
- estimate a justified bounded continuation only from captured progress;
- form no additional trial and make no production-policy or timing claim.

This is a global work-budget question, not another grace-envelope problem.
