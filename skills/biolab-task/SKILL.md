---
name: biolab-task
version: 0.2.0
description: "Use when the user asks in natural language to do, implement, run, arrange, schedule, or execute a Biolab task. First check available task types, then create either a single-stage task or a multi-stage workflow task once the required inputs are clear."
metadata:
  requires:
    bins: ["biolab"]
  cliHelp: "biolab tasks --help"
---

# Biolab Task Natural-Language Workflow

Use this skill when the user asks to create, arrange, execute, or inspect a Biolab task in the task scheduling system.

Examples:

- `帮我建一个样品 QC 任务`
- `帮我安排一个 Tm 计算任务`
- `帮我建一个先算 Tm 再做人工复核的多阶段任务`
- `执行 batch-03 的序列比对任务`
- `看看有没有任务类型适合做这个`
- `create a workflow task for compute first, then staff review`

Do not use this skill for generic coding requests unless the user clearly means a Biolab task in the task scheduling system.

Before API calls, read `../biolab-shared/SKILL.md`.

## Core Rule

Never assume the task type exists. Always check available task types first:

```bash
biolab tasks types -f json
```

Then decide:

1. If one task type clearly matches a single-stage request, collect missing required inputs and create a normal task.
2. If the user describes multiple stages, different assignees, or explicit dependencies, plan a workflow task.
3. If multiple task types may match, show the best 2-3 candidates and ask the user to choose.
4. If no task type matches, say no suitable task type is currently available and do not create a task.

## Matching Heuristics

Compare the user request with each task type's:

- `display_name`
- `key`
- `description`
- `category`
- `input_schema`
- `output_schema`
- `documents`

Prefer enabled task types. Ignore disabled task types unless the user explicitly asks about unavailable options.

Do not inspect, infer, or report task type staff bindings. Staff assignment or binding details are not part of the user-facing task type contract; if an API response includes `assigned_staff` or similar internal fields, ignore them.

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
biolab tasks create <json_file>
```

### Workflow command

```bash
biolab tasks create-workflow <json_file>
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
      "input_data": {}
    }
  ]
}
```

Use this when one task type is enough for the full request.

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
      "input_data": {},
      "sort_order": 10
    },
    {
      "client_key": "stage_b",
      "name": "<next stage>",
      "task_type_id": "<stage_task_type_id>",
      "input_data": {},
      "assignee_ids": ["<user_id>"],
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
- each stage should use the task type that best matches that stage only
- put staff assignees on the relevant stage with `assignee_ids`

## Lab Context

Do not hardcode `lab_id` into JSON payloads. The CLI will use the current lab or the user can pass `--lab-id`.

## Required Input Checks

If `input_schema.required` exists for a matched task type, ensure required fields are present before creating the task.

For workflow tasks:

1. validate each stage's required inputs against that stage's task type when practical
2. ask concise follow-up questions for missing values
3. do not invent dependency structure that the user did not imply

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

Ask for confirmation if the task would start external work, notify staff, spend resources, or if the request is ambiguous.

For clearly requested, low-risk task creation with all inputs present, proceed after the preview according to the user's intent.

## Inspecting Workflow Tasks

Use:

```bash
biolab tasks workflow <TASK_ID> -f json
```

This is the preferred detail view for multi-stage tasks because it includes:

- root task metadata
- stage list
- dependencies
- assignments

Do not rely on `biolab tasks get` alone when the user wants workflow structure.

## Reading Results

Use:

```bash
biolab tasks results <TASK_ID> -f json
```

Behavior:

- single-stage compute task: reads compute output
- single-stage staff task: reads submitted results
- workflow task: groups output by stage

For workflow tasks, results are stage-aware:

- compute stages read from `part.output_data`
- staff stages read from submitted results linked by `part_id`

If the user asks why a workflow task appears empty, inspect both the workflow structure and the stage result shape before assuming a backend bug.

## Examples

### Single-stage task

User: `帮我建一个样品 QC 任务`

Workflow:

1. Run `biolab tasks types -f json`.
2. Match the best single task type.
3. Inspect required fields such as `sample_ids`.
4. Ask for missing inputs.
5. Create with `biolab tasks create`.

### Multi-stage workflow task

User: `帮我建一个先算 Tm 再做人工 QC 的任务`

Workflow:

1. Run `biolab tasks types -f json`.
2. Match one compute task type for Tm and one staff task type for QC.
3. Collect the sequence, sample identifiers, and assignee if needed.
4. Build a workflow payload with two parts and one dependency.
5. Create with `biolab tasks create-workflow`.

### No match

User: `帮我做质谱分析`

If no task type mentions mass spectrometry or related schema fields, explain that no suitable task type is currently available and do not create a generic placeholder unless the user explicitly asks for a manual or custom task and the API supports it.

## Output

After task creation, report:

- task id
- title
- status
- task type for single-stage tasks, or stage summary for workflow tasks

Use `biolab tasks get <TASK_ID> -f json` only if the create response is missing important fields.
