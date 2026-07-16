---
name: scitex-admin
description: "Use when creating, deleting, or managing Scientex admin-only task type catalog definitions. Normal task execution remains under scitex-task; this skill is for catalog administration such as creating reusable task types."
metadata:
  requires:
    bins: ["scitex"]
  cliHelp: "scitex admin --help"
---

# Scientex Admin Task Type Catalog

Use this skill when the user wants to define or manage reusable Scientex task types, such as adding a new staff review task type or compute task type to the platform catalog. Also use it when binding or unbinding staff users who are allowed to handle a task type.

Examples:

- `创建一个新的任务类型`
- `把这个实验流程包装成任务类型`
- `添加一个 staff task type`
- `新增一个 compute task type，输入是 fasta 文件`
- `delete this temporary task type`
- `把这个任务类型绑定给某个员工`
- `让这个员工可以处理 sample_qc 任务类型`
- `移除某个员工和任务类型的绑定`
- `给这个任务类型上传 SOP 文件`
- `上传工单模板到任务类型`
- `查看任务类型的文档列表`
- `删除任务类型上挂的某个文档`

Do not use this skill for creating executable task instances. For actual task execution, read `../scitex-task/SKILL.md` and use `scitex tasks create` or `scitex tasks create-workflow`.

Before API calls, read `../scitex-shared/SKILL.md`.

## Core Rule

Use the top-level admin command group:

```bash
# Task type lifecycle
scitex admin task-types create <task_type.json> [--sop <file>] [--work-order <file>]
scitex admin task-types delete <TASK_TYPE_ID>

# Document management (SOP, work order, attachment)
scitex admin task-types upload-doc <TYPE_ID> <FILE> --doc-type sop|work_order|attachment
scitex admin task-types list-docs <TYPE_ID>
scitex admin task-types delete-doc <TYPE_ID> <DOC_ID>
scitex admin task-types feedback <TYPE_ID>

# Staff binding
scitex admin task-types staff list <TASK_TYPE_ID>
scitex admin task-types staff add <TASK_TYPE_ID> <USER_ID>
scitex admin task-types staff remove <TASK_TYPE_ID> <USER_ID>
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

1. Search `scitex tasks types --search <key-or-name> -f json` before creating; treat matches as catalog candidates and avoid duplicate keys.
2. Translate the user's task-type idea into a clear `TaskTypeCreate` JSON payload.
2. Prefer stable lowercase snake_case keys for `key` and schema property names.
3. Include only fields supported by the OpenAPI schema.
4. Save the payload as a temporary JSON file.
5. If the user provides SOP or work order content, write those to separate files now.
6. Show the payload, key, category, and documents to be created; obtain confirmation before the write operation. Then run create with inline document flags when files are ready:

```bash
# Create with SOP and work order in one step
scitex admin task-types create <task_type.json> --sop <sop_file> --work-order <wo_file> -f json

# Or create first, then upload documents separately
scitex admin task-types create <task_type.json> -f json
scitex admin task-types upload-doc <TYPE_ID> <sop_file> --doc-type sop -f json
scitex admin task-types upload-doc <TYPE_ID> <wo_file> --doc-type work_order -f json
```

7. Report the created task type id, key, display name, category, and any uploaded document ids.

**When to upload documents:** Always upload an SOP when the task type represents a procedure with steps, safety rules, or inspection checklists. Always upload a work order template when the task type requires staff to fill out a structured daily/per-run record. Do not leave `documents: []` if the user provided procedure content.

## Document Management

Task type documents are files (SOP, work order template, or attachment) stored permanently on the task type catalog entry. Staff members see them when working on task instances of this type.

Document types accepted by the backend (`TaskDocumentType` enum):
- `sop` — Standard operating procedure, steps, safety rules
- `work_order` — Per-run or daily record template staff fill out
- `attachment` — Any supplementary reference file
- `result_attachment` — Output reference (not supported by `upload-doc`; attached at result submission)

### Upload

```bash
scitex admin task-types upload-doc <TYPE_ID> <FILE> --doc-type sop -f json
scitex admin task-types upload-doc <TYPE_ID> <FILE> --doc-type work_order -f json
scitex admin task-types upload-doc <TYPE_ID> <FILE> --doc-type attachment -f json
```

Accepted file formats: Markdown (`.md`), PDF (`.pdf`), Word (`.docx`), plain text, or any binary.

The response includes `feishu_doc_url` and `feishu_sync_status` — when `feishu_sync_status` is `synced`, the document is also accessible in Feishu.

### List

```bash
scitex admin task-types list-docs <TYPE_ID> -f json
```

Use this to verify uploads or to retrieve doc IDs before deletion.

### Delete

```bash
scitex admin task-types delete-doc <TYPE_ID> <DOC_ID>
```

### Feedback

Staff can submit feedback on documents (e.g. corrections to an SOP) when completing a task. View feedback with:

```bash
scitex admin task-types feedback <TYPE_ID> -f json
```

## Deletion

Use deletion only when the user asks to remove a task type or when cleaning up a temporary test fixture:

```bash
scitex admin task-types delete <TASK_TYPE_ID>
```

When deleting a task type that may be used by existing tasks, warn the user that removal can affect catalog availability and obtain explicit confirmation. Apply the same confirmation rule to staff bindings and document uploads/deletions.

## Permission Handling

If the backend returns 401, 403, `forbidden`, `permission`, `not authorized`, or `platform_admin` in the error detail, explain that the current account does not have permission for the admin operation. Do not retry with another identity unless the user explicitly logs in or switches credentials.

## Output

After creation, summarize:

- task type id
- key
- display name
- category
- any required input fields
- uploaded documents (id, document_type, filename, feishu_sync_status) — report these even when using `--sop`/`--work-order` inline flags, as the combined JSON response includes an `uploaded_documents` array

For a standalone `upload-doc`, report: doc id, document_type, filename, feishu_sync_status, and feishu_doc_url if present.

For deletion, report the deleted task type id.

## Staff Binding

Task type staff binding controls which staff users can handle a reusable task type. It is platform catalog configuration, not a normal task instance assignment.

Use:

```bash
scitex admin task-types staff list <TASK_TYPE_ID>
scitex admin task-types staff add <TASK_TYPE_ID> <USER_ID>
scitex admin task-types staff remove <TASK_TYPE_ID> <USER_ID>
```

Rules:

- Use this workflow when the user asks to bind, assign, allow, authorize, remove, or unbind a staff user for a task type.
- `remove` takes `USER_ID`, not `assignment_id`, because the backend DELETE path is `/task-types/{type_id}/staff/{user_id}`.
- If the user gives a name or email rather than `USER_ID`, first use available user/lab-member lookup commands to resolve the user ID. If no reliable lookup is available, ask the user for the user ID.
- Do not use `scitex tasks create` or workflow `assignee_ids` for task type catalog binding. Those are for task instances.
- After binding or unbinding, summarize the task type id and user id. Use `staff list` when the user wants to verify current bindings.
