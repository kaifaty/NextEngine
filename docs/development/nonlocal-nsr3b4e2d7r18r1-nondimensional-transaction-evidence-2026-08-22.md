# NSR3-B4E2D7R18R1 nondimensional AL transaction evidence

Date: `2026-08-22`

Status: `PASS / NONDIMENSIONAL_AL_TRANSACTION_CANDIDATE / D7R19_BLOCKED`

Implementation commit: `0e568232961b334df7c58f33190f03d8963d0b6f`.

## Result

The tiny directly normalized transaction gate passes and selects:

```text
NONDIMENSIONAL_AL_TRANSACTION_CANDIDATE
```

The normalized dual state is `u=lambda/kappa`, the sole combined scale is
`theta=kappa*dt^2/M`, and both the reference and aligned-substep profiles use
the exact binary64 value `0x3fc5cccccccccccd`. This is formulation evidence,
not a nominal solve or production-scale result. D7R19 remains blocked.

## Direct formula and precision evidence

The reference and aligned profiles are byte-exact across direct dense and
static-bound sparse objective, gradient, HVP and divided-difference paths.
Their common normalized root is:

```text
c750a70c83bb36b724c557e8232326aa1ab2bc635c7236b1536ac49ef51d0601
```

Independent directly normalized precision evaluations also agree across the
two profiles:

| Oracle | Root | Reduction sign |
|---|---|---:|
| binary64 divided difference | included in normalized root | `-1` |
| Linux x86-64 long double | `3d612fa964f02705704b0b508ebde7c41197e89c0a9a53a78f14809fb596c777` | `-1` |
| IEEE binary128 | `fd2eae0be2411fcdf6094d80edb90b8e74ba3a37b9090b97564280094c88c534` | `-1` |

One-ULP mutation of `theta` from `0x3fc5cccccccccccd` to
`0x3fc5ccccccccccce` changes the normalized root to
`4c8faa25a2e9704d6f3d17bf2520f8c95a3308e9127cbc9ca32f484891e943f8`.

## Dimensional reconstruction certificate

The old dimensional HVP was independently evaluated at both physical scales
and compared componentwise with `M/dt^2` times the normalized HVP. The frozen
absolute forward certificate uses maximum directed degree `107`, operation
count `10528` and `gamma_n=2.3376856006561943e-12`.

| Profile | Maximum observed error | Maximum component bound | Minimum margin | Result |
|---|---:|---:|---:|---|
| reference | `4.7184478546569153e-14` | `3.812746729180834e-10` | `7.696960729647833e-11` | PASS |
| aligned `dt/78`, `6084*kappa` | `3.80168785341084e-10` | `2.3196751100336193e-6` | `4.683059382934425e-7` | PASS |

This closes D7R18's isolated relative-HVP mismatch without weakening its
post-observation tolerance. The mismatch was caused by dimensional scaling and
floating-point accumulation order, not by a different normalized operator.

## Dual admission mapping

The exact transformed limits are retained:

```text
dual-u             0x3da1eed347666340
complementarity-u  0x3d6cb1520bd70533
```

The reference legacy decision and normalized decision agree. The maximum
reference `lambda -> u` update mapping error is
`4.336808689942018e-19`, and all primal, normalized stationarity, dual,
complementarity, position-state and `u_next=max(0,u+c)` values are byte-exact
between reference and aligned profiles.

Raw equivalent pressure change grows from `2.89686246161596` to
`17624.5112164715` under the `6084x` penalty scale and is therefore retained
only as a diagnostic. A later nominal stage must still pass the independent
kinematic pressure-impulse and support-reaction ledger.

## Failure, work and lifecycle controls

- all `18` zero/negative/NaN/positive-infinity/negative-infinity scale and
  non-finite-dual cases reject before topology, workspace, pair, precision or
  HVP work;
- four workspaces build and release exactly, maximum live count is two;
- two long-double and two binary128 normalized audits execute;
- candidate all-pair calls remain zero;
- position, prediction and trial roots remain unchanged;
- D7R18 reproduces at stdout SHA-256
  `0267e094641aaddd9b604a71b527db976aaa699f51dacd558624512bd3b1b925`.

An initial local probe retained both two-workspace profiles simultaneously and
correctly failed the frozen lifecycle gate at maximum live count four. The
implementation now releases the reference pair before constructing the
aligned pair; no formula value changed. Do not reintroduce cross-profile
workspace retention.

## Reproducibility

Raw evidence directory:
`/home/kaifaty/.cache/nextengine/run-nonlocal-b4e2d7r18r1.BsadIf`.

Two independent clean GCC 15.2 Release builds produce identical
`5,420,608`-byte executables:

```text
SHA-256  b9b8c9f606159496cab2ee67469795d9a476402b170bf083f0567d0970fd6788
Build ID 5b67c1423e59794e5a085e8e633d1042a4dac16d
```

Both R1 processes exit zero with empty stderr and emit the same `2,862`-byte
stdout:

```text
stdout SHA-256  b63aa9851e40658c80737ddc2138419bad8833bf8879646de1bab9544d745fbd
semantic result eeb29e67830518104c347105bcf600998eb00a3394e7c4b8169f701863bd570e
```

Each clean binary also directly reproduces D7R17's complete `7,549`-byte
stdout at SHA-256
`a2a8de930d41d8e49c255a3fcf987a8794b4da067ddadf658732946fca0ffced`
with empty stderr.

Build directories:

```text
/home/kaifaty/.cache/nextengine/build-nonlocal-b4e2d7r18r1-a.AQr7hK
/home/kaifaty/.cache/nextengine/build-nonlocal-b4e2d7r18r1-b.gYTeW0
```

## Decision

Retain the directly normalized AL representation for the next private
research stage. Research and freeze one tiny full normalized transaction that
reproduces D7R13 confirmation, holdout, accepted-sign precision and rollback.

Do not run D7R19, another nominal substep, a macro, trajectory or timing lane.
No production penalty, runtime state, parallel/GPU path or authority is
selected by this result.
