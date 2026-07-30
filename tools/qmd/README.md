# QMD documentation search

NextEngine uses [QMD](https://github.com/tobi/qmd) as a local discovery layer
over repository documentation. QMD does not define product truth: agents must
retrieve full source documents and apply the precedence rules in `AGENTS.md`.

## Bootstrap on a new workstation

QMD 2.5.3 requires Node.js 22 or newer:

```text
npm install --global @tobilu/qmd@2.5.3
node tools/qmd/setup.mjs
node tools/qmd/daemon.mjs start
node tools/qmd/daemon.mjs status
```

The setup creates an ignored project-local `.qmd/` index, indexes `docs/**/*.md`,
downloads the configured GGUF models, generates embeddings and runs diagnostics.
Run the setup again after changing the collection or model configuration.
Model downloads are performed lazily by `qmd embed`; the setup intentionally
does not run `qmd pull`, because `pull` replaces cached model files and is not an
idempotent refresh operation on Windows.

The bootstrap is complete only when all of the following are true:

1. `qmd doctor` reports valid models, a fresh vector index and the intended
   CPU/GPU backend. Vulkan fallback on an NVIDIA Windows host is valid when a
   compatible CUDA runtime is unavailable.
2. `node tools/qmd/daemon.mjs status` reports `status: "ok"` and
   `nextengine-docs` rooted at this checkout's `docs` directory.
3. After restarting Codex or opening a new task, `/mcp` lists `qmd`.
4. One initial structured query is allowed to warm the models. A second query
   in the same daemon lifetime should be materially faster.

The daemon survives Codex task restarts but not a workstation reboot. Run
`node tools/qmd/daemon.mjs start` once after reboot. The command is idempotent.

## Models

- Embedding: `Qwen3-Embedding-0.6B-Q8_0`, selected for multilingual retrieval,
  including Russian and English.
- Reranking: `Qwen3-Reranker-0.6B-Q8_0`, also multilingual.
- Query expansion: QMD's `qmd-query-expansion-1.7B`. Agents should normally
  author structured `intent`/`lex`/`vec` queries themselves rather than rely on
  automatic expansion.

Models are cached outside the repository under the QMD user cache. The generated
`.qmd/` directory is a reconstructible local cache and must not be committed.

## Local model stability

The project launchers default `QMD_EMBED_PARALLELISM` to `1` for embedding and
reranking as a conservative cross-host stability guard. QMD allows parallel
contexts on Vulkan, but this Windows/RTX 3080 host previously stalled with
automatic context counts and a two-context benchmark did not materially improve
cold-start latency. The setting applies to setup, MCP and the benchmark
launcher.

On a different host, raise the value only after `qmd doctor` confirms the
backend and a measured comparison shows a benefit. Set the environment variable
before starting the daemon; the project launcher preserves an explicit
per-host override:

```text
# PowerShell
$env:QMD_EMBED_PARALLELISM = "2"
node tools/qmd/daemon.mjs start

# POSIX shell
QMD_EMBED_PARALLELISM=2 node tools/qmd/daemon.mjs start
```

Stop and restart an existing daemon before changing the value. Values above `4`
are not recommended without VRAM measurement.

Model loading and ranking-context creation dominate a cold query. For repeated
queries, prefer QMD's long-lived HTTP daemon so models remain resident in VRAM:

```text
node tools/qmd/daemon.mjs start
node tools/qmd/daemon.mjs status
node tools/qmd/daemon.mjs stop
```

The launcher pins the project root, models and port, and applies the stable
single-context default unless the host supplies an explicit override. Codex
connects to `http://127.0.0.1:8181/mcp`. The first model-backed query still
warms the daemon; later queries reuse the loaded models across Codex tasks.

Direct `qmd` commands do not pass through the project launchers. When diagnosing
the same Windows/Vulkan host from PowerShell, set the equivalent environment
override for that shell:

```text
$env:QMD_EMBED_PARALLELISM = "1"
qmd doctor
```

For search degradation, keep semantic retrieval when possible:

1. Retry a structured hybrid query without reranking (`qmd query --no-rerank`,
   or MCP `rerank: false`).
2. Use a small reranking pool (`-C 12`, or MCP `candidateLimit: 12`) when
   reranking is necessary.
3. Fall back to `qmd search` only when embedding/vector search is also
   unavailable or too slow.

Use this diagnostic order when reranking is slow:

1. If every query takes tens of seconds, verify that Codex is connected to the
   HTTP URL in `.codex/config.toml`. Standalone `qmd query` and stdio processes
   repeatedly pay cold model-loading and are not representative benchmarks.
2. Check `node tools/qmd/daemon.mjs status`. If port `8181` belongs to another
   checkout, the launcher rejects it; stop that daemon and start this one.
3. Keep `candidateLimit` around `8`-`12`. Lowering it does not remove cold model
   loading, but it bounds warm reranker work.
4. If the model still stalls or exhausts VRAM, return to
   `QMD_EMBED_PARALLELISM=1`, restart the daemon, and use `rerank: false` until
   the backend is healthy.
5. Treat a lower-quantization reranker or CUDA installation as a measured
   host-specific optimization, not a bootstrap requirement.

## Codex integration

Project-scoped MCP configuration lives in `.codex/config.toml`. It uses QMD's
long-lived local HTTP daemon so model loading is not repeated for every Codex
task. Start it with `node tools/qmd/daemon.mjs start`. The vendored QMD skill
lives in `.agents/skills/qmd`; a new Codex task or application restart is
required after first installation or an MCP configuration change. Verify that
`qmd` appears in `/mcp` after restarting. Codex loads the project MCP layer only
for a trusted project.

Useful checks:

```text
qmd doctor
qmd status
node tools/qmd/daemon.mjs status
codex mcp list
node tools/qmd/bench.mjs
```

Use the benchmark launcher instead of calling `qmd bench` directly. QMD 2.5.3's
benchmark API can fall back to its default embedding model even when the local
index selects Qwen3; the launcher explicitly pins all three project models.

For routine documentation changes:

```text
qmd update
qmd embed -c nextengine-docs
```

Do not configure a QMD collection update command that runs `git pull`: index
maintenance must never mutate or hide the developer's working tree.
