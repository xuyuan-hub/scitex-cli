---
name: scitex-project
description: "Use when listing projects, viewing project details by slug, or delegating to a project-specific skill (e.g. scitex-tashan). Do not invent project commands — follow the target project's skill."
metadata:
  requires:
    bins: ["scitex"]
  cliHelp: "scitex project --help"
---

# Scientex Project Workflows

**Before starting, read `../scitex-shared/SKILL.md` for auth, safety, and OpenAPI rules.**

This skill covers generic project administration. For project-specific workflows, use the corresponding `scitex-<project>` skill.

## Project Administration

```bash
scitex projects list -f json
scitex projects get <PROJECT_ID> -f json
scitex projects create '<JSON>' -f json
scitex projects update <PROJECT_ID> '<JSON>' -f json
scitex projects members <PROJECT_ID> -f json
scitex projects add-member <PROJECT_ID> <USER_ID> [--role member] -f json
scitex projects remove-member <PROJECT_ID> <USER_ID> -f json
```

## Project by Slug

```bash
scitex project <SLUG> info -f json
```

If the user already knows the slug, query `scitex project <SLUG> info -f json` directly. Otherwise use `projects list` to identify it, then check its dedicated skill (e.g. `scitex-tashan`) for available workflow commands.

## Rules

- Do not invent commands — check `scitex project <SLUG> --help` or the project-specific skill.
- Prefer `-f json` for machine-parsable output.
- Confirm before creating or updating project records, or changing project membership.
- For project-specific workflows, delegate to the corresponding `scitex-<project>` skill.
