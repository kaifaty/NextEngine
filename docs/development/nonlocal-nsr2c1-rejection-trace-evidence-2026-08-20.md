# Nonlocal NSR2-C1 rejection trace evidence -- 2026-08-20

Status: `PASS / ARITHMETIC_FLOOR_IDENTIFIED / REMEDIATION_REQUIRED`

## Outcome

The 512-particle work failure is caused by objective-difference resolution at
the arithmetic floor, not neighborhood topology, pressure active-set changes
or a missing global preconditioner.

After trial 11 the current scaled displacement residual is
`4.2012516705799287e-8`. The next unconstrained Newton-CG step has length
`1.0985245026012308e-9 m` and predicted decrease
`4.4382493242539421e-14`. The binary64 resolution guard for the accumulated
scene energy is approximately `3.27e-10`, over three orders of magnitude
larger. Direct subtraction reports `-8.1854523159563541e-12`, so the trust
ratio rejects the step.

The controller then multiplies the radius by `0.25` for 13 trials. For trials
12--24 the radius remains larger than the unconstrained step, so Newton-CG
returns the exact same step and the solver reevaluates the exact same trial.
All 13 rejections have:

- zero pair additions/removals;
- zero pressure-active additions/removals;
- identical step, predicted reduction, actual reduction and ratio;
- classification `ARITHMETIC_FLOOR` under the frozen contract.

At trials 25 and 26 the radius finally clips the step. Both trials are accepted
only because subtraction noise flips sign and reports actual decreases of
`1.59e-12` and `2.27e-12`, respectively, despite predicted decreases of only
`3.75e-14` and `6.93e-15`. Their ratios (`42.49` and `328.29`) are therefore
not credible model-agreement measurements. The final low residual is real, but
the acceptance decisions that reach it are numerically unresolved.

## Exact artifacts

| Artifact | SHA-256 |
|---|---|
| diagnostic raw report, run 1 | `223433a92f59b35e3ebadb11628c509db09cf801f44d80e16c3dd1c5c5ec76e6` |
| diagnostic raw report, run 2 | `223433a92f59b35e3ebadb11628c509db09cf801f44d80e16c3dd1c5c5ec76e6` |
| semantic result | `c6f6a6018101a5083a61bef392d6c8be9ab31385572974be7d77f40597971db2` |
| parent NSR2-C after instrumentation | `efc5a14d4b096afc15ed099b166786da1c30a276eecfec8018e2f4b6fcf720a4` |

## Decision

Freeze NSR2-C2 `numerical-energy-floor-stop-v1`. It may terminate only when
the model decrease is below a conservative binary64 energy-resolution bound
and both the scale-aware force residual and proposed displacement are at most
`1e-7` of particle spacing. At `0.05 m` spacing this caps each at `5 nm`, still
200 times below the existing micrometre report quantum. This is a guarded
numerical termination state, not a relaxed ordinary convergence tolerance.

