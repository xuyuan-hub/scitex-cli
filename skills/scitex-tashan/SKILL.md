---
name: scitex-tashan
description: "Use when operating the Tashan (他山) project workflows: project info lookup and seed intake object types, batches, records, and stocks."
metadata:
  requires:
    bins: ["scitex"]
  cliHelp: "scitex project tashan --help"
---

# Scientex Tashan Project Workflows

**Before starting, read `../scitex-shared/SKILL.md` for auth, safety, and OpenAPI rules.**

Use this skill for Tashan (他山) project-scoped APIs under `scitex project tashan ...`. Other projects have their own skill packages.

## Info

```bash
scitex project tashan info -f json
```

## Seed Intake

```bash
scitex project tashan seed object-types list -f json
scitex project tashan seed object-types create '<JSON_OR_FILE>' -f json
scitex project tashan seed object-types get <CONFIG_ID> -f json
scitex project tashan seed object-types update <CONFIG_ID> '<JSON_OR_FILE>' -f json

scitex project tashan seed batches list -f json
scitex project tashan seed batches create '<JSON_OR_FILE>' -f json
scitex project tashan seed batches get <BATCH_ID> -f json
scitex project tashan seed batches import-manifest <BATCH_ID> --file <PATH> -f json
scitex project tashan seed batches create-intake-task <BATCH_ID> --record-id <RECORD_ID> -f json

scitex project tashan seed records list --batch-id <BATCH_ID> --status <STATUS> -f json
scitex project tashan seed records public --batch-id <BATCH_ID> -f json
scitex project tashan seed field-catalog -f json
scitex project tashan seed records get <RECORD_ID> -f json
scitex project tashan seed records update <RECORD_ID> '<JSON_OR_FILE>' -f json
scitex project tashan seed records complete <RECORD_ID> -f json

scitex project tashan seed stocks list -f json
scitex project tashan seed stocks get <STOCK_ID> -f json
```

## Schema

Inspect `<SCIENTEX_BASE_URL>/openapi.json` before preparing create/update JSON.

Relevant schemas:

- `SeedObjectTypeConfigCreate`
- `SeedObjectTypeConfigUpdate`
- `SeedIntakeBatchCreate`
- `SeedIntakeRecordUpdate`
- `IntakeTaskRequest`

JSON inputs may be inline JSON strings or local JSON file paths.

Before creating or updating a record, query `seed field-catalog -f json` and construct dynamic record fields from that catalog. When a batch or status is known, use the documented `records list --batch-id` / `--status` filters rather than a broad list.

## Rules

- Confirm before creating or updating object types, batches, and records.
- Before `batches import-manifest`, retrieve the batch, preview the source file and expected import effect, then obtain confirmation.
- Before `batches create-intake-task`, retrieve the batch and list the target records. Supplying no `--record-id` targets all eligible records in that batch; state the count/IDs and obtain explicit confirmation. Use one or more `--record-id` values for an intentional subset.
- Confirm before completing intake records.
- Prefer `-f json` for all project workflows.
- Do not use this skill for user administration, signup, password recovery, or audit/log endpoints.
