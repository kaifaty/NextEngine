# TRAIN-4 coupled complete-clip trajectory research

| Field | Value |
| --- | --- |
| Date | 2026-08-14 |
| Scope | Optimizer-free learned-policy lane; offline constraint-solver research for `REQ-HUM-DATA-005/007` |
| Status | `R93 V9 OFFLINE PASS / R94 FRESH FAIL / STOP_AND_RESEARCH` |
| Current implementation | `nextengine.dimensionless-contact-trajectory-qp.v9` |
| Claim ceiling | Research infrastructure only; no V19, TRAIN-4 Advance, visual gate or learned optimization |

## Frozen boundary

The cycle retained active tangential displacement `2000 µm/frame`, active
normal displacement `1000 µm/frame`, active normal residual `5000 µm`, the
collider floor `-2 µm`, joint-velocity reserve `2500` basis points and root
vertical velocity `200060 µm/s`. It added no grace window, settling interval,
velocity zeroing, contact-point deletion, ROM/impact relaxation or post-solve
safety projection. Fresh scenes remain acceptance authority; indexed partial
reset remains report-only. Learned optimizer steps and training runs remain
zero.

The accepted offline question was narrower: can one complete immutable clip
be changed by one bounded root/leg trajectory solve so contact pose, finite and
analytic contact velocity, every collider sample, soft ROM and root/joint
velocity all pass simultaneously?

## Research sequence

All generated artifacts remain under the external TRAIN-4 evaluation root.

| Evidence | Result | Disposition |
| --- | --- | --- |
| R58 per-frame coupled Gauss-Newton | Root/contact corrections spike and do not preserve the temporal envelopes. | Reject independent frame solves. |
| R59 global normal equations | Improves several residuals but stagnates before simultaneous feasibility. | Keep the clip-global variable domain; reject weighted least squares as the constraint mechanism. |
| R60 stronger weights plus hard root closure | The post root operation repairs its own bound and re-breaks contact. | Reject every sequential root/joint post-pass. |
| R61 contact/joint baseline | Contact passes (`4999 µm` residual, `574/430 µm` finite, `573/457 µm` analytic) and joint reserve is `2500 bp`, but collider minimum is `-30748 µm` and root vertical velocity is `478470 µm/s`. | Proves contact plus joint feasibility, not complete feasibility. |
| R62 simultaneous weighted inequalities | Contact, collider, root and joint facts still trade against each other after a hard projection. | Reject penalty-only active constraints. |
| R63 all-collider coupled solve | Stable but stalls at collider `-30100 µm` and root `469680 µm/s`. | The active set is correct; scaling/solver is not. |
| R64–R67 root envelope, active corridor and smaller line search | Feasible directions collapse to tiny steps; R67 accepts steps as small as `1/512` while useful metrics barely improve. | Reject line-search tuning; normalize variables and rows explicitly. |
| R68 dimensionless sparse QP | Reaches contact `4999/1999/999`, but quantization leaves collider `-57 µm`, root `200100 µm/s` and joint `2501 bp`. | Feasible mechanism confirmed; add only internal quantization reserve. |
| R69 reserved sparse QP from R61 | Complete `cmu05` passes: residual `4900`, finite tangent/normal `1978/980`, analytic `1968/996`, collider `+30`, joint `2500`, root `199800`; zero dropped points. | First simultaneous complete-clip proof. |
| R70 raw canonical clip, eight outer iterations | Contact passes and all velocity/ROM facts pass; thirteen collider samples remain, minimum `-146 µm`. | R61 is not a semantic input, but eight direct iterations are insufficient. |
| R71 exact R70 continuation | Two more relinearizations reach collider `+1 µm` and complete PASS while contact and velocity facts remain within bounds. | Freeze a direct-source V8 candidate with at most twelve outer iterations; require a single-invocation reproduction before evidence promotion. |
| R72 raw canonical clip, one invocation with at most twelve iterations | Stops on exact PASS at iteration ten: residual `4900`, finite tangent/normal `1981/980`, analytic `1968/996`, collider `+1`, joint `2500`, root `199740`; zero dropped points. An independent FK audit of the emitted integers gives residual `4901`, finite normal `981` and collider `0`, still PASS. | Confirms the direct-source iteration bound and exposes that final metrics must be recomputed from emitted quantized root/joint values. R72 ran from a dirty research worktree, so clean-commit all-clip evidence remains required. |
| R73 clean V8 all-three/all-17 invocation | `cmu05` passes at iteration ten and `cmu16` at iteration five. The stronger raw `cmu139` mask takes a `231006 µm` first root step, reaches collider `-34056 µm` and residual `31581 µm`, then OSQP reports primal infeasible. | Reject V8 as an all-clip solver. The clean run confirms two controls and localizes the blocker to `cmu139`; no fresh PhysX run is authorized. |
| R74 point-entry stencil on raw `cmu139` | The 16 point-level edge differences are real, but the second QP is still primal infeasible. | Keep the semantic observation; reject it as sufficient remediation. |
| R75 applied-step cap, original V8 stencil | All twelve QPs remain feasible. Final collider is `-5682 µm` and residual `5811 µm`; collider alternates between approximately `-7..-9 mm` and `-20..-27 mm`. | A bounded step removes artificial infeasibility, but blind acceptance creates an active-set oscillation. |
| R76 point-entry plus applied-step cap | All twelve QPs remain feasible, but final residual/tangent/normal are `6296/2594/1078 µm` and collider is `-2732 µm`; alternate iterations still regress collider to `-26..-32 mm`. | Reject the combined semantic hypothesis. The remaining defect is step globalization, not a missing scalar or point-edge switch. |
| R77 full group-elastic in-QP trust | The first trial improves exact normalized merit `18.0222 -> 14.09237`, but actual/predicted ratio `0.204` correctly rejects it and shrinks the radius. Duplicating all row sides then reaches the OSQP iteration limit. | Exact rejection behaves as intended; reject the dense/full elastic encoding. |
| R78 selective category-elastic trust | The first trial is accepted at ratio `0.274` and merit `12.8099`, but temporarily creates collider `-33386 µm`; the next QP reaches the iteration limit. | Global category slack columns destroy useful clip-local sparsity and are not a production encoding. |
| R79 selective per-row L1 elastic trust | The first direction worsens gate-aligned max merit `18.0222 -> 24.4408`; both model and exact audit reject it. | A row-average L1 objective is not interchangeable with the current max-based acceptance metric. Stop representation tuning until model and exact merit are identical. |
| R80 exact max/sum backtracking | Five accepted hard-QP steps reduce worst normalized violation `6.0602 -> 2.4908` and total violation `18.0222 -> 9.5412`, eliminating the R75/R76 alternation. Acceptance nevertheless contracts from factor `1` to `0.5`, `0.125` and the minimum `0.0625`; final collider/residual remain `-12456/17402 µm`. Rejected full steps at trials 3–5 reduce total violation to `9.5286`, `8.7025` and `8.9733` while temporarily increasing the worst component. | Partial discriminator, not a candidate. Monotone max-first backtracking prevents oscillation but stalls at contact curvature and discards useful non-dominated steps. Stop step-factor tuning; test an exact filter, then second-order correction/restoration if rejection persists. |
| R81 exact worst/total feasibility-filter observation | The first two full candidates improve both measures. The filter then accepts the R80-rejected full third step (`3.3688/12.7776 -> 3.7554/9.5286`) and another non-dominated step (`3.7554/9.5286 -> 5.9160/8.8038`). Thus it crosses the max-first plateau, but permits worst violation to return close to the immutable source ceiling `6.0602`. The run was stopped between QPs before a report/candidate was emitted. | Support the filter diagnosis; reject this two-violation filter as a solver identity. It is an explicitly non-promotable interactive observation, not evidence. Move to exact-row second-order restoration rather than spend twelve more `100000`-iteration QPs on the permissive filter. |
| R82 exact-row hard second-order correction | The first two capped primary steps reproduce the useful R80 path (`6.0602/18.0222 -> 4.7570/15.2353 -> 3.3690/12.7778`). The third full step lowers total violation to `9.5300` but raises the worst component to `3.7556`, so max-first acceptance correctly routes it to correction. The bounded correction with the original `58488`-row Jacobian and nonlinear trial-state bounds is `primal infeasible` after `16925` OSQP iterations; no corrected exact state is evaluated. | Reject hard all-row SOC as sufficient, not the second-order diagnosis. The report remains `FAIL`, the pending trial/candidate is non-admissible, and the last accepted state remains the second primary point. Next isolate artificial infeasibility with iteration-only normalized nonlinear restoration slack; final exact-zero audit remains unchanged. |
| R83 minimax nonlinear phase-I restoration | Relaxing only nonlinear rows makes the correction model feasible. Four bounded bisections bracket minimum normalized slack between `0.235364` infeasible and `0.470727` solved; linear/trust rows remain hard. The selected model balances every nonlinear group near `0.4707` and improves exact contact violations, but its `50 mm` root-saturated correction drives exact collider from baseline violation `3.1438` to `8.8592` (`-44298 µm`). Exact max-first audit rejects it and restores the baseline. | Reject the old-Jacobian minimax restoration identity. Its model predicts collider violation `0.4707` while integer FK measures `8.8592`, directly localizing stale linearization over the correction radius. Recompute the correction Jacobian at the nonlinear trial point before changing slack, trust or requirements. |
| R84 hard trial-Jacobian restoration | The correction uses the Jacobian already evaluated at the quantized rejected trial, with identical `252303` base nonzeros, nonlinear row bounds and component trust. It remains `primal infeasible` after `66000` iterations with primal residual `0.06245`; no corrected exact state is evaluated. | Trial-point relinearization alone cannot remove local incompatibility. R83 and R84 isolate two independent requirements: trial geometry for exact fidelity and iteration-only phase-I slack for model feasibility. Test their direct composition before considering another mechanism. |
| R85 trial-Jacobian minimax phase-I | The direct R83/R84 composition makes all tested slack levels feasible; the selected `0.235364` solve reaches `solved inaccurate` at `100000` iterations and balances trial-model nonlinear rows near `0.2354`. Exact contact violations improve, and total violation drops `12.7778 -> 11.9054`, but collider violation rises `3.1438 -> 6.6700` (`-33352 µm`). Exact max-first rejects the correction and restores the baseline. | Reject this local composition as an acceptance mechanism. Trial relinearization improves R83 collider error `8.8592 -> 6.6700` but still underpredicts it by a large margin over a `1.7887` normalized correction. Stop local solver-knob composition and map exact/model error versus frame and step scale before choosing a new geometry or trust model. |
| R86 directional model-fidelity audit | The hash-bound R85 direction reproduces the reference exact violations with zero normalized delta. At scales `0.125` and `0.25`, model/exact worst merits are `3.3245/3.2958` and `2.8832/3.1418`; both improve the accepted `3.3690` baseline, with actual/predicted ratios `1.042` and `0.695`. At `0.5`, the ratio becomes `-0.390`, exact merit regresses to `4.4442` and maximum collider prediction error grows to `13030 µm`. The dominant error is `collider.right-foot`, especially source frames `1383..1392`, worst at `1388`. | Accept only a measured local model-valid interval, not any audited scale as a candidate. A quarter direction uses at most `8944 µm` root component / `9555 µm` root norm / `24992 µrad` joint component, while the half direction is outside reliable geometry. R87 must re-solve the trial-point phase-I inside `10000 µm` root-component and `25000 µrad` joint-component trust bounds; it must not scale the stored R85 direction. |
| R87 model-valid bounded re-solve | A fresh trial-Jacobian phase-I solve brackets minimum model slack between `0.941454` infeasible and `1.176818` feasible. The exact candidate improves the retained baseline from `3.3690/12.7778` to `2.4324/5.9482`; model merit is `1.1769/4.7073` and the actual/predicted worst-reduction ratio is `0.511`. Contact groups fall to `1.1535..1.2048`; collider remains dominant at `2.4324` (`-12163 µm`). Root/joint component use is `10000.001 µm / 25000.044 µrad`, the sub-unit excess being the reported OSQP residual rather than a postprojection. | Accept the research step and the measured trust mechanism, not the candidate: report status is `COMPLETE / EXACT_IMPROVING / NOT_ADMISSIBLE`, exact gate is still `FAIL`. Keep the radius unchanged because agreement is positive but not strong enough to expand into R86's invalid half-step region. R88 relinearizes once at the R87 exact state and repeats the same bounded phase-I plus exact max-first audit. |
| R88 unchanged-trust continuation | Relinearizing at R87 lowers feasible model slack to `0.763320` and predicts merit `0.7634/3.0533`. Exact contact groups all fall below one, but collider regresses `2.4324 -> 2.9816`; exact merit becomes `2.9816/5.6120` and actual/predicted ratio is `-0.327`. Maximum collider prediction error grows to `11143 µm` at `collider.right-foot`, source frame `1387`. | Exact max-first rejects R88 and retains R87. Do not repeat the same radius or accept lower total violation. Contract trust by `0.5` and re-solve from R87, not from the rejected R88 candidate: R89 uses `5000 µm / 12500 µrad`, matching the R86 scale-`0.125` interval whose ratio was `1.042`. |
| R89 half-radius contracted re-solve | Starting again from retained R87, the contracted phase-I brackets slack between `0.915984` infeasible and `1.068648` feasible. Model merit `1.0686/4.2746` becomes exact `1.7226/4.8639`, improving both R87 coordinates with ratio `0.517`. Collider error falls from R88's `11143 µm` to `3321 µm`; root component/norm use is `3937/4676 µm`, while joint use reaches `12500 µrad`. | Accept R89 only as the new research iterate; exact collider/contact groups remain `1.7226/1.0348..1.0590`. The contraction restores agreement and validates an adaptive loop. R90 may run at most six relinearized attempts, retain trust after ratio `>=0.25`, halve it after rejection, never expand, and accept only exact max-first improvement. |
| R90 capped adaptive trust loop | Six hash-bound attempts accept two and reject four. Exact merit improves `1.7226/4.8639 -> 1.1100/4.3599`; the accepted steps have ratios `0.697` and `0.999`. The final accepted state still violates collider/contact groups by `1.1098/1.1100/1.0625/1.0776`. A sixth solve at the unchanged minimum `625 µm / 1562 µrad` predicts improvement but measures exact `1.2382/4.3191`, ratio `-1.832`, and is rejected at minimum trust. | Record `COMPLETE / FAIL / NOT_ADMISSIBLE / minimum_trust_rejection`. Stop radius/acceptance tuning: at the fifth step collider prediction error is only `1.6 µm`, but the next same-radius step jumps to `992 µm` at the right foot. R91 audits whether the scalar box-minimum row switches its active support vertex; no candidate, PhysX or training run is authorized. |
| R91 stable box-feature audit | The audit reproduces R90's scalar model to `9.1e-13 µm` and proves scalar-minimum/vertex-minimum exact identity to `2.3e-10 µm`. Across `15414` box/frame samples it finds `85` baseline-to-exact feature switches, `84` on foot boxes, plus `246` one-sided-probe switches. At the source-`1386` hotspot the active right-foot vertex changes `x-,y-,z+ -> x-,y-,z-`; scalar error is `992.020 µm`, while the stable-vertex prediction error is `3.706 µm`. Global maximum error falls to `3.874 µm`, ratio `0.00390`. | Record `COMPLETE / SUPPORTS_PER_VERTEX_BOX_ROWS / NOT_EVALUATED`: all predeclared discriminators pass, with no candidate or gate claim. R92 may implement stable vertex rows only for the two contact-role foot boxes, retaining scalar rows for the other 17 colliders. That bounded choice adds `14` rather than `98` rows per frame and keeps exact all-collider FK authority unchanged. |
| R92 clean V9 raw `cmu139` | Commit `04005c7` adds eight stable rows for each contact-role foot box and preserves scalar rows elsewhere. The raw immutable clip reaches emitted-integer exact PASS at outer iteration three: residual `4901`, finite tangent/normal `1981/981`, analytic `1969/996`, collider `+49`, joint `2500`, root `199770`; every normalized violation is zero. Each QP has `73902` rows / `344787` nonzeros; solve iterations are `100000`, `84025`, `4350`, wall time `527.8 s`. | Accept the shared V9 identity for clean offline promotion testing, not the single-clip debug candidate. R92 remains `NOT_ADMISSIBLE`, optimizer/training/PhysX stay zero, and performance is report-only. R93 must reproduce all three complete clips, all 17 exact slices and overlap identity from one clean invocation before any fresh scene. |
| R93 clean V9 all-three/all-17 | One clean invocation from commit `cffad89` solves each immutable complete clip exactly once. `cmu05`, `cmu16` and `cmu139` pass after `3/2/3` outer iterations; all 17 exact slices pass, all `390` compared arrays across `30` overlap pairs agree byte-for-byte, and contact-point deletion remains zero. Optimizer steps and training runs remain zero. | Accept the V9 offline promotion checkpoint and authorize the next ordered fresh-scene all-17 PhysX acceptance only. `gate_decision=NO_CHANGE` retains the claim ceiling: no corpus admission, V19, visual gate or training is implied by this solver-only PASS. |
| R94 clean V9 fresh all-17 | Seventeen separate fresh-scene workers cover all cases from commit `5cedc41`. Initial root/joint state is exact within quantization, but seven cases fail: hard impact `4`, hard ROM `4`, joint safety/velocity `1`; ordinals `2/7/10/14` regress previously passing controls. Partial reset is not run; optimizer/training remain zero. | Record `FAIL / STOP_AND_RESEARCH`. Reject reset-state mismatch and the claim that offline V9 geometry is sufficient for fixed-PD native tracking. Freeze V9 and perform the linked report-only R95 V7↔V9 native-dynamics differential audit before any new PhysX counterfactual. |

R69 report SHA-256 is
`4e840f9f9d91f4b13ffbda8f23ab33b2158f61ad87bcdd1e12c6932ae9606b56`;
candidate SHA-256 is
`81c659703304db6341125011901ae98daab67afcceb51008f75801d3400c0266`.
R70 report/candidate SHA-256 are
`047b09c418ee33273ebcba0776a16d861bb7bba42085d5496b406f1f5b4dec80` /
`c8abd88d17673de2bc44411e1fdf3af607415e3dd6421fd5af1a78bca5d283df`.
R71 report/candidate SHA-256 are
`8cde1b6c210efdb6f2f9b3205d426de25817b8016a6c4c5b77d63a0704486f73` /
`917fccc2f7737bc29144411e1fdf3af607415e3dd6421fd5af1a78bca5d283df`.
R72 report/candidate SHA-256 are
`04247d340c0e533facdf71acd7a3c026ccd8a8f139b57340c7210ff04b5e7b5d` /
`10acecd2e9adad6cd850d906e9807d93059a16b9f62266f65913ab20b05e884e`;
the research oracle SHA-256 is
`81f7265a7bc706dba3f31f302ede95191cd3da0c75fbac4bfb55c2a27afe2491`.

R73 manifest SHA-256 is
`d0b3897545af22bfefa68e69562eb27e5bfc182b240e09baa12325d0ab31d37c`.
R74 report/candidate SHA-256 are
`c8cfcb6d52f80ef7c68b34bf0ab602a6919ece6cb2bee1cc0e5137c6c8710b72` /
`fe8a61023bac2d35eadec1488cd92761055d0d66243a3c1dd94335a363afc840`.
R75 report/candidate SHA-256 are
`8e14416db70286873230e42c0419080e4799f66aafddf7b2bde81d2cddc82c86` /
`5fbab9156e76981c88c6dbd9ba3168d04fa0a37eff081910bff3b9588796b433`.
R76 report/candidate SHA-256 are
`6e5093796cf1d786285813fef92c21682cb129e3dd0cd5ec3bf046b13590d00c` /
`32d62f740151a462ce8faef379752b0029fe9268122904d782efed5fc81733d1`.
R77/R78/R79 report SHA-256 are respectively
`acedf2f3fe68d98fe89cb0fbd5e7a7c5ec7f4d07db7e1e9d28c0571d16e83fa`,
`ff844f676a52f01faf1ac38b200346db64c9328f2bca23d30deed1d02f69bd87`
and `53b2d262b8843eb8ecbec66c9f4d14386d57a9093e0e0d045a09cf084a7ab933`.
R80 report/candidate/research-adapter SHA-256 are
`23d73e47b70c236079ea1d52f67e81de7313433b795e06f69525e06c3e90a60e` /
`570ae002ade352d5cc6e5fcb99ca6db2ca48871bf73f180ecc5d187a3688677c` /
`157cc9fc9ee296fb07375916b2e5b7b3f6abc24664792f6a1c87930c2cffeb88`.
The run was deliberately interrupted while solving trial six after the
minimum factor had already been consumed; its report `FAIL` and
`solver_failure/interrupted` termination are retained rather than relabelled.
The non-promotable R81 adapter SHA-256 is
`34f8ffcadaea0e51d05a512fcd6741aaa8379310caccd0cf38273c7f054fb0ea`.
R81 has no report/candidate hash and cannot be cited as acceptance evidence.
R82 report/candidate/research-adapter SHA-256 are
`c6f8db607228b081594bc8cc43b87a05477ecfb2290c9dd5f95b543fa666a44e` /
`f0240e47757f1bac3394045f6287bd15d111d19e376fe7ab6651a9ec2e5cb4ea` /
`a13860b1be3a2f802bc80f0eba54fd4bd750e291b8627315b933018097f993f2`.
The candidate captures the rejected pending trial after correction solver
failure and is explicitly non-admissible; the report's
`final_accepted_exact_violation` identifies the retained baseline.
R83 report/candidate/research-adapter SHA-256 are
`7d8da4fe0a17510a65aa818fa551132b0ce881b6d6f69e7bfac0fcc1b0285c0b` /
`8f40a26c7a896700cc1047b3dbe44e2b5cd3bdb82f1ff18d0efa22e9a4642460` /
`339bf7cd7085caabbc80089e5a2d895dc84a0528f1adf26ef3c16780f6e09604`.
The R83 candidate is the restored accepted baseline, not the rejected
correction, and remains non-admissible because the unchanged gate is `FAIL`.
R84 report/candidate/research-adapter SHA-256 are
`513b52aa1ff500060962fe68dd910494a43b9e369bf52105077b1a369a144485` /
`f0240e47757f1bac3394045f6287bd15d111d19e376fe7ab6651a9ec2e5cb4ea` /
`1e66abebc8a01c890c93ae969af6771aa3a4403f2ab5188709f222da18340f06`.
The R84 candidate again captures the pending rejected primary trial after
correction solver failure and is explicitly non-admissible.
R85 report/candidate/research-adapter SHA-256 are
`228117a508ff22c9f3a5927862af8b79a40635446c3b02a305651231aeb69adf` /
`8f40a26c7a896700cc1047b3dbe44e2b5cd3bdb82f1ff18d0efa22e9a4642460` /
`bc23a1535deb1e5235ef312e5903f6d1b490d97fba85bd1a6a34a30f0a5732f1`.
The R85 candidate is the restored accepted baseline and remains
non-admissible because the unchanged gate is `FAIL`.
R86 report/direction/research-adapter SHA-256 are
`18e395edb33ed80a6f8946988e377f401943bdad4523813f648d42429eb15036` /
`23cdcfe5ec5ccc9d080cfdf893b1563976b4516a94c256fe86f013f535f3c515` /
`92676ca3cbb3d3351224d8922a207b537388db1a86623b0eeaa0fdbf74570f2f`.
R86 emits no candidate and has `candidate_status=NOT_EVALUATED`; every scale
is a research measurement rather than admissible TRAIN-4 evidence.
R87 report/direction/candidate/research-adapter SHA-256 are
`318b1c9e18a3ffcb60ea69e1eb75faee0f97d28c7fda4c8640b158f4872ff062` /
`ae0ef25f4dc8fddf51631cd7abaa837c4915ad73b059b55f7d620c7b54d6aa27` /
`e5c5de0772c2da503dc2f3e0069b798c588afcb4ea0f25c85967e3460386f5bb` /
`2fa0dd674b9d06a85238090a4a22e0c3358fe2a7a92f271e0d2905d0b9d899e5`.
The candidate is an exact-improving research iterate, not admitted evidence.
R88 report/direction/rejected-candidate/bridge/research-adapter SHA-256 are
`fe3c437a462a52d8aca5209f0389977de8b70ee65d73d4d7b239a7470d4ee88c` /
`5ca09a098ae5072f840be6aaa4ff8d9962db8731b55196b43668906a8d60bfad` /
`44ea0ff9ec0357ea16831bd8bb576ac9ff2792aea3d62c0ea7a4f1240aeebc27` /
`9438fda09d9c4f06ee27fe54855f646c11da2530cc0653faa4fba14009c46a1d` /
`6577318057f5385e8d987d057e77896bfecb52d30007533bda56d494d83728b0`.
R88 is a completed rejection record; R87 remains the research iterate.
R89 report/direction/candidate/bridge/research-adapter SHA-256 are
`7de07519892f416065f1b4479125181af0820e9a27b87511ff766000a992c4cc` /
`396845cf2978ab1d7a526bbf0703668293964bc38454965142a64e66ae081597` /
`db716c4a3f14e0b296ca316923da956df2e4cbccd12c9d9919174b9db13479ab` /
`dbb3b263eaa1c26139492ce330183f0f8e78438367508b49cefb7485111b4a95` /
`6eabdc3ff5377441b15bf191e78c4ab650180bd692306409bf8794786274fd1c`.
R89 is exact-improving but remains `NOT_ADMISSIBLE` and exact `FAIL`.
R90 report/accepted-candidate/research-adapter SHA-256 are
`e6abf145e7fafa47423c77ca59985dcc687102b3cc3079dd2fd4d935a4a4efc3` /
`9905394fb254d0d238c4edb011f9d9f7939fdcedcf9e7fa77b5965ef000879b1` /
`687df77b8624ddff851b675fa98f0e5e2c64be9ac3324ed9fe3636fd0e40125f`.
R90 terminates at minimum trust with exact `FAIL`; its accepted state is a
research baseline only and cannot enter TRAIN-4 evidence or a corpus.
R91 report/debug/research-adapter SHA-256 are
`a84cfe542d40ef30ff5d2efa2cf458c7829a2d3be117336d3974a499471040a7` /
`25cdd801974c56e46b3f685d4ed621ead51b3144ecac596aad82e00b1607c764` /
`0ca2ab6e2d2136897c6350516fa3211b0b862c18c2ae5a4bad72b66a4eb59e22`.
R91 emits no motion candidate and leaves exact TRAIN-4 status unevaluated.
R92 report/candidate/research-adapter/profile SHA-256 are
`753d4070e7a31076c4873df182e297963c53942db1295e23e16b67203b97d209` /
`1fa35712652d063336121e7d0fed813bea79d18cb86f0203ea754ca7496b6b8d` /
`5a89854309efbcff5915087cd950e451b2340bfeacd44786de65885215122bf1` /
`bccf5eb12c8de2e1c0fa20df74c55f30cc9e964e7d5cbfcbf596bb6d672815ab`.
The production V9 implementation is clean commit
`04005c747057a08246d8f25b615effa51b043792`; R92 is a raw-clip proof, not an
admitted corpus or fresh-scene result.
R93 canonical/file manifest SHA-256 are
`7ee041b7710302b9fae909b0cb34c0de3a6c2df257032cd0e1c7dcaf16afefeb` /
`53984129cc45d295ced9ef4f49ea8532d0b224c3ccc19980e8b986cee7604d70`.
The run is bound to clean commit
`cffad89dfa02fa5a6d30062ed0fa21c08547f93f`; complete-clip artifact hashes are
`73ac3b2e4c9fa82344006b5b6f38453f232179287e5c54f24fe099beb0f17912`,
`16cacfc0d6a5a57a7e2e2fad75faef1681e9b17c02cd3b78bafc7bb4ed3b5b7d` and
`8dbb178fa9d7a4ae1fe4cdb50133e285b8f236e523c4be90c277f6ecbe0968c0`.
R94 canonical/file report SHA-256 are
`0d429faff356e3240a7f2906115566b565fc0800043dd7ffa04544b2edb38925` /
`4aab74888d50fae2ab644445597d3f7f7a64217f078ceca4b4f8045ed34f0929`.
The run is bound to clean commit
`5cedc41d23958023f7b4d7dcee46c34f2f230b73`; probe profile SHA-256 is
`935bdd369c7ef3b485a114b8adadd7476b151345806f2957e1080595017d40e1`.

## Primary-source research decision

The observed failure is a known globalization failure of sequential convex
optimization, not evidence that the unchanged physical problem is infeasible:

- [TrajOpt](https://escholarship.org/uc/item/6km506db) places a box trust
  region inside each convex subproblem, converts infeasible constraints to
  L1 penalties, and accepts a step from true/model improvement rather than
  applying every convex answer.
- [SCvx](https://arxiv.org/abs/1608.05133) names the two matching defects
  *artificial infeasibility* and *approximation error*. It uses penalized
  virtual controls plus an adaptive trust region and rejects/re-solves when
  actual reduction disagrees with the linear prediction.
- [CRISP](https://arxiv.org/abs/2502.01055) is the closest contact-planning
  analogue: an L-infinity trust region is part of the QP, constraint penalties
  are individual, the radius follows actual/predicted merit reduction, and a
  second-order correction is reserved for persistent Maratos-effect rejection.
- [Trust-region SQP-filter methods](https://doi.org/10.1137/S1052623499357258)
  provide the alternative when one arbitrary penalty sum is not defensible:
  objective quality and constraint violation are filtered separately, with a
  feasibility-restoration path.
- [Contact Trust Region](https://arxiv.org/abs/2505.02291) shows that a
  symmetric geometric trust region can be inconsistent with unilateral
  contact. That is a secondary escalation if a correctly globalized V8 still
  fails at heel/forefoot entry; R74 proves it is not the first fix.
- [ContactIPM](https://arxiv.org/abs/2608.11731) demonstrates elastic contact
  relaxation with termination gated by the unrelaxed physical residual. Its
  full contact-implicit MPCC formulation would change the frozen-mode scope
  and is therefore a fallback, not the current intervention.
- [Convex Optimization, section 3.2.3](https://web.stanford.edu/~boyd/cvxbook/bv_cvxbook.pdf)
  establishes the relevant pointwise affine construction. For a box above a
  plane, `min(vertex_y) >= floor` is exactly the intersection of one affine
  floor inequality per stable-labelled vertex after local kinematic
  linearization; differentiating the scalar minimum is unnecessary.
- The [TrajOpt paper](https://rll.berkeley.edu/trajopt/ijrr/2013-IJRR-TRAJOPT.pdf)
  likewise incorporates a polyhedral collision approximation directly in the
  convex subproblem. This supports testing feature-preserving rows, without
  changing the exact collision gate or adopting its optimizer wholesale.

The immediate conclusion is narrower than adopting any paper wholesale. V8
lacks a globalization contract: it solves a hard local feasibility QP and
blindly applies the whole answer. R75/R76 show that bounding the applied step
prevents false infeasibility but not oscillation. R77/R78 show that an exact
actual/predicted rejection rule detects bad steps. R79 shows that the QP and
the exact audit must use the same violation functional; category-max and
row-average penalties cannot be mixed.

R80 supplied the missing discriminator without closing the clip. Exact
max-first acceptance makes monotone progress, unlike rejected R64–R67 weighted
least-squares line search, but reaches the minimum factor with all dominant
contact categories still roughly balanced at `2.3..2.49` times tolerance.
More importantly, its rejected full steps reduce total violation by more than
the accepted small steps while temporarily moving the worst category. That is
the signature for which a filter is preferable to a single scalar or
lexicographic merit.

R81 confirms that a filter can retain a useful non-monotone step, but also
rejects the naive choice of worst and total violation as its two coordinates.
Both coordinates measure feasibility, so reducing the total can conceal a
large regression in the exact-zero bottleneck. This is an evidence-specific
observation, not a claim that the SQP-filter paper prescribes those measures.

R82 tested the hard form of the standard second-order correction without an
explicit Hessian. Shared row identity stayed stable, and exact max-first
acceptance routed the expected third trial into correction, but the old
Jacobian cannot satisfy the full trial-state row set inside the retained
correction trust. This is direct evidence of artificial infeasibility inside the restoration step,
not evidence that the unchanged physical clip is infeasible.

R83 then tested lexicographic phase-I restoration. Sparse bound relaxation
avoids the dense R77/R78 encodings and proves a normalized nonlinear slack can
remove artificial infeasibility while linear and trust rows stay hard. It also
provides a stronger discriminator: the selected old-Jacobian model limits the
collider violation to `0.4707`, but exact integer FK measures `8.8592`. The
correction therefore leaves the validity radius of that Jacobian even though
the convex subproblem itself is solved accurately.

R84 reused the already computed Jacobian at the rejected nonlinear trial point
for one hard bounded correction. This removes the stale-geometry variable but
does not remove artificial infeasibility: the correction fails before exact
audit. Together R83/R84 now form a controlled two-factor result rather than an
invitation to tune another scalar.

R85 directly composed the R84 trial-point Jacobian and unchanged R83 phase-I.
It halves the selected model slack and reduces the exact collider regression,
but the corrected FK still disagrees qualitatively with a tightly balanced
linear model. This is the predeclared stop condition for local SOC/restoration
composition, not a reason to add a fifth scalar or relax the exact gate.

R86 completed that predeclared discriminator. The exact/model relationship is
reliable through a quarter of the stored direction and reverses before half:
the worst-merit actual/predicted ratio changes from `0.695` at `0.25` to
`-0.390` at `0.5`. Collider prediction error grows nonlinearly at the right
foot around source frame `1388`, from `2485 µm` to `13030 µm` over the
same interval. This supports an adaptive local trust contract and rejects both
the original `50 mm / 100 mrad` correction and blind post-scaling.

R87 completed that fresh solve inside root-component `10000 µm` and joint-
component `25000 µrad` trust. It improves both exact merit coordinates and
keeps the actual/model ratio positive at `0.511`, unlike the R86 half step.
This validates re-solving inside the measured radius and rejects blind
post-scaling. It does not validate one-shot correction as a complete solver:
the candidate still has four nonzero contact/collider groups and exact status
`FAIL`.

R88 completed that continuation and triggered the predeclared rejection path.
Its model predicts a `1.6793` reduction in worst violation while integer FK
measures a `0.5492` increase. Lower total violation cannot override the exact
max regression. The rejected state is retained only as diagnostic evidence;
R87 remains the research baseline.

R89 completed that standard trust response. It restores a positive ratio
`0.517`, improves exact max/sum and cuts maximum collider model error by more
than three times relative to R88. The remaining `1.03..1.72` violations are
close enough to test the actual adaptive iteration contract, but not to claim
feasibility or tune a tolerance.

R90 completed that bounded loop. Attempts two and five were accepted, reducing
exact worst merit to `1.2698` and then `1.1100`; four other candidates were
discarded. The final rejection occurred at the declared minimum trust, so the
loop stopped instead of weakening its ratio rule or taking a seventh attempt.
The remaining violations are small but nonzero, hence the exact gate remains
`FAIL` and the accepted debug state remains `NOT_ADMISSIBLE`.

The last two attempts provide a sharper geometric discriminator. At attempt
five the maximum scalar-collider model error is approximately `1.6 µm`; after
relinearizing at that accepted integer state, a step with the same `625 µm /
1562 µrad` component bounds produces approximately `992 µm` error at
`collider.right-foot`, source frame `1386`. The implementation computes box
height as `center_y - abs(rotation_row_y) @ half_extents` and differentiates
that scalar minimum with a one-sided angular probe. This map is nonsmooth when
the lowest box vertex changes, especially around a nearly flat foot. A smaller
trust radius alone therefore cannot guarantee a consistent active feature.

R91 completed that report-only audit and passed every predeclared discriminator.
The stable-vertex construction is exactly the same box/floor geometry, yet its
worst prediction error is only `3.874 µm`, versus `992.020 µm` for the scalar
minimum. The worst right-foot frame switches the expected `z` support feature,
and the vertex model selects the same feature as exact FK. This rejects the
radius-only explanation and supports changing the local row representation;
it does not itself prove that the complete coupled constraints are feasible.

R92 implemented that smallest experiment. Only box colliders with contact role
`8` receive eight stable-labelled floor rows; every sphere and nonfoot box keeps
its existing scalar-minimum row. The two foot boxes add `14` rows per frame
(`15414` over `1101` frames), rather than the `107898` extra rows required to
decompose all fourteen boxes. The exact emitted-integer audit still evaluates
the unchanged scalar minimum of all `19` colliders against `-2 µm`.

The clean raw `cmu139` run passes after three relinearizations. Its first large
step remains expensive and ends `solved inaccurate` at the `100000`-iteration
cap, but exact FK is retained and the next two QPs converge without a rejected
oscillation. Final normalized violation is exactly zero in all seven groups.
R93 then reproduces the unchanged V9 identity on all three complete clips and
all 17 exact slices from one clean invocation. The three solves terminate on
exact quantized PASS after `3/2/3` iterations, while all `390` compared arrays
over `30` overlap pairs are identical and no contact point is deleted. This
closes the offline geometry and cross-clip blocker. R92 wall time remains a
report-only optimization target and cannot replace the fresh PhysX safety gate.

## V8/V9 solver identity

V8 starts from the exact V18 complete source clip and freezes every
point-consistent mode inferred from that source before solving. Production
review found that R57 had already removed `11` initially inferred active
points from `cmu139`: frames `628..629` and `724..732`. The first pair overlaps
the selected `@626/@627/@630` discriminator region. R69 used the already
reduced R57 mask and could not expose this because `cmu05` has no such mask
difference. V8 therefore does not inherit the R57 mask and gives the solver no
point-deletion operation. This is a stricter all-clip test, not a changed
tolerance.

V8 has `13` variables per frame: root XYZ and five selected joints on each
leg. Every SQP outer iteration constructs one dimensionless sparse QP with
explicit rows for:

- active normal residual;
- shared-active finite X/Y/Z displacement;
- hybrid-stencil analytic X/Y/Z contact velocity;
- every collider sample against the common ground floor;
- root vertical velocity;
- selected-joint velocity and soft ROM.

The hybrid stencil is forward at contact entry, backward at exit and centered
otherwise, with entry precedence. There is no root closure or joint-velocity
projection after the QP. Internal margins are stricter targets, not changed
requirements: collider `+50 µm`, root velocity reserve `300 µm/s`, joint
velocity factor `999/1000`, normal residual `100 µm`, finite normal/tangent
`20/20 µm`, analytic normal/tangent `300/2000 µm/s`.

The private lab adapter is pinned to NumPy `2.5.2`, SciPy `1.18.0` and OSQP
`1.1.3`. These are replaceable preprocessing dependencies and have no runtime,
physics, corpus-admission or public-schema authority. `optimizer_steps=0`
continues to mean no learned-policy optimizer; the QP is reported separately
as a bounded trajectory constraint solve.

## Decision and remaining gate

The repository retains V8 and adds the V9 profile, complete-clip builder
dispatch, exact-slice path and focused tests. The production path recomputes
final FK, effectors, center of mass and collider facts from emitted integer
root/joint values, and rejects either profile if it changes a frozen contact,
collider, root or joint bound. Clean R93 proves the unchanged all-clip/slice
identity, while R94 disproves its dynamic sufficiency under the frozen fixed-PD
plant. Detailed native evidence, source synthesis and hypotheses are in the
[native-dynamics research decision](humanoid-train4-native-dynamics-research-2026-08-14.md).
The next accepted evidence sequence is:

1. compare hash-bound R47 V7 and R93 V9 exact slices in report-only R95,
   including trajectory derivatives, implied fixed-PD load and contact entry;
2. select at most a one- or two-case fresh discriminator from measured evidence;
3. require that discriminator, then fresh all-17, to have exact-zero required
   safety and zero control regressions before full-corpus construction.

R94 has returned the work to bounded contact/physics research. Full
27-clip V19, visual/exhaustive admission, TRAIN-4 Advance, TRAIN-5 and training
remain forbidden until the ordered gates pass.
