# NSR2-C1 -- 512-particle rejection trace contract

Status: `FROZEN / DIAGNOSTIC_IMPLEMENTATION_AUTHORIZED / REPORT_ONLY`

Identity: `nuv-newton-krylov-r0`

Parent evidence: NSR2-C fails only the work gate on `scale_8x8x8`, with 13
rejected trials and 153 HVP calls. This stage changes no objective, derivative,
stopping threshold, trust radius rule or acceptance rule.

## Required trace

For every outer trial of the unchanged 512-particle case, record:

- current objective and scale-aware residual;
- radius before and after the trial;
- inner stop reason, iterations, HVP calls and step norm;
- predicted reduction, actual reduction and their ratio;
- acceptance decision;
- current/trial pair count and pair additions/removals;
- current/trial pressure-active count and active-set additions/removals.

The trace implementation must be observational: the existing NSR2-C report
must retain raw SHA-256
`efc5a14d4b096afc15ed099b166786da1c30a276eecfec8018e2f4b6fcf720a4`.
Two diagnostic reports must also be byte-identical.

## Classification rules

- `ARITHMETIC_FLOOR`: rejected trials begin only when both predicted and
  actual reductions are within `1024 * epsilon * max(abs(E), 1)` and the
  current scaled residual is within one order of the `1e-8` stop.
- `ACTIVE_SET_MODEL_MISMATCH`: rejected trials coincide with a pressure-active
  membership change while reductions remain above that arithmetic floor.
- `SUPPORT_TOPOLOGY_MODEL_MISMATCH`: rejected trials coincide with pair
  additions/removals, without a pressure-active change, while reductions
  remain above that arithmetic floor.
- `SMOOTH_MODEL_CONDITIONING`: rejected trials occur above the arithmetic floor
  with neither active-set nor support membership change.
- `MIXED_OR_UNRESOLVED`: more than one material class occurs or the trace does
  not uniquely identify a class.

The classification is evidence, not authorization to change the solver. A
remediation, if any, requires its own frozen contract and one discriminating
control. No coefficient, radius, tolerance or iteration sweep is allowed.

