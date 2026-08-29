# Nonlocal corrected GPU term audit — independent revision-1 review

| Field | Value |
| --- | --- |
| Research ID | `NCGA0` revision 1 |
| Verdict | `NO-GO / REFUTED_AS_WRITTEN` |
| Candidate | `349f12e66be970276d91c96bda1f33fad1a9919e` |
| Tree | `5856ea2a15de806e50d504f920ba6a9757436aeb` |
| Parent | `9b5021aa29b763c4db3d7ed8c975ab9f99a861fc` |
| Diff SHA-256 | `d5eba020404a4e8559dbae1aea63d11067a787faa3ba9319dc41108e09afe26a` |
| Contract SHA-256 | `71538590e6a6dfa8f2a7c55ffe02ccc91785be13e280d5ab6b997851a2e29bc2` |

## Load-bearing finding

Revision 1 names the viscosity coefficients of one directed edge as
`lambda/2` and `mu`, but the host and CUDA evaluators both emit the full
undirected-pair energy gradient using `lambda` and `2*mu`. The parent FCR
contract distinguishes those two observables explicitly. Because both sides
made the same interpretation, their agreement did not prove the literal
revision-1 claim.

The negative control covered only the kernel-gradient chain factor and could
not expose this shared factor-of-two interpretation in the viscosity terms.
On the frozen fixtures, the literal half-coefficient forces differ from the
reported forces by at least `0.029`, far beyond the `2e-5` correspondence
bound; pair closure cannot distinguish them because either convention remains
equal and opposite.

## Reviewer verification

- All candidate, tree, parent, diff, contract and four source identities
  matched.
- A fresh Release build passed twice with byte-identical stdout SHA-256
  `5cbf70a7a26c428d1eee0ee0c3e85aa9fd935e0079a48bb78f6155f035c77d41`.
- `memcheck`, `initcheck` and `synccheck` each reported zero errors.
- The retained source-shaped GPU control passed `11/11`; NPR1-B reproduced its
  expected gradient failure; FCR0 passed.
- An independent probe extracted doubled versus literal-contract values for
  both viscosity fixtures.

These successful execution checks do not override the observable mismatch.

## Required repair

The package must choose and state one observable. The selected physical
observable for the next revision is the full derivative of the frozen
undirected-pair energy, because that is the endpoint force checked by FCR0.
Revision 2 must:

1. state explicitly that the force coefficients are `lambda` and `2*mu`, while
   the energy coefficients remain `lambda/2` and `mu`;
2. add an independent finite-difference derivative of that energy;
3. require a half-force/directed-edge negative control to reject both viscosity
   fixtures;
4. retain all revision-1 inputs, numeric bounds, source-gradient negatives and
   repeatability gates unchanged.

Only one batched repair and one re-review are allowed. No neighborhood, matrix,
solver, trajectory, performance or product claim is admitted by this review.
