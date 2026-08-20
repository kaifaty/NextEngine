# Nonlocal FCR3-B1 term-local evidence — 2026-08-20

Status: `PASS / ISOLATED_PRESSURE_SELECTED / REPORT_ONLY`

## Result

The frozen `P, V, S, PV, PS, VS, PVS` discriminator reproduced the known
combined failure and selected `isolated-P` as the minimal remediation scope.
The viscosity-only, surface-only and viscosity-plus-surface controls pass the
unchanged FCR3-B quality gate. Every mask containing pressure fails it.

| Mask | Gate | FCR2 gradient | SISSM v1 gradient | SISSM iterations | Backtracks |
|---|---:|---:|---:|---:|---:|
| P | FAIL | `6.4706636074863672e-8` | `4.7192795652880601e-3` | 80 | 0 |
| V | PASS | `1.2992638963598233e-6` | `1.1690924487344373e-7` | 80 | 1618 |
| S | PASS | `1.3188093437259378e-11` | `1.3189254235289553e-11` | 10 | 1 |
| PV | FAIL | `1.5666493679180159e-5` | `3.6688927249202054e-3` | 80 | 0 |
| PS | FAIL | `3.634398477320999e-6` | `9.7716721485485974e-3` | 80 | 0 |
| VS | PASS | `1.7624669860091208e-6` | `1.1448306028041191e-10` | 80 | 756 |
| PVS | FAIL | `3.3092290056831499e-6` | `3.6826938583085457e-3` | 80 | 0 |

The pressure masks monotonically reduce the objective and accept `alpha=1`,
but their residual plateaus above the fixed-budget reference gate. This rules
out line-search rejection as the first cause and does not authorize a larger
iteration cap.

## Reproducibility

Command:

```text
nonlocal-formula-reclosure --sissm-term-local-self-test
```

- two reports were byte-identical;
- raw report SHA-256:
  `2ef351755844b658c05e89b19a4063282cd964246711bef6377c9d2d576b79dd`;
- semantic result SHA-256:
  `39e5218118d667c46e7a2443db00b7233237d54c086d978b4c2043a5d995a854`;
- implementation commit: `ee91248d57c8b80803cde536489b8bb438e62248`;
- FCR0, FCR1 and FCR2 raw report hashes remain respectively
  `996eff3d61126491a3c1c92b6147d1c1f3eec0487d4b9dee45caadc6588b4345`,
  `ead18de38f7e5fa68602c99f69cd891d11034502fe935fd473978040f2672140`
  and `10b98cb4d29935062d25a649994112c09598ece552023d88b22110340a92a2a8`.

## Consequence

Exactly one pressure-only remediation is authorized. It must compare the
released-code/SISPH conservative split used by v1 with the literal Nonlocal
Eq. 20/26 split, without changing material coefficients, tolerances, geometry
or the 80-iteration budget. Chebyshev, profile reclosure, CUDA and runtime
promotion remain blocked.
