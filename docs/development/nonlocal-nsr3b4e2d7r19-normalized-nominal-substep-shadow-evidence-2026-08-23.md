# NSR3-B4E2D7R19 normalized nominal-substep shadow evidence

Date: `2026-08-23`

Status: `FAIL / PRECISION_LEDGER / NO_ROUTE / NO_STATE`

## Outcome

D7R19 reaches a narrower boundary before any nominal route is admissible. The
normalized solver accepts five private inner trials at outer update zero, but
three accepted long-double audits classify compact-support membership
differently from the binary64 solver. The frozen precision control therefore
fails hard. The simultaneous per-trust-step HVP watchdog is reported but does
not outrank this hard control.

The result is not a failed physical ledger or a reason to relax the HVP cap.
No confirmed state exists, so density, boundary and impulse route gates are
intentionally not evaluated.

## Exact boundary

All parent and alignment controls pass:

- R4R2 stdout SHA-256:
  `5bb8f5abcb6abf0903c681529771ee6814b567f09d6e12f138514c3f0dab0dc3`;
- D7R17 stdout SHA-256:
  `a2a8de930d41d8e49c255a3fcf987a8794b4da067ddadf658732946fca0ffced`;
- frame-zero root:
  `0d567ba5512ba237a48e5e0b828a670a398f1bf23a35ac269729cad535f374d7`;
- `dt/kappa/theta` bits:
  `0x3f0c01c01c01c01c / 0x415c75a640000000 / 0x3fc5cccccccccccd`;
- particles / lower-Y clamps / free samples: `6000 / 400 / 5600`.

The private work ledger is:

```text
outer updates                   1
completed accepted/rejected     5 / 0
completed-trial HVPs            85
total budget HVPs               117
dimensionless trust selections  6
inherited trust selections      0
workspaces build/release/live   6 / 6 / 2 maximum
precision audits                5 long double / 0 binary128
all-pair candidate calls        0
```

The sixth trust solve consumes its explicit dimensionless policy selection and
then reaches `STRUCTURAL_BUDGET_HVP_PER_STEP` before producing a completed
trial record. This is why policy provenance is `6`, while the completed trial
ledger is `5`; the accounting is exact.

Three accepted audits set `topology_precision_mismatch=true`. Repeated
current/trial observations total `10,989`; the three unique successive states
contain `3,641`, `2,346` and `1,328` mismatches, totalling `7,315` unique-state
observations.

| Outer/trial | Current mismatches | Trial mismatches | Long-double root | Sign |
|---|---:|---:|---|---|
| `0/0` | `3,641` | `2,346` | `00188e8e74bf7bc16e9be0d1b7df3a9555f23d4d22f3bad8355b0a36dd4aac02` | resolved positive |
| `0/1` | `2,346` | `1,328` | `823812828a7386e3120762cab508b76577fae92c692a39152420ace1ff5d8f95` | resolved positive |
| `0/2` | `1,328` | `0` | `7298f0f80106642c72a6912736e30b5c55e5cb0898f58f85e8146967b6fb1ded` | resolved positive |

Every row has zero observed minimum long-double distance from the support
horizon. This suggests a compact-support representation edge, but does not
prove that every mismatched pair is harmless. The hard precision gate remains
unchanged.

## Reproducibility

Implementation commit:
`085056d6593f3685a7d399c620a86cb560d80cb6`.

Two clean Release builds:

- `/home/kaifaty/.cache/nextengine/build-nonlocal-b4e2d7r19-a.G4ji3L`;
- `/home/kaifaty/.cache/nextengine/build-nonlocal-b4e2d7r19-b.YZByRC`.

Both binaries are `5,691,960` bytes, have SHA-256
`8068d09787b41e301d16ffeee557991bee0964ac72ddd63d1b70918b1836fd1d`
and GNU build ID `39d118b04f30b8e1d68331ab7ae184f86d820373`.

Both fresh processes exit `1`, emit empty stderr and reproduce:

- stdout-with-LF bytes: `4,100`;
- stdout SHA-256:
  `f5811bfc7d5e986d72b9130f8e6cb90c5ae21bd347ce7f0b476c171fe8cff7bb`;
- semantic result:
  `bcc6f588010999f23664209037a0daae961606ca29fdcc0d7a31e661902e185b`;
- private transaction root:
  `a1030f9b23abf0322c6c7acaba17da84da7861c1ac776ad78d5747b5989fcca1`.

Raw local outputs are under
`/home/kaifaty/.cache/nextengine/run-nonlocal-b4e2d7r19-clean.gnmcXc`.

## Decision

Preserve D7R19 as a hard `PRECISION_LEDGER` failure. Do not reinterpret it as
`NORMALIZED_NOMINAL_STRUCTURAL_WATCHDOG_EXHAUSTED`, relax topology precision,
increase the HVP cap or run another nominal substep.

Research one replay-only D7R19R1 discriminator over the exact three mismatched
accepted trials. It must distinguish a C2 zero-shell representation edge from
a nonlocal membership or acceptance-sign change before any precision-policy
reclosure.
