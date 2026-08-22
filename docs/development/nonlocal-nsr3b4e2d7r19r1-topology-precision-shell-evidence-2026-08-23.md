# NSR3-B4E2D7R19R1 topology-precision shell replay evidence

Date: `2026-08-23`

Status: `PASS / RUNTIME_TOPOLOGY_PRECISION_CANDIDATE / REPLAY_ONLY`

## Outcome

D7R19R1 proves that D7R19's hard precision failure is a discrete topology-
ownership conflict at the compact-support horizon, not an energy-sign or
formula failure. Every observed binary64/extended membership disagreement is
inside the frozen `64*epsilon_binary64*h` shell. At every mismatched radius the
observed cubic `W`, `W'` and `W''` contributions are zero, and the kernel is
exactly C2-closed at `r=h` in binary64, long double and binary128.

All three membership policies preserve resolved positive reductions in both
extended formats:

1. live extended membership;
2. binary64-owned membership;
3. horizon-canonicalized mismatch membership.

The result selects only research/freeze of a narrow runtime topology-policy
reclosure. It does not change D7R19, accept a state, authorize binary128 in the
runtime, increase a watchdog or permit another nominal substep.

## Parent and targets

The replay invokes the D7R19 parent once and captures its already computed
private trial states. It does not rerun the nominal transaction.

- parent stdout SHA-256:
  `f5811bfc7d5e986d72b9130f8e6cb90c5ae21bd347ce7f0b476c171fe8cff7bb`;
- parent semantic result:
  `bcc6f588010999f23664209037a0daae961606ca29fdcc0d7a31e661902e185b`;
- parent transaction root:
  `a1030f9b23abf0322c6c7acaba17da84da7861c1ac776ad78d5747b5989fcca1`;
- parent completed / total-budget HVPs: `85 / 117`;
- replay HVPs: `0`.

The exact outer-0 target roots reproduce:

| Trial | Parent long-double root | Pair-union size |
|---:|---|---:|
| `0` | `00188e8e74bf7bc16e9be0d1b7df3a9555f23d4d22f3bad8355b0a36dd4aac02` | `386,402` |
| `1` | `823812828a7386e3120762cab508b76577fae92c692a39152420ace1ff5d8f95` | `386,402` |
| `2` | `7298f0f80106642c72a6912736e30b5c55e5cb0898f58f85e8146967b6fb1ded` | `386,402` |

## Shell evidence

Long double reproduces D7R19's `10,989` repeated current/trial observations
and `7,315` unique successive-state observations. Binary128 sees more exact
boundary disagreements -- `15,298` repeated and `10,566` unique -- because it
resolves additional distances around the same horizon shell. This count
difference is expected evidence of discrete topology ownership; it is not a
nonlocal interaction.

| Trial | Long current/trial | Binary128 current/trial |
|---:|---:|---:|
| `0` | `3,641 / 2,346` | `5,817 / 3,019` |
| `1` | `2,346 / 1,328` | `3,019 / 1,713` |
| `2` | `1,328 / 0` | `1,713 / 17` |

The maximum distances and kernel values are:

```text
long-double maximum abs(r-h)  1.38777878078144567553e-17
binary128 maximum abs(r-h)    0x1.0000000000002c2p-56
maximum abs(W)                 0 in both formats
maximum abs(W')                0 in both formats
maximum abs(W'')               0 in both formats
W/W'/W'' at exactly h          0 in binary64/long-double/binary128
```

Successive trial/current mismatch-set roots are exact in both extended
formats. All unions are sorted, sparse and use no all-pair fallback.

## Sign and candidate evidence

Each of the nine policy/trial lanes resolves positive under both naive and
compensated long-double arithmetic at at least `1024` ULP and under both
binary128 reductions at at least `4096` ULP. Long-double and binary128 signs
agree in all 18 comparisons.

The binary128 relative error of the existing binary64 candidate reduction is
identical across all three topology policies for a given trial:

| Trial | Exact binary128 relative-error encoding | Frozen bound |
|---:|---|---:|
| `0` | `0x1.69ea42e86fdd97db87afb51e2f9p-32` | `<= 0.05` |
| `1` | `0x1.f02517c34e663a4a305a8ee00c0dp-30` | `<= 0.05` |
| `2` | `0x1.088bd6f4561462ec3ed7d6d56c41p-28` | `<= 0.05` |

No topology policy changes a reduction sign or violates the candidate bound.

## Structural work and rollback

```text
static-index builds/canonicalizations  1 / 1
superset builds                        6
pair-union observations                1,159,206
long-double audits                     9
binary128 audits                       9
long-double pair visits                13,910,472
binary128 pair visits                  13,910,472
replay HVPs                            0
all-pair candidate calls               0
accepted replay states                 0
```

Frame zero, parent transaction state and every target current/trial position
root remain exact after replay. The replay emits no public commit, physics
mutation, second substep, macro, trajectory or timing claim.

## Reproducibility

Implementation commit:
`df7ac391` (`research: replay topology precision shell`).

Two clean Release builds:

- `/home/kaifaty/.cache/nextengine/build-nonlocal-b4e2d7r19r1-a.Xhxr0x`;
- `/home/kaifaty/.cache/nextengine/build-nonlocal-b4e2d7r19r1-b.lW34ME`.

Both binaries are `5,742,656` bytes, have SHA-256
`5a73a98f6c57ae9692dd5599754b2ba7dd68ed75b667855e69ef43390aa7bc9c`
and GNU build ID `8f98fe9087e7fb7431fa189494966cbbd66e698c`.

Both fresh processes exit `0`, emit empty stderr and reproduce:

- stdout-with-LF bytes: `8,473`;
- stdout SHA-256:
  `f77eb3da05b6ddb2da815a228aef1709917a4761af1eea8d1c71a3a2f2cf0aa1`;
- semantic result:
  `c5cc2c128104e006e491afd40b2ebb474508aa80d97a21c0750f3bec4758cd7e`;
- route: `RUNTIME_TOPOLOGY_PRECISION_CANDIDATE`.

Raw outputs are under:

- `/home/kaifaty/.cache/nextengine/run-nonlocal-b4e2d7r19r1-a.P4q30b`;
- `/home/kaifaty/.cache/nextengine/run-nonlocal-b4e2d7r19r1-b.0DdUCU`.

## Decision

Preserve D7R19 as its historical hard precision failure. For the next
experiment, make the primary binary64 solver topology the explicit owner of
membership masks used by independent precision audits. Extended arithmetic
may reevaluate continuous values over that frozen discrete topology, but it
must not silently define a different neighbor graph.

Research and freeze a separate D7R19R2 first-substep policy reclosure. Its only
solver-visible delta may be explicit binary64-owned topology in precision
audits. It must preserve D7R19 parent bytes, the binary64 candidate trajectory,
work and rollback, and it must retain the existing 32-HVP per-trust-step
watchdog. A likely structural-watchdog route is an observation to test, not a
preauthorized result.
