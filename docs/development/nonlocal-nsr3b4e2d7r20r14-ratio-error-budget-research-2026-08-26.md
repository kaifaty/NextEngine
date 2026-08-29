# NSR3-B4E2D7R20R14 ratio error-budget research

Status: `RESEARCH COMPLETE / INVERSE COUNTERFACTUAL SELECTED`.

## Mathematical decomposition

For a leaving coordinate, R13 encloses

```text
r = x / (x - z)
b = gamma_512 (1 + |r|)
  + (e_x + e_z) (1 + |r|) / (x - z),
```

where `e_x` is the accumulated current-direction enclosure and `e_z` is the
new passive principal-solve enclosure. The observed total bound alone cannot
tell which error source is causal.

## Competing hypotheses

| ID | causal hypothesis | discriminator |
|---|---|---|
| E1 | candidate principal-solve error `e_z` dominates both collisions | a verified inverse sharply reduces `e_z` and makes both ratio orderings strict |
| E2 | candidate error explains shear, while accumulated `e_x` retains the corner collision | only shear becomes strictly ordered |
| E3 | current-direction error dominates both | verified candidate refinement resolves neither |
| E4 | the inverse itself cannot be certified on the failing principal systems | verified-inverse audit rejects before the counterfactual ratio |

## Selected counterfactual

After the unchanged ratio test has already returned ambiguous, retain its
decision and run the existing verified-inverse audit on that same final
principal factor/solution. Re-evaluate the ratio intervals with only `e_z`
replaced by the refined enclosure. Record both components and the new ordering,
but do not feed it back into NNQP.

This is cheaper and more discriminating than immediately changing the
direction-error recurrence. A subset result would justify a distinct shear
remedy and a separate derivation for the accumulated corner enclosure.

