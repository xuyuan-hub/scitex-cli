---
name: biolab-admin
description: "Use when creating, updating, deleting, or managing Biolab admin-only task type catalog definitions. Normal task execution remains under biolab-task; this skill is for catalog administration such as creating reusable task types."
metadata:
  requires:
    bins: ["biolab"]
  cliHelp: "biolab admin --help"
---

# Biolab Admin Task Type Catalog

Use this skill when the user wants to define or manage reusable Biolab task types, such as adding a new staff review task type or compute task type to the platform catalog.

Examples:

- `创建一个新的任务类型`
- `把这个实验流程包装成任务类型`
- `添加一个 staff task type`
- `新增一个 compute task type，输入是 fasta 文件`
- `delete this temporary task type`

Do not use this skill for creating executable task instances. For actual task execution, read `../biolab-task/SKILL.md` and use `biolab tasks create` or `biolab tasks create-workflow`.

Before API calls, read `../biolab-shared/SKILL.md`.

## Core Rule

Use the top-level admin command group:

```bash
biolab admin task-types create <task_type.json>
biolab admin task-types delete <TASK_TYPE_ID>
```

Do not wrap these normal task execution endpoints as admin catalog commands:

- `POST /api/v1/tasks`
- `POST /api/v1/tasks/{task_id}/parts`

Do not ask the user to confirm they are an administrator before using this workflow. If the backend returns a permission or forbidden error, report that the current account lacks permission for the admin operation.

## Create Payload

The create command accepts a JSON file matching the backend `TaskTypeCreate` schema:

```json
{
  "key": "sample_qc",
  "display_name": "Sample QC",
  "description": "Manual sample quality-control task",
  "scope": "lab",
  "category": "staff",
  "input_schema": {
    "type": "object",
    "properties": {
      "sample_id": {
        "type": "string",
        "title": "Sample ID",
        "description": "Sample identifier to inspect"
      }
    },
    "required": ["sample_id"]
  },
  "output_schema": {
    "type": "object",
    "properties": {
      "qc_result": {
        "type": "string",
        "title": "QC Result"
      }
    }
  }
}
```

Required fields:

- `key`: stable machine key, 1-100 characters.
- `display_name`: user-facing name, 1-255 characters.

Common optional fields:

- `description`: task type description.
- `scope`: backend-defined scope string when needed.
- `category`: `staff` or `compute`; defaults to `staff` when omitted by the backend.
- `input_schema`: JSON object describing required inputs.
- `output_schema`: JSON object describing outputs.
- `command_template`: string array for compute task execution.
- `timeout_seconds`: positive integer timeout for compute task execution.

## Schema Rules

Task type schemas should use this shape:

```json
{
  "type": "object",
  "properties": {
    "field_key": {
      "type": "string",
      "title": "Field Label",
      "description": "Optional help text"
    }
  },
  "required": ["field_key"]
}
```

Supported property `type` values are:

- `string`
- `integer`
- `number`
- `object`

For upload fields, use `type: "object"` with `format: "file"`:

```json
{
  "type": "object",
  "format": "file",
  "title": "Input File"
}
```

If `required` is present, every required field should exist in `properties`.

## Workflow

1. Translate the user's task-type idea into a clear `TaskTypeCreate` JSON payload.
2. Prefer stable lowercase snake_case keys for `key` and schema property names.
3. Include only fields supported by the OpenAPI schema.
4. Save the payload as a temporary JSON file.
5. Run:

```bash
biolab admin task-types create <task_type.json> -f json
```

6. Report the created task type id, key, display name, and category.

## Deletion

Use deletion only when the user asks to remove a task type or when cleaning up a temporary test fixture:

```bash
biolab admin task-types delete <TASK_TYPE_ID>
```

When deleting a task type that may be used by existing tasks, warn the user that removal can affect catalog availability.

## Permission Handling

If the backend returns 401, 403, `forbidden`, `permission`, `not authorized`, or `platform_admin` in the error detail, explain that the current account does not have permission for the admin operation. Do not retry with another identity unless the user explicitly logs in or switches credentials.

## Output

After creation, summarize:

- task type id
- key
- display name
- category
- any required input fields

For deletion, report the deleted task type id.
