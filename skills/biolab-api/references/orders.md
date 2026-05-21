# Orders Reference

Use this reference before creating, updating, downloading, or resending Biolab orders.

## Core Concepts

- `primer_synthesis`: primer synthesis order.
- `sequencing`: sequencing order.
- Status values come from `OrderStatus`: `draft`, `pending_approval`, `approved`, `pending`, `ordered`, `received`, `stored`.
- Suppliers currently mentioned by project docs: `sangon`, `biosune`.

## Recommended Commands

```bash
biolab orders list -f json
biolab orders get <ORDER_ID> -f json
biolab templates get-default primer_synthesis -f json
biolab templates list -f json
biolab templates get <TEMPLATE_ID> -f json
biolab orders create-primer order.json
biolab orders create-sequencing order.json
biolab orders update <ORDER_ID> '{"status":"received"}'
biolab orders download <ORDER_ID> order.xlsx
```

## Schema Check

Before creating or updating order JSON, inspect the backend OpenAPI schema instead of guessing request fields:

```text
<BIOLAB_BASE_URL>/openapi.json
```

Use `PrimerOrderCreate`, `PrimerItemCreate`, `SequencingOrderCreate`, and `OrderUpdate`. For order-info templates, use `OrderInfoTemplateCreate`.

## Order Template Workflow

Before creating a primer synthesis order, check the user's default order-info template:

```bash
biolab templates get-default primer_synthesis -f json
```

If there is no default template, list templates and inspect the user's chosen template:

```bash
biolab templates list -f json
biolab templates get <TEMPLATE_ID> -f json
```

If no suitable template exists, ask the user for the template fields described in `references/templates.md`, create it, and set it as default:

```bash
biolab templates create template.json
biolab templates set-default <TEMPLATE_ID>
```

If there is no template and the user asks to reuse previous order information, inspect an existing order only when one is available:

```bash
biolab orders list -f json
biolab orders get <ORDER_ID> -f json
```

Show the selected template's key order information to the user for confirmation before creating the order. Use the confirmed template as defaults for company, invoice, PI, payment, recipient, and recurring notes. Contact fields such as `customer_name`, `customer_phone`, and `customer_email` are order fields, not template fields; prefill them from `biolab me -f json` only after user confirmation. User-provided order values always override template defaults.

## Primer Order JSON Workflow

After the template is confirmed, collect the primer-specific fields from the user:

- `supplier_name`
- `customer_name`, `customer_phone`, and `customer_email`
- order-level optional fields from `PrimerOrderCreate`, such as `supplier_email`, `company_phone`, `total_price`, `notes`, `confidential`, `weekend_delivery`, and `partial_delivery`
- one or more `items` matching `PrimerItemCreate`

Build the `orders create-primer` JSON by merging the confirmed template defaults with the primer item fields. Preserve `notes` from the template unless the user overrides them for the order. Show the final JSON summary to the user and get explicit confirmation before running:

```bash
biolab orders create-primer order.json
```

## Agent Rules

- Inspect and confirm the relevant order-info template before writing a create JSON file.
- Prefer `-f json` when reading order details for follow-up automation.
- Confirm user intent before updating order status, resending mail, or creating an order.
- For Excel workflows, use download/upload template commands instead of inventing spreadsheet columns.

## Minimal Primer Order Shape

Check OpenAPI before using this shape. At the time of writing, `PrimerItemCreate` requires only `primer_name` and `sequence`; all other item fields are optional.

```json
{
  "type": "primer_synthesis",
  "supplier_name": "sangon",
  "customer_name": "Name",
  "customer_phone": "13800000000",
  "customer_email": "name@example.com",
  "notes": "Special supplier instructions",
  "items": [
    {
      "primer_name": "FWD",
      "sequence": "ATGC"
    }
  ]
}
```

Optional `PrimerItemCreate` fields currently include:

- `base_count`
- `purification_method`
- `nmoles`
- `scale_od`
- `tube_count`
- `deliverable_form`
- `five_modification`
- `three_modification`
- `double_modification`
- `primer_type`
- `remarks`
