# NSR3-B4E2D7R6 cap/accuracy discriminator evidence

Date: `2026-08-22`

Status: `FAIL / INNER_SUBPROBLEM_NUMERICAL_FLOOR / PRIVATE_ONLY`

## Reproducibility

Two clean Release builds produce byte-identical 4,858,008-byte executables at
SHA `26ab20ef38cc0e874f88841c12a347d2596070e8050cce8358c1f516de8de13e`
and Build ID `4e0e28fadf482d8ba8c8a1c689d341384dfeba28`.

Both fresh D7R6 processes exit one with empty stderr and byte-identical
44,094-byte stdout reports at SHA
`6979ebf9f0b88fb57cbcbbaed6721951cb7fa05acb8f7d4b8c39962f257b9f6f`.
The semantic result is
`9e93beb38632b10d9a7a1b8473edcb65d7d5ed672d268597071aa5bbbda00dfa`.
Raw evidence is under
`/home/kaifaty/.cache/nextengine/external/run-nonlocal-b4e2d7r6.OdJC1S`.

D7 through D7R5 retain their exact historical stdout hashes and exit states:

```text
D7    e0542abc4e0c7ff0b38acc0fe38095e270dec030617ad5183665414ce9b3db11  exit 1
D7R   4b0272df2d46bb53ec09175ea7095b0e8c699ac9e0e4e093c6e20dfd00240801  exit 1
D7R1  9a9582d09897f63ed7cd9953cc2fb6153b4266813fb6797ba71a1de98140cbfe  exit 1
D7R2  f6b18542b1a25a548a11925552f014148152c817c8de557a86f9e97f4c628bcb  exit 1
D7R2R 36a6c45b08e595818a4ce86a7bde3d88d3a923f250e0ed9d859973aeb083786e  exit 0
D7R3  b52f9a599f2b0312fbe73ed8686f623a7142a967b497f507345ed7a47bc00f3c  exit 0
D7R4  975da3f5adc13bba6c5fec3cfe08f0bc395f8fac58882b841ba5026774d6e886  exit 0
D7R5  7ea5489fa90346f385ce8db08dba445f2e2389e65fdf556a13a17e837ba916d7  exit 0
```

## Frozen controls

The complete D7R5 parent is exact. The shared first-eight outer root remains
`9bffc61a...82c2`, its state root remains `04a9c033...d95e`, and the
`eta=1e-8` lane through outer 13 reproduces D7R5 exactly with private state
root `1fdbcb76...6546`.

Inactive, reset and forced-rollback controls pass. The public root remains
`3af35d5d...91b`; trajectory steps, public commits and physics mutations are
zero.

## Lane results

No frozen positive route is reached. Every lane stays finite and
dual-feasible, but every lane eventually fails its unchanged inner policy:

| `eta` | completed post-prefix outers | failed outer | failure | failed trials / HVP | final stationarity |
|---:|---:|---:|---|---:|---:|
| `1e-8` | 50 | 58 | `REJECT_LIMIT` | 11 / 20 | `1.1975523969668175e-8` |
| `1e-9` | 5 | 13 | `MINIMUM_TRUST_RADIUS` | 9 / 16 | `1.1785409868777534e-9` |
| `1e-10` | 3 | 11 | `MINIMUM_TRUST_RADIUS` | 6 / 10 | `1.534119330122981e-10` |
| `1e-11` | 3 | 11 | `MINIMUM_TRUST_RADIUS` | 6 / 10 | `1.534119330122981e-10` |
| `1e-12` | 3 | 11 | `MINIMUM_TRUST_RADIUS` | 6 / 10 | `1.534119330122981e-10` |

The three tightest lanes are identical through the same outer-11 failure.
That is direct negative evidence against treating another fixed tolerance
decade as progress.

The baseline also violates the frozen exact primal-monotonicity gate. After
reaching zero positive violation it returns to a small positive violation at
outer indices `15,20,25,30,35,40,45,50,55`. The positive peaks decrease, but
the five-update cycle is not monotone and therefore cannot be reclassified as
cap exhaustion.

## Near-gate observation

The `eta=1e-10` lane completes outer 10 at:

```text
positive constraint          8.3502094128107274e-12
absolute dual change         1.0239444292459154e-8 J
equivalent pressure change   8.1915554339673236e-5 Pa
position update              1.4998551094366294e-10 dx
inner stationarity           1.0849482654499924e-14
```

The primal, absolute-dual and pressure values are the same dimensionally
derived bound and miss it by only `1.0239444292459154x`. The immediately
following inner solve nevertheless bottoms out at stationarity
`1.534119330122981e-10`, or `1.5341x` its requested tolerance, after one
step-norm interpolation and four quarter-radius updates.

This is not evidence that the gate should be relaxed by 2.4%. It is evidence
that the current nested solver cannot certify which side of the gate the
pressure state occupies.

## Research interpretation

Birgin and Martinez explicitly study the practical AL case in which a
subproblem cannot be solved to its prescribed precision despite being
solvable in exact arithmetic, and warn that the algorithm must classify and
handle that numerical difficulty rather than silently continue. Sun and
Nocedal likewise show why classical trust-region acceptance can fail when
function differences reach an evaluation-error floor. Scaled AL stopping
criteria can be useful, but they do not by themselves authorize weakening our
absolute pressure-state contract.

Primary sources:

- [Birgin and Martinez, nonmonotone-penalty AL under numerical difficulty](https://optimization-online.org/2010/06/2662/)
- [Andreani et al., scaled safeguarded-AL stopping criteria](https://optimization-online.org/2020/08/7985/)
- [Sun and Nocedal, trust regions for noisy function values](https://arxiv.org/abs/2201.00973)
- [Nonlocal authors' publication page](https://peridynamics.com/publications.html)

The announced `Semi-Implicit Pairwise Descent for Nonlocal Continuum
Mechanics` paper and code remain `to appear` as of this evidence date. They
cannot yet be used to replace or validate the local pressure-state solver.

## Decision

Preserve D7R6 as a hard `INNER_FAILURE`. Do not select a larger outer cap, a
tighter fixed inner tolerance, a relaxed pressure gate, a new `beta`, or a
semismooth solver from this result.

Freeze D7R7 as a replay-only failure-mechanism discriminator over the three
unique failed states. It must expose trial-level model/direct/raw reductions,
ULP scale, trust-radius ownership and exact active/pair topology before one
next mathematical remedy may be researched.

