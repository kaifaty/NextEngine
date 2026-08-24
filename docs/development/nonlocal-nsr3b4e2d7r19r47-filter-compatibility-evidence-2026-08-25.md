# NSR3-B4E2D7R19R47 filter-compatibility evidence

Date: `2026-08-25`

Status: `PASS / RESTORATION_COMPATIBILITY_REQUIRED / ROLLBACK ONLY`.

Implementation commit: `f35734aa`.

Frozen identity SHA-256:
`09b1ce5efc452cbbd4220f6bb09f720d23fdb777b65de7d796e9c64d0e41799f`.

## Result

R47 replays exact R46 and classifies the R43 normal step as filter-admitted
but strictly incompatible with an ordinary filter-SQP transaction:

```text
source psi             3.2918458374056909e-15
predicted reduction    2.6064437667433723e-15
direct linear psi      6.8540207066231842e-16
direct linear h        3.7024372261047680e-08
positive rows          464
strictly compatible    false
route                   RESTORATION_COMPATIBILITY_REQUIRED
```

The step is far inside the inherited global-L2 trust radius:
`1.0290544256239871e-6 <= 0.25`. All 18,000 contact components were audited;
the 1,290 source-active components have zero tangent violations. Exact R46
filter admission, immutable source/endpoint/trial/owner roots and rollback all
close.

No correction, candidate, runtime filter state or following outer was
committed. No restoration solve was executed. R47 authorizes only restoration
entry research; it does not authorize restoration exit, runtime use or
production.

## Binary64 reclosure

V1 incorrectly required floating-point subtraction to be invertible. V2 keeps
the directly evaluated R43 linear metric authoritative and uses the frozen
two-subtraction forward bound:

```text
reconstructed psi      6.8540207066231862e-16
absolute discrepancy   1.9721522630525295e-31
gamma(4) bound          5.8474928675268488e-30
direct h bit-exact      true
```

The discrepancy is positive, so the dense control exercises the intended
rounding path; it remains about `0.033728x` of the bound. No observed tolerance
was fitted, and exact-zero compatibility was not weakened. The v1 diagnostic
is retained separately with no scientific compatibility credit.

## Work and controls

The stage replays R46 once and adds no scientific work: zero new workspaces,
JVPs, VJPs, HVPs, nonlinear models, trials and outers. Nine precedence routes
close with route root
`b72ff40f3cbea3189e785351e278f37a51513db1336d0264a3570b0ff76728ca`.
Rollback is exact.

Semantic result SHA-256:
`cbc082189cf1813be74123be818fa55649c7b2feac07d87f31626063b046286c`.

## Clean Release reproducibility

```text
/home/kaifaty/.cache/nextengine/build-nonlocal-b4e2d7r19r47-a.QkJpul
/home/kaifaty/.cache/nextengine/build-nonlocal-b4e2d7r19r47-b.ZAGK9Q
binary SHA-256 45a7ce38c5fc5610547d11336f68c23572039c2d431a8231d129cbbbc5a92b1e
size           7764896
ELF build-id   0a88cb9485e0467099f904e3bc54bb7b9c4ae9e5

/home/kaifaty/.cache/nextengine/run-nonlocal-b4e2d7r19r47-a.jDVNcM
/home/kaifaty/.cache/nextengine/run-nonlocal-b4e2d7r19r47-b.gKmsel
stdout bytes   1533
stdout SHA-256 dac0ffd871c927cb62253f64a078866d9515faefda59e48a54d923089dbc371d
```

Both binaries and stdout payloads are byte-exact. Each run reproduces exact
R46 parent stdout SHA
`ecbdc13b1cf1ca889fed1f0ce98f4f4613c06eb222d2e132271cd23cdddaf4e3`.
These are correctness/reproducibility runs, not timing evidence.

## Consequence

R46 answered whether the candidate makes admissible filter progress; R47
answers whether that candidate can participate in an ordinary transaction.
The answers are respectively yes and no. The next stage must research a
matrix-free bounded primal/dual restoration-compatibility certificate at the
exact R43 moved state, under the remaining trust region and exact contact
tangent cone, before returning to filter switching or trust-radius policy.
