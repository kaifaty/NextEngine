# NSR3-B4E2D7R20R63ZI original-operator twofold recurrence evidence

Status: `AUTHOR_PASS / REVIEW_NOT_TESTED / BOUNDED_REJECTED`.

## Snapshot and command

- Author snapshot: `994efc5aad67d0c45cf1e10c8f70bfeac6b91bac`.
- Parent: `db03f4b3cc3b6c60429ee40259adc8f719e17f6d`.
- Parent-to-snapshot executable diff SHA-256:
  `8e1031ab9f387213eb676ebababe76d66582a66c7df9eb551624530a97731558`.
- Frozen parent cache SHA-256:
  `23dbf605ad7b6ae12c4cf6a80404ead9617354ff2848bd010b52c7fa7f83bb84`.
- Release binary SHA-256:
  `d718c88aa77beb90230594bfc3cd95b740d8838b8380546e64b97f162845b278`.
- Command:

  ```text
  /tmp/nextengine-r63zh-build/nonlocal-formula-tangent-twofold-recurrence-release /tmp/nextengine-r63zh-parent-cache.bin
  ```

Two clean Release executions produced byte-identical stdout SHA-256
`98736993d50ae29deebdb48361c0079e0aa3443ca3bfd1775ecbddbcd0080a1e`.
The sealed result SHA-256 is
`844a07a0c157e0e468f48509f7f6e0dc643e7894a2a37c0e1eefac3935427329`.

## Result

The frozen discriminator returns `R63ZI_STATE2_REJECTED` with
`candidate=false`. All apparatus, exact audits and controls pass, but the
tangent-Gram width-two recurrence produces reject/reject/reject rather than
the required reject/reject/pass ladder. Every state has `12+/24-/66?` and sign
root `4dc6f1bc...f04150c`; state 2 does not reproduce the required
`89b2908b...6094` root.

This is not the common recurrence aliased under a new label. The tangent
candidate and common-negative state values differ, while the common lane also
independently closes as a complete rejecting control.

## Exact correspondence and work

- exact tangent-Gram source root:
  `1875cca6c378cefcd99d7c2d35a1ae28a25aa74fd15eb89791b5798e94abdca9`;
- binary128 basis materialization root:
  `2cc68c081e78615dd0f52968ed465e251bb55e5d4b236b1434ec9033dfa8eb10`;
- twofold component/radius roots:
  `1a6b42b...4b8ea` / `95617408...ca41`;
- all `10,404/10,404` exact coefficients are contained;
- all `306/306` rows of the three executed products are contained;
- exact audit root:
  `9c54bd9fdce5e4daa351765383270c748f1ccb3e75b8245e86184c16b31b55af`;
- builder work is exactly 102 basis products, 42,534 dots, 6,554,520 terms
  and 10,404 scale products;
- exact oracle work is 5,253 upper dots, 1,654,695 Gram products and 5,151
  mirrors;
- recurrence work closes at three operator products, three factor solves, two
  rho dots, two denominator dots, three scalar divisions, 204 solution
  updates, 204 residual updates and 102 direction updates.

Stale/unsealed, resealed coefficient, dropped-low, radius-only, nonfinite,
dimension/source identity, result-sealing and classifier-precedence controls
all pass. Control root:
`3f59b9656722d5f7dbc2c442581a549051116365a86f0c7e5b125d59c44b2ca9`.

## Parent regressions and checks

- focused dev and Release targets build under strict FP flags and `-Werror`;
- `git diff --check`: pass;
- R63ZG stdout remains byte exact at
  `ca2a0f80b692a466aa97b4725fcc0ac3f553db95bb7ddf72494a754a7e2029c9`;
- a clean R63ZH rebuild using the frozen external Boost headers returns byte-
  exact stdout
  `181ac246c7a0d6e1c24543fb70e684a9357a0c183858c371867dcd068fe96722`;
- no Cargo, host-check or continuum ProductCheck was run because this is a
  localized offline C++ research tool with no runtime/public-contract change.

## Conclusion and next discriminator

R63ZI rejects the width-two dense materialization as a portable producer for
this frozen weak direction. Coefficient and observed-product enclosure do not
imply that the K2 centers retain enough information for the unchanged PCG
trajectory.

The result does not yet distinguish loss in stored-matrix multiplication from
loss in the remaining K2 factor/update recurrence. The smallest next
experiment must keep every K2 solve, dot, divide and update unchanged while
replacing only the three operator calls by the frozen binary128 tangent
product followed immediately by K2 projection. A passing hybrid localizes the
failure to operator storage/product precision; a rejecting hybrid localizes it
to product projection or the residual recurrence and precedes any wider-
expansion design.

No dynamic builder, width increase, adaptive stop, corpus, timing, runtime,
GPU or production authority follows. Independent review has not been run.
