# NSR3-B4E2D7R20R63R common-operator inverse evidence

Status: `PASS / COMMON_OPERATOR_TWO_SIDED_CANDIDATE`.

Claim status: `SUPPORTED_EXACT` for the complete captured common operator.
Evidence classes: `EXACT_CERTIFICATE`, `CORRESPONDENCE`, `NUMERICAL`.

## Reproducible result

Implementation commit: `af5898f8`.

Command:

```text
/tmp/nextengine-r20r4-build/nonlocal-formula-reclosure \
  --nonlocal-al-generalization-v5-common-operator-inverse
```

Two independent executions are byte-identical:

```text
stdout sha256   bdf1ae684b2df0836d3ef67b22586e0112be9ac6f757d7d95a0e2c1db1d8d5ca
semantic sha256 b8783c5390382c3169ff49c9d7df7c8facce50a15c7f46804ed10eb0bb65f382
route           COMMON_OPERATOR_TWO_SIDED_CANDIDATE
controls root   9863caf57c2968af4bda1186cf84a5575e986cf1fdcdf6e22910eb92e941cc75
```

Platform, controls, exact R63Q reconstruction, original factor root, all
canonical solves, exact products, work and lifecycle gates pass.

## Two-sided contraction

The immutable original binary128 tangent QR root is
`fde9aef5ee0233c920ab76c855c612cc722422c67f44d18598211ae901082ec4`.
Its 102 canonical two-triangular solves assemble raw, unsymmetrized `Z` at root
`32508a73b43eab2afd6737761d00b5d166a2b415151fb6ff4d92fd3df0ed4a07`.

Exact rational multiplication against common `H*` gives:

```text
quantity                         value
max entry |I-ZH*|                2.761231737205025e-4
max entry |I-H*Z|                1.979993963381460e-4
||I-ZH*||inf                     9.061521879470768e-3
||I-H*Z||inf                     3.569948137506210e-3
||Z||inf                         1.040672052806690e33
||Z||inf/(1-rho_left)            1.050188357586526e33
max raw Z asymmetry              1.953125e-2
```

Both strict exact comparisons with one pass. The raw asymmetry is reported and
root-bound; no symmetrization was used. Relative to the `1e33` inverse scale it
is negligible, but no relative-asymmetry threshold enters the result.

Defect roots:

```text
exact inverse dyadics  9aa68591d9a13ea82cd8e42815865b91e9ffd63c31ca6982d18ecb77070a815a
left defect            6b51e8483052fb268a658a6cf2d6697e5890e3c469ab71dffbf0d25458253a31
right defect           29db9a342bcc0bca7eb5ec57067668f33c4f0ea2bc2bb66d5ea0e9dc0c2c9b28
defect record          fea187b0a8583cdc08f71521cfcb64f996c4c09c93db71fa94878d24b018b9d5
inverse audit          a86c14bad30ffb45cc65491420b885f41f6bcaf5930014efe71b427a4f092aee
```

## Exact work

The stage performs 102 verifier column solves, 525,402 forward and 525,402
backward triangular terms, 20,808 divisions, and exactly 1,061,208 products
per defect orientation. It applies zero physical RHS values and performs zero
PCG/refinement updates, sparse constructions, timing samples or state changes.

## Meaning and next discriminator

The common operator now has an independent residual-to-solution-error
certificate:

```text
||x-x*||inf <= 1.050188357586526e33 ||b-H*x||inf.
```

The large amplification is expected from the weak direction and makes an
exact residual essential. It does not invalidate the verifier because both
defects contract by a wide strict margin.

R63S may now replay the two frozen direct tangent-PCG lanes, compute exact
common-operator residuals for every iterate and derive component signs from
the bound above without dense `H`, dense `X` or legacy signs. The first
certified iterate must be located under a fixed eight-update budget before any
runtime stopping policy or sparse work.

R63B--R63Q stdout files remain byte-identical to all frozen hashes. Build
passed. No CPU/wall performance comparison, runtime state, GPU or production
inference occurred.

