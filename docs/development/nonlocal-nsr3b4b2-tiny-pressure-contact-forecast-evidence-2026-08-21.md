# NSR3-B4B2 tiny pressure contact-forecast evidence -- 2026-08-21

Status: `PASS / TINY_PRESSURE_CONTACT_FORECAST_CANDIDATE / B4C_DESIGN_AUTHORIZED`

## Reproduction

```text
nonlocal-formula-reclosure --tiny-pressure-contact-forecast-self-test
```

Three complete reports are byte-identical:

```text
raw JSON plus LF  43477c6d539896194c6c8d6df52781821d47eb90b2b4b37712442dbca64aa74f
JSON without LF   8f9c0fb92216770a11f8f0b603bb66f5ce27858815d73b90b2e09266ec19e64b
semantic result   b79e537d44519391d9ec65f56134f409122c10ca7135d62f797b026d062b7286
```

Observed wall times are `21.53 / 20.25 / 20.14 s`; these are reproducibility
observations, not a benchmark or production-performance claim.

## Full corpus result

Both frozen cases complete with the unchanged KKT substep, fixed
`48/96/192` references and physical, comparison and work gates:

| Case | Accepted / executed | Spectral / nonlinear HVP | Max density strain | Max speed | Max ledger |
|---|---:|---:|---:|---:|---:|
| supported-column startup | `296 / 444` | `384 / 3007` | `6.409e-4` | `0.2162 m/s` | `5.635e-10` |
| released-block impact | `82 / 124` | `96 / 218` | `1.0804e-2` | `0.6149 m/s` | `1.898e-10` |

The fixed-reference convergence ratios are `1.782/1.837` for P1 position /
velocity and `2.001/1.873` for P2. Maximum per-frame fixed-192 kinetic error
is `2.053%` for P1 and `2.979%` for P2, below the frozen `15%` limit. Neither
case creates mechanical energy.

## Forecast behavior

P1 frame zero is pressure-inactive at the committed start but
`FORECAST_ACTIVE`; its predicted spectrum selects `21` initial and `42`
accepted substeps. The remaining seven frames are `START_ACTIVE` and select
`16` or `21` initial substeps.

P2 retains the exact inactive path for frames `0--13`: zero spectral HVPs and
one initial substep. Frame 14 alone becomes `FORECAST_ACTIVE`, selects
`13/26`, and frame 15 is `START_ACTIVE` at the same count. First executed
contact occurs at `0.044791666666666667 s`; the frozen contact-time gate
passes.

The forecast never mutates the committed state or owns an impulse. Every
forecast HVP is charged, and every embedded comparison restarts from the
immutable frame state.

## Non-regression

The following raw reports remain exact:

```text
B4BF  309c99aec8299e3c03afff45d7cc6e2ec0036730aaa7f26408693d7eeaf0df13
B4B1  949b590057827de329b60b9b7a9979672b800f73ad0fa1a43b7a330cd6b2c791
B4BK1 0da0758e2e375f403b6cfb061b7c2e6a1e7f499a07d29c299a7a4b8d92542343
B4BK  21d63ff86b2d248623bd5bdd8ba363e7caca4142ff9a2cbdcb337be6553438e9
B4B   a8fdaaf0d2ac358783b13f6beb02c506e67cb023bdd50a0bf2739e6d900624d8
B4A   6e46cb7650e48b156bf5d837d3590b4019792787d5f7fbd4ca1af9eeea5afd48
B3R   89ede039a67b8e0f7f4a27d5ecd412f4691245ee8acaec612dd0894b2153edad
D5    38845883a1f689aa1f126f58633d94605d8662e914c2165211c3b52577ec4261
B3    c64ad0b8d73f7ada62364a3daa2d3bed66a8bfc5a1fe6c0148b7c1f8f2e5fb2f
B2    d6ba5f8e802966c25283d0c8384ed01beec20b347acb343cf5c7c2bf360d69d9
```

## Decision

Select `TINY_PRESSURE_CONTACT_FORECAST_CANDIDATE`. This closes only the tiny
pressure/contact trajectory gate and authorizes B4C joint fluid/support
neighborhood plus canonical-runner design. Nominal water execution, general
collision, viscosity, surface tension, internal aperture, CUDA, runtime and
production integration remain blocked.
