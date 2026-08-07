# AGENTS.md

Tool-agnostic entry point for coding agents (Codex, Cursor, OpenCode, Copilot, etc.) working **on this repository's source code**. Claude Code should read [CLAUDE.md](CLAUDE.md) as well — it repeats the rules below plus Claude-specific architecture depth.

## Two different kinds of "agent" here — do not confuse them

| | Reads | Purpose |
|---|---|---|
| **Contributor agent** (you, right now) | `AGENTS.md` (this file), `CLAUDE.md` | Modifies the `scitex-cli` Rust source, tests, and skill files themselves |
| **End-user agent** | `skills/*/SKILL.md` (bundled with each Release and installed via `scitex skills install`) | Operates the *built* `scitex` binary on behalf of a lab user — creates orders, checks inventory, etc. |

If you were invoked to help a user "place an order" or "check inventory" against a running Scientex lab, you almost certainly want the end-user skills under `skills/`, not this file. This file is for changing how `scitex-cli` itself is built.

## What this project is

**scitex-cli** — the official Rust CLI client for the Scientex lab management system (primer synthesis + sequencing orders, inventory, lab administration, projects, tasks). It talks to a FastAPI backend at `https://scientex.cn/api/v1` over Feishu OAuth. Full detail: [README.md](README.md) (user-facing) and [CLAUDE.md](CLAUDE.md) (architecture).

## Scientex Multi-Repository Boundary

Scientex is a three-repository project. This repo is the **HTTP-only client**; it does not own backend state.

| Concern | Source of truth |
|---------|-----------------|
| HTTP API, OpenAPI, request/response schemas | `xuyuan-hub/scientex` backend |
| Database models and Alembic migrations | `xuyuan-hub/scientex` backend |
| Task/part/assignment status machines | `xuyuan-hub/scientex` backend |
| Queue names and Worker payload/output protocol | `xuyuan-hub/scientex` backend + `worker/` deployment assets |
| CLI commands, DTOs, output formatting, Agent skills | this repository |

Rules when changing this repo:

- No direct database access, SQLModel assumptions, Procrastinate queue calls, Worker filesystem access, or deployment logic.
- Treat the backend's OpenAPI spec as the contract. When fields, paths, enums, or schemas change on the backend, update CLI DTOs (`src/types.rs`), command handlers, tests, and `skills/*/SKILL.md` together.
- Use only `SCIENTEX_*` environment variables in code, docs, and skills. Do not introduce new `BIOLAB_*` names (legacy compatibility only, and only if explicitly documented).
- Do not invent task/assignment status values, queue names, or Worker `output_data` fields — match backend enums exactly; reject or omit unsupported values client-side.
- Cross-repo changes are sequenced: backend contract/migration first, then CLI adaptation + tests, then CLI release. Record the compatible backend commit/OpenAPI baseline in change notes when a CLI change depends on a backend change.
- Not every backend endpoint has CLI coverage — see `docs/命令参考.md` for what's actually implemented. Don't assume a command exists; run `scitex <group> --help` or grep `src/commands/` to confirm before documenting or relying on it.

## Common Commands

```bash
cargo build --release        # build
cargo test                   # must pass before every submission (unit + OpenAPI contract tests)
cargo fmt --check             # must pass before every submission
cargo test -q -- --list       # get the current, accurate test count — don't trust a hardcoded number in docs
./target/release/scitex --help
```

## Development Workflow

All feature work follows a doc-driven process:

1. **Write a plan first** — before coding, create a plan file under `docs/feishu/YYYY/MM/` (see naming below) that confirms requirements, lists affected files, and includes a `[ ]` TODO checklist.
2. **Implement and check off** — update `[ ]` to `[x]` with a commit reference as work lands.
3. **Update on interruption** — if a plan stalls or is abandoned, record *why* in the TODO list rather than deleting it.
4. **Keep the index current** — the TODO list at the end of each plan file is the single source of truth for implementation status. When you pick up someone else's plan, re-verify the "仍未完成 / not yet done" items against the actual code before trusting them — they go stale (see the worked example in `docs/feishu/2026/07/`, where "fmt failing" and "upload-field missing" were both already fixed by the time of a later check).

Non-plan docs (installation guides, command references) live directly under `docs/` and are committed to git; they don't follow the dated-plan naming pattern.

## Document Organization / Feishu Sync

All plan documents are synced with Feishu Drive.

**Root folder:** https://v1md2ogd1v3.feishu.cn/drive/folder/SPNcfvJX9ldQVjdAuRGcQJZknXc

Local mirror: `docs/feishu/YYYY/MM/` — gitignored, organized by year/month of creation.

```
docs/
├── 安装指南.md          # git-tracked — user installation guide
├── 命令参考.md          # git-tracked — full command reference, keep in sync with actual CLI
└── feishu/              # Feishu Drive mirror, gitignored
    └── 2026/
        ├── 05/          # 16 plans: 3 CLI + 13 backend
        ├── 06/          # 1 plan: experiment-to-execution feedback flow
        └── 07/          # OpenAPI drift remediation (client/backend mismatch audit + fix plan)
```

### Naming

Name plan files by the year/month of creation:

- `ScientexCli-YYYY-MM-DD-Name.md` — CLI-side plans (current project name; older plans in `docs/feishu/2026/05-06/` may still use the legacy `BiolabCli-` prefix from before the rename to `scitex-cli` — don't "fix" old filenames, just use the current prefix going forward)
- `Scientex-YYYY-MM-DD-Name.md` — backend-side plans (legacy prefix: `Biolab-`)

## Where to Go Next

- [README.md](README.md) — install instructions, AI Agent Skills table, security warnings, command overview
- [CLAUDE.md](CLAUDE.md) — full source layout, architectural patterns, CI/contract-test detail
- `docs/命令参考.md` — the authoritative, up-to-date command list
- `skills/scitex-shared/SKILL.md` — the shared baseline every end-user domain skill points back to (auth, credential chain, OpenAPI schema rules)
