# Templates Reference

Use this reference before creating or applying order-info templates.

## Core Concepts

Templates store recurring order metadata such as company, invoice title, PI, payment method, recipient address, and recurring order notes. Contact fields belong to orders, not templates.

## Recommended Commands

```bash
biolab templates list -f json
biolab templates get <TEMPLATE_ID> -f json
biolab templates get-default primer_synthesis -f json
biolab templates create template.json
biolab templates update <TEMPLATE_ID> template.json
biolab templates set-default <TEMPLATE_ID>
```

## Schema Check

Before creating or updating templates, inspect `OrderInfoTemplateCreate` / `OrderInfoTemplateUpdate` from the backend OpenAPI schema:

```text
<BIOLAB_BASE_URL>/openapi.json
```

## Agent Rules

- Prefer reading the default template before creating an order JSON.
- If no default template exists for an order type, ask the user whether to create one before building an order JSON.
- Preserve user-provided `notes` for recurring supplier instructions.
- Confirm before creating templates, deleting templates, or changing the default template.
- Treat template data as defaults, not as guaranteed final order values; user-provided values win.
- Use `biolab me -f json` to prefill order contact fields when the user agrees; do not save those contact fields into `OrderInfoTemplateCreate` unless the OpenAPI schema adds them.

## First-Time Setup Workflow

When `biolab templates get-default <ORDER_TYPE> -f json` returns no default template and `biolab templates list -f json` has no suitable template:

1. Ask the user for company, invoice, PI, payment, recipient, and contact fields.
2. Optionally prefill order contact fields from `biolab me -f json`.
3. Write a template JSON file using the common fields below.
4. Run `biolab templates create template.json -f json`.
5. Ask for confirmation, then run `biolab templates set-default <TEMPLATE_ID>`.

## Common Fields

`OrderInfoTemplateCreate` currently requires only `name`. Optional fields include `order_type`, `is_default`, `principal_investigator`, `company_name`, `company_phone`, `invoice_title`, `payment_method`, `recipient_address`, and `notes`.

```json
{
  "name": "Default primer order",
  "order_type": "primer_synthesis",
  "is_default": true,
  "company_name": "Company",
  "company_phone": "021-00000000",
  "invoice_title": "Invoice title",
  "principal_investigator": "PI",
  "payment_method": "bank_transfer",
  "recipient_address": "Address",
  "notes": "Special supplier instructions"
}
```
