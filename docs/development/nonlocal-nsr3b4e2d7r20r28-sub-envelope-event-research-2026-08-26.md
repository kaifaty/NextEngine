# NSR3-B4E2D7R20R28 sub-envelope event research

Status: `RESEARCH COMPLETE / BIDIRECTIONAL EVENT BRACKET SELECTED`.

## Question

Which exact projector event lies between the R26 iteration-20 current state
and its smallest existing trial `alpha=2^-20`, and is there a rigorously safe
Armijo-valid point immediately before or after that event?

## Competing hypotheses

| ID | hypothesis | observable discriminator |
|---|---|---|
| E1 | one zero-bound component event truncates an otherwise valid current-face model | one nearest linear root; the certified old-side point has current mask and positive Armijo |
| E2 | the direction encounters coupled component or ball events | co-nearest roots, a ball root, or the certified new-side point crosses more than one face |
| E3 | the model is acceptable only after the event | old-side margin non-positive and new-side margin positive |
| E4 | neither local side is an ascent step | both rigorously signed margins are non-positive |
| E5 | finite precision prevents a side certificate | root, forward bound, representable side or margin cannot be rigorously signed |

## Selected discriminator

Replay exact R27 shear and retain only the rejected iteration-20 direction.
On its fixed current face, enumerate every component lower/upper event using
the R21 linear/quadratic KKT equations. Independently enumerate active-ball
entry/exit roots from the free-component norm quadratic. Keep every finite
admissible root in the open interval `(0, 2^-20]`, sort by exact binary128
alpha and expose multiplicity and the distance to the next event.

Only if the nearest event is a unique zero-bound component with nonzero affine
derivative, construct the symmetric certified bracket from the R25
componentwise evaluation bound:

```text
alpha_old = nextafter(root - Bz / abs(dz), 0)
alpha_new = nextafter(root + Bz / abs(dz), +infinity)
```

The two points must be positive, ordered, lie below `2^-20`, remain before any
second event and select exactly the expected old/new masks with unchanged ball
activity. Shadow-evaluate both with the unchanged direction and exact R19
Armijo formula. Do not select or apply either point.

## Claim ceiling and stop rule

This may identify a pre-event relinearization candidate, a safe crossing
candidate, local model rejection or event ambiguity on one rejected shear
direction. It cannot extend the line cap, alter the trajectory, establish
12/12 convergence, select runtime policy or make a performance/production
claim. Stop after the two conditional shadow evaluations; do not add a ULP or
dyadic sweep if the bracket fails.
