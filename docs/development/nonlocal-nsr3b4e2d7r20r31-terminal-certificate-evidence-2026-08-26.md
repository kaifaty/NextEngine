# NSR3-B4E2D7R20R31 terminal certificate precedence evidence

Status: `PASS / TERMINAL_CERTIFICATE_PRECEDES_ARMIJO`.

Implementation `7bb3c886` emits twice:

```text
3425772c1e7480d690a31b52e2456857b17feb15f9967892e3830b01c907de5b
```

The R29 shear root `3f4798c8...e473`, iteration-22 step root and all 21 R30
trials reproduce exactly. Every trial's independently recomposed bounded
Armijo margin is bit-exact with the R19 implementation, every complete KKT
predicate agrees with `metrics.certified`, and no trial or state is added or
applied.

Exactly one inherited trial is terminally certified: the first, full Newton
step at `alpha=1`. Its complete tuple is:

```text
finite             true
primal             1.0527085776690160e-32
dual mapping       2.8432901288346389e-34
complementarity    1.3051115190508769e-35
stationarity       0
gap                0
gap bound          1.0247683900617747e-30
gap scaled         4.0000401912440675e-30
```

All residual and gap predicates pass the unchanged
`2^-70 = 8.4703294725430034e-22` certificate. Its metrics root is
`05b71f6f...9b53` and R31 trial root is `e36e8d5e...26d7`. Powers 1 through
20 are finite with valid primal, stationarity and gap sign, but fail dual
mapping, complementarity and scaled gap; therefore the full step is not a
rounded-display accident.

The full-step nominal dual change is positive,
`1.0833355937178202e-34`, and its nominal Armijo margin is also positive,
`1.0811658122698316e-34`. The rigorous comparison must additionally carry
old/new dual bounds of about `5.143149475479e-30` each, the lower-slope term
`2.169781434375e-37` and RHS rounding bound `2.753187461112e-33`. Their total
burden is `1.0289052355397224e-29`, about `94,976x` the nominal dual change,
so the rigorous margin is negative `-1.0288940390376748e-29`. All 21 trials
are bound dominated.

Thus R30 is not a failed Newton formula or a hidden face event. At the exact
terminal scale the bounded merit comparison can no longer prove ascent even
though the stronger end-state KKT oracle already proves convergence. This is
`SUPPORTED_BOUNDED` evidence for the exact 12-case binary128 research profile,
not an independent proof of the shared KKT implementation or production
readiness.

The next candidate may select this existing trial only as a terminally
certified result after the inherited line has rejected. It must not weaken
ordinary Armijo, the KKT tolerance, the cap or any nonterminal step.
