# NCGP14 confined Nonlocal pressure/contact evidence

Date: `2026-09-01`

Final result: `CONFINED_PRESSURE_CONTACT_SUPPORTED / INDEPENDENT_GO`

Claim ceiling: `TWO-STEP CPU LONG-DOUBLE PRESSURE/CONTACT BASELINE ONLY`

## Question and answer

NCGP14 tested whether the NCGP13 failure was caused by an invalid open-column
fixture rather than by the corrected density-pressure operator or frictionless
analytical contact. It retained the open 128-particle failure, added an open
512-particle size control, and placed the same 128-particle state in a tight
side- and bottom-supported tank. Physics, tolerances and time step were not
retuned.

The result selects the fixture explanation. Both open columns reject their
second private trial at the velocity gate. The tight lane with a 4096-sweep QP
ceiling stops honestly on work, while the otherwise identical 16384-sweep lane
converges and commits both trials. Its velocity, density, containment,
momentum, energy and topology observables pass. The conditional 16-round lane
is correctly not run because the 8-round lane already supports the fixture.

This establishes a bounded CPU pressure/contact baseline. It is not a
surface- or viscosity-coupled result, a long trajectory, correct-water proof,
CUDA correspondence result, 4k/16k/50k result, performance measurement or
runtime/product result.

## Frozen identity

- frozen Revision-5 contract commit: `6d7b71ef`;
- final implementation and reviewed repair commit:
  `d1cfe76c4d2a342e99dc0954ba7e322709af1784`;
- final tree: `bbdf9f83918bd21698396e069907b6c3aa198255`;
- wrapper SHA-256:
  `c1c1b454a383b0d5c53eda5b4567da809597dbefc9664e2dd260ce7b56ad786e`;
- CMake SHA-256:
  `f44e47563054f9e67cdcf7fd9685a0f946e9447fa6fd240a637c043375d9c7e9`;
- aggregate source root:
  `a3f980e56dff5ce0e599f74b2048092ebac4d4b2d875f8d1c2a4d918a40ccd16`;
- contract file SHA-256:
  `4573690e22e79c999b2cdcd609747d0cb061ee59df475dd9040450c6678fd880`;
- nested contract root:
  `e6d9cdce3a67818024c68ad2da7f4d2405613b7b27953678530b4a415609779f`;
- clean Release binary A/B SHA-256:
  `f924209d34ce75ed679bda043156d0d4f79ec13f266ea7ce4bdd6cb5141cab14`;
- byte-identical stdout A/B SHA-256:
  `15a92dff1e927f4137c43a607a9b31e30200a76e1ebd49a99127543d7a98bd37`;
- final result root:
  `54c89f7a0bd4fd13920db325a2b401591cd8cfca3690fc694440d28b42f54221`;
- total expected/actual work root:
  `32212119d1635e1124986ceef9b6726801a9ab167dd72260aeee497b8882833f`;
- finalization root:
  `9bc58d6407d0245bb6486a9452c71fa2f900392a5a39a1742964b23fe01b179a`.

Author build directories outside Git:

- `/tmp/nextengine-ncgp14-final-a.4RZ6tP`;
- `/tmp/nextengine-ncgp14-final-b.wjpcKW`.

## Exact command

```text
cmake -S crates/continuum-water/tools/nonlocal-feasibility \
  -B <fresh> -DCMAKE_BUILD_TYPE=Release
cmake --build <fresh> \
  --target nonlocal-corrected-cpu-confined-pressure-contact -j2
<fresh>/nonlocal-corrected-cpu-confined-pressure-contact \
  --confined-pressure-contact-discriminator
```

Both valid scientific runs exit zero. Declared apparatus failures emit typed
JSON and exit `2`; executable-read or unexpected failures exit `3` without a
scientific classification.

## Frozen lane results

| Lane | Result | Accepted trials | Key evidence |
| --- | --- | ---: | --- |
| OPEN-128-CAP4096 | physical reject | `1/2` | retained trial-2 velocity RMS `0.0764701228422 m/s > 0.05` |
| OPEN-512-CAP4096 | physical reject | `1/2` | trial-2 velocity RMS `0.0739900860230 m/s > 0.05` |
| TIGHT-128-CAP4096 | QP work ceiling | `0/1` | first round uses exactly `4096` sweeps |
| TIGHT-128-CAP16384 | supported | `2/2` | `5/8` rounds, maximum `8111` QP sweeps |
| TIGHT-128-CAP16384-R16 | typed not run | `0/0` | `NOT_RUN_R8_SUPPORTED` |

The supported tight-tank lane has:

| Observable | Trial 1 | Trial 2 | Frozen limit |
| --- | ---: | ---: | ---: |
| velocity RMS | `0.017535844394 m/s` | `0.022846294063 m/s` | `<=0.05 m/s` |
| maximum speed | `0.035531628259 m/s` | `0.061264136241 m/s` | `<=0.10 m/s` |
| position RMSE | `0.073066 mm` | `0.155559 mm` | `<=2.5 mm` |
| maximum displacement | `0.148048 mm` | `0.403316 mm` | `<=5 mm` |
| maximum positive strain | `0.069739%` | `0.078844%` | `<=0.1%` |
| RMS positive strain | `0.022004%` | `0.024136%` | `<=0.025%` |

The exact sweep lists are `[6887,7663,6759,6521,6042]` for trial 1 and
`[8111,7274,6523,5720,4486,4314,3870,3117]` for trial 2. Every final primal
residual remains below the unchanged `1e-8` limit. Exact inset, identity,
mass, pressure balance, momentum, energy and single-component topology gates
also pass.

## Geometry, surface census and controls

The independent initial tight-tank census reproduces `6560` dynamic-dynamic
pairs including self, `6988` dynamic-ghost pairs, `6480` unique lateral pairs,
face memberships `[1809,1865,1809,1865,885,0]` and maximum row degree `117`.
Moving the exact 100-record `z_index==12` ghost layer by `-0.125 m` creates
exactly `144` top-face pairs while preserving the original integer face
identity, and changes the sealed census root.

The corrected surface census remains diagnostic only. Candidate and
independent oracle agree at `2976/17764/3216` active pairs for OPEN-128,
OPEN-512 and TIGHT-128. OPEN-128's two-step direct surface velocity scale is
only `0.000319011436 m/s`, far below the open-column excess; its zero-net-force
bound cannot remove the separately frozen mean-speed failure. Surface is not
used in the supported pressure/contact trajectory.

All eight top-level controls pass. Control 6 publishes exactly ten work-root
evidence entries and the required 21 ordered bool scalars; its result root is
`02374f4087cebf83224ec61378f9d049dd7f8bc7e96a8228f3d43302297d12d6`.
All canonical/permuted lane routes, semantic roots, work receipts and result
roots are exact.

## Verification and independent review

- two fresh Release configure/build/runs are byte-identical;
- both stderr streams are empty;
- retained NCGP12 exits zero with
  `NONLOCAL_SUPPORT_REDESIGN_REQUIRED`; stdout SHA-256 is
  `9916f076c05115c4034682e00ef3ff6346dd1d1e9164933f771be515f105a0f1`;
- retained NCGP13 exits zero with `PRESSURE_CONTACT_TRAJECTORY_REFUTED`;
  stdout SHA-256 is
  `7d6cbaedea4e9f8055b037f4aef5baa46b5392c3b000792a03a99c31979ebff5`;
- the final ASan+UBSan run exits zero with empty stderr SHA-256
  `e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855`;
  its instrumented stdout SHA-256 is
  `bdd7c2bc8a591a0052aa2991bb39499faf4100ce23e1e6228ade587006e90270`;
- `git diff --check` passes and both author and reviewer worktrees remain clean.

The initial independent review reproduced the numerical result but found that
Control 6 omitted five required mutation booleans and used non-frozen row
names. The one allowed apparatus repair changed only that scalar publication.
The re-review independently rebuilt and reran the exact repaired commit,
recomputed all `85` expected/actual work pairs, `37` receipt work roots, the
20-child total work root, Control 6, finalization and `result.v3`, and returned
`GO`. The repair/re-review allowance is exhausted.

## Interpretation and next action

The NCGP13 trial-2 failure was not evidence that pressure/contact cannot hold
confined water. It was primarily an invalid hydrostatic fixture: most samples
had neither lateral wall support nor enough pressure/surface reaction to avoid
nearly ballistic acceleration. In the tight tank, the unchanged pressure and
frictionless contact composition supports both frozen trials once the QP is
given the predeclared sufficient work ceiling.

The next smallest stage is a separately frozen CPU long-double trajectory that
reintroduces the corrected viscosity and surface terms on top of this exact
confined baseline, first over a bounded short horizon and then over the longer
correctness window. Only after that stage passes independent review should the
same corpus be moved to CUDA and then scaled to 4k/16k/50k timing. CPU DFSPH
remains the product fallback; SPEC-38 and ADR-076 remain Proposed.
