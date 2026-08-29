# NSR2-C2 -- numerical energy-floor stop contract

Status: `FROZEN / IMPLEMENTATION_AUTHORIZED / REPORT_ONLY`

Identity: `nuv-newton-krylov-r0`

Candidate: `numerical-energy-floor-stop-v1`

## Hypothesis

When the trust model predicts a decrease below the representable resolution of
the accumulated binary64 energy, an actual/predicted ratio is not meaningful.
If both the physical force-to-displacement residual and the proposed step are
already nanometric, terminating explicitly is safer than accepting or rejecting
steps according to subtraction noise.

## Frozen rule

After Newton-CG and the model HVP, but before trial objective evaluation, define:

```text
energy_scale = max(1,
  abs(E_inertia) + abs(E_pressure) + abs(E_viscosity) + abs(E_surface))
energy_floor = 1024 * epsilon_binary64 * energy_scale
scaled_step = ||p||_2 / spacing
```

Return `NUMERICAL_ENERGY_FLOOR` success without mutating state only if all hold:

```text
0 < predicted_reduction <= energy_floor
scaled_displacement_residual <= 1e-7
scaled_step <= 1e-7
```

The ordinary `RAW_GRADIENT <= 1e-10` and
`SCALED_DISPLACEMENT <= 1e-8` stops retain priority and remain unchanged. The
new state is not an accepted trial and performs no trial objective or pair
build. No coefficient, trust-radius, forcing, acceptance or iteration setting
changes.

At spacing `0.05 m`, both physical guards are `<=5 nm`, below the existing
`1 um` report quantum. The `1024` factor is fixed before execution and must not
be swept.

## Corpus and gates

- exact modified all-pairs/neighborhood solver correspondence on 8/27/64;
- 125/512/1000 scaling with the unchanged NSR2-C fixtures;
- every normal stop retains residual `<=1e-8`;
- every floor stop proves all three frozen predicates and residual/step
  `<=1e-7`;
- outer trials `<=32`, rejects `<=8`, HVP `<=128`, momentum `<=1e-12`, pairs
  `<=80*N`, neighbors `<=160`;
- the 512 case must not accept a trial that satisfies all three numerical-floor
  predicates;
- two reports byte-identical;
- NSR0, NSR1, NSR2-A, NSR2-A1, NSR2-B, NSR2-C and NSR2-C1 raw reports remain
  byte-identical.

PASS selects the bounded CPU solver candidate and authorizes NSR3 measurement
and physical-corpus specification. Failure stops this remediation; it does not
permit a tolerance or factor sweep.
