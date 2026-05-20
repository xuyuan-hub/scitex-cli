# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with this repository.

## Project Overview

**biolab-cli** — A Rust CLI client for the Biolab lab management system (primer synthesis + sequencing orders, inventory, lab administration). Replaces the Python scripts embedded in `.claude/skills/biolab-api/scripts/`.

The system communicates with a FastAPI backend at `http://8.136.56.203/api/v1` using Feishu OAuth for authentication.

## Common Commands

```bash
# Build
cargo build --release

# Run
./target/release/biolab <cmd>    # release binary

# Help
biolab --help
biolab orders --help
biolab inventory --help

# Install AI agent skills for this project
biolab skills install
biolab skills check -f json

# Login (Feishu OAuth)
biolab login
biolab status
biolab logout

# Output in JSON (for machine parsing)
biolab me -f json
biolab orders list -f json
```

## Architecture

### Source Layout

```
Cargo.toml          # Rust project, binary: biolab, lib: biolab
src/
├── main.rs         # Thin CLI entry — imports from lib crate, clap router
├── lib.rs          # ALL mod declarations here; binary imports via biolab::...
├── config.rs       # Token management (env → file → OAuth), base URL
├── client.rs       # BiolabClient factory, re-exports BiolabError
├── errors.rs       # BiolabError enum (thiserror)
├── types.rs        # Serde request/response structs + custom deserializers
├── auth.rs         # Feishu OAuth login flow (tiny_http callback server)
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
└── services/       # impl BiolabClient blocks, domain-specific API methods
    ├── orders.rs   # Order API path builders + unit tests
    ├── users.rs    # User API path builders + unit tests
    ├── templates.rs# Template API path builders + unit tests
    ├── inventory.rs# Stock/location API path builders + unit tests
    ├── lab.rs      # Lab/member/approval API path builders + unit tests
    └── helpers.rs  # Shared: empty_body(), single_field_body(), url_encode()
```

### Key Patterns

- **Credential chain**: `BIOLAB_TOKEN` env var → `~/.biolab_token` file → interactive OAuth
- **Token storage**: `~/.biolab_token`, valid 8 days
- **Module ownership**: All `mod` declarations live in `lib.rs`; `main.rs` imports from `biolab::...` (the library crate). This avoids the binary crate's `crate::` resolving differently from the library's `crate::`
- **HTTP client**: `BiolabHttp` (in `http.rs`) wraps reqwest with Bearer token injection; `api_response.rs` provides `extract_array`/`extract_object`/`envelope_data` for response unwrapping
- **Domain services**: `impl BiolabClient` blocks in `services/*.rs` call `self.http.get/post/...` then `extract_array`/`extract_object` — all methods unwrap the `{ "data": ... }` envelope consistently
- **Shared helpers**: `services/helpers.rs` — `empty_body()`, `single_field_body()`, `url_encode()`
- **Custom deserializers**: `string_or_f64` / `opt_string_or_f64` in `types.rs` — backend sometimes returns numeric fields as JSON strings
- **Errors**: `BiolabError` in `src/errors.rs` (not `error.rs` — avoids collision with `std::error`)
- **Output modes**: `-f json` for machine-readable, default text for human (colored status badges)
- **Agent skills**: `biolab skills install` copies the bundled `skills/biolab-api/SKILL.md` into `.claude/skills/biolab-api` and `.codex/skills/biolab-api`, then writes a version stamp for `biolab skills check`
- **Tests**: `cargo test` must pass before every submission — CI gate enforces this (23 unit tests across api_response, services, types)

### API Base URL

Default: `http://8.136.56.203/api/v1` — overrideable via `BIOLAB_BASE_URL` env var.

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
Detailed API schemas are bundled in `skills/biolab-api/references/` and installed into `.claude/skills/biolab-api/references/`:
- `orders.md` — Order schemas, status machine, supplier differences
- `inventory.md` — Stock/checkin/checkout schemas
- `templates.md` — Template fields for order defaults
- `lab.md` — Lab CRUD, member management, approval rules
- `users.md` — User info, permission model, signup

## CI

`.github/workflows/release.yml` builds for 4 platforms on push:
- Linux (x86_64 via musl)
- Windows (x86_64)
- macOS (x86_64)
- macOS (arm64)

`cargo test` runs before build — all tests must pass.

Tagged pushes (e.g. `v0.1.0`) auto-create GitHub Releases with binaries.
