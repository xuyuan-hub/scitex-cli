---
name: scitex-tashan
description: "Use when operating the Tashan (他山) project workflows: project info lookup and seed intake object types, batches, records, stocks, or when users need the current seed manifest Excel upload requirements, headers, mappings, or template."
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

## Seed Manifest Requirements And Template

When a user asks what seed manifest to upload, which Excel columns are accepted, or for a manifest template, retrieve the live requirements before proposing a table. Do not infer them from a past upload or use `batches import-manifest` merely to discover validation rules.

Use the shortest applicable read-only path:

1. **An intake task ID is known**: run `scitex tasks get <TASK_ID> -f json`. Read the matching part from `input_requirements[].requirements`: its `description` contains the file layout, standard headers, and example row; `input_schema` gives required task fields and accepted file extension. Read that part's `input_data.object_type_config_id`, then fetch the configuration below for project-specific mappings.
2. **A batch ID is known**: run `scitex project tashan seed batches get <BATCH_ID> -f json`, take its `object_type_config_id`, then run `scitex project tashan seed object-types get <CONFIG_ID> -f json`.
3. **No intake task exists yet**: discover the lab-visible task type, then read its detail:

   ```bash
   scitex tasks types --search "种子清单导入" -f json
   scitex tasks type <TASK_TYPE_ID> -f json
   ```

4. **Object type is known**: retrieve it directly:

   ```bash
   scitex project tashan seed object-types get <CONFIG_ID> -f json
   ```

   If only its code or name is known, list configurations first and ask the user to select an ambiguous match:

   ```bash
   scitex project tashan seed object-types list -f json
   ```

Build the user-facing requirements from these sources, in this order:

- State the `.xlsx` acceptance and all worksheet/header/row rules from the task type `description` and `input_schema.properties.source_file`.
- Copy the standard headers and sample row from the task type description; do not hardcode a stale local header list.
- If `import_mapping` is present on the object type, show its spreadsheet-header-to-Scientex-field mappings and state that these project-specific aliases override the standard mapping.
- Treat `completion_required_fields` as requirements for completing a later intake record, not as mandatory columns for uploading the manifest, unless the backend's task description expressly says otherwise.
- If a user wants an actual `.xlsx` template, create a single-sheet workbook using the retrieved headers and one clearly labelled example row; do not upload it until the user confirms the intended batch and import effect.

When no object type or batch is specified, provide the task type's standard requirements and ask which object type applies before claiming that its custom mappings have been covered.

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
