---
name: scitex-tashan
description: "Use when operating Tashan (他山) project seed workflows: designing, creating, inspecting, or updating a custom seed type; creating frozen intake batches and manifests; managing intake records or field tasks; or operating the formal SeedLot weight ledger. In particular, use when a project leader needs help turning a seed intake process into a validated new seed-type configuration."
metadata:
  requires:
    bins: ["scitex"]
  cliHelp: "scitex project tashan --help"
---

# Scientex Tashan Project Workflows

**Before starting, read `../scitex-shared/SKILL.md` for authentication, confirmation, and OpenAPI rules.**

Use only `scitex project tashan ...` for Tashan APIs. Prefer `-f json` whenever the result informs another step.

## Read only what the request needs

- **Create, design, inspect, or change a seed type:** Read [references/seed-object-types.md](references/seed-object-types.md). This is the project-leader workflow from requirements to post-create verification.
- **Create a batch, download/import a frozen manifest, update/complete records, or dispatch field intake:** Read [references/seed-intake.md](references/seed-intake.md).
- **Reserve, check out, transfer, or adjust formal weight inventory:** Read [references/seed-lot-ledger.md](references/seed-lot-ledger.md).

Do not load unrelated references merely to list project information or inspect one resource.

## Start safely

```bash
scitex project tashan info -f json
scitex project tashan seed object-types list -f json
```

Before preparing a create or update payload, inspect the live `SeedObjectTypeConfigCreate` or `SeedObjectTypeConfigUpdate` schema at `<SCIENTEX_BASE_URL>/openapi.json`; the live backend is authoritative. Do not use the generic `seed field-catalog` output to invent fields for a custom type: the type's `main_schema.fields` is its own frozen field contract.

## Command map

```bash
# Seed type configurations
scitex project tashan seed object-types list -f json
scitex project tashan seed object-types create '<JSON_OR_FILE>' -f json
scitex project tashan seed object-types get <CONFIG_ID> -f json
scitex project tashan seed object-types update <CONFIG_ID> '<JSON_OR_FILE>' -f json

# Intake batches and frozen templates
scitex project tashan seed batches list -f json
scitex project tashan seed batches create --object-type-config <CONFIG_ID> [--batch-code CODE] -f json
scitex project tashan seed batches get <BATCH_ID> -f json
scitex project tashan seed batches download-template <BATCH_ID> [--out FILE.xlsx] [--force] -f json
scitex project tashan seed batches import-manifest <BATCH_ID> --file <PATH> -f json
scitex project tashan seed batches create-intake-task <BATCH_ID> [--record-id <RECORD_ID> ...] -f json

# Records and formal inventory
scitex project tashan seed records list --batch-id <BATCH_ID> [--status <STATUS>] -f json
scitex project tashan seed records public --batch-id <BATCH_ID> -f json
scitex project tashan seed records get <RECORD_ID> -f json
scitex project tashan seed records update <RECORD_ID> '<JSON_OR_FILE>' -f json
scitex project tashan seed records complete <RECORD_ID> -f json
scitex project tashan seed lots list [--type CODE] [--all] -f json
```

## Non-negotiable safety rules

- Confirm explicitly before every create, update, manifest import, intake task, record completion, or inventory write. Before confirmation, state the project, affected IDs, full configuration or field changes, and expected downstream effect.
- Treat a type code as immutable. Never silently update an existing type when creation reports a duplicate; retrieve the existing configuration and ask how to proceed.
- Creating a batch freezes that type's field contract. Verify a newly created type before offering a batch, and obtain a separate confirmation before creating that batch.
- Download a template only from the intended batch; never create a replacement workbook or reuse headers from another batch.
- Do not use this skill for user administration, signup, password recovery, or audit/log endpoints.
