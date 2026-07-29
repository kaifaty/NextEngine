# QMD documentation search

NextEngine uses [QMD](https://github.com/tobi/qmd) as a local discovery layer
over repository documentation. QMD does not define product truth: agents must
retrieve full source documents and apply the precedence rules in `AGENTS.md`.

## One-time setup

QMD 2.5.3 requires Node.js 22 or newer:

```text
npm install --global @tobilu/qmd@2.5.3
node tools/qmd/setup.mjs
```

The setup creates an ignored project-local `.qmd/` index, indexes `docs/**/*.md`,
downloads the configured GGUF models, generates embeddings and runs diagnostics.
Run the setup again after changing the collection or model configuration.
Model downloads are performed lazily by `qmd embed`; the setup intentionally
does not run `qmd pull`, because `pull` replaces cached model files and is not an
idempotent refresh operation on Windows.

## Models

- Embedding: `Qwen3-Embedding-0.6B-Q8_0`, selected for multilingual retrieval,
  including Russian and English.
- Reranking: `Qwen3-Reranker-0.6B-Q8_0`, also multilingual.
- Query expansion: QMD's `qmd-query-expansion-1.7B`. Agents should normally
  author structured `intent`/`lex`/`vec` queries themselves rather than rely on
  automatic expansion.

Models are cached outside the repository under the QMD user cache. The generated
`.qmd/` directory is a reconstructible local cache and must not be committed.

## Codex integration

Project-scoped MCP configuration lives in `.codex/config.toml`. The
`tools/qmd/mcp.mjs` launcher pins the MCP process to this repository so the
project-local index is found even when the client launches the server from
another working directory. The vendored QMD skill lives in
`.agents/skills/qmd`; a new Codex task or application restart is required after
first installation. Codex loads the project MCP layer only for a trusted
project.

Useful checks:

```text
qmd doctor
qmd status
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
