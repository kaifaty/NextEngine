# Physical sound V32 M0 — physics-locked residual research

| Field | Value |
| --- | --- |
| Date | `2026-09-02` |
| Status | `BOUNDED_RESEARCH_COMPLETE / PHYSICS_LOCKED_RESIDUAL_SELECTED` |
| Trigger | V24–V28 variants moved execution symptoms but V28 R2 still failed decay, remesh and all physical counterfactual gates |
| Scope | Synthetic known-truth representation only; no real signal, material quality, admission or runtime claim |

## Falsifiable problem

V28 proved that the closed `23,142`-parameter M0c field can fit frequency,
contact gain and short-window spectrum while behaving almost invariantly to
Young's modulus, density, thickness and scale. It also missed remesh gain by
three orders of magnitude and made decay worse than ridge.

The next experiment must explain why those failures become structurally
impossible. A nearby seed, capacity, loss, horizon or end-to-end waveform model
does not satisfy that requirement.

## Competing hypotheses

| Hypothesis | Evidence for | Evidence against | Decision |
| --- | --- | --- | --- |
| H1: another network should predict the complete modal record | M0c improved several output-space metrics | M0c's frequency head ignored exact physical axes; nearby variants are closed by the spent R2 values | `REJECT` |
| H2: move directly to waveform/codec generation | Could represent residual timbre not present in ten modes | R2's hardest failures are causality and remesh, not only waveform fidelity; an opaque decoder weakens those gates | `NOT_SELECTED` |
| H3: retain analytic frequencies and learn only bounded unknown corrections | P1 already passes physical ratios, force scaling, nodes and remesh exactly; a residual can add capacity without owning those invariants | Synthetic success cannot establish real timbre or validator quality | `SELECTED_FOR_M0` |
| H4: keep P1 unchanged and skip ML | Lowest complexity and deterministic | Cannot test whether published real evidence can later calibrate damping/radiation/contact variation beyond fixed presets | `CONTROL` |

## Primary-source evidence

- [DiffSound (SIGGRAPH 2024)](https://arxiv.org/abs/2409.13486) retains a
  physics-based modal-analysis and differentiable synthesis pipeline while
  optimizing physical, shape and impact parameters. It supports differentiable
  modal rendering, not replacing every physical relation with a waveform net.
- [NeuralSound (TOG 2022)](https://arxiv.org/abs/2108.07425) explicitly allows
  learned vibration/radiation modules to be combined independently with
  numerical solvers. Its mixed vibration solver uses neural output as a warm
  start for a convergent numerical method rather than treating approximate
  neural eigenvalues as unquestioned authority.
- [Learning Neural Constitutive Laws (ICML 2023)](https://proceedings.mlr.press/v202/ma23a.html)
  argues that known governing PDEs should remain explicit while the unknown
  constitutive part is learned inside the simulator. This directly supports a
  small learned correction around P1 rather than relearning its frequency law.
- [Physics-informed architectures and constraints (L4DC 2022)](https://proceedings.mlr.press/v168/djeumou22a.html)
  represents dynamics as a composition of known and unknown functions and
  constrains network outputs/internal state. It supports enforcing bounds and
  dependency isolation in topology, not hoping a penalty discovers them.

These sources establish relevant prior art, not evidence that the proposed
NextEngine family will pass. DiffSound and NeuralSound solve broader and more
expensive geometry/radiation problems; M0 deliberately tests a much smaller
CPU-only residual representation.

## Selected representation

P1 remains the sole owner of:

```text
modal frequency and ordering
analytic contact and pickup mode shapes
impulse scaling
support formula
remesh-invariant common-vertex values
canonical damped-modal rendering
```

The candidate receives a frozen modal record and may predict only:

```text
log_decay_multiplier(mode, material, support)
global_log_gain_delta(mode, geometry, material)
contact_log_participation_multiplier(mode, contact, support)
```

All three outputs are bounded by construction. Decay and magnitude corrections
are positive exponentials. Contact correction multiplies P1 participation, so
an analytic node remains exactly zero and its sign cannot flip. Impulse remains
outside the model, so force scaling stays exact. Frequency is not an output,
loss target or trainable path.

## Smallest discriminator

Build a deterministic synthetic correction oracle over a group-disjoint grid
of P1 plates/beams, three disclosed synthetic materials, eight geometry cells
and eight contacts. Train one `1,491`-parameter three-head MLP once and compare
it with unchanged P1/identity, ridge and nearest-retrieval controls.

The family passes only if it beats every control on unopened geometry groups
while the structural gates remain exact. Development failure closes before
method holdout. A method-holdout miss closes the family without retry.

## Consequences and remaining uncertainty

- M0 can test representation capacity and automatic selection mechanics without
  real signal or per-sound listening.
- A pass authorizes only a later real-data model protocol after independent
  source roles exist. It does not prove Metal, Glass or Wood quality.
- Real damping/radiation may require features absent from this residual family.
  That question remains sealed until source power and validator qualification.
- A failure should distinguish branch capacity from optimization/resource
  failure; it cannot select another width, seed, teacher coefficient or bound.

Smallest next action: freeze the exact M0 corpus, oracle, topology, controls,
losses, gates, resource envelope and one-shot access order before implementation
or candidate values exist.
