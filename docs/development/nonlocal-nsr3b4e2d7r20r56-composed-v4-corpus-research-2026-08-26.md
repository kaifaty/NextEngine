# NSR3-B4E2D7R20R56 composed v4 corpus research

Status: `PASS / COMPOSED POLICY CERTIFIES V4 5 OF 5`.

## Question

Do the root-agnostic R50 failed-inverse refiner and R55 slope refiner compose
without cross-triggering and certify all five immutable v4 cases?

## Bounded discriminator

Replay each v4 case once with both default-off hooks installed. Give each case
a fresh R50 centered trace capped by the inherited semismooth structural cap and
a fresh R55 slope trace capped at one inverse audit. The hooks share no counter,
certificate or mutable trace state.

Require the torsion case to reproduce the exact R50 trajectory and use only
centered failed-inverse refinement. Require counterflow to reproduce R55 and use
only slope refinement. Corner, helical compression and alternating layers must
trigger neither hook and retain their R51 roots.

## Hypotheses

| ID | hypothesis | discriminator |
|---|---|---|
| C1 | mechanisms compose independently | 5/5 certify; trigger matrix is torsion `5/0`, counterflow `0/1`, others `0/0` |
| C2 | one mechanism perturbs another case | any cross-trigger or historical root/work mismatch |
| C3 | composition exposes a later boundary | exact hook policy holds but fewer than five cases certify |

C1 would close the current v4 correctness corpus for this default-off research
policy. It would authorize production-roadmap design and broader blind-source
validation, not immediate default/runtime promotion.

## Outcome

C1 is confirmed exactly. The trigger matrix matches the contract, all five
cases certify ordinarily, and no mechanism triggers outside its intended
numeric predicate. Broader validation now requires a newly committed blind
source corpus.
