# NSR3-B3 boundary-composition research -- 2026-08-21

Status: `COMPLETE / POST_SOLVE_SWEEP_HYPOTHESIS_SELECTED / EXECUTION_PENDING`

## Question

After B2 proved the smooth support formula and hard contact independently,
what exact order can compose them without silently changing the selected
Nonlocal objective or losing auditable momentum accounting?

## Primary-source boundary

The Nonlocal Unified solver predicts positions, minimizes its unified smooth
energy, commits the final position and reconstructs velocity from the
position increment. Its paper explicitly lists fluid-solid collision as
future work. It therefore supplies no inherited ordering or correctness claim
for collision ([Liu et al., 2026](https://doi.org/10.1145/3799902.3811196)).

The separate semi-analytical boundary paper performs reduced-order CCD to
construct a nonpenetrating **initial guess**, then iterates a different SISPH
bulk-plus-contact-energy update. Its collision potential, adaptive contact
parameters and line-search filter are part of that solver identity; the paper
also identifies two-way feedback to moving/deformable solids as future work
([Liu et al., 2026, CGF](https://peridynamics.com/publications/2026-Liu-SAM.pdf)).

Consequently, neither of these orders can be attributed to the selected FCR2
objective. B3 must test an explicit engine-side composition and preserve its
operator-splitting error as evidence.

## Competing compositions

### A. Sweep the predictor before the smooth solve

```text
predict y* -> contact sweep -> use contacted y* as inertia target -> solve
```

This keeps the optimizer's initial target outside the wall, but folds the
contact correction into inertia. The resulting velocity change no longer has
a separately observable contact impulse, and the smooth solve can cross the
wall again. A second sweep would still be required.

Disposition: rejected for B3. It obscures rather than removes the split.

### B. Solve first, sweep the accepted segment

```text
predict y*
  -> solve inertia + fluid-centred pressure support
  -> tentative smooth state ys
  -> analytical sweep x -> ys
  -> accepted nonpenetrating y
  -> v = (y - x) / h
  -> rebuild support, active set and spectrum before reuse
```

This preserves the exact smooth objective and exposes a separate contact
impulse. It can, however, move the state away from smooth stationarity and
reduce temporal order at impact. Rebuilding before the next substep is
mandatory; cached curvature cannot survive contact.

Disposition: selected as the smallest falsifiable B3 hypothesis.

### C. Add the semi-analytical contact potential to FCR2

This would couple contact inside the nonlinear solve and might reduce
splitting artifacts. It also changes the objective, globalization, geometry
representation and parameter set. The existing gradient/HVP and B1R1 roots
would no longer be sufficient.

Disposition: deferred as a new formula identity only if B3 rejects B.

## Momentum ledger

For one smooth substep with

```text
y* = x + h (v_n + h g)
E = m/(2h^2) ||y-y*||^2 + Phi(y;q),
```

stationarity gives

```text
m (v_s - (v_n + h g)) = -h grad_y Phi.
```

Translation closure gives `sum grad_y Phi + sum grad_q Phi = 0`, so define:

```text
fluid support impulse     Jf_support = -h sum grad_y Phi
static support reaction   Jb_support = -h sum grad_q Phi
contact fluid impulse     Jf_contact = m (v_after - v_s)
contact boundary reaction Jb_contact = -Jf_contact.
```

The complete step ledger is

```text
Delta P - M h g + Jb_support + Jb_contact = 0.
```

Support and contact reactions must remain separate report fields. Adding
their vectors early would hide double counting or a sign error.

## Risks that B3 must discriminate

1. **Post-contact nonstationarity:** the sweep can re-activate pressure or
   increase smooth energy; the next substep must rebuild rather than reuse the
   pre-contact tape.
2. **Impact order reduction:** a smooth `1/2` convergence ratio is not assumed
   across the first-contact event; candidate accuracy is compared directly to
   a converged fixed-step reference.
3. **Chatter:** gravity can create a contact every substep after rest. Contact
   count and repeated feature IDs are published rather than hidden.
4. **Negative curvature:** B2's support Hessian is indefinite. Steihaug
   negative-curvature exits remain valid and are counted.
5. **Reaction ambiguity:** pressure support reaction and geometric contact
   reaction must close independently and together.
6. **Tunnelling:** post-solve endpoint clamping is insufficient for a general
   triangle mesh, but an exact swept sphere against the frozen axis plane/box
   is sufficient for this B3 discriminator only.

## Wall-anchored layer invariant

An adjacent counterexample was checked before implementation. If ghost
positions are incorrectly anchored to a fluid sample initially at
`y=0.035 m`, a nominal third sample at `-0.115 m` moves from `r=H` to
`r=0.14 m` when the fluid reaches contact and changes density by about
`5.6e-5 rho0`. Such a state would invalidate the B2 layer claim.

That geometry is not the B2 side-oriented lattice. For a lower wall at `a`,
the wall-owned layers are

```text
q_l = a + R - l*dx,  l=1,2,...
```

while every nonpenetrating fluid centre satisfies `y>=a+R`. Therefore the
omitted third layer obeys

```text
|y-q_3| >= 3dx = H.
```

It is exactly zero at contact and outside support everywhere else; diagonal
edge/corner distances are no smaller. Two layers are therefore sufficient
through wallward motion **only with a boundary-owned lattice origin**. B3
freezes this ownership and rejects any fluid-relative ghost translation.

## Decision

Freeze and execute the bounded
[B3 contract](../plans/nonlocal-nonlinear-solver-research/03b3-boundary-composition-smoke-contract.md).
PASS would establish only that the selected split composes on tiny static
plane/corner trajectories. Failure preserves B2 and either changes ordering
under one bounded remediation or opens a new unified-contact formula lineage.
