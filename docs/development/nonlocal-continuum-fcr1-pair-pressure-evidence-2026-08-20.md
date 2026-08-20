# Nonlocal FCR1 pair/pressure evidence — 2026-08-20

Status: `PASS / FCR2_AUTHORIZED / REPORT_ONLY`

## Outcome

FCR1 selects two previously implicit parts of `nuv-variational-fcr1`:

```text
CPU pair traversal: unique unordered pair i<j + both endpoint writes
pressure potential: compression-only max(rho/rho0-1,0)^2
surface mapping:    source-shaped strength = physical gamma * particle mass
```

The discriminator passes twice byte-identically. It authorizes a minimal f64
solver, not physical coefficients, CUDA or runtime integration.

## Pair enumeration results

| Strategy | Viscosity error | Surface error | Decision |
|---|---:|---:|---|
| unique pair, full coefficient | `0` | `0` | selected CPU reference |
| both directed edges, half coefficient | `0` | `0` | mathematically equivalent; not CPU canonical |
| both directed edges, full coefficient | `0.5` | `0.5` | rejected: doubles the update |
| unique pair, half coefficient | `0.5` | — | rejected: halves the update |

Every tested endpoint accumulation has exact zero momentum-closure residual.
The selected order is lexicographic `SampleId`, which gives FCR2 one explicit
deterministic accumulation rule. A later CUDA gather must prove correspondence
rather than inherit this claim.

## Pressure results

All particles in the bounded controls are below rest density. The selected
compression-only rule produces exactly zero pressure force. The literal
two-sided comparator conserves momentum but produces tensile attraction:

| Case | Density range (`kg/m^3`) | Selected maximum force | Two-sided inward corner force | Momentum residual |
|---|---:|---:|---:|---:|
| pair | `14.1855` | `0` | `1862.20` | `0` |
| `3x3x3` patch | `49.0651 .. 107.091` | `0` | `16527.37` | `3.59e-17` |

This is a semantics discriminator, not a calibrated water fixture. FCR3 must
reclose kernel normalization/support and coefficients at product scale.

## Surface parameter result

With `m=0.125 kg`, treating source `strength` as physical `gamma` has relative
error `0.875`. Passing `strength=gamma*m` matches the declared energy-derived
update exactly. The result freezes parameter units for the new identity but
does not calibrate macroscopic surface tension.

## Exact artifacts

| Artifact | SHA-256 |
|---|---|
| FCR1 raw report, run 1 | `ead18de38f7e5fa68602c99f69cd891d11034502fe935fd473978040f2672140` |
| FCR1 raw report, run 2 | `ead18de38f7e5fa68602c99f69cd891d11034502fe935fd473978040f2672140` |
| result root in report | `3713f1a5b9cf9cb620559c21055bbb716e159b5eabf0325e0f10f7cba5de19b1` |
| executable | `e94999d6fc924642de08c6aa648ddfb6c9e2cae1e2eebfbf72285a69911e17dc` |
| implementation `.cpp` | `d21b52c027610a498628d973f20772d2ab42e94d5b694218b5564126d0262922` |
| unchanged FCR0 report | `996eff3d61126491a3c1c92b6147d1c1f3eec0487d4b9dee45caadc6588b4345` |
| unchanged NPR1-A report | `464a55bc33741f18ddc4b3c1cda6b46b5248bb653789a5e31585482f871ee207` |
| unchanged NPR1-B failing report | `e8ed6950e33fb9388887c40e304c74b517761698c0c7accbaff9a99899bf2c3a` |

Implementation commit: `f12061492c80e6bc9a35a646d62a87c524f45f77`.

## Decision

Proceed to FCR2 with the selected CPU traversal and compression-only
potential. Product profile selection, long corpus, CUDA, performance credit
and runtime authority remain blocked.
