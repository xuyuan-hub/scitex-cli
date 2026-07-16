---
name: scitex-shared
version: 0.1.0
description: "Use when first setting up scitex CLI, logging in, checking account status, handling token storage, checking updates, or preparing any Scientex API JSON payload that must match backend OpenAPI schemas."
metadata:
  requires:
    bins: ["scitex"]
  cliHelp: "scitex --help"
---

# Scientex Shared Rules

Use this shared skill before using any domain-specific Scientex skill.

## First Setup And Diagnostics

On first setup, after an installation problem, or when the user explicitly asks about updates, check the CLI:

```bash
scitex --help
scitex update check
```

Before API calls, use `scitex status`; only run `scitex login` when it reports unauthenticated.

```bash
scitex login
scitex status
```

If `login` prints an authorization URL, send that exact URL to the user and wait for them to complete browser auth before continuing.

## Credentials

Token lookup order:

1. `SCIENTEX_TOKEN`
2. container-local token file when running in Docker/K8s
3. OS keychain
4. explicit plaintext fallback only when `SCIENTEX_INSECURE_TOKEN_FILE=1`

Do not print tokens or secrets.

## OpenAPI First

For any create/update JSON payload, inspect the backend OpenAPI schema before choosing fields.

Default schema URL:

```text
<SCIENTEX_BASE_URL>/openapi.json
```

If `SCIENTEX_BASE_URL` is unset, use the CLI default base URL.

Do not invent CLI commands for backend endpoints that `scitex <domain> --help` does not expose.

## Output And Safety

- Prefer `-f json` when the next step needs machine parsing.
- Use `scitex <domain> --help` before guessing flags.
- Prefer an exact `get` by ID, a documented server-side search/filter, or a status-scoped list. Use an unfiltered list only when the API has no narrower query or the user asks to browse the full set.
- Confirm before write operations that mutate lab state, orders, templates, inventory, or profile data.

## Task Views

Do not treat all task endpoints as the same view:

- **Lab tasks** (`scitex tasks create/list/get/results/...`) use `/lab/tasks`; they are the normal member view, scoped to the current lab or an explicit `--lab-id`.
- **Platform tasks** (`scitex tasks workflow`, `scitex tasks update`, `scitex tasks update-file`) use `/tasks`; they are cross-lab administrator operations and require `platform_admin` or superuser permission.
- **My Tasks** (`scitex tasks my ...`) use `/staff/tasks`; they show only task stages assigned to the current staff member, not tasks created by them or all tasks in their lab.

The same task can appear in all three views. Route by the user's role and intent, and never use an administrator endpoint as a fallback for a lab member.

Task definition discovery follows the same boundary: `scitex tasks types --search <keyword>` and `scitex tasks type <TYPE_ID>` use the current lab's enabled, user-submit-able definitions. The list is a lightweight candidate view; fetch the selected definition's detail before constructing task input JSON. Do not use the administrator `/task-types` catalog for a normal lab user.

- Use the domain skill matching the task:
  - Orders: `../scitex-orders/SKILL.md`
  - Templates: `../scitex-templates/SKILL.md`
  - Inventory: `../scitex-inventory/SKILL.md`
  - Lab: `../scitex-lab/SKILL.md`
  - Project administration: `../scitex-project/SKILL.md`
  - Tashan project workflows: `../scitex-tashan/SKILL.md`
  - Task execution: `../scitex-task/SKILL.md`
  - Task type catalog administration: `../scitex-admin/SKILL.md`
  - Error Report: `../scitex-error-report/SKILL.md`
  - Users: `../scitex-users/SKILL.md`
