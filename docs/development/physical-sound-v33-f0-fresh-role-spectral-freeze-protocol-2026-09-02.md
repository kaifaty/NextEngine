# Physical sound V33 F0 — fresh-role spectral freeze protocol

| Field | Value |
| --- | --- |
| Date frozen | `2026-09-02` |
| Status | `FROZEN_BEFORE_NEW_ORACLE_OR_MODEL_VALUES / SYNTHETIC_ONLY` |
| Roadmap package | V33 `F0`, prerequisite for value-independent `I0` |
| Research | [Mode-local spectral successor](physical-sound-v33-mode-local-spectral-successor-research-2026-09-02.md) |
| Canonical profile | [`physical-sound-v33-f0-mode-local-spectral-residual.v1.json`](../../lab/profiles/physical-sound-v33-f0-mode-local-spectral-residual.v1.json) |
| Product effect | None; external experiment only, authored clips remain authoritative |

## Question and allowed claim

Can one fixed surface-spectral and P1 mode-local lift let a small bounded MLP
generalize a contact correction across unseen geometry, unseen contact and
both together, while P1 retains exact physical authority?

F0 freezes the experiment but opens no target or model value. I0 may execute
discarded non-official fixtures only. D0 is the first stage allowed to
materialize train and development targets; H0 remains inaccessible unless D0
passes completely. Even an H0 pass establishes synthetic representation
capacity only. It cannot establish Steel, Glass or Wood quality, naturalness,
validator release, admission, cooking, demo behavior or runtime authority.

## Immutable lineage and isolation

The canonical profile binds the V31 P0/P1 causal owners, V32 M0 representation
and terminal M1 reject, completed T0/V0a mechanics and V33 R0 research. The
rejected V32 weights, predictions, rows, optimizer state and opened values are
not inputs. V33 may reuse P1 formula ownership and the V32 branch decomposition
as controls, but no V32 material row, geometry multiplier cell or contact pair
appears in the fresh corpus.

All generated corpus rows, model values, predictions, reports and caches stay
in fresh external storage. F0 validation performs no network request, signal
decode, P1 solve, oracle evaluation, tensor initialization or training.

## Fresh synthetic corpus

Three new synthetic materials cross the same three P1 family/support controls:
simply supported rectangular plate, cantilever beam and simply supported beam.
The material rows, ten geometry-multiplier cells and `12/6/6` contact sets are
exactly those in the canonical profile and are disjoint from V32 M1.

Every case retains ten P1 modes, the P0 pickup for its family and exactly
`1 N·s` impulse. Geometry components retain the P1 family-specific order.

### Role construction

Train uses geometry cells `0..5` and all 12 train contacts. Development uses:

- `geometry-only`: cells `6,7` with the 12 train contacts;
- `contact-only`: already trained cells `4,5` with six development contacts;
- `joint`: cells `6,7` with the six development contacts.

Method holdout mirrors this without reusing development identities:

- `geometry-only`: cells `8,9` with the 12 train contacts;
- `contact-only`: already trained cells `2,3` with six holdout contacts;
- `joint`: cells `8,9` with the six holdout contacts.

Exact case identity is `(family, material, geometry cell, contact set, contact
index)`. No case crosses roles or strata. Geometry identity intentionally
crosses from train into each `contact-only` stratum; otherwise contact-only
generalization would be inseparable from geometry generalization. No
development case crosses into holdout.

| Role/stratum | Cases | Modal rows | Role-local geometry groups |
| --- | ---: | ---: | ---: |
| train | `648` | `6,480` | `54` |
| development / geometry-only | `216` | `2,160` | `18` |
| development / contact-only | `108` | `1,080` | `18` |
| development / joint | `108` | `1,080` | `18` |
| method holdout / geometry-only | `216` | `2,160` | `18` |
| method holdout / contact-only | `108` | `1,080` | `18` |
| method holdout / joint | `108` | `1,080` | `18` |
| total | `1,512` | `15,120` | `126` role-local / `90` unique |

The counts are algebraic commitments. F0 validates them without solving a
fixture or computing a target.

## New truth bank

The truth bank remains log-space bounded corrections around P1. It is not a
copy of V32: all three branch expressions and coefficients change, and contact
combines two low-order surface fields with local P1 mode-field differences.

For the local stencil, `h = 1/16`. Sample coordinates use
`clamp(x ± h, 0, 1)` with the actual clamped distance retained by the profile;
P1 evaluates participation at the four resulting points. The truth contact
components are:

```text
du = phi(u_plus,v) - phi(u_minus,v)
dv = phi(u,v_plus) - phi(u,v_minus)
cu = phi(u_plus,v) + phi(u_minus,v) - 2*phi(u,v)

surface_a = sin(pi*u) * cos(2*pi*v)
surface_b = cos(3*pi*u) * sin(pi*v)

p* = 0.22*tanh(
       0.26*phi + 0.18*du - 0.14*dv
     + 0.20*surface_a + 0.17*surface_b
     + 0.12*ordinal*support + 0.10*family*(2*u-1)
     + 0.08*log_frequency*cu)
```

Decay and global gain use the distinct expressions frozen in the profile.
All functions use Python `math` binary64 in I0/D0/H0. The oracle coefficients
are unavailable as model inputs. Synthetic truth is a discriminator, not a
statement about a real acoustic material.

## Fixed representation

Decay and global-gain branches retain their exact V32 field lists and
`11→16→16→1` / `13→16→16→1` SiLU/tanh topology. The contact branch begins
with the 12 V32 contact fields, then appends:

1. for orders `k=1..4`, exactly
   `sin(k*pi*u), cos(k*pi*u), sin(k*pi*v), cos(k*pi*v)` in that order;
2. P1 participation at `u_minus`, `u_plus`, `v_minus`, `v_plus` in that order.

The exact contact input count is `32`; its topology is `32→16→16→1`, with
`817` parameters. The whole candidate has `1,811` binary64 parameters. Its
contact output remains a tanh-bounded `±0.25` log multiplier of P1 contact
participation. Therefore P1 contact zeros and sign remain exact. P1 frequency,
mode order, pickup, support, impulse and remesh values are not model outputs.

No learned/random positional embedding, object/record/role ID, geometry-cell
index, mesh resolution, vertex index, expected target, validator result,
waveform, listener field or pretrained feature may enter the model.

## Candidate, controls and training

There is exactly one candidate and these frozen controls:

1. P1 identity with zero corrections;
2. nearest train row using branch-local normalized candidate features and a
   canonical row-ID tie-break;
3. float64 ridge on raw V32 features;
4. float64 ridge on the complete spectral/mode-local features;
5. the exact V32 `12→16→16→1` raw-coordinate contact MLP topology, retrained
   from scratch on V33 train only.

Candidate and raw MLP use their separately frozen seeds and no seed grid. Both
train once with full-batch AdamW for `1,200` steps, learning rate `0.003`,
weight decay `1e-6`, gradient clip `1`, deterministic float64 CPU and one
intra/inter-op thread. There is no scheduler, checkpoint selection, early
stop, resume, retry or development-informed model selection.

The normalized three-branch candidate loss remains the mean of
`MSE(decay)/0.20²`, `MSE(global_gain)/0.16²` and `MSE(contact)/0.22²`.
The raw-MLP control uses the same loss and branch decomposition. Ridge and
nearest fit only train-role rows.

Three inference-only candidate ablations zero all appended lift fields, only
the four stencil fields, or all contact-coordinate/lift fields. They do not
retrain and cannot select another model.

## Development and holdout gates

D0 opens train and development in two fresh complete processes A/B. Every
artifact, decision and stdout must repeat byte-for-byte. All P1 hard gates and
the exact resource/access envelope are conjunctive.

Contact development must meet all of these frozen comparisons:

- aggregate RMSE `<=0.80x` raw MLP, `<=0.85x` nearest,
  `<=0.90x` spectral ridge and `<=0.85x` raw ridge;
- each of geometry-only, contact-only and joint RMSE `<=0.95x` the best of
  raw MLP, nearest and spectral ridge;
- aggregate absolute RMSE `<=0.035`, each stratum `<=0.045`;
- full-lift zero ablation worsens contact RMSE by at least `10%`, stencil-only
  zero worsens by at least `3%`, and full contact-context zero worsens by at
  least `10%`.

Decay and global gain each retain absolute RMSE `<=0.035` and must be
`<=0.90x` both ridge and nearest. Their material-zero ablation must worsen the
mean branch RMSE by at least `5%`. Candidate aggregate normalized RMSE must be
`<=0.85x` identity, raw ridge and nearest.

Any miss publishes a terminal development reject and exact zero holdout rows.
On complete D0 pass only, the candidate weights and implementation closure are
hash-frozen before H0.

H0 opens the committed method holdout once per A/B evidence process. Candidate
aggregate normalized RMSE must be `<=0.90x` the best non-neural control; every
branch must be `<=0.95x` its best non-neural control. Contact must also be
`<=0.95x` raw MLP overall, each contact stratum must be `<=0.98x` its best
raw-MLP/nearest/spectral-ridge control, aggregate contact absolute RMSE must be
`<=0.040` and each stratum `<=0.050`. Zero denominators require an exact-zero
candidate numerator. Any miss closes the family permanently.

## Hard invariants, access and resources

Every official phase requires:

- P1 frequency and order bit identity;
- exact P1 contact zero and signed-gain sign preservation;
- exact `0.5/1/2` impulse ratios and common-contact remesh equality;
- branch isolation, finite values, positive decay and exact correction bounds;
- corrected render peak strictly below `0.95`;
- unchanged T0 mutation/reason and V0a parent identities;
- no forbidden field, role leakage, real/protected signal or network access;
- canonical sorted binary64 serialization and atomic fresh-output publication.

Each complete process is bounded by `300 s`, `1 GiB` peak RSS, `64 MiB`
published output and zero network/real-signal values. A/B repetition is
evidence replication, never a retry.

## Failure and stop rule

F0 contract/profile failure publishes no partial freeze. I0 may repair only a
value-independent implementation defect without changing F0 semantics; a
semantic defect requires a successor profile before official values.

Once D0 opens, basis order, stencil, boundary rule, material/cell/contact
roles, truth coefficients, topology, seeds, optimizer, steps, controls, metric
or gate cannot change inside V33. D0 failure leaves H0 unopened. H0 failure is
terminal. If spectral ridge passes while the neural candidate loses, the basis
may remain diagnostic but the ML family closes. P1 and authored clips remain
complete behavior in every outcome.
