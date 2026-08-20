# NSR3-B0R -- normalized cubic formula-reclosure contract

Status: `FROZEN / IMPLEMENTATION_AUTHORIZED / PHYSICAL_EXECUTION_BLOCKED`

New objective identity: `nuv-variational-fcr2`.

Parent solver identity: `nuv-newton-krylov-r0`.

## One authorized change

Add immutable `kernel_scale` to the research configuration:

```text
kernel_scale = 1 / [dx^3 sum_{k in Z^3} W_raw(|k|dx;H)]
```

where the finite sum includes the self sample and every integer offset inside
support. Apply that same factor to:

```text
W
dW/dr
d2W/dr2.
```

`H/dx=3` is the only admitted profile in B0R. The expected binary64 scale is
`7.985668078772472`. No material coefficient, support, time step, active-set
rule, pair enumeration, solver tolerance, trust policy or floor stop may
change.

FCR1 remains available with implicit scale one. Its code paths and raw reports
must remain byte-identical. FCR2 must be selected explicitly; it cannot replace
the default of an old command.

## Required controls

1. **Normalization algebra:** repeat NSR3-B0 integral, lattice-density,
   derivative and radial-moment controls under FCR2.
2. **Energy gradient:** pressure, normal viscosity, tangential viscosity,
   surface and combined directional derivatives agree with centered binary64
   finite differences at relative error `<=1e-7` away from branches.
3. **Pair closure:** every internal pair remains equal/opposite with normalized
   momentum residual `<=1e-12`; unique-pair/full-coefficient remains canonical.
4. **Physical-density active control:** the center of a sufficiently large
   uncompressed `H=3dx` lattice is `rho0` within `1e-12`; a declared `0.99`
   isotropic spacing compression activates pressure and the objective/gradient
   remain finite.
5. **Curvature:** analytic FCR2 HVP agrees with centered gradient differences
   under the NSR0 tolerances and with a tiny dense Hessian assembled from basis
   products. Symmetry and active-margin reports are mandatory.
6. **Trust solve:** the unpreconditioned safeguarded Newton-CG path has positive
   actual/model reduction on every accepted trial, no state mutation after a
   reject, and terminates only under an existing declared stop.
7. **Tape correspondence:** A1 and A2 remain bit-exact under FCR2; performance
   is not remeasured in B0R because the coefficient scale changes work, not the
   validity of the already selected storage optimization.
8. **Historical immutability:** the eight frozen FCR1/NSR raw report SHA-256
   values and A1 semantic result hash remain unchanged.
9. **Repeatability:** two FCR2 reports are byte-identical and contain a new
   timing-independent result root.

## Capacity and failure

- B0R controls admit at most `512` particles and `80*N` unique pairs.
- Kernel-scale enumeration is bounded by
  `(2*ceil(H/dx)+1)^3`, which is `343` samples at the admitted ratio.
- Non-finite scale, scale outside `[7.5,8.5]`, mismatched support, capacity
  overflow or any derivative/HVP mismatch fails before publication.
- One implementation/transcription defect may be corrected without changing
  this contract. A second formula mismatch stops FCR2.

## Exit

PASS selects only `FCR2_NORMALIZED_OBJECTIVE_CANDIDATE` and returns to
NSR3-B1 manufactured multi-step design. Static boundary reclosure is still
required before hydrostatic or dam-break trajectories. PASS grants no CUDA,
runtime, public-schema, save/replay or production authority.

