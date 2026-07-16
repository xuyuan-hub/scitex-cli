---
name: scitex-templates
description: "Use when listing, creating, updating, deleting, selecting, or retrieving Scientex order-info templates for primer synthesis or sequencing orders, then applying their defaults while composing an order."
metadata:
  requires:
    bins: ["scitex"]
  cliHelp: "scitex templates --help"
---

# Scientex Templates

**Before starting, read `../scitex-shared/SKILL.md` for auth, safety, and OpenAPI rules.**

## Commands

```bash
scitex templates list -f json
scitex templates get <TEMPLATE_ID> -f json
scitex templates get-default primer_synthesis -f json
scitex templates create template.json -f json
scitex templates update <TEMPLATE_ID> template.json -f json
scitex templates delete <TEMPLATE_ID> -f json
scitex templates set-default <TEMPLATE_ID> -f json
```

## Schema

Inspect `<SCIENTEX_BASE_URL>/openapi.json` before writing template JSON.

Relevant schemas:

- `OrderInfoTemplateCreate`
- `OrderInfoTemplateUpdate`
- `OrderInfoTemplateResponse`

At the time of writing, `OrderInfoTemplateCreate` requires only `name`. Optional fields include `order_type`, `is_default`, `principal_investigator`, `company_name`, `company_phone`, `invoice_title`, `payment_method`, `recipient_address`, and `notes`.

Contact fields such as `customer_name`, `customer_phone`, and `customer_email` belong to orders, not templates.

## First-Time Order Template Workflow

1. Run `scitex templates get-default <ORDER_TYPE> -f json`.
2. If no default exists, run `scitex templates list -f json`.
3. If no suitable template exists, ask the user for template fields.
4. Create the template JSON from `OrderInfoTemplateCreate`.
5. Confirm with the user.
6. Run `scitex templates create template.json -f json`.
7. If needed, retrieve `scitex templates get <TEMPLATE_ID> -f json`, verify its order type and contents, then run `scitex templates set-default <TEMPLATE_ID> -f json`.

## Rules

- Confirm before creating, updating, deleting, or changing the default template.
- Preserve user-provided `notes` for recurring supplier instructions.
- Treat template data as defaults; user-provided order fields override template fields.
