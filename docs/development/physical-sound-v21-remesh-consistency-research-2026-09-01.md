# Physical sound V21 — remesh-consistency research decision

| Field | Value |
| --- | --- |
| Date | `2026-09-01` |
| Status | `COMPLETE / F1_DIRECTION_SELECTED / VALUES_1801_2101_SEALED` |
| Trigger | [V20 I1 repeat-exact reject](physical-sound-v20-i1-frozen-integration-result-2026-09-01.md) |
| Scope | Select the smallest falsifiable successor family; do not select thresholds or hyperparameters from I1 |
| Product effect | None |

## Falsifiable problem

F0's learned pointwise prior plus graph-Laplacian residual extension passes
absolute gain, gradient, controls, acoustic endpoints and direct canonical
primary/twin probe disagreement. One fresh Wood/Bowl pair nevertheless exceeds
the frozen relative primary/twin gain-metric drift gate.

The successor question is not whether a larger generic network can lower test
error. It is whether the contact-gain operator can represent one continuous
surface function whose quality remains stable when the same physical object is
sampled by a different valid mesh.

## Competing hypotheses

| Hypothesis | Evidence for | Evidence against | Discriminator |
| --- | --- | --- | --- |
| H1: only the old metric is unstable | Direct probe disagreement passes and the primary/twin NRMSE gap is about `0.0218` | The metric and `0.10` gate were frozen before I1 and cannot be waived after opening it | Keep the old gate on fresh data and add symmetric absolute diagnostics only |
| H2: graph-harmonic residual is discretization-sensitive | F0 residual depends on mesh edges, lengths, context vertices and a discrete Laplacian solve | Eleven of twelve I1 pairs pass, so the defect is narrow rather than universal | Compare exact F0 with a continuous-coordinate residual on fresh paired development groups |
| H3: the learned prior is insufficiently stable | A stronger prior reduces the residual that the mesh solver must transport | Absolute F0 gain/gradient and every topology gate already pass | Pair-aware prior ablation with the same residual and fixed capacity ceiling |
| H4: a large geometry neural operator is required now | Neural-operator literature provides discretization-convergent constructions | Current truth is eight smooth fields on known two-dimensional topology charts with only 24 train groups; a large GINO adds cost and confounds before simpler controls fail | Escalate only if all preregistered small continuous operators reject on fresh development |

## External evidence and limits

- [GINO](https://arxiv.org/abs/2309.00583) defines graph-kernel integration
  with quadrature weights and reports discretization convergence across
  arbitrary geometry meshes. It also warns that fixed nearest-neighbour graph
  updates can become discretization-dependent. This supports giving F1 a
  continuum integral/evaluation meaning; it does not transfer a threshold or
  prove that the large GINO architecture is appropriate here.
- [MeshGraphNets](https://arxiv.org/abs/2010.03409) demonstrates learned
  mesh-based dynamics across adaptive discretizations. It supports paired
  resolution evaluation, but its rollout task and scale are different from a
  static modal-gain field.
- [Beyond Regular Grids](https://proceedings.mlr.press/v235/lingsch24a.html)
  evaluates spectral operators directly on non-equispaced point sets. It
  motivates a small topology-native spectral residual in continuous UV before
  a large latent-grid operator.
- [Can neural operators always be continuously discretized?](https://openreview.net/pdf?id=cyJxphdw3B)
  shows that discretization invariance is not automatic merely because a model
  is called a neural operator. F1 must prove paired consistency empirically and
  cannot claim it from architecture alone.

## Selected smallest experiment

V21 will preregister a bounded `ContinuousResidualOperatorV1` tournament:

1. retain the exact B0 global scaffold and C0 coverage/fallback policy;
2. learn a small topology/material/support-conditioned continuous prior under
   the existing parameter-byte ceiling;
3. fit observed residual coefficients in a fixed topology-native UV basis and
   evaluate that function directly at arbitrary query and canonical-probe
   coordinates, without using mesh adjacency in the residual representation;
4. train/evaluate primary-remesh pairs explicitly and make the old relative
   metric-drift plus direct probe disagreement independently blocking;
5. compare against exact frozen F0, prior-only, graph harmonic and fixed-kernel
   residual controls under one deterministic development ranking;
6. permit only a predeclared tiny grid of basis order, regularization and
   pair-consistency weights; choose once on development, then freeze before
   one-shot test.

The continuous residual is the primary candidate because it changes the layer
causally implicated by I1 while preserving the successful decomposition. A
pair-aware F0-prior ablation may determine whether prior quality alone explains
the drift. A GINO/GNO family is `ESCALATION_ONLY`, not part of F1, unless the
small continuous family closes on fresh development evidence.

## Data and anti-leakage decision

| Role | Physical groups | Values |
| --- | ---: | --- |
| train | `1801…1824` plus fixed remesh partners | Open only after F1 metadata/protocol freeze |
| development | `1901…1912` plus partners | May select one preregistered candidate once |
| one-shot F1 test | `2001…2012` plus partners | Sealed until implementation is committed |
| one-shot reintegration | `2101…2112` plus partners | Sealed until F1 test pass and integration protocol freeze |

I1 identities `1701…1712` are attribution-only and cannot rank candidates,
weights, basis order, regularization, context, threshold or contact. A rejected
F1 test closes the family; it cannot select an F1 retry. Any F2 must use new
metadata bands frozen before values.

## Decision and reconsideration

Proceed with protocol correction P0d, then freeze F1 metadata, candidate family,
development selection rule and unchanged test gates before value generation.
Reconsider a larger neural operator only if the small continuous family and
pair-aware ablation both fail fresh development while exact controls prove the
corpus and evaluator are functioning. No outcome authorizes real-data access,
runtime inference or fallback removal.
