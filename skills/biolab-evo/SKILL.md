---
name: biolab-evo
description: "Use when operating Biolab evo compute TaskTypes for molecular design workflows: Tm calculation, codon optimization, NGS primer design/verification, barcode checks, complete primer assembly, correspondence files, and EXP2 BsaI Golden Gate primer/library design with plasmid file inputs."
---

# Biolab Evo Workflows

Use this skill for evo compute tasks in the Biolab task scheduling system. Before API calls, read `../biolab-shared/SKILL.md`. For generic non-evo task routing, use `../biolab-task/SKILL.md`.

## Core Rule

Always query the live task type before creating a task:

```bash
biolab tasks types --search <keyword> -f json
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

Use `biolab tasks create <json_file> -f json` for JSON-only tasks.

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
biolab tasks create task.json --file-field plasmid=path/to/file.dna -f json
```

Rules:

- The left side of `--file-field` must equal the file key in `input_schema`, such as `plasmid`.
- Do not put the file field itself in `input_data`; the server will insert a `FileFieldRef` with `storage_key`, `filename`, `content_type`, `size`, and `document_id`.
- Multiple file fields are allowed by repeating `--file-field key=path`.
- If a task already exists and a file field must be uploaded separately, use:

```bash
biolab tasks upload-field <task_id> <file_path> <field_key> -f json
```

## EXP2 Primer Design

For `evo-design-exp2-primers`, first search:

```bash
biolab tasks types --search exp2 -f json
```

Required inputs normally include:

- `plasmid`: file field, commonly `.dna`, `.gb`, `.fasta`, `.fa`
- `gene`
- `aa_start`
- `aa_end`

Useful optional inputs include `output_dir`, `mode`, `max_oligos`, `target_tm`, `gpu`, `top_positions`, `top_mutations`, `max_variants`, `max_library_size`, `batch_size`, `seed`, and `use_mixture`; only include fields needed by the request or defaults you intentionally override.

Example payload:

```json
{
  "title": "Design EXP2 primers for CasY7 aa 1-47",
  "description": "Design degenerate DNA library and BsaI Golden Gate primers from CasY7 plasmid file.",
  "task_type_id": "<evo-design-exp2-primers id>",
  "input_data": {
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
biolab tasks create task.json --file-field plasmid=data/evo/Y70001_CasY7_plasmid.dna -f json
```

This is a GPU/long-running task. Confirm before starting if the user did not clearly ask to submit it.

## Results

Check status:

```bash
biolab tasks get <task_id> -f json
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
