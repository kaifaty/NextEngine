# NSR3-B4BF -- contact-onset forecast controller contract

Status: `FROZEN / IMPLEMENTATION_AUTHORIZED / B4B2_BLOCKED`

Parent B4B1 is exact FAIL with semantic SHA-256
`11302033bacf1a3656c3b584f9db68f573e088b786c56ceea10b32dfee509a2e`
and JSON-without-final-LF SHA-256
`2b40f5662f9aa38f2bc6bae35d4e9d2e21c9b0f967af40c1831562d7fa4c98c9`.

## Identity and immutable state

```text
feasible-contact-onset-spectrum-r0
```

Use only the exact first P1 macro-frame start and detached P2 initial state.
Reuse the B4B1 KKT substep, fixed-192 state, aggregate gates, macro frame,
spectral target and all coefficients unchanged. No trajectory continues past
one macro frame.

## Published level curve

From the immutable P1 start execute independent KKT intervals with

```text
n = 1,2,4,8,16,32,48,96,192.
```

Report adjacent embedded gates and every level versus fixed-192: RMS
position/velocity, COM, q99 height/front and kinetic error. This curve is
diagnostic; no count is selected by best-of-level search.

## Forecast policy

1. Form the macro predictor with the existing semi-implicit recurrence.
2. Clamp displacement to the exact box without committing it.
3. If projected position has no active pressure centre, choose `n=1` and
   charge zero spectrum HVPs.
4. Otherwise run the unchanged 48-HVP pressure spectrum at the projected
   position and choose
   `ceil(Hf*sqrt(lambda_max/M)/0.15)`.
5. Execute immutable `(n,2n)` KKT intervals and commit neither.

Require `1 <= n <= 192`, both intervals pass all KKT/ledger/work gates, their
embedded gate passes, and the fine `2n` interval passes every inherited
fixed-192 frame comparison including kinetic `<=0.15`. Charge forecast HVPs
and both candidate intervals.

For detached P2 require projected pressure inactive, `n=1`, zero forecast
HVPs and the one-step interval bit-identical to exact free flight with zero
multiplier/reaction.

## Repeatability and exit

Two reports must be byte-identical. B4B1, B4BK1, B4BK r0, B4B r0, B4A, B3R,
D5, original B3 and B2 raw reports remain exact.

PASS selects `CONTACT_ONSET_SPECTRAL_FORECAST_CANDIDATE` and authorizes only a
separately frozen B4B2 full-corpus retry. FAIL preserves B4B1 and opens
controller research; it cannot relax the kinetic gate or introduce a fitted
minimum count.

No P2 impact, general mesh, nominal water, neighborhood, CUDA, runtime or
production authority is granted.
