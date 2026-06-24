# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with this repository.

## Project Overview

**scitex-cli** — A Rust CLI client for the Scientex lab management system (primer synthesis + sequencing orders, inventory, lab administration).

The system communicates with a FastAPI backend at `http://8.136.56.203/api/v1` using Feishu OAuth for authentication.

## Scientex Multi-Repository Boundary

This repository is the **HTTP-only client** for the larger Scientex project. The governance source is `xuyuan-hub/scientex/docs/Scientex-2026-06-24-三仓库同步治理方案.md`.

Authoritative ownership:

| Concern | Source of truth |
|---------|-----------------|
| HTTP API, OpenAPI, request/response schemas | `xuyuan-hub/scientex` backend |
| Database models and Alembic migrations | `xuyuan-hub/scientex` backend |
| Task/part/assignment status machines | `xuyuan-hub/scientex` backend |
| Queue names and Worker payload/output protocol | `xuyuan-hub/scientex` backend + `worker/` deployment assets |
| CLI commands, DTOs, output formatting, Agent skills | this `scitex-cli` repository |

Client rules:

- Do not add direct database access, SQLModel assumptions, Procrastinate queue calls, Worker filesystem access, or deployment logic to this repository.
- Treat Web OpenAPI as the contract. When API fields, paths, enums, task payloads, or output schemas change, update CLI DTOs, command handlers, tests, and skills from the Web contract.
- Use only `SCIENTEX_*` environment variables in code and skills. Do not introduce new `BIOLAB_*` names except as explicitly documented legacy compatibility.
- Do not invent task status, assignment status, queue, or Worker output fields in the CLI. Match backend enums and output schemas exactly; unsupported values should be rejected or omitted client-side.
- For compute tasks, the CLI creates/reads tasks through HTTP only. It must not assume how Worker executes jobs beyond the documented `output_data` contract returned by the API.
- Cross-repository changes must be sequenced as: Web contract/migration first, then CLI adaptation and tests, then CLI release. Record the compatible Web commit/OpenAPI baseline in the change notes when a CLI change depends on a backend change.

## Common Commands

```bash
# Build
cargo build --release

# Run
./target/release/scitex <cmd>    # release binary

# Help
scitex --help
scitex orders --help
scitex inventory --help

# Install AI agent skills for this project
scitex skills install
scitex skills check -f json

# Login (Feishu OAuth)
scitex login
scitex status
scitex logout

# Output in JSON (for machine parsing)
scitex me -f json
scitex orders list -f json
```

## Architecture

### Source Layout

```
Cargo.toml          # Rust project, binary: scitex, lib: scitex_cli
src/
├── main.rs         # Thin CLI entry — imports from lib crate, clap router
├── lib.rs          # ALL mod declarations here; binary imports via scitex_cli::...
├── config.rs       # Token management (env → keyring → optional token file), base URL
├── client.rs       # ScientexClient factory, re-exports ScientexError
├── errors.rs       # ScientexError enum (thiserror)
├── types.rs        # Serde request/response structs + custom deserializers
├── auth.rs         # Feishu OAuth login (custom CLI poll flow)
├── output.rs       # Formatting: JSON (--format json) vs colored text
├── http.rs         # Raw HTTP methods: get/post/patch/put/delete/upload/download
├── api_response.rs # Response envelope unwrapping (data/items/results)
├── commands/       # clap subcommand args + run() handlers
│   ├── users.rs    # me / update-me / change-password
│   ├── orders.rs   # list / get / create / update / resend / download / upload
│   ├── templates.rs# CRUD + default for order-info templates
│   ├── inventory.rs# list / get / stats / checkin / checkout / locations
│   ├── lab.rs      # lab info / members / invite / join / approval rules
│   └── skills.rs   # AI agent skill installation and check
└── services/       # impl ScientexClient blocks, domain-specific API methods
    ├── orders.rs   # Order API path builders + unit tests
    ├── users.rs    # User API path builders + unit tests
    ├── templates.rs# Template API path builders + unit tests
    ├── inventory.rs# Stock/location API path builders + unit tests
    ├── lab.rs      # Lab/member/approval API path builders + unit tests
    └── helpers.rs  # Shared: empty_body(), single_field_body(), url_encode()
```

### Key Patterns

- **Credential chain**: `SCIENTEX_TOKEN` env var → OS keyring (`scitex-cli`) → optional `~/.scitex_token` file → OAuth poll
- **Token storage**: OS keyring by default, valid 8 days; plaintext file fallback requires explicit `SCIENTEX_INSECURE_TOKEN_FILE=1`
- **Module ownership**: All `mod` declarations live in `lib.rs`; `main.rs` imports from `scitex_cli::...` (the library crate). This avoids the binary crate's `crate::` resolving differently from the library's `crate::`
- **HTTP client**: `ScientexHttp` (in `http.rs`) wraps reqwest with Bearer token injection; `api_response.rs` provides `extract_array`/`extract_object`/`envelope_data` for response unwrapping
- **Domain services**: `impl ScientexClient` blocks in `services/*.rs` call `self.http.get/post/...` then `extract_array`/`extract_object` — all methods unwrap the `{ "data": ... }` envelope consistently
- **Shared helpers**: `services/helpers.rs` — `empty_body()`, `single_field_body()`, `url_encode()`
- **Custom deserializers**: `string_or_f64` / `opt_string_or_f64` in `types.rs` — backend sometimes returns numeric fields as JSON strings
- **Errors**: `ScientexError` in `src/errors.rs` (not `error.rs` — avoids collision with `std::error`)
- **Output modes**: `-f json` for machine-readable, default text for human (colored status badges)
- **Agent skills**: `scitex skills install` delegates to `npx skills add xuyuan-hub/scitex-cli`, so supported agents refresh their own skill indexes.
- **Tests**: `cargo test` must pass before every submission — CI gate enforces this (130 unit tests + 21 OpenAPI contract tests; see `tests/openapi_contract.rs`)

### API Base URL

Default: `http://8.136.56.203/api/v1` — overrideable via `SCIENTEX_BASE_URL` env var.

### Agent Skills

AI agent skills (SKILL.md files) live under `skills/` — this directory is tracked in git.
The `.agents/skills/` directory is **gitignored** and contains symlinks managed by
the skills installer (`scitex skills install` / `npx skills add`). When adding or
editing a skill, always work in `skills/scitex-<name>/SKILL.md`, never in `.agents/`.

## Business Domain

### Order Status Machine
```
pending → ordered → received → stored
(待下单)  (已下单)  (已收货)   (已入库)
```

### Order Types
| Type | Supplier(s) |
|------|-------------|
| `primer_synthesis` | `sangon` (生工) / `biosune` (铂尚) |
| `sequencing` | `biosune` |

### Lab Permission Model
Five workflow roles: `pi` > `procurement` > `finance` > `warehouse` > `member`

### Reference Docs
Detailed API schemas are bundled with the domain skills installed by the standard `skills` installer:
- `orders.md` — Order schemas, status machine, supplier differences
- `inventory.md` — Stock/checkin/checkout schemas
- `templates.md` — Template fields for order defaults
- `lab.md` — Lab CRUD, member management, approval rules
- `users.md` — User info, permission model, signup

## Development Workflow

All feature work follows a doc-driven process using plan files under `docs/` with `ScientexCli-YYYY-MM-DD-Name.md` naming (Name in Chinese). Non-plan docs (installation guides, command references) do not follow this pattern.

### Document Organization

All plans (CLI and server-side) live under `docs/feishu/YYYY/MM/`, organized by year and month. This directory is a local mirror synced from Feishu Drive and is gitignored. Non-plan docs (installation guides, command references) remain under `docs/` directly and are committed to git.

### Process

1. **Write a plan first** — Before coding, create a plan under `docs/feishu/YYYY/MM/` that confirms requirements, lists affected files, and includes a TODO list with `[ ]` checkboxes for each task.
2. **Implement and check off** — As tasks are completed, update `[ ]` to `[x]` with the commit reference.
3. **Update on interruption** — If work is interrupted or a plan is abandoned, update the TODO list to explain **why** it stopped and **why** the plan was abandoned (if applicable).
4. **Keep the index current** — Every plan file ends with a TODO list section. This is the single source of truth for implementation status.

### Feishu Sync

All plans are stored in Feishu Drive. The `docs/feishu/` directory is a local mirror synced from Feishu and excluded from git.

Feishu root folder: https://v1md2ogd1v3.feishu.cn/drive/folder/SPNcfvJX9ldQVjdAuRGcQJZknXc
Local mirror: `docs/feishu/YYYY/MM/`

Existing plans (under `docs/feishu/2026/`):
- `05/` — 16 plans: 3 CLI (架构、CLI命令、项目种子API) + 13 服务端方案（库存、用户、订单、实验记录等）
- `06/` — 1 plan: 实验方案到执行反馈流程方案

Other docs:
- `docs/2026-05-21-安装指南.md` — User installation guide
- `docs/2026-05-27-命令参考.md` — Full command reference

## CI

`.github/workflows/release.yml` builds for 4 platforms on push:
- Linux (x86_64 via musl)
- Windows (x86_64)
- macOS (x86_64)
- macOS (arm64)

`cargo test` runs before build — all tests must pass.

## OpenAPI Contract Tests

`tests/openapi_contract.rs` checks that CLI enum values and API paths stay aligned with the backend's OpenAPI spec. The fixture `tests/fixtures/openapi.json` is a pinned snapshot of `http://8.136.56.203/api/v1/openapi.json`.

When the backend changes its API, refresh the fixture:

```bash
curl http://8.136.56.203/api/v1/openapi.json -o tests/fixtures/openapi.json
# or: cargo test --test openapi_contract refresh_openapi_fixture -- --ignored
```

Then run `cargo test` — if a CLI enum value or path no longer matches the backend, the relevant test will fail.

Tagged pushes (e.g. `v0.1.0`) auto-create GitHub Releases with binaries.
