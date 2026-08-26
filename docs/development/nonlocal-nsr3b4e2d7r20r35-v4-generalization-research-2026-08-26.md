# NSR3-B4E2D7R20R35 v4 solver generalization research

Status: `RESEARCH COMPLETE / ONE-SHOT GENERALIZED POLICY SELECTED`.

## Question

Does the mechanism selected through R32 certify all five blind, excited v4
problems without case-specific constants or post-observation tuning?

## Competing hypotheses

| ID | hypothesis | discriminator |
|---|---|---|
| G1 | the ordinary R26 path already generalizes | all cases certify without exhausted recovery or terminal selection |
| G2 | terminal precedence generalizes | an existing Armijo-rejected trial is fully KKT-certified and terminates |
| G3 | the one-shot zero-bound recovery generalizes | one analytically unique forward-certified release restores progress |
| G4 | a new mechanism is required | active-set/direction failure, ambiguous/nonzero/ball event, second exhaustion or cap remains |

## Selected policy

Use the existing verified-inverse, dual-ratio and certified-event solver with
the unchanged 32-step cap. After a complete ordinary line rejection:

1. scan only its already evaluated trials in inherited power order and return
   the first full unchanged KKT certificate as terminal state;
2. otherwise, if no exhausted recovery has yet occurred for that case and all
   21 trials cross component masks without ball change, analytically enumerate
   every component/ball event in `(0,2^-20]`;
3. only for one unique nearest admissible zero-bound release from mask `-1/+1`
   to `0`, derive the R25 sparse forward bound, construct the first
   representable safe new-side point, and require one predicted mask change,
   unchanged ball and positive rigorous Armijo before applying it;
4. continue unchanged, but never permit a second exhausted recovery.

Any other rejection is classified and retained. No v4 root, iteration, scalar
or event may be hardcoded. The v4 cases execute once after the contract is
committed; no retry or tuning follows from their result.

## Controls and ceiling

R34 problem roots are immutable inputs. R32 must separately retain semantic
`e7b9acaf...0f73`. Report complete case roots, terminal/recovery counts, event
identities, line work and final KKT tuples.

Even 5/5 is bounded blind binary128 solver evidence, not an independent oracle,
binary64/runtime correspondence, a physical-model validation, performance,
GPU suitability or production readiness.
