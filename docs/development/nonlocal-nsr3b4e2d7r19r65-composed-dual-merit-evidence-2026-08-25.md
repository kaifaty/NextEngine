# NSR3-B4E2D7R19R65 composed dual-merit evidence

Date: `2026-08-25`

Status: `PASS / COMPOSED_DUAL_PATH_CANDIDATE`.

Implementation commit: `db578f0e`.

## Strongest honest result

The exact convex projection dual resolves the v6/v8 merit conflict on the
frozen fixture. Every one of the 235 normal-safe projected candidates has
strict positive composed-dual change, including candidates in all 15 outers
where strict intermediate primal-inertia descent rejected the complete path.

This supports the bounded conclusion that strict post-Hildreth primal inertia
was the wrong inner acceptance measure. It does not yet prove that applying
the best dual candidate dominates FISTA, that the accelerated block converges,
or that the outer nonlinear Nonlocal transaction can omit filter-SQP.

Route: `COMPOSED_DUAL_PATH_CANDIDATE`.

## Formula correspondence

For `s=P_D(t-A^T lambda)`, both forms were evaluated:

```text
d = 0.5||s-t||^2 + lambda^T(c+A*s)

d_alt = 0.5||s-(t-A^T lambda)||^2
      + lambda^T c
      + (A^T lambda)^T t
      - 0.5||A^T lambda||^2.
```

Across 16 baselines and 239 candidates:

```text
maximum direct/completed-square gap    5.169878828456423e-26
maximum frozen formula bound           6.991903031031288e-19
maximum physical scaling gap           6.883572491054685e-23
maximum physical scaling bound         1.553608101451456e-22
```

The formula gap is more than six orders of magnitude below its conservative
bound. The physical R63 inertia also re-closes as `SPACING^2*F` within its
predeclared numerical bound.

## Path atlas

| outer | blocked by v6 | safe ascent candidates | best exponent | best dual ascent |
|---:|:---:|---:|---:|---:|
| 1 | no | 16 | 1 | `2.3357354711e-11` |
| 2 | yes | 15 | 1 | `4.6467265130e-11` |
| 3 | yes | 15 | 1 | `3.5602090758e-11` |
| 4 | yes | 14 | 2 | `2.9989975634e-11` |
| 5 | yes | 15 | 2 | `2.4648404360e-11` |
| 6 | yes | 14 | 2 | `1.8207042352e-11` |
| 7 | yes | 14 | 2 | `1.2728503282e-11` |
| 8 | yes | 14 | 2 | `9.6993387247e-12` |
| 9 | yes | 14 | 2 | `7.6787122984e-12` |
| 10 | yes | 14 | 2 | `5.7337761523e-12` |
| 11 | yes | 15 | 2 | `4.8165485469e-12` |
| 12 | yes | 15 | 2 | `3.8522833122e-12` |
| 13 | yes | 15 | 2 | `3.2088530025e-12` |
| 14 | yes | 15 | 2 | `2.6796964739e-12` |
| 15 | yes | 15 | 2 | `2.2393436942e-12` |
| 16 | yes | 15 | 2 | `1.8542427204e-12` |

All four scalar/sign/box-active controls pass. Old solver, proportioning and
filter-pair semantics remain respectively `bd568e0f...f0e2`,
`9203252f...ea03` and `bdeeab4b...8ab2`. v9 semantic is
`77cbed07c8cdc615110677cbae0266946b1fed777f2ba6ceba875a1ba4f583af`.
The old sparse and diagnostic work ledgers are unchanged; no candidate was
applied.

## Claim ledger

| Claim | Status | Evidence | Ceiling |
|---|---|---|---|
| M1: strict primal inertia is the wrong inner merit | `SUPPORTED_BOUNDED` | 235/235 safe candidates ascend exact dual; 15/15 blocked outers covered | one convex linearized fixture |
| M2: inner filter is required on this path | `REFUTED_BOUNDED` | exact scalar dual already orders every safe candidate | outer nonlinear filter remains separate |
| M3: merit ownership changes by phase | `REFUTED_BOUNDED` | uniform ascent coverage | one 16-outer trajectory |
| M4: direct sign/scale is wrong | `FALSIFIED_BOUNDED` | two formula forms, physical reclosure and analytic controls pass | binary64 numerical correspondence |

Evidence classes: `ANALYTIC_DERIVATION`, `NUMERICAL`, `CORRESPONDENCE`.
This is not a machine-checked proof.

## Consequence

Freeze a new transaction experiment that selects the largest strict
completed-square dual ascent under the inherited fixed-density and
cached-normal gates, applies it, and verifies the committed direct dual from
the already required fresh all-row audit. Keep filter-SQP for the outer
nonlinear globalization roadmap; do not make it an inner convex-block merit.
