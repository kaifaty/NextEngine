# NSR3-B4E1S -- nominal Hydro spectrum contract

Status: `FROZEN / IMPLEMENTATION_AND_EXECUTION_AUTHORIZED / NO_KKT_TRAJECTORY`

Identity projection (exact bytes, no final LF):

```text
nextengine.nonlocal.nsr3b4e1s-hydro-spectrum|v1|parent=c53a112830cb94c4139da75c2044d28f61673cb1aa05c116098967aee41f7bbe:79a8932136a64c1cfa953caafe323cb9e860cb134c1d4dc0907be7f40891587a|scenario=c430b679dfeec33a6ac12c51df75ddee7e7bc48484c6c05188219f0a727909a0|initial-frame=999cc0c925e52dc873be53f911d3effc0a2bf48fe8c5538e9fe3286b14fc76c7|static=daafa32e95eea258c51704d30d7654a702778d560d59fab749a96180b0a6b297|pairs=26ab8b79194d53686510d59a11e013900c79ccad57585549acc2007aa1e66414:354630:615072:118|active=9;strain=6.6613381477509392e-16|spectrum=lanczos48;target=.15;mass=.125;macro-dt=1/240|runs=2-byte-exact;timing=external|gate=finite,repeat,initial-substeps<=96|trajectory=none|credit=b4e1m-design-only
```

Identity SHA-256:
`76453ea9d74f996c52a710a126c444b57662485aaad74c71e041e9e952fe17ac`.

## Frozen parent and command

B4E0 PASS is mandatory. Add only:

```text
nonlocal-formula-reclosure --nominal-hydro-spectrum-probe
```

Reconstruct the exact B4E0 Hydro fluid/support, immutable static index and
one-pass flat neighborhood. Require all frozen parent counts/roots before the
first HVP. Do not rebuild through nested rows or execute an all-pairs audit.

## Spectrum and policy discriminator

Run the selected `boundary_pressure_spectrum_joint_workspace` twice over
separate same-state workspaces. Each estimate uses the existing deterministic
direction, full reorthogonalization and exactly 48 Lanczos HVP calls. Require:

- both workspaces have 6,000 fluid, 5,824 support, 354,630 pairs, 615,072
  directed records, maximum degree 118 and the frozen pair/static roots;
- initial evaluation has exactly nine active centres and maximum positive
  strain exactly `6.6613381477509392e-16` as a parent diagnostic;
- both estimates pass, have finite nonnegative and bit-identical maximum
  eigenvalue, eigenfrequency and derived substep count;
- `frequency=sqrt(max_eigenvalue/0.125)` and
  `initial_substeps=max(1,ceil((1/240)*frequency/0.15))`;
- each trace records 48 joint HVPs, zero candidate/audit all-pairs HVPs, exact
  lifecycle and zero live workspaces after explicit release;
- `initial_substeps <= 96`.

No expected eigenvalue is frozen before execution. If the final substep gate
fails, report `TEMPORAL_POLICY_CAPACITY` and do not redesign/tune in the same
identity.

## Repeatability, cost and exit

Run the command twice in fresh processes and require byte-identical canonical
JSON. The report contains no clock, RSS or host path. `/usr/bin/time` facts are
recorded separately and may motivate B4EP, but cannot affect PASS.

Keep `kkt_started=false`, `trajectory_started=false`,
`reference_curve_decoded=false`, `b4e_comparison_execution_authorized=false`,
`runtime_authority=false` and `production_authority=false`.

PASS selects only `NOMINAL_HYDRO_SPECTRUM_CANDIDATE` and authorizes B4E1M
one-macro research/contract design. Failure grants no later execution.
