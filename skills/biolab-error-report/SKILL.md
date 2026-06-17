---
name: biolab-error-report
version: 0.1.0
description: "Use when the user encounters a Biolab CLI error and wants to submit an error report, or when the CLI detects repeated errors and suggests reporting. Categorizes errors into ui-display, functional, data, performance, permission, or other."
metadata:
  requires:
    bins: ["biolab"]
  cliHelp: "biolab error-report --help"
---

# Biolab Error Report

Before using, read `../biolab-shared/SKILL.md` to ensure the CLI is set up and authenticated.

## When to Use

Use this skill when:

- The user wants to report a Biolab CLI bug, crash, or unexpected behavior.
- The CLI has detected repeated errors and suggests submitting a report.
- The user describes an error pattern and asks to "report it" or "send feedback."

## Command

```bash
biolab error-report \
  --category <CATEGORY> \
  --title "<TITLE>" \
  --description "<DESCRIPTION>" \
  --url "<URL>"  # optional
```

## Error Categories

| Category CLI Flag | API Value | Use When |
|---|---|---|
| `ui-display` | `ui_display` | Terminal output is garbled, misaligned, or truncated |
| `functional` | `functional` | HTTP 4xx/5xx, API returns a business error, command behaves unexpectedly |
| `data` | `data` | JSON parse failure, missing/malformed fields, type mismatch |
| `performance` | `performance` | Timeout, slow download, excessive latency |
| `permission` | `permission` | 401/403, NotAuthenticated, insufficient role |
| `other` | `other` | Anything that does not fit the above |

## Guidelines

1. **Infer the category** from the error message:
   - `<status> <method> <path>` → `functional` (e.g. "422 POST /api/v1/orders/...")
   - `401` / `403` / `NotAuthenticated` → `permission`
   - `Failed to parse` / `missing field` / `Serde` → `data`
   - `timeout` / `Timeout` → `performance`
2. **Title** should be concise: `"<command>: <short error>"`, at most ~80 chars.
3. **Description** should include:
   - Exact command that was run (sanitized: remove `--token` values, passwords, file paths)
   - Full error message
   - CLI version (from `biolab --version`)
   - OS the user is on
4. **Use `-f json`** to capture the report ID on success.
5. If the user is not logged in (`biolab status` shows unauthenticated), remind them to login first — error reports require authentication.

## Example

```bash
biolab error-report \
  -c functional \
  -t "orders create: 422 when primer_name missing" \
  -d "Running 'biolab orders create --supplier sangon' without --primer-name returned HTTP 422 with no clear hint. CLI v0.4.10 on Windows." \
  -f json
```
