---
name: biolab-shared
version: 0.1.0
description: "Use when first setting up biolab CLI, logging in, checking account status, handling token storage, checking updates, or preparing any Biolab API JSON payload that must match backend OpenAPI schemas."
metadata:
  requires:
    bins: ["biolab"]
  cliHelp: "biolab --help"
---

# Biolab Shared Rules

Use this shared skill before using any domain-specific Biolab skill.

## Setup

Check the CLI:

```bash
biolab --help
biolab update check
```

Authenticate before API calls:

```bash
biolab login
biolab status
```

If `login` prints an authorization URL, send that exact URL to the user and wait for them to complete browser auth before continuing.

## Credentials

Token lookup order:

1. `BIOLAB_TOKEN`
2. container-local token file when running in Docker/K8s
3. OS keychain
4. explicit plaintext fallback only when `BIOLAB_INSECURE_TOKEN_FILE=1`

Do not print tokens or secrets.

## OpenAPI First

For any create/update JSON payload, inspect the backend OpenAPI schema before choosing fields.

Default schema URL:

```text
<BIOLAB_BASE_URL>/openapi.json
```

If `BIOLAB_BASE_URL` is unset, use the CLI default base URL.

Do not invent CLI commands for backend endpoints that `biolab <domain> --help` does not expose.

## Output And Safety

- Prefer `-f json` when the next step needs machine parsing.
- Use `biolab <domain> --help` before guessing flags.
- Confirm before write operations that mutate lab state, orders, templates, inventory, or profile data.
- Use the domain skill matching the task:
  - Orders: `../biolab-orders/SKILL.md`
  - Templates: `../biolab-templates/SKILL.md`
  - Inventory: `../biolab-inventory/SKILL.md`
  - Lab: `../biolab-lab/SKILL.md`
  - Project administration: `../biolab-project/SKILL.md`
  - Tashan project workflows: `../biolab-tashan/SKILL.md`
  - Users: `../biolab-users/SKILL.md`
