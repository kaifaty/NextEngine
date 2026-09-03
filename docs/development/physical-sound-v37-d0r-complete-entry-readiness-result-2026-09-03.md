# Physical sound V37 D0R complete-entry readiness result

## Decision

| Field | Result |
| --- | --- |
| Terminal | `Pass` on both full-count artificial runs |
| Readiness seal | `e3c33767…e3e5f` |
| Official target/capability access | `0` |
| Real/protected signal access | `0` |
| Runtime or product authority | None |

D0R closes the last known pre-access implementation ambiguity. The checked
profile now gives one executable meaning to the optimizer, row schedule,
microbatch reduction, three classical controls, every metric ratio including
zero denominators, hard/resource gates, terminal precedence and publication
payloads. The same owner can later accept an official D0 provider, but D0R
itself issued only a `SURROGATE_D0` capability and used discarded artificial
targets.

## Artificial discriminator

The full `6,480` train and `4,320` development rows use a nonzero artificial
teacher: the frozen untrained QSO at seed `370201` plus a small declared
nonlinear query residual. The residual prevents a zero-error denominator from
making the ablation gates vacuous. It is mechanics evidence only and says
nothing about how real Steel, Glass or Wood sounds.

The trained candidate reached aggregate RMSE:

| Axis | RMSE |
| --- | ---: |
| decay | `0.0004616956801256649` |
| global gain | `0.00013779437755409926` |
| contact | `0.0004968328396440674` |

All `22/22` metric gates passed. Candidate contact error was `0.0818×` local
interpolation, `0.1675×` ridge, `0.0911×` nearest and `0.0901×` the pointwise
MLP. Frozen QSO ablations were `4.52×` to `9.71×` worse in their declared
strata, so field, query, interaction and topology paths were all observable.

## Exact execution evidence

Two independent CPU processes produced byte-identical stdout, empty stderr and
the same four-file terminal tree:

- owner SHA-256: `2ee81490…500e`;
- profile SHA-256: `75baf8cc…65f3`;
- terminal tree: `5f50eccb…46eb`;
- topology: `617c0cb2…a59e`;
- run times: `29.41 s` and `29.17 s`;
- peak RSS: `852,260 KiB` and `855,856 KiB`;
- published size: `719,301` bytes per run.

The checked readiness seal binds the current owner/profile, frozen
NumPy `2.3.5` / Torch `2.8.0+cu128` CPU environment, exact D0 lifecycle, two
identical executions and zero forbidden access. Before training, the owner also
revalidates the earlier E0 execution seal and T0 exact-truth composite seal.
Generated model bytes and terminal trees remain under `/tmp` and are not in Git.

## Verification

- Ruff `0.16.3`: owner and focused tests pass;
- mypy `2.0.0 --strict`: owner passes;
- focused D0R unit suite: `7/7` pass;
- combined V37 Python suite: `59/59` pass;
- focused xtask physical-sound registry suite: `161/161` pass;
- final process A/B and readiness-seal construction: pass;
- official/real/protected counters: all zero.

`boundary-scan` retains the known unrelated repository-baseline failure
`SOURCE_LAYOUT_ESCAPE_HATCH` in
`tools/xtask/src/physical_sound_registry_command/realimpact_transfer_fixture.rs`;
it reported no new D0R-specific boundary finding.

## Consequence

The next critical-path action is the one-shot official D0 opening through a
provider that verifies E0, T0 and this D0R seal before materializing any target.
D0 scientific quality is still unknown. H0 remains unopened and is permitted
only after an exact D0 `Pass` freeze. The internet source lane remains
independent and still needs six exact-Steel plus 27 non-Metal parent groups
before S1 can freeze real roles.
