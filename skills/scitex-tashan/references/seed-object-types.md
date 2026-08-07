# Custom Seed Types for Project Leaders

Read this reference when a project leader wants to create a new seed type, turn an Excel intake process into a type configuration, review a proposed field schema, or change an existing type.

## Outcome and boundaries

A seed type defines the reusable field contract for later intake batches. A batch freezes a copy of that contract, so changing a type only affects batches created afterwards. Use the project type configuration, not a generic catalog or a historical Excel workbook, as the source of truth.

Only a project leader can create or update a type. If the current user lacks that role, help them prepare and review the draft but do not attempt the write.

## 1. Discover before designing

First inspect the project and avoid code collisions:

```bash
scitex project tashan info -f json
scitex project tashan seed object-types list -f json
```

For a create or update, inspect the live backend OpenAPI at `<SCIENTEX_BASE_URL>/openapi.json`, especially `SeedObjectTypeConfigCreate` and `SeedObjectTypeConfigUpdate`. The latest contract requires `code`, `name`, `category`, `numbering_prefix`, and `main_schema` on creation; `main_schema` must contain `fields`.

Ask only for the information that is still unknown:

1. Type code, display name, category, and numbering prefix.
2. One row per business field: Chinese display label/Excel header, stable English `key`, value type, and whether the value is known before intake or only at completion.
3. Who supplies each field (`leader`, `staff`, or `both`), whether staff may see it, and whether it is sensitive.
4. Allowed values, numeric range/precision, pattern, or length limits.

Do not choose a business field, visibility setting, or validation rule on the leader's behalf. Propose a draft when enough facts are available, then show it for review.

## 2. Design a valid field contract

Use an English `key` beginning with a letter and containing only letters, digits, and underscores. Keep keys and Excel `header` values unique within the type. Keys are the stable API/export identifiers; labels and headers can be Chinese.

| Need | Configuration |
|---|---|
| Identify each sample before import | One `sample_name` role field; `required_at: "preparing"`. |
| Create formal weight inventory | One `inventory_weight` role field; `value_type: "decimal"`, `required_at: "completion"`, non-negative numeric validation. |
| Record physical packaging | One `container` role field; `required_at: "completion"`. |
| Let staff fill a field in the field work order | Set `editor` to `staff` or `both`, `employee_visible` to `true`, and `sensitive` to `false`. |
| Keep an eventual field out of the import manifest | Use `required_at: "completion"` or `"optional"`, not `"preparing"`. |

Each recognized role can occur only once. Optional recognized roles include `seed_count`, `source_parent`, `generation`, `storage_site`, `storage_position`, `intake_date`, and `operator`.

Use these `value_type` values when supported by the live contract: `string`, `integer`, `decimal`, `date`, `enum`, `multi_enum`, and `json_object`. Typical `validation` members are `minimum`, `maximum`, `precision`, `allowed_values`, `multi_allowed_values`, `pattern`, and `max_length`. An optional field is still validated when a value is supplied.

## 3. Draft and review the payload

Use the following as a design template after checking it against the live OpenAPI. Replace every illustrative value; do not submit it unchanged.

```json
{
  "code": "MAIZE_TRIAL",
  "name": "玉米试验材料",
  "category": "breeding",
  "numbering_prefix": "MT",
  "main_schema": {
    "schema_version": 2,
    "fields": [
      {
        "key": "material_code",
        "label": "材料编号",
        "header": "材料编号",
        "order": 0,
        "value_type": "string",
        "required_at": "preparing",
        "editor": "leader",
        "employee_visible": false,
        "sensitive": false,
        "role": "sample_name",
        "validation": {"max_length": 100}
      },
      {
        "key": "net_weight_g",
        "label": "实收净重(g)",
        "header": "实收净重(g)",
        "order": 1,
        "value_type": "decimal",
        "required_at": "completion",
        "editor": "staff",
        "employee_visible": true,
        "sensitive": false,
        "role": "inventory_weight",
        "validation": {"minimum": 0, "precision": 4}
      },
      {
        "key": "package_type",
        "label": "容器",
        "header": "容器",
        "order": 2,
        "value_type": "string",
        "required_at": "completion",
        "editor": "staff",
        "employee_visible": true,
        "sensitive": false,
        "role": "container",
        "validation": {}
      }
    ]
  }
}
```

Run the deterministic preflight before requesting confirmation. It does not contact a write endpoint; `--openapi` additionally checks the downloaded current `SeedObjectTypeConfigCreate` contract. Run it from the installed `scitex-tashan` skill directory, or substitute that directory for `<SKILL_ROOT>`:

```bash
python3 <SKILL_ROOT>/scripts/validate_seed_type_config.py <PATH_TO_REVIEWED_JSON> \
  --openapi "${SCIENTEX_BASE_URL:-https://scientex.cn/api/v1}/openapi.json" --json
```

Review this checklist with the leader before writing:

- The code is unique in the type list, short enough for the live schema, and will not need to change later.
- Only values available before physical intake are `preparing`; measurement and packaging fields are normally `completion`.
- The contract has exactly one each of `sample_name`, `inventory_weight`, and `container` with the compatible type/stage constraints above.
- Staff visibility, editor assignment, and sensitivity are compatible; sensitive fields must not be exposed in a staff work order.
- Enum choices, decimal precision, and bounds match the real-world process.

Present the complete JSON, preflight result, and a concise field table to the leader. Obtain explicit confirmation naming the new code before executing the command. The preflight complements, but never replaces, the live OpenAPI check and server validation.

## 4. Create and verify

Save the reviewed JSON locally or pass it inline, then create it only after confirmation:

```bash
scitex project tashan seed object-types create <PATH_TO_REVIEWED_JSON> -f json
```

Report the returned configuration ID, code, name, and field count. Then retrieve the exact object and check that its `main_schema.fields`, role bindings, and visibility rules match the approved draft:

```bash
scitex project tashan seed object-types get <CONFIG_ID> -f json
```

Only after that verification, offer to create an intake batch. Explain that the batch will freeze this exact contract and require a new, separate confirmation:

```bash
scitex project tashan seed batches create --object-type-config <CONFIG_ID> [--batch-code CODE] -f json
```

## 5. Inspect or change an existing type

Retrieve a type by ID before proposing a change. The code cannot be changed. When changing `main_schema`, send the full revised `fields` array rather than one field fragment, preserve intended existing fields, and explain that existing batches retain their old frozen template.

```bash
scitex project tashan seed object-types get <CONFIG_ID> -f json
scitex project tashan seed object-types update <CONFIG_ID> <PATH_TO_REVIEWED_JSON> -f json
```

Confirm the exact diff and the impact on future batches before the update. If the user instead needs an incompatible process, prefer creating a new type with a new code.

## Failure handling

- **Duplicate code or conflict:** Do not retry with an altered payload. Retrieve the matching type and ask whether to use it, update it, or choose another code.
- **Validation error:** Preserve the server error, compare the payload with the live OpenAPI and the approved field table, correct the draft, and reconfirm.
- **Permission error:** Explain that the write requires project-leader permission; do not attempt a different endpoint or identity.
- **Unexpected returned contract:** Stop before batch creation, show the mismatch, and ask the leader whether to correct the type.
