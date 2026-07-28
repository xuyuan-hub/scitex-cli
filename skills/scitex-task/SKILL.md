---
name: scitex-task
description: "Use when the user asks to create, arrange, inspect, or execute a task in the Scientex lab task scheduling system. First resolve a live task type, then create either a single-stage task or a multi-stage workflow task, including per-stage scheduled release, once the required inputs are clear."
metadata:
  requires:
    bins: ["scitex"]
  cliHelp: "scitex tasks --help"
---

# Scientex Task Natural-Language Workflow

Use this skill when the user asks to create, arrange, execute, or inspect a Scientex task in the task scheduling system.

Examples:

- `帮我建一个样品 QC 任务`
- `帮我安排一个 Tm 计算任务`
- `帮我建一个先算 Tm 再做人工复核的多阶段任务`
- `执行 batch-03 的序列比对任务`
- `看看有没有任务类型适合做这个`
- `create a workflow task for compute first, then staff review`

Do not use this skill for generic coding requests unless the user clearly means a Scientex task in the task scheduling system.

Before API calls, read `../scitex-shared/SKILL.md`.

## Task View Boundary

This skill defaults to the **Lab tasks** view. Create, list, get, documents, upload-field, results, confirm, and reject commands operate through `/lab/tasks` and are scoped to the current lab or explicit `--lab-id`.

Do not use `scitex admin tasks ...` for a normal lab member: it is the platform administrator's global `/tasks` view. The old `scitex tasks workflow/update/update-file` aliases are hidden compatibility commands and must not be used for new workflows. Use `scitex tasks my ...` only when the authenticated staff member needs their own assigned task stages; it is neither a lab list nor a creator-filtered list.

## Core Rule

Never assume the task type exists. Search the current lab's enabled, submit-able task definitions first, then fetch only the selected candidate's detail:

```bash
# Lightweight, lab-scoped candidate search.
scitex tasks types --search <keyword> -f json

# Confirm the selected candidate's schema and user-visible documents.
scitex tasks type <TASK_TYPE_ID> -f json
```

`--search`, `--category`, `--skip`, and `--limit` are all lab-scoped and may be combined with `--lab-id`. The list returns only selection summaries; do not expect `input_schema`, output schema, document content, staff bindings, command templates, or queue settings until `tasks type <ID>`.

Use `--category` only when it materially narrows the candidates. For example:

```bash
scitex tasks types --search <keyword> --category compute -f json
```

If a lab search has no result, retry with a concise synonym. Do not conclude that no type exists until the relevant `input_schema` from a selected detail response has been checked. If `has_next` is true, inspect further pages before claiming the candidate set is complete.

## Inventory Gate For Experiment Tasks

When the task represents an experiment, lab execution, sample processing, PCR, sequencing prep, reagent use, consumable use, primer use, or any workflow that may consume inventory:

1. Read `../scitex-inventory/SKILL.md`.
2. Extract inventory requirements and assign stable `requirement_key` values.
3. **Actively search** for each requirement. Start with an exact catalog number/name plus category, supplier, or filters when known; expand to synonyms only for zero or ambiguous results. After selecting a precise `item_id`, use `scitex inventory check` as a non-reserving stock aggregate and inspect the matching batches.
4. If any requirement cannot be matched to in-stock items after thorough searching, do not create an executable task. Report the missing inventory and move to ordering/restock discussion.
5. Do not checkout inventory during task planning or task creation.
6. During actual execution, re-search inventory and use `checkout` or `checkout-item` with `task_id`, `part_id`, and `requirement_key`.

Active search is a point-in-time snapshot — it is not a reservation or atomic stock lock. Re-search at execution time.

`--filters` is a JSON array of filter objects, for example:

```json
[
  {
    "field": "category",
    "operator": "eq",
    "value": "COMPUTE"
  }
]
```

Then decide:

1. If one task type clearly matches a single-stage request, collect missing required inputs and create a normal task.
2. If the user describes multiple stages, different assignees, or explicit dependencies, plan a workflow task.
3. If multiple task types may match, show the best 2-3 candidates and ask the user to choose.
4. If no task type matches, say no suitable task type is currently available and do not create a task.

## Matching Heuristics

Compare list candidates with their:

- `display_name`
- `key`
- `description`
- `category`

Then fetch `tasks type <ID>` for the selected candidate and compare its `input_schema`, `output_schema`, and user-visible document metadata. The detail response is already limited to enabled lab-available types.

Do not inspect, infer, or report task type staff bindings, command templates, queues, or timeouts; those are not part of the user-facing task type contract and are not returned by lab endpoints.

## Single-Stage vs Workflow

Use a normal task when the request is one stage with one task type.

Use a workflow task when any of these are true:

- the user explicitly asks for multiple stages
- different stages need different `task_type_id` values
- one stage depends on another stage finishing first
- one stage is compute and another is staff review or submission
- the user mentions assignees for only part of the flow

### Single-stage command

```bash
scitex tasks create <json_file>
```

### Workflow command

```bash
scitex tasks create-workflow <json_file>
```

## Creating A Single-Stage Task

The JSON payload should follow the normal task shape:

```json
{
  "title": "<short user-facing title>",
  "description": "<optional description>",
  "task_type_id": "<matched_task_type_id>",
  "input_data": {},
  "parts": [
    {
      "name": "<optional part name>",
      "input_data": {
        "<required_field>": "<value>"
      }
    }
  ]
}
```

Use this when one task type is enough for the full request.

**Critical: input_data placement.** The backend validates each part's `input_data` against that part's task type `input_schema`. Required fields from the task type schema MUST go into `parts[].input_data`, not the root-level `input_data`. Putting them at the root will fail with HTTP 422:
`Part '<name>' input validation failed: 缺少必填字段: <field>`.

## Creating A Workflow Task

The JSON payload should follow the workflow task shape:

```json
{
  "title": "<workflow title>",
  "description": "<optional description>",
  "input_data": {},
  "parts": [
    {
      "client_key": "stage_a",
      "name": "<stage name>",
      "task_type_id": "<stage_task_type_id>",
      "input_data": {
        "<required_field>": "<value>"
      },
      "sort_order": 10
    },
    {
      "client_key": "stage_b",
      "name": "<next stage>",
      "task_type_id": "<stage_task_type_id>",
      "input_data": {
        "<required_field>": "<value>"
      },
      "assignee_ids": ["<user_id>"],
      "release_schedule": {
        "mode": "at_time",
        "not_before_at": "<RFC 3339 date-time with UTC offset>",
        "timezone": "Asia/Shanghai"
      },
      "sort_order": 20
    }
  ],
  "dependencies": [
    {
      "prerequisite_client_key": "stage_a",
      "dependent_client_key": "stage_b",
      "condition_type": "completed"
    }
  ]
}
```

Important rules:

- do not put a root-level `task_type_id` on workflow payloads
- every workflow part must have a unique `client_key`
- dependencies must point to existing `client_key` values
- use `condition_type: "completed"` when the dependent stage should unlock only after the prerequisite stage completes successfully
- each stage should use the task type that best matches that stage only
- put staff assignees on the relevant stage with `assignee_ids`
- task type required fields go into `parts[].input_data`, not the root `input_data` — same rule as single-stage tasks

### Scheduled release for a workflow stage

Set a stage's optional `release_schedule` only inside that stage's `parts[]` object; do not put it at the workflow root. Omit it for the normal immediate-release behavior.

For an absolute scheduled release, use:

```json
"release_schedule": {
  "mode": "at_time",
  "not_before_at": "<RFC 3339 date-time with UTC offset>",
  "timezone": "Asia/Shanghai"
}
```

- `mode` must be `"at_time"` for a scheduled release. The only other supported value is `"immediate"`.
- `not_before_at` is the absolute earliest release time and must be an OpenAPI `date-time` value. Collect a full date, time, and UTC offset; never guess an ambiguous local time.
- `timezone` is optional, but when supplied it must be an IANA timezone name such as `Asia/Shanghai`.
- A scheduled release may be used on the first stage, or as an additional earliest-release boundary on a stage that already has dependencies. Dependencies still belong in `dependencies`; do not replace them with a time schedule.
- Do not use `release_schedule` to describe a relative delay after an upstream stage completes. That is a completed-dependency rule, not an absolute release time.

## Workflow Status Semantics

When inspecting a workflow, interpret stage status as:

- `LOCKED`: the stage is waiting for dependency conditions and/or its scheduled release time
- `READY`: dependency conditions and any scheduled release boundary are satisfied, so the stage is eligible to run
- `IN_PROGRESS`: the stage is running
- `COMPLETED`: the stage finished successfully
- `BLOCKED`: the stage cannot proceed (e.g. upstream failure)

For compute-only workflows, a dependent compute stage may move from `LOCKED` to `READY`, then to `IN_PROGRESS` and `COMPLETED` automatically once the prerequisite stage completes. The root task can remain `IN_PROGRESS` or `WAITING_LAB_CONFIRM` even when compute stages have output; check stage statuses and `part.output_data.exit_code` before reporting whether compute work finished.

## Rescheduling A Locked Stage

Inspect the specific Lab-visible part first. Its detail includes `not_before_at`, incoming dependencies, schedule-change history, and visible runs:

```bash
scitex tasks part <TASK_ID> <PART_ID> --timezone Asia/Shanghai -f json
```

Only a `LOCKED` stage can be rescheduled. Do not call the command for a `READY`, `IN_PROGRESS`, `COMPLETED`, or `BLOCKED` stage, and do not invent a `SCHEDULED` status.

For an absolute release boundary, collect an unambiguous RFC3339 time with offset and a valid IANA timezone:

```bash
scitex tasks part-reschedule <TASK_ID> <PART_ID> \
  --release-at 2026-08-01T09:00:00+08:00 --timezone Asia/Shanghai \
  --reason "move to next shift" --yes
```

For an additional relative delay after a dependency condition, use one or more exact dependency IDs:

```bash
scitex tasks part-reschedule <TASK_ID> <PART_ID> \
  --delay <DEPENDENCY_ID>=1:week --reason "allow recovery" --yes
```

Supported units are `minute`, `hour`, `day`, and `week`; values are integers from 0 through 9999. To remove an absolute boundary, use `--release-immediately --reason <TEXT>`. Always state the affected stage, old rule, new rule, and non-empty reason before approval. This changes scheduling metadata only; it does not use a client timer, cron, queue, or Worker connection.

## Lab Context

Do not hardcode `lab_id` into JSON payloads. The CLI will use the current lab or the user can pass `--lab-id`.

## File Inputs

For direct task creation, attach local files without inventing a reference shape:

```bash
scitex tasks create task.json --file-field field_key=path/to/input.bin -f json
```

When preparing an `input_data` object separately, upload first and use the returned object exactly:

```bash
scitex files upload path/to/input.bin -f json
```

## Required Input Checks

If `input_schema.required` exists for a matched task type, ensure required fields are present before creating the task.

For workflow tasks:

1. validate each stage's required inputs against that stage's task type when practical
2. ask concise follow-up questions for missing values
3. do not invent dependency structure that the user did not imply

### Downstream Stages That Need Upstream Outputs

When a downstream stage's required input comes from an upstream compute stage's output (e.g. Design NGS Primer needs `template` from Codon Optimize output), you **cannot** create the full workflow at once — the backend validates all required inputs at creation time and will reject missing fields with HTTP 422.

Two approaches:

1. **Two-step (reliable, preferred)**: Create the upstream stage(s) as a task or workflow first. After they complete, download the results (`scitex tasks get <id> -f json` gives signed download URLs), then create the downstream stages with the concrete output data.
2. **When all inputs are known upfront**: If every stage's required inputs are available at creation time (no data dependency between stages), create the full workflow in one shot.

Do not leave a required field empty or use a placeholder hoping the backend will fill it — the API validates all parts at submission time.

## Confirmation

Before creating a task, show a short preview.

For single-stage tasks include:

- matched task type
- task title
- input data
- parts

For workflow tasks include:

- workflow title
- stage list in order
- each stage's matched task type
- each stage's key input data
- dependencies
- assignees, if any
- scheduled release time and timezone for every stage using `release_schedule`

For a scheduled release, confirm the exact stage, release time, and timezone before creating the task. Ask for confirmation if the task would start external work, notify staff, spend resources, or if the request is ambiguous.

For clearly requested, low-risk task creation with all inputs present, proceed after the preview according to the user's intent.

## Inspecting Workflow Tasks

For a normal lab member, use:

```bash
scitex tasks get <TASK_ID> -f json
scitex tasks results <TASK_ID> -f json
```

`tasks get` includes each visible part's user-facing task-type input requirements under `input_requirements`. Inspect those requirements before uploading or replacing a task input file; for a seed manifest import they include the accepted Excel format, table headers, and example rows.

Use `scitex tasks part <TASK_ID> <PART_ID> -f json` when the lab-visible part ID is already known. It includes the selected part's incoming dependencies and schedule history, but it is not a substitute for a global workflow graph or hidden assignments. Exact global structure remains available only through `scitex admin tasks workflow <TASK_ID> -f json` to an authorized platform administrator. Do not instruct a lab member to call it or infer hidden structure when it returns permission denied.

## Staff Assignment Completion

Use the staff view only for stages assigned to the authenticated employee:

```bash
scitex tasks my list --search <keyword> --exclude-status completed -f json
scitex tasks my get <ASSIGNMENT_ID> -f json
scitex tasks my upload-field <TASK_ID> <FILE> <FIELD_KEY> --visibility lab-and-staff -f json
scitex tasks my submit-result <ASSIGNMENT_ID> result.json [--feedback feedback.json] -f json
scitex tasks my complete <ASSIGNMENT_ID> result.json [--feedback feedback.json] -f json
```

Prefer `tasks my complete` for normal completion because it atomically submits the result, completes the assignment, and unlocks downstream stages. Use `submit-result` or `status` separately only when the user explicitly needs a partial or staged action. Writable assignment statuses are `pending`, `in-progress`, and `completed`; `BLOCKED` is a task-part status, not an assignment status.

`result.json` should normally contain the output object defined by the assigned stage's
`output_schema`, for example:

```json
{
  "checked_samples": 12,
  "passed": true
}
```

The CLI sends this as `{ "output_data": <result.json contents> }`. A complete
`TaskResultCreate` request object is also accepted for compatibility. If the raw output
schema itself has a top-level `output_data`, `comment`, or `document_feedback` field, use
the explicit request form `{ "output_data": { ... } }` to avoid that ambiguity.
`--feedback` always writes the feedback file at the request's top-level
`document_feedback` field.

## Reading Results

Use:

```bash
scitex tasks results <TASK_ID> -f json
```

Behavior:

- single-stage compute task: reads compute output
- single-stage staff task: reads submitted results
- workflow task: groups output by stage

For workflow tasks, results are stage-aware:

- compute stages read from `part.output_data`
- staff stages read from submitted results linked by `part_id`

If the user asks why a workflow task appears empty, inspect the lab-visible task and results first. Escalate to a platform administrator only when global workflow structure is required and the user has that authority.

## Examples

### Single-stage task

User: `帮我建一个样品 QC 任务`

Workflow:

1. Run `scitex tasks types --search "sample qc" -f json`, then fetch the selected ID with `scitex tasks type <ID> -f json`.
2. Match the best enabled lab-available single task type.
3. Inspect required fields such as `sample_ids`.
4. Ask for missing inputs.
5. Create with `scitex tasks create`.

### Multi-stage workflow task

User: `帮我建一个先算 Tm 再做人工 QC 的任务`

Workflow:

1. Search the lab for `tm` and `qc`, then fetch both selected IDs with `scitex tasks type <ID> -f json`.
2. Match one enabled lab-available compute task type for Tm and one enabled lab-available staff task type for QC.
3. Collect the sequence, sample identifiers, and assignee if needed.
4. Build a workflow payload with two parts and one dependency.
5. Create with `scitex tasks create-workflow`.

### No match

User: `帮我做质谱分析`

If no task type mentions mass spectrometry or related schema fields, explain that no suitable task type is currently available and do not create a generic placeholder unless the user explicitly asks for a manual or custom task and the API supports it.

## Output

After task creation, report:

- task id
- title
- status
- task type for single-stage tasks, or stage summary for workflow tasks

Use `scitex tasks get <TASK_ID> -f json` only if the create response is missing important fields.
