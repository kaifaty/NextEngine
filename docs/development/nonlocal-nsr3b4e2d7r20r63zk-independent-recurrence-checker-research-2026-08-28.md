# NSR3-B4E2D7R20R63ZK independent recurrence checker research

Status: `CONTRACT_FROZEN / IMPLEMENTATION_NEXT`.

## Decision

R63ZJ is reproducible but `INCONCLUSIVE`: its author validator permits
resealed state/certificate and recurrence-work drift. The next experiment is
not a third repair. R63ZK treats the complete R63ZJ return value as untrusted
trace data and independently schedules the same declared recurrence from the
immutable fixture through small K2 arithmetic primitives.

The complete scope, controls and stop rules are in the
[R63ZK contract](../plans/nonlocal-nonlinear-solver-research/03b4e2d7r20r63zk-independent-recurrence-checker-contract.md).

## Bounded research synthesis

The local evidence distinguishes four live hypotheses:

- stale/mutable certificate data creates the apparent ladder;
- the raw hybrid trace is numerically correct despite the broken validator;
- callback products are correct but a later K2 transition differs;
- the trace is correct but validation work is inseparable from candidate work.

R63N/R63O already reject a direct return to matrix-free `T(T^T p)`, so that
path would not test these hypotheses. A whole-recurrence width increase would
change too many boundaries. Full trace translation validation is the smallest
counterfactual: independently regenerate the declared semantics and compare
each value, while treating producer self-roots as diagnostics only.

Two primary-source patterns support this shape without adding authority to the
numerical claim:

- Pnueli, Siegel and Singerman's translation-validation line checks each
  produced result against an independently stated semantics instead of proving
  the producer implementation once; the practical constraint is that the
  validator itself becomes trusted.
- Appel et al.'s proof-checker work therefore minimizes and names the trusted
  checker boundary rather than trusting a large proof producer. Becker et al.
  apply the same separation to finite-precision error-bound certificates: an
  analysis tool produces a certificate and a separately justified checker
  validates it.

Sources:

- [Foundational Proof-Carrying Code project and trustworthy checker papers](https://www.cs.princeton.edu/~appel/fpcc.html)
- [A Trustworthy Proof Checker](https://www.cs.princeton.edu/~appel/papers/flit.pdf)
- [A Verified Certificate Checker for Finite-Precision Error Bounds in Coq and HOL4](https://arxiv.org/abs/1707.02115)

These sources motivate only the producer/checker split and small trusted
boundary. R63ZK remains an ordinary C++ research checker, not a formal proof.

## Implementation boundary

Expose only generic research K2 primitives required to schedule the recurrence
outside the author transaction: projection, factor solve, dot, divide, update
and certificate-from-components. Implement the R63ZK scheduler and classifier
in a separate translation unit. It may share arithmetic primitives and the
frozen R63Y verifier, but it must not call the R63ZJ validator or use author
route/certificate/work fields to construct expected values.

The first preflight is a feasibility stop: if the public boundary cannot
reconstruct every state vector, scalar and certificate without calling the
author transaction scheduler, stop and export an immutable full trace before
writing any classifier.

## Authority ceiling

R63ZK can at most recover a reviewed fixed-fixture correspondence claim for
the raw hybrid trace. It cannot rehabilitate R63ZJ itself, select a portable
representation, authorize width three, or grant timing, corpus, runtime, GPU
or production authority. SPEC-38/ADR-076 remain `Proposed`; later continuum
ProductChecks remain `NOT_RUN`.
