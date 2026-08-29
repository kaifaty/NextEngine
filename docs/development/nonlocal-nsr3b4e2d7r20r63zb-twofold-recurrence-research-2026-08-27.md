# NSR3-B4E2D7R20R63ZB twofold recurrence research

Status: `RESEARCH_COMPLETE / CONTRACT_FROZEN / REVIEWED_NEGATIVE`.

Evidence: the final reviewed producer takes
`TWOFOLD_RECURRENCE_LADDER_REJECTED` at result semantic
`acfe6bafe9569eba101ed1e352cc01e149023e295bd8c082bdac58e49a709ebf`.
All finite apparatus, identity, arithmetic, positivity, verifier and work
boundaries close, but states `x0,x1,x2` each remain `12+/24-/66?`. See the
[R63ZB evidence](nonlocal-nsr3b4e2d7r20r63zb-twofold-recurrence-evidence-2026-08-27.md).

## Strongest bounded conclusion

R63ZB should execute one complete deterministic exported-factor K2 PCG lane
through states `0..2`. This is the first end-to-end portable candidate-
generation discriminator. It combines only boundaries already selected:

```text
K2 start/preconditioner centers     R63ZA
immutable K2 common coefficients   R63Z
K2 affine solution verifier         R63Y
fixed recurrence shape              R63S/R63V
```

The huge R63ZA dependency radii are excluded from recurrence state. They bound
distance to an ideal triangular solve but are not uncertainty in the exact
deterministic `(hi,lo)` center. Propagating them would recreate R63W's lost-
correlation failure and answer a different question.

## Competing hypotheses

| ID | Hypothesis | Evidence | Decision |
|---|---|---|---|
| H0 | Sealed K2 centers reproduce a certifiable state-2 exported PCG candidate | R63Y verifies imported state 2 at `1.01e-3`; R63Z products are contained; R63ZA center/reference differences are at most `0.0424` | selected discriminator |
| H1 | Propagate R63ZA triangular radii through PCG | radii already reach `1e25` from dependency, not center error | explicitly rejected |
| H2 | Keep binary128 scalar/update arithmetic around portable products | would evade the production arithmetic question | offline comparator only |
| H3 | Stop as soon as state 2 passes | state 2 is a frozen research frontier, not a runtime policy | execute exactly states 0,1,2 and report only |
| H4 | Add residual replacement, flexible CG or GMRES now | changes recurrence before the minimal port is tested | fallback only after a localized failure |

## Frozen recurrence

1. Generate `x0` with the R63ZA exported-factor K2 solve of the immutable RHS.
2. Compute `r0=b-H*x0`, `z0=M^-1 r0`, `p0=z0`, and positive `rho0=r0^Tz0`.
3. For iterations 1 and 2, compute `Hp`, positive `den=p^THp`,
   `alpha=rho/den`, then K2 `x+=alpha*p`, `r-=alpha*Hp`.
4. After iteration 1 only, solve `z=M^-1r`, compute positive `rho_new`,
   `beta=rho_new/rho`, and `p=z+beta*p`.
5. Seal states `x0,x1,x2` before invoking the R63Y verifier. Require states 0
   and 1 to reject and state 2 to resolve `24+/78-/0?` with the frozen sign
   root.

Every scalar/vector operation uses canonical twofold add/multiply/divide from
R63ZA with radius zero on the deterministic operands. Each matvec accumulates
a new K2 center directly from the immutable R63Z K2 coefficient artifact in
the same flattened order. R63Z's proof-oriented `Dot2Err` publishes a
one-component center plus radius; both remain independent audit telemetry and
are not recurrence operands. Positivity is decided from the sealed K2 scalar
itself, never from a binary128 comparator.

## Fixed work

```text
states/certificates             3
common operator products        3
exported-factor solves          3
triangular terms           30,906
triangular divisions          612
Krylov dot products             4
Krylov scalar divisions         3
solution updates              204
residual updates              204
direction updates             102
total vector updates          510
adaptive stops                  0
```

All three certificates execute. A failed state-0/1 expectation or state-2
certificate cannot change the recurrence already executed.

## Primary-source grounding

- Hida, Li and Bailey, [double-double arithmetic
  algorithms](https://www.davidhbailey.com/dhbpapers/qd.pdf), ground the fixed
  twofold scalar operations selected and controlled by R63ZA.
- Ogita, Rump and Oishi, DOI
  [10.1137/030601818](https://doi.org/10.1137/030601818), ground accurate dot
  reductions without native wider types.
- Iakymchuk et al., *Reproducibility strategies for parallel Preconditioned
  Conjugate Gradient*, DOI
  [10.1016/j.cam.2019.112697](https://doi.org/10.1016/j.cam.2019.112697),
  supports fixed reproducible reduction identities for later parallel ports.
- Simoncini and Szyld, DOI
  [10.1137/S1064827502406415](https://doi.org/10.1137/S1064827502406415),
  motivates bounding inexact operator/preconditioner actions before admitting
  a Krylov method. The current local gates, not the citation, decide success.

## Result interpretation and next boundary

State 2 fails under both the finite producer and an observational binary128
replay of the same finite artifacts. This selects neither a portable
arithmetic producer nor a general failure of PCG/K2. It localizes the next
question to the changed operator/input lineage rather than recurrence
rounding alone.

R63S and the comparator already use the same exported factor, so cross tangent
versus K2 common-block products with original versus projected RHS/inverse-
scale inputs under the same recurrence and final R63Y certificate. The fixed
`2x2` experiment must identify the first state/product/solve at which the
selected state-2 lineage is lost. Do not fit a tolerance, add a third word
after observing the result, use exact signs, change Krylov method, or start
timing in the same gate.

No result authorizes an adaptive two-update runtime stop, nonlinear state
commit, timing, GPU or production.
