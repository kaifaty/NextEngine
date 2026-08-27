# NSR3-B4E2D7R20R63ZA finite factor-consumption evidence

| Field | Result |
|---|---|
| Status | `PASS` |
| Route | `TWOFOLD_EXPORTED_FACTOR_CONSUMPTION_CANDIDATE` |
| Semantic SHA-256 | `797cfbf4d6a96e76e7d099096711aad3468f2ab10cb2ada854168e621837124b` |
| Stdout SHA-256, both final executions | `ed2511615eb0b488b4337133860971f86a0d7ad48c757db49c5fcda32b325f72` |
| Parent R63Z semantic | `e27ee8616c1a34a1a224f65f32d5cebe1cd6dac973ccc2ae89c3fbad8343e4be` |
| Controls root | `1f48563f8ce47ee3f125991f5a7b270e22a1967b87433dab80515effc8531b23` |
| Identity root | `78ce6fc93f9dc5d2f979eb1d401d9ee883c7d469571e16799baf73cce2700dc1` |
| Fixture root | `05e520b948d93029d18f6b2b34e87b356ab2dbd2f0a354b55a10f3920fbc1c8e` |
| Authority | offline arithmetic research only |

## Observation

R63ZA consumed the once-exported binary64 upper factor with a fixed twofold
binary64 implementation of add, subtract, multiply and three-correction long
division. It executed exactly three complete solves: start generation, the
initial exported-lane PCG residual and the iteration-1 residual.

Each deterministic scalar is the exact sum of two stored binary64 words. A
separate row-local interval propagates source and prior-row uncertainty and
adds an a-posteriori quotient residual bound. The finite root is sealed before
the independent binary128/dyadic containment audit.

## Results

| Subject | Intermediate/output containments | Maximum center/reference difference | Maximum propagated radius |
|---|---:|---:|---:|
| start | `204/204` | `6.8055966667169288e-3` | `3.3039071838266965e24` |
| initial residual | `204/204` | `4.2363265311507138e-2` | `3.9554019182519786e25` |
| iteration-1 residual | `204/204` | `4.0437387492250191e-3` | `1.6718991421320941e24` |

All `612/612` frozen reference components are contained. The deterministic
K2 centers remain close to the binary128 reference on the absolute scale
above. The sequential dependency interval is extraordinarily wider than the
observed center difference. This is evidence of dependency amplification in
the triangular enclosure, not evidence that the deterministic centers are
wrong.

Consequently, R63ZB may consume only the sealed K2 centers and must use the
independent final R63Y affine certificate as correctness authority. It must
not propagate the `1e24..1e25` R63ZA radii as if they were useful stochastic
or physical uncertainty.

## Work and controls

The fixed new work is exact:

```text
forward terms             15,453
backward terms            15,453
divisions                    612
row equation audits          612
reference containments       612
operator products              0
PCG scalar dots                0
candidate updates              0
factor builds                  0
```

All disclosed controls pass: scalar normalization, nonidentity-permutation
solve, cancellation, three-correction division, necessary radii, zero/
subnormal/nonfinite/negative rejection, overflow, orientation, stale
identity, mutation, exact-oracle independence and classifier precedence.
Mutating an oracle changes the audit root while the finite root remains exact.

## Invalid/apparatus-only executions

The first execution did not receive the private consumption fixture through
the R63X reconstruction and stopped at apparatus. A second execution executed
all positive subjects, but its oracle-independence control changed a reference
by one binary128 ulp that correctly remained inside the finite enclosure. Both
runs have no scientific route credit. The final control uses a deliberately
external reference mutation; no positive radius, source or classification
threshold changed.

## Regression and conclusion

The focused build passes and both final R63ZA stdout payloads are byte-
identical. Direct post-change regressions retain:

```text
R63S 15568ae0d50ea3285da6d12cc07ba80d650994bd95b833daaae71868c6404dd1
R63X 9e218dbd248fc079fd058a44753d5d09b4dbaa8ed5ac9695eca2be68ccf25bf7
R63Z de22f5f638fa434ca61857f09b5b9f6988d40f33a5e7fcec4f43a4b9b21bda54
```

R63ZA closes portable consumption of the frozen exported factor for the three
RHS values needed to reach the selected state-2 frontier. It does not prove a
portable factor builder or an end-to-end PCG candidate.

The next gate must run a deterministic K2 exported-lane recurrence through
states `0..2`: R63Z operator products, R63ZA preconditioner centers, twofold
rho/denominator/alpha/beta and vector updates. All three states execute; the
independent R63Y affine verifier alone decides whether state 2 resolves the
frozen signs. No adaptive stop, corpus, timing, GPU, runtime or production
claim is admitted.
