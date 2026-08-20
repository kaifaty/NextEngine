# FCR3-B1 — SISSM term-localized coupling discriminator

Status: `FROZEN FOR IMPLEMENTATION / REPORT_ONLY`

## Fixture

Use the exact FCR2 combined tetrahedron geometry, velocities, rest-density
derivation, time step, support and iteration cap. Do not alter any nonzero
coefficient. Execute these masks in order:

```text
P     pressure
V     bulk + shear viscosity
S     surface
PV
PS
VS
PVS   known failing control
```

Each mask runs both the FCR2 reference and corrected SISSM v1. Apply the same
objective, gradient, finite, monotonic, momentum and physical-direction gates
as FCR3-B. Performance is reported but does not gate this localization run.

## Decision rule

1. If an isolated term fails, remediation scope is that term's split.
2. Otherwise, the first failing two-term mask in the fixed `PV, PS, VS` order
   identifies the minimal coupling scope.
3. If all pairwise masks pass but `PVS` fails, remediation scope is the
   three-way composition/overshoot policy.
4. No coefficient, iteration, relaxation or tolerance sweep is allowed.

Two reports must be byte-identical and FCR0–FCR2 remain unchanged. The result
authorizes one implementation remediation only inside the selected scope. It
does not authorize Chebyshev or profile reclosure.
