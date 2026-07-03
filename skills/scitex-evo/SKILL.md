---
name: scitex-evo
description: "Use when operating Scientex evo compute TaskTypes for molecular design workflows: Tm calculation, codon optimization, NGS primer design/verification, barcode checks, complete primer assembly, correspondence files, EXP2 BsaI Golden Gate primer/library design, and multi-stage evo workflows that chain compute TaskTypes with dependencies."
---

# Scientex Evo Workflows

Use this skill for evo compute tasks in the Scientex task scheduling system. Before API calls, read `../scitex-shared/SKILL.md`. For generic non-evo task routing, use `../scitex-task/SKILL.md`.

## Core Rule

Always query the live task type before creating a task:

```bash
scitex tasks types --search <keyword> -f json
```

Match by `key`, `display_name`, `description`, and `input_schema`. Prefer enabled compute TaskTypes. Do not hardcode stale TaskType IDs if a live search can find the type.

## Common Evo TaskTypes

| Key | Use |
| --- | --- |
| `evo-compute-tm` | Calculate DNA primer Tm. |
| `evo-codon-optimize` | Back-translate protein to optimized DNA while avoiding restriction sites. |
| `evo-design-ngs-primer` | Pick a primer from a DNA template near a target Tm. |
| `evo-verify-ngs-primer` | Check primer length, GC, Tm, and related NGS primer quality fields. |
| `evo-build-complete-primer` | Assemble Illumina adapter + barcode index + specific primer. |
| `evo-check-barcodes` | Check uniqueness and Hamming separation of barcode pairs. |
| `evo-build-correspondence` | Generate NGS handoff correspondence/barcodes outputs from window specs. |
| `evo-design-exp2-primers` | GPU workflow for ESM2 saturation scan, variant selection, degenerate library design, and BsaI Golden Gate primers. |

## Creating Tasks

Use `scitex tasks create <json_file> -f json` for JSON-only tasks.

The payload should follow the live `input_schema`:

```json
{
  "title": "<short title>",
  "description": "<optional description>",
  "task_type_id": "<live task type id>",
  "input_data": {}
}
```

Do not include `lab_id`; the CLI uses the current lab unless `--lab-id` is explicitly supplied.

## Evo Multi-stage Workflows

Use `../scitex-task/SKILL.md` plus this skill when the user asks to chain evo compute steps, such as "calculate Tm first, then design an NGS primer after that completes".

Rules:

- Query live TaskTypes for every stage, for example `scitex tasks types --search tm -f json` and `scitex tasks types --search ngs -f json`.
- Use `scitex tasks create-workflow <json_file> -f json`.
- Put `task_type_id` on each `parts[*]`; do not put a root-level `task_type_id` on workflow payloads.
- Give each stage a stable `client_key`.
- Add `dependencies` with `condition_type: "completed"` when a later stage should unlock only after an earlier stage completes.
- Include each stage's required `input_data` explicitly. Do not assume a previous stage's output is passed into the next stage unless the live schema or documented API explicitly supports output references.

Example: compute primer Tm, then unlock NGS primer design:

```json
{
  "title": "Compute Tm then design NGS primer",
  "description": "First calculate Tm, then design an NGS primer after the Tm stage completes.",
  "parts": [
    {
      "client_key": "compute_tm",
      "name": "Compute Tm",
      "task_type_id": "<evo-compute-tm id>",
      "input_data": {
        "sequence": "ATGGTCTCAGGAAACCTAGACCCAGAAAAACACGAATGG"
      }
    },
    {
      "client_key": "design_ngs_primer",
      "name": "Design NGS Primer",
      "task_type_id": "<evo-design-ngs-primer id>",
      "input_data": {
        "template": "ATGGACGCTTCCCCGAGCATCTCCCCATTCCATGAGCGGGGAAGCGTCCATTGGCTGCCTTTAAAGTGCAGAAGTCAGAA"
      }
    }
  ],
  "dependencies": [
    {
      "prerequisite_client_key": "compute_tm",
      "dependent_client_key": "design_ngs_primer",
      "condition_type": "completed"
    }
  ]
}
```

After creation, inspect progression with:

```bash
scitex tasks workflow <task_id> -f json
```

Expected compute workflow progression: dependent stages may start as `LOCKED`, become `READY` after prerequisites complete, then run automatically. Report per-stage status and `output_data.exit_code`; the root workflow status may lag behind completed compute stages.

## File Inputs

For TaskTypes with file fields, the `input_schema` marks them as:

```json
{
  "type": "object",
  "format": "file"
}
```

Create the task with multipart input:

```bash
scitex tasks create task.json --file-field plasmid=path/to/file.dna -f json
```

Rules:

- The left side of `--file-field` must equal the file key in `input_schema`, such as `plasmid`.
- Do not put the file field itself in `input_data`; the server will insert a `FileFieldRef` with `storage_key`, `filename`, `content_type`, `size`, and `document_id`.
- Multiple file fields are allowed by repeating `--file-field key=path`.
- If a task already exists and a file field must be uploaded separately, use:

```bash
scitex tasks upload-field <task_id> <file_path> <field_key> -f json
```

## EXP2 Primer Design

For `evo-design-exp2-primers`, first search:

```bash
scitex tasks types --search exp2 -f json
```

Required inputs normally include:

- `plasmid`: file field (`.dna` / `.gb` / `.fasta` / `.fa`), typed as `string` in schema
- `gene`
- `aa_start`
- `aa_end`

Because `plasmid` is typed as `string` (not `format: file`), you **must** include a placeholder value in `input_data` to pass validation:

```json
{
  "title": "Design EXP2 primers for CasY7 aa 1-47",
  "description": "Design degenerate DNA library and BsaI Golden Gate primers from CasY7 plasmid file.",
  "task_type_id": "<evo-design-exp2-primers id>",
  "input_data": {
    "plasmid": "placeholder",
    "gene": "CasY7",
    "aa_start": 1,
    "aa_end": 47,
    "output_dir": "out_CasY7_1_47",
    "mode": "zero_shot",
    "max_oligos": 10,
    "target_tm": 55.0,
    "gpu": 0
  }
}
```

Create it:

```bash
scitex tasks create task.json --file-field plasmid=data/evo/Y70001_CasY7_plasmid.dna -f json
```

**Known limitation**: The server replaces the root task's `input_data.plasmid` with a `FileFieldRef`, but the part's `input_data.plasmid` keeps the `"placeholder"` string. The worker reads from the part's input_data and may fail. This is a backend bug — file references are not propagated from root task to parts during multipart creation.

## Results

Check status:

```bash
scitex tasks get <task_id> -f json
```

If `status` is `completed` and `output_data.exit_code` is `0`, download files from `output_data.files[*].download_url`. Preserve filenames. Important EXP2 outputs often include:

- `result.json`
- `pipeline_summary.txt`
- `primers.csv`
- `oligos.csv`
- `selected_positions.csv`
- `position_analysis.csv`
- `variant_sequences.csv`
- `all_mutations.csv`
- `degenerate_summary.csv`
- `plasmid_info.json`
- `oligo*_details.csv`

If `output_data.exit_code` is nonzero, report the `error`, `stderr_log_url`, and task id.
