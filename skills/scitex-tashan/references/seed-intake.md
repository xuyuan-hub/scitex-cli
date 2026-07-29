# Frozen Intake Batches, Manifests, and Records

Read this reference for intake work after the seed type is already chosen or created.

## Batch and manifest workflow

Creating a batch freezes the seed type's field contract. Retrieve the type and state this consequence before asking for confirmation. After creation, report the batch ID, batch code, and frozen specification ID.

```bash
scitex project tashan seed object-types get <CONFIG_ID> -f json
scitex project tashan seed batches create --object-type-config <CONFIG_ID> [--batch-code CODE] -f json
scitex project tashan seed batches get <BATCH_ID> -f json
```

Always download the template from the exact draft batch:

```bash
scitex project tashan seed batches download-template <BATCH_ID> --out manifest.xlsx -f json
```

Do not generate a blank workbook, copy headers from another batch, or overwrite a file without an explicit `--force`. Confirm the intended batch and that the supplied file is a readable, non-empty `.xlsx` before importing.

```bash
scitex project tashan seed batches import-manifest <BATCH_ID> --file <PATH> -f json
```

The import creates an asynchronous task, not records immediately. Report `task_id`, `part_id`, `batch_id`, and `source_file_document_id`, then inspect its progress through the batch or `scitex tasks part <TASK_ID> <PART_ID> -f json`. For a conflict such as `SED_006`, do not retry against another batch; ask whether to create a new draft batch or use the existing manifest.

## Records and field tasks

Use server-side filters when batch or status is known:

```bash
scitex project tashan seed records list --batch-id <BATCH_ID> [--status <STATUS>] -f json
scitex project tashan seed records public --batch-id <BATCH_ID> -f json
scitex project tashan seed records get <RECORD_ID> -f json
scitex project tashan seed records update <RECORD_ID> '<JSON_OR_FILE>' -f json
scitex project tashan seed records complete <RECORD_ID> -f json
```

Use field keys from the batch's frozen type schema, not Chinese labels or arbitrary catalog names, in record updates. Confirm before updating or completing a record.

Before creating a field-intake task, retrieve the batch and list its target records. Omitting `--record-id` targets all eligible records; state the count and IDs and obtain explicit confirmation. Use one or more `--record-id` arguments only for an intentional subset.

```bash
scitex project tashan seed batches create-intake-task <BATCH_ID> [--record-id <RECORD_ID> ...] -f json
```

Field staff may receive only fields that are editable by staff, employee-visible, and non-sensitive. Do not compensate for a missing field by adding an unconfigured Excel column.
