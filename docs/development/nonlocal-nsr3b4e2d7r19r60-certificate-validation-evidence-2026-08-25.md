# NSR3-B4E2D7R19R60 certificate-validation evidence

Date: `2026-08-25`

Status: `PASS / ROW_LOCAL_CERTIFICATE_VALIDATION_CANDIDATE / ROLLBACK ONLY`.

Implementation commit: `08457348`.

Frozen identity SHA-256:
`15313184ff0f552c9731da249d135d1075ead0334988bf9834a0bf72e3ce42ca`.

## Result

R60 reproduces exact R59, then independently validates the row-local
certificate at structural, arithmetic-domain and full-binary128 levels.

### Structural controls

The nominal workspace's degree vector equals every flat-offset difference and
its derived maximum equals 113. One valid dense case passes. Four mutations are
rejected before classification:

```text
forced one-row undercount       rejected
forced one-row overcount        rejected
decreasing offset               rejected
declared maximum mismatch       rejected
dense control cases                     5
```

Degree-control root:
`b366e42f357ef118c6eed6a8f3a08fe9f000158229d8ab79d62229bd101a9cee`.

### Arithmetic-domain audit

An explicit scalar binary64 replay in stable row/slot order is bit-identical to
the captured directed JVP values and absolute sums. It audits displacement,
relative direction, Jacobian, component products, dot partials, terms,
accumulators, raw values, bounds and uppers:

```text
nonzero subnormal intermediates    0
nonfinite intermediates            0
runtime replay exact            true
```

Thus the retained standard relative-error model is not crossing an unmodelled
underflow or overflow boundary on this exact witness.

### Independent binary128 containment

The fresh binary128 traversal forms position and witness differences directly
from the binary64-owned source components, instead of promoting candidate-path
binary64 differences. Compensated row folds and a row-local binary128 forward
bound produce:

```text
binary64 local-upper containment failures       0
minimum containment margin       3.345662094648636e-22
minimum-margin row / degree                 0 / 104
maximum full128 upper             -5.198748252209752e-22
maximum-full-upper row                         5196
```

The minimum margin is strictly positive in binary128,
`0x1.94773a03e85102b67cp-72`. Frozen precedence therefore selects
`ROW_LOCAL_CERTIFICATE_VALIDATION_CANDIDATE`.

```text
full traversal root  30ed1a37eb96f0967f10a36c9f014459ab4ffe4c5b26609f74a653a1798f3a50
comparison root      f0f18b90c88931ec6dc417ccf257109bfb094e541b17b0f69c06154dfab52ff8
route root           c2fda0f84cb53ddff41cc67deab63fef14262fa2cd230b8584956ef66f959caa
semantic             d93dbdc95eb66cf45f074541ae44b621755f45f011d2f1589af834a45238aec9
```

New work is one row-local scan, five dense degree cases and one binary128 row
traversal. There are zero new pair/JVP/VJP/solver passes. One moved workspace
is built and released; rollback is exact.

## Clean Release reproducibility

```text
/home/kaifaty/.cache/nextengine/build-nonlocal-b4e2d7r19r60-final-a.60uQB6
/home/kaifaty/.cache/nextengine/build-nonlocal-b4e2d7r19r60-final-b.ATPTdP
binary SHA-256 4dd6fa61269ae252eedcc7eff4f15554da0b601b021d8633b2599170273ac471
size           8365000
ELF build-id   db8e688f921f0b87af1795c975df03dcde3cdfcf
stdout bytes   2206
stdout SHA-256 6c6c62b6efb196d97dc4210ad7bcb56732f0c2d107a2df6aa10e9de84005c7f7
```

Both clean Release binaries and outputs are byte-exact; both processes exit
zero. Wall time is not performance evidence. R59 remains exact internally at
stdout SHA-256
`495bbd213784c4e4f7500c33e2be32c05d41b859145282770364a8fc3a143a4b`.

## Consequence

R60 validates R59 strongly enough to research and freeze a separate R61
integration boundary. R61 may introduce a topology-owned row-local audit
candidate with the legacy global bound retained as a shadow comparison and
with the R60 structural negative controls. It may not carry binary128 into the
runtime path, change operator arithmetic, apply the witness, authorize
restoration exit or claim production. Those transitions remain separately
gated after integration evidence.
