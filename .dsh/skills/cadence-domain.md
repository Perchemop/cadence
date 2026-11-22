---
name: cadence-domain
description: Cadence-specific domain knowledge — BMAD methodology, project scaffolding, agent orchestration, craft-launcher integration
whenToUse: Use when working on Cadence codebase — implementing BMAD workflows, project scaffolding, agent teams, or craft-launcher integration
disable-model-invocation: false
user-invocable: true
---

# Cadence Domain Skill

## What is Cadence

Cadence is a **BMAD (Build, Measure, Analyze, Decide) methodology orchestrator** for AI-assisted software development. It provides:

1. **Project scaffolding** — creates new projects from templates with proper structure
2. **Agent orchestration** — manages multi-agent teams for complex workflows
3. **Craft-launcher integration** — launches and manages development environments
4. **Workflow execution** — runs BMAD cycles through structured phases

## Architecture (from ROADMAP.md and IMPLEMENTATION_PLAN.md)

### Crate Structure
```
cadence/
├── crates/
│   ├── cadence-core/       # Headless BMAD logic, workflow engine
│   ├── cadence-commands/   # CLI command definitions
│   ├── cadence-persistence/# Project state, workflow history
│   ├── cadence-ui/         # egui/eframe desktop UI
│   └── cadence-app/        # Wiring, eframe setup
├── apps/
│   └── cadence-desktop/    # Desktop entry point
```

### Key Domains
- **BMAD Cycle** — Build → Measure → Analyze → Decide phases
- **Project Templates** — Rust/egui, Node/React, Python/FastAPI, etc.
- **Agent Teams** — Orchestrator + specialised teammates (Coder, Reviewer, Designer, etc.)
- **Workflow State** — Persistent workflow execution with checkpoints
- **Craft Integration** — Launches dev environments, manages processes

## Current Implementation Status

Check `ROADMAP.md`, `PROGRESS.md`, `REQUIREMENTS_TRACEABILITY.md` for current state.

### Implemented (as of last update)
- Core crate structure with headless tests
- CI pipeline (fmt, clippy, test, build)
- Basic BMAD cycle types
- Project scaffolding commands
- Desktop UI shell with egui/eframe

### In Progress
- Multi-agent team orchestration
- Craft-launcher integration
- Workflow persistence and resume
- BMAD phase execution engine

## Domain-Specific Rules

### BMAD Workflow Implementation
- Each phase (Build, Measure, Analyze, Decide) = separate command + test
- Phase transitions must be explicit and auditable
- State persisted after each phase for crash recovery
- Human gate required before Decide phase actions

### Agent Team Management
- Teams follow guide §5.7 — only for Case 4 (Parallel Multi-Session Work)
- Team definitions in `.dsh/agents/*.yaml`
- Shared task list in `docs/tickets/`
- Inter-agent logs in `docs/team-log/`

### Craft-Launcher Integration
- Launches isolated dev environments per project
- Manages process lifecycle (start, stop, restart, logs)
- Provides health checks and resource monitoring
- Communicates via local RPC/HTTP

## Testing Requirements

- **Headless workflow tests** for every BMAD phase (guide §17)
- Core crate tests run on CI without display
- Integration tests for craft-launcher communication
- UI tests via `egui_kittest` + AccessKit where harness exists

## Configuration

Key config files:
- `Cargo.toml` — workspace and crate definitions
- `rust-toolchain.toml` — pinned toolchain (1.83.0)
- `deny.toml` — licence/advisory/bans config
- `.ci-required.txt` — authoritative CI checks
- `.dsh/settings.local.json` — DSH feature flags (local only)

## Integration Points

| System | Interface |
|--------|-----------|
| GitHub Actions | CI via `ci.sh` |
| Craft-launcher | Local RPC/HTTP API |
| DSH | Agent presets, skills, workflow tool |
| egui/eframe | Desktop UI rendering |
| SQLite | Persistence (cadence-persistence) |

## References

- `docs/architecture.md` — system architecture
- `docs/inspiration/` — BMAD methodology references (public sources only)
- `docs/capability-inventory.md` — all features with acceptance criteria
- `docs/design/brand.md` — visual identity