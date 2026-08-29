# Nonlocal NPR0 profile-bridge evidence — 2026-08-20

Status: `REPORT_ONLY / NPR0_A_B_SCALE_LAW_COMPLETE / BOUNDARY_OPEN`

## Result

The quarantined Nonlocal feasibility tool now names and audits the three
one-axis product bridge profiles. Every bridge completes the retained fused
P1 plus checked compact-P2 correspondence preflight with finite output and
exact P1/P2 output/CSR identity. A separate binary64 algebraic test proves the
dimensionally derived scale law through four SISSM iterations.

This closes scale/cadence/support *execution preflight*, not physical
reclosure. None of these profiles has sealed density support or contact, and
the single-call CUDA timings below are diagnostic rather than percentile
performance evidence.

## Frozen bridge records

| Profile | Profile SHA-256 | P2 report SHA-256 | Result |
| --- | --- | --- | --- |
| `nuv-basin-48k-source-scale.v2` | `5594d9a1ca7cfd594695d64d503f87d30232e197b68c23b52258be9843cca1a6` | `6816648d7b49850a7dbc9aa637ea62f857befaea96fc28989c0706baf1a64064` | exact P1/P2; finite; `20,917,770` compact bytes |
| `nuv-basin-48k-cadence.v2` | `b65b270f238ed887f18a49858d0101f61cb2d9e1a5375461fa34a3ad508189a3` | `92e76ac4ed49ae2b46df1b50ac0cb360ddb07f04d979596467de3df8bb0fd23b` | exact P1/P2; finite; `20,917,770` compact bytes |
| `nuv-basin-48k-spec-support.v2` | `a6bdf3be0673e322ca76e0aa26c27e6f50849a99bf1d2186db668717e0b149d5` | `b6fae5820d211faf05db3876d8f0a4476d74ff79867b7930c3fe8982b4248cc6` | exact P1/P2; finite; `12,325,706` compact bytes |

The machine audit report SHA-256 is
`04b0c4c482c54b32a582c72b24158df4a0cc80f2312d563f0be1015b948d3a70`.
Its command result is `PASS`, while its semantic result intentionally remains
`PROFILE_RECLOSURE_REQUIRED`.

The preflights each execute one diagnostic GPU call. Their observed totals
were approximately `3.27 ms`, `3.31 ms` and `1.06 ms` in bridge order. They
are not warmup/percentile campaigns and award no integrated-budget credit.

## Dimensionless scale-law discriminator

For length ratio `s` and time ratio `t`, the source/matrix update is
algebraically similar under:

```text
x'       = s x
v'       = (s/t) v
g'       = (s/t^2) g
m'       = s^3 m
kappa'   = (s^4/t^2) kappa
lambda'  = (s^3/t) lambda
mu'      = (s^3/t) mu
gamma'   = (s/t^2) gamma
```

The executable discriminator uses `s=10`, `t=25/6`, enables every term and
compares four binary64 gather iterations. It passes at tolerance `5e-11`:

| Quantity | Maximum scale-aware error |
| --- | ---: |
| Density | `6.90e-16` relative |
| Source | `1.18e-14` / scaled spacing |
| Local matrix | `3.55e-15` absolute |
| Next position | `1.56e-17` / scaled spacing |
| Velocity | `1.55e-17` / scaled spacing-per-step |
| Momentum residual | `1.03e-17` absolute |

The report SHA-256 is
`b14c73b0de3f0c28f22a8c5431c1ff4d0983220ed3c82764b816adaba3f7eb56`.
The test makes no energy-similarity, calibration, stability or physical-quality
claim. In particular, changing `h/dx` from three to two remains an empirical
product-profile question even though the pure `s,t` transformation closes.

## Regression preservation

Adding the v2 records did not mutate the canonical retained profiles:

- `nuv-water-48k.v0` remains
  `cb1868b86b4d9d40e996ebfa8e529982f73e647e358dd9f0a7c9269fce1211e8`;
- `nuv-water-50k-coherent.v1` remains
  `5463af89fccc1b9dcb79fd522b2399fc10ac80f5b0490378e0ecb2bb73d5cc9e`.

## Consequence

NPR0 may advance to the boundary discriminator with two clearly separated
coefficient identities: unchanged source coefficients as the counterfactual
control and dimensionally derived coefficients as a hypothesis. Neither is a
selected water calibration.

The next step must bind the existing SPEC-38 two-layer `REST_VOLUME` support
and an actual non-penetration mechanism separately. Fixed ghost density
support alone cannot be relabelled as a sealed analytical boundary.
