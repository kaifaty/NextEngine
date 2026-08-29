# Nonlocal NSR3-B1S2 spectral-policy evidence -- 2026-08-20

Status: `FAIL / SPECTRAL_ONLY_ERROR_POLICY_REJECTED / REPORT_ONLY`

## Outcome

The frozen spectral target `dt*omega_max<=0.15` repairs the B1S position error
and substantially reduces velocity error, but all three 2% compression cases
remain just above the unchanged normalized velocity limit. B1S2 therefore
rejects `PRESSURE_SPECTRUM_SUBSTEPS_R0`.

Every spectrum, substep count, trajectory convergence, conservation and
capacity gate otherwise passes. The result separates two roles: spectrum is a
valid stiffness/initial-step estimator, but it is not by itself an error
estimator for finite-amplitude motion.

## Matrix

| Spacing | `kappa` factor | Substeps | `dt*omega` | Error / `dx` | Error / `c` | Result |
|---:|---:|---:|---:|---:|---:|---|
| `0.99` | `0.25` | 20 | `0.1453` | `0.00400` | `0.000394` | PASS |
| `0.99` | `1` | 39 | `0.1490` | `0.00906` | `0.000402` | PASS |
| `0.99` | `4` | 78 | `0.1490` | `0.01900` | `0.000402` | PASS |
| `0.98` | `0.25` | 22 | `0.1461` | `0.01030` | `0.0010005` | FAIL |
| `0.98` | `1` | 43 | `0.1495` | `0.02316` | `0.0010212` | FAIL |
| `0.98` | `4` | 86 | `0.1495` | `0.04844` | `0.0010212` | FAIL |

All self-convergence ratios lie between `1.884` and `1.916`. Position,
kinetic-energy and pressure-exit gates pass. Each trajectory remains finite,
ends pressure-inactive, has no density overshoot or rejected trust trial and
stays under `3 / 0 / 6` maximum outer/reject/HVP work per step.

The exact counts `20/39/78` and `22/43/86`, all 48-HVP spectra and their B1S1
eigenvalues are reproduced. `kappa=0` and inactive states select one substep
without running Lanczos.

## ULP correction

The first pilot classified several mathematically one-step pressure-exit
differences as greater than `dt` because separately accumulated binary64 times
differed by a few ULPs. The one implementation correction adds only a
`32*epsilon` comparison allowance in the time observable. It changes no
trajectory, target or physical threshold. After correction, every 1% case
passes and the remaining failures are solely the normalized velocity error.

## Interpretation

The miss is small but systematic: `0.05%` at low stiffness and about `2.12%`
at the two larger stiffnesses. Lowering the spectral target after observing
this matrix would be another fitted global safety factor. It would not prove
generalization to a new compression amplitude.

The better next hypothesis is an embedded controller:

```text
spectral omega_max -> initial n
n versus 2n       -> measured local/interval error
accept or refine  -> 2n versus 4n
```

Spectrum remains useful for avoiding a dangerously coarse first attempt;
step-doubling measures the amplitude-dependent truncation error that spectrum
does not encode.

## Repeatability and lineage

- B1S2 semantic result SHA-256:
  `ce64874a7f891bae7e4f6387d8d444b556a15c26734e6bbc8b7dfd05f9115081`;
- two byte-identical raw reports:
  `29cd40a7e0fb7eeef292de2751b1deb4b2b415543604ce2f26c7c4744688fb12`;
- B1S1 remains byte-identical PASS at
  `3b8dcccc49303e7fb27d1c4791352e9b03d8dc5ab201fec9dbb3d1d56225d64d`.

## Decision

Reject `PRESSURE_SPECTRUM_SUBSTEPS_R0` as a standalone error policy while
retaining B1S1's spectral premise. Freeze B1S3 for a bounded embedded
step-doubling controller and include a previously unseen 3% compression
holdout. B1R/B2, CUDA and runtime remain blocked.

