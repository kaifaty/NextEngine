# NSR3-B4E2D7R20R57 v5 source-manifest evidence

Status: `PASS / V5_HOLDOUT_MANIFEST_FROZEN`.

## Identity

- semantic `2e5b935306919c0797a4f3b6f77edd8dfc4dcead3281ed274fc5b7b885523b9a`;
- six immutable cases, 637 fluid particles total;
- fluid counts `100/108/96/105/108/120`;
- boundary counts `548/620/576/588/592/600`;
- all six source roots and all six geometry roots are pairwise distinct.

Source roots in frozen order:

```text
387154ed7deeeabec32507253444a43164969dd404c8b05a85247432c8b285f7
71fd973b6b829c5a85d9e4dc5b7f3c6026d08df29857fd5f8166fce33940e251
86669c6efe90f65972690ec6c6366a323bbadd913596be73f8b7226ba0536606
ae338cbb8ecbf61228bb0bfee092aa1acababc889136815b95f8412959ce185f
23c8db3e8abdc451f48f58a9d223b404e23ebc86bb91152b2bd5d9099ededca3
e2e84cb2fa1cec6d56120c8ee989db0c70dd97e14a4875d345cd1bda67a65e33
```

## Blindness and regression

Operator, projection, preflight, solver and timing flags are all false. Two
independent executions are byte-identical. Parent R56 retains semantic
`daf7c2ce...20b7`.

No excitation claim exists yet. Sources are now immutable even if a future
preflight rejects them. Only an input-only operator/scaling/excitation preflight
is authorized next.
