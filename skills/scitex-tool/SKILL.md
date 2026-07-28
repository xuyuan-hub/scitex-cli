---
name: scitex-tool
description: "Use when discovering, validating, submitting, rerunning, or reviewing a published Scientex Tool Catalog entry. Keeps execution on the public Tool Catalog contract and requires validation before a run."
metadata:
  requires:
    bins: ["scitex"]
  cliHelp: "scitex tool --help"
---

# Scientex Published Tool Catalog

**Before starting, read `../scitex-shared/SKILL.md` for authentication, confirmation, and OpenAPI rules.**

Use this skill only for published, user-visible Tool Catalog entries. Do not use administrator Tool Catalog routes and do not infer or request Worker queues, container images, entrypoints, host paths, or execution configuration.

## Discover and inspect

Search first, then inspect a selected key and version before preparing inputs:

```bash
scitex tool search --query <TEXT> --domain <DOMAIN> --family <FAMILY> --tag <TAG> -f json
scitex tool show <TOOL_KEY> -f json
```

The detail response contains the published version ID, parameter schema, artifact schema, licensing, citation, and submission availability. If no version is marked `submission_configured`, explain that submission is unavailable; do not attempt a run.

## Validate then run

Write one JSON object matching the chosen version's `parameter_schema` and validate it before execution:

```bash
scitex tool validate <TOOL_KEY> --version <TOOL_VERSION_ID> --input input.json -f json
scitex tool run <TOOL_KEY> --version <TOOL_VERSION_ID> --input input.json \
  --title "<TITLE>" [--description "<DESCRIPTION>"] [--lab-id <LAB_ID>] --yes -f json
```

`run` performs validation again in the same invocation and sends no run request when validation is unsuccessful. It requires interactive confirmation, or `--yes` in automation/non-interactive use. Present the exact tool key, version ID, manifest digest, input summary, expected artifacts, and static timeout before giving approval.

For a parameter whose schema has `format: "file"`, first upload a local file and copy the returned object exactly into `input.json`:

```bash
scitex files upload source.fasta -f json
```

Use that returned FileFieldRef object. The CLI rejects a local path, URL, bare storage key, or incomplete reference shape; it cannot independently attest where an otherwise valid-looking JSON object originated. Do not repeat storage keys in summaries or confirmations.

## Exact rerun and artifact review

An exact rerun starts from a visible immutable run on a Lab task part:

```bash
scitex tasks part <TASK_ID> <PART_ID> -f json
scitex tool rerun <TASK_ID> <PART_ID> <RUN_ID> [--lab-id <LAB_ID>] --yes -f json
```

Before rerunning, confirm that the selected run has published ToolVersion provenance, manifest/runtime digests, execution profile, normalized parameters, and the intended input artifacts. The CLI performs these checks and does not silently fall back to a newer version.

Use only bounded server previews to review declared result artifacts:

```bash
scitex tool artifact-preview <TASK_ID> <PART_ID> <RUN_ID> <ARTIFACT_INDEX> [--lab-id <LAB_ID>] -f json
```

Do not assume a submitted run has started or completed; tool dispatch and execution are asynchronous. Inspect the Lab task part or the returned task/part IDs for status.

## Safety rules

- Treat `validate` as read-only and `run`/`rerun` as state-changing.
- Never bypass validation, confirmation, version selection, or Lab scoping.
- Report backend validation errors as returned; do not invent status values or operational diagnoses.
- Never display tokens, storage keys, Worker commands, queues, runtime entrypoints, or host paths.
