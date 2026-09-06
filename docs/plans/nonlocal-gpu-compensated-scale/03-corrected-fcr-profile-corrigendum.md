# NCGP3 corrected-FCR profile corrigendum

| Field | Value |
| --- | --- |
| Research ID | `NCGP3` revision 4 |
| Status | `FROZEN / CORRECTNESS_FIRST / REPORT_ONLY` |
| Supersedes | The ambiguous material-coefficient sentence and density-gate interpretation in NCGP3 revision 1 |
| Mathematical parent | FCR2 selected by NSR3B0R kernel-normalization reclosure |
| Claim class | Unchanged: bounded 4k correctness; no 50k or performance claim |

## Reason for the corrigendum

The first NCGP3 4k implementation inherited the historical raw FCR1/NCGP1
parameters. Its independent infinite-lattice check reproduced the already
known raw-kernel density ratio `0.12522433816880058`. NSR3B0 and NSR3B0R had
previously established that this raw cubic kernel integrates to `1/8` and is
physically ineligible. Consequently, that run is a retained negative control,
not a new refutation of the corrected-FCR model requested by the water plan.

This corrigendum freezes the corrected profile before it is implemented in the
NCGP3 CUDA, CPU and long-double paths. It does not change a measured result or
relax a numerical tolerance.

## Frozen corrected profile

Retain the NCGP3 geometry, mass, time step, capacity and state representation.
Select these immutable FCR2 material values:

```text
kernel_scale = 7.985668078772472
kappa        = 1226.25 J
lambda       = 1.4138231728735551e-5 kg m/s
mu           = 0
gamma        = 0.010664424039285813 m/(kg s^2)
```

`kernel_scale` multiplies the cubic kernel value and both radial derivatives
identically: `W`, `dW/dr` and `d2W/dr2`. It is part of profile, input, work and
result identity. Surface shape functions are not multiplied by this scale.

The historical raw profile remains executable only as a named negative
control. It must reproduce a lattice-density ratio near
`0.12522433816880058` and must not be admitted as physical water evidence.

## Density gate semantics

FCR2 uses the compression-only pressure coordinate
`J_i = max(rho_i / rho0, 1)`. A free surface has a truncated neighbor support
and therefore a deficient SPH density estimate even when the material is not
under tensile pressure. The physical pressure gate consequently measures the
same one-sided compression observable:

```text
e_i = max(rho_i / rho0 - 1, 0)
density_rmse = sqrt(mean(e_i^2))
density_max  = max(e_i)
```

The frozen limits remain `density_rmse <= 5%` and `density_max <= 10%`.
Raw minimum, maximum and mean density are still reported for diagnosis, but a
free-surface support deficit is not reclassified as compressive density error.
The active-pressure ID signature must remain identical between CPU, corrected
GPU and permuted GPU routes.

## Required gates before trajectories

Before any 16-step or 240-step trajectory:

1. independently sum the corrected infinite reference lattice and require a
   density ratio within `1e-12` of the selected NSR3B0R value;
2. prove that the raw profile still fails the physical admission control;
3. run the retained graph, boundary and executable rollback controls;
4. run one corrected hydrostatic step on CPU, corrected GPU and permuted GPU;
5. require the frozen position, HVP, active-signature and one-sided density
   gates without coefficient tuning or retry-to-green.

Failure of the corrected route is first-specific and blocks later timing. A
raw-profile failure cannot be used to claim `PHYSICS_REFUTED` for FCR2.
