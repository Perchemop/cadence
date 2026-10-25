# Repository radar

Periodic build-vs-adopt pass over public GitHub projects (guide.md §4A). One
section per pass; verdicts are `adopt`, `pilot`, `watch` or `reject`, and a
rejected candidate is not re-proposed unless its reason has changed.

## 2026-10-25 — first pass

Scope: the CPM engine, the Exchange/IFC interchange crates, and release,
publishing and supply-chain infrastructure.

Already covered in this repository (no action, do not re-propose): `cargo-deny`
(licences + advisories), gitleaks, SBOM, dependency review, lockfile freshness,
Dependabot, SHA-pinned actions.

| Candidate | Verdict | Reason |
| --- | --- | --- |
| `axodotdev/cargo-dist` | **adopt** | Declarative targets/installers instead of the hand-maintained release matrix. Adopted incrementally: `dist-workspace.toml` + `dist plan` on every pull request; the release workflow stays hand-written and SHA-pinned (`allow-dirty = ["ci"]`). |
| `obi1kenobi/cargo-semver-checks` | **adopt** | The workspace keeps an explicit version beside every path dependency so the crates stay publishable; nothing enforced that their public APIs only change compatibly. Gate runs against the pull request's base commit. |
| `sourcefrog/cargo-mutants` | **adopt** | The CPM forward/backward passes produce plausible dates even when subtly wrong. Weekly, scoped to `crates/core/src/engine.rs`. |
| `petgraph/petgraph` | **reject** | The hand-written `topological_order` is small, dependency-free and covered by the engine tests; petgraph would add a graph API surface without changing behaviour. Revisit only if the engine needs multi-edge or incremental graphs. |

Re-check date for the radar as a whole: 2027-01-25.
