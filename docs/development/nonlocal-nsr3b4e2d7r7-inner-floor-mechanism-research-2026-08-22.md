# NSR3-B4E2D7R7 inner-floor mechanism research

Date: `2026-08-22`

Status: `RESEARCH_COMPLETE / CONTRACT_FROZEN / NOT_RUN`

## Question

D7R6 rules out a simple fixed cap/tolerance selection: its baseline develops a
five-update primal cycle and its tighter lanes hit the unchanged trust-radius
floor. Which mechanism owns those failures?

The next step must distinguish three possibilities without accepting a new
trial:

1. the PHR active-set kink changes the local branch near zero constraint;
2. raw binary64 objective subtraction loses the physically descending signal;
3. the trust model or its derivatives cease to predict the direct energy
   change even on stable topology.

## Evidence carried forward

There are only three unique failed states:

```text
eta=1e-8   failed outer 58   REJECT_LIMIT
eta=1e-9   failed outer 13   MINIMUM_TRUST_RADIUS
eta<=1e-10 failed outer 11   MINIMUM_TRUST_RADIUS
```

The `1e-10`, `1e-11` and `1e-12` lanes are identical. Replaying all five
would add no information and would obscure the common-state proof.

D7R1/D7R2 already showed that counts alone are insufficient: active PHR
membership and compact-support radial branches must be identified exactly.
D7R3/D7R4 already selected step-norm-aware radius interpolation. D7R7 must
observe that exact policy rather than return to the stopped legacy quartering
experiment.

## Source findings

- Practical augmented-Lagrangian algorithms need an explicit path when an
  inner problem cannot meet its requested tolerance in finite arithmetic;
  silently increasing work is not a convergence argument.
- Trust-region ratios become unreliable when objective differences approach
  the function-evaluation error scale. A raw/model disagreement must therefore
  be compared to an explicit ULP/error scale.
- Scaled stopping criteria may reduce unnecessary work, but our public
  pressure-state gate is absolute and dimensioned. A scaled inner criterion
  cannot retroactively waive it.
- Primal-dual semismooth methods are plausible only after the replay proves an
  active-set/complementarity mechanism. They are not selected by citation.
- The Nonlocal group's pairwise-descent successor is still unavailable for
  formula/code review and remains only a reconsideration trigger.

Primary sources:

- [Birgin and Martinez, AL subproblem numerical difficulty](https://optimization-online.org/2010/06/2662/)
- [Andreani et al., scaled safeguarded-AL stopping](https://optimization-online.org/2020/08/7985/)
- [Sun and Nocedal, error-aware trust regions](https://arxiv.org/abs/2201.00973)
- [Ouyang and Milzarek, trust-region semismooth Newton](https://arxiv.org/abs/2106.09340)
- [Nonlocal authors' current publication status](https://peridynamics.com/publications.html)

## Selected discriminator

Add one private `--nonlocal-al-inner-floor-mechanism-discriminator` command.
It reproduces the complete D7R6 report, then replays only the three unique
failed inner solves from their exact pre-failure position/multiplier states.

For every live trial it records:

- current state and topology roots;
- trust radius, step norm and radius owner;
- HVP count and negative-curvature exit;
- gradient-step and predicted reduction;
- raw and fixed-order factored/direct actual reductions and both ratios;
- current/trial total-energy ULP and reduction-to-ULP ratios;
- active, fluid-pair and boundary-pair roots before and after;
- minimum absolute PHR coefficient and minimum radial horizon margin;
- the exact would-accept decision under unchanged raw admission.

The command is observational. It may mutate only private replay copies and
must end with the same public root and zero commits.

## Frozen routing intent

Route precedence is:

1. common PHR/active-set kink;
2. common binary64 merit-resolution floor on stable topology;
3. mixed active-set and merit-resolution mechanisms;
4. trust-model/derivative reclosure.

Every route authorizes only one next research/contract stage. No route
selects a cap, tolerance, pressure gate, penalty parameter, solver family or
trajectory.

