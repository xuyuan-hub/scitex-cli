---
name: scitex-tashan
description: "Use when operating the Tashan (他山) project workflows: project info lookup, seed intake batches and frozen manifests, seed records, or formal SeedLot weight ledger operations."
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
scitex project tashan seed batches create --object-type-config <CONFIG_ID> [--batch-code CODE] -f json
scitex project tashan seed batches get <BATCH_ID> -f json
scitex project tashan seed batches download-template <BATCH_ID> [--out FILE.xlsx] [--force] -f json
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

scitex project tashan seed lots list [--type CODE] [--all] -f json
scitex project tashan seed lots get <LOT_ID> -f json
scitex project tashan seed lots movements <LOT_ID> -f json
scitex project tashan seed lots reservations <LOT_ID> -f json
scitex project tashan seed lots reserve <LOT_ID> --weight-g <DECIMAL_G> --yes -f json
scitex project tashan seed reservations release <RESERVATION_ID> --yes -f json
scitex project tashan seed lots checkout <LOT_ID> --weight-g <DECIMAL_G> [--reservation ID] --yes -f json
scitex project tashan seed lots transfer <LOT_ID> [--location-id ID] [--site TEXT] [--location-text TEXT] [--note TEXT] --yes -f json
scitex project tashan seed lots adjust <LOT_ID> --type <adjustment|loss|migration_correction> --weight-delta-g <DECIMAL_G> --reason TEXT --yes -f json
```

## Frozen Seed Manifest Template

An intake batch freezes its own manifest contract. When a batch is known, always download that batch's template before asking the user to fill an Excel file:

```bash
scitex project tashan seed batches get <BATCH_ID> -f json
scitex project tashan seed batches download-template <BATCH_ID> --out manifest.xlsx -f json
```

Do not generate a blank workbook, copy headers from an older batch, or substitute another batch's template. The download is the only authority for the frozen `SeedTypeSpec`; it refuses to overwrite an existing file unless `--force` is explicit.

Before upload, confirm the intended batch and preview that the file is a non-empty readable `.xlsx`. `import-manifest` sends the file as multipart field `file` and only creates an asynchronous import task. After success, report its `task_id`, `part_id`, `batch_id`, and `source_file_document_id`, then inspect the batch or `scitex tasks part <TASK_ID> <PART_ID> -f json` for progress. If the server reports an import conflict such as `SED_006`, do not retry with a different batch; ask whether to create a new draft batch or use the existing manifest.

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

## SeedLot Weight Ledger

`seed stocks` is a read-only migration view. Use `seed lots` for formal inventory. All weights are decimal strings in grams with at most four decimal places; do not use floating-point arithmetic or maintain a local balance cache.

Read the current lot, movement, or reservation before proposing a write. `reserve`, `release`, `checkout`, `transfer`, and `adjust` are state-changing and require `--yes` in non-interactive use. Describe the exact lot, decimal weight, reservation (when any), target placement, movement type, and reason before approving. Use only `adjustment`, `loss`, or `migration_correction` for adjustments. Report server-returned movement/reservation/placement identifiers and balances rather than calculating them locally.

## Rules

- Confirm before creating or updating object types, batches, and records.
- Before `batches import-manifest`, retrieve the batch, preview the source file and expected import effect, then obtain confirmation.
- Before `batches create-intake-task`, retrieve the batch and list the target records. Supplying no `--record-id` targets all eligible records in that batch; state the count/IDs and obtain explicit confirmation. Use one or more `--record-id` values for an intentional subset.
- Confirm before completing intake records.
- Prefer `-f json` for all project workflows.
- Do not use this skill for user administration, signup, password recovery, or audit/log endpoints.
