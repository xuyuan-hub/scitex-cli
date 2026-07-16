---
name: scitex-orders
version: 0.1.0
description: "Use when creating, listing, inspecting, updating, downloading, uploading, approving, or placing Scientex primer synthesis or sequencing orders, including primer purchase workflows."
metadata:
  requires:
    bins: ["scitex"]
  cliHelp: "scitex orders --help"
---

# Scientex Orders

**Before starting, read `../scitex-shared/SKILL.md` for auth, safety, and OpenAPI rules.**

Use `../scitex-templates/SKILL.md` whenever an order should use saved order-info defaults.

## Commands

```bash
scitex orders list --order-type primer_synthesis --status pending_approval --supplier-name sangon -f json
scitex orders pending-approvals -f json
scitex orders get <ORDER_ID> -f json
scitex orders create-primer order.json -f json
scitex orders create-sequencing order.json -f json
scitex orders update <ORDER_ID> '{"status":"received"}' -f json
scitex orders download <ORDER_ID> order.xlsx
scitex orders download-primer-template primer_template.xlsx
scitex orders download-sequencing-template sequencing_template.xlsx
scitex orders upload-primer-excel primer.xlsx -f json
scitex orders upload-sequencing-excel sequencing.xlsx -f json
scitex orders resend <ORDER_ID> -f json
scitex orders send <ORDER_ID> -f json
scitex orders approve <ORDER_ID> -f json
scitex orders reject <ORDER_ID> -f json
```

Use `--price-min` / `--price-max` and `--date-from` / `--date-to` when the user supplies those bounds. Prefer `pending-approvals` for the current user's approval queue instead of scanning all orders.

## Schema

Inspect `<SCIENTEX_BASE_URL>/openapi.json` before writing order JSON.

Relevant schemas:

- `PrimerOrderCreate`
- `PrimerItemCreate`
- `SequencingOrderCreate`
- `SequencingItemCreate`
- `OrderUpdate`

`OrderType`: `primer_synthesis`, `sequencing`

`OrderStatus`: `draft`, `pending_approval`, `approved`, `pending`, `ordered`, `received`, `stored`

## Primer Purchase Workflow

### Step 1: Collect primer information — two paths

Ask the user how they want to provide primer information: Excel upload or manual input.

**Path A — Excel upload**

1. Run `scitex orders download-primer-template primer_template.xlsx` to get the standard primer ordering spreadsheet.
2. User fills out the template or their own Excel file.
3. Run `scitex orders upload-primer-excel <file.xlsx> -f json` to parse and validate.
4. Review the parsed items with the user, then proceed to Step 2.

**Path B — Provide primer fields for manual input**

Present the user with the complete `PrimerItemCreate` field reference below. Ask them to provide primer information in any format (text, chat, JSON). Build the items array from their input, then proceed to Step 2.

#### `PrimerItemCreate` — Complete Field Reference

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `primer_name` | string (max 255) | **Yes** | Primer identifier, e.g. `FWD-JK968` |
| `sequence` | string (max 5000) | **Yes** | Nucleotide sequence, e.g. `ATGCGATCGATCGATCGA` |
| `base_count` | integer \| null | No | Length of the sequence in bases |
| `purification_method` | string \| null | No | e.g. `Desalt`, `HPLC`, `PAGE` |
| `nmoles` | number \| string \| null | No | Synthesis scale in nanomoles |
| `scale_od` | number \| string \| null | No | OD scale value |
| `tube_count` | integer \| null | No | Number of tubes requested |
| `deliverable_form` | string \| null | No | e.g. `Dry`, `Liquid` |
| `five_modification` | string (max 128) \| null | No | 5' end modification |
| `three_modification` | string (max 128) \| null | No | 3' end modification |
| `double_modification` | string (max 128) \| null | No | Dual-end modification |
| `primer_type` | string (max 100) \| null | No | e.g. `PCR`, `Sequencing` |
| `remarks` | string (max 1000) \| null | No | Free-text notes |

Only `primer_name` and `sequence` are required. All other fields are optional.

### Step 2: Confirm order-level fields

1. Read the default `primer_synthesis` template through `scitex-templates`.
2. Confirm contact fields, usually from `scitex me -f json`.
3. Confirm supplier (`sangon` or `biosune`).
4. Preserve order-level `notes` from the confirmed template unless the user overrides.

#### `PrimerOrderCreate` — Order-Level Fields

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `type` | string | `primer_synthesis` | Must be `primer_synthesis` |
| `status` | OrderStatus | `pending` | Initial order status |
| `lab_id` | uuid \| null | | Lab to place the order under |
| `supplier_name` | string \| null | | `sangon` or `biosune` |
| `supplier_email` | string \| null | | Supplier contact email |
| `customer_name` | string \| null | | Contact person name |
| `customer_phone` | string \| null | | Contact phone |
| `customer_email` | string \| null | | Contact email |
| `company_name` | string \| null | | Company for invoicing |
| `company_phone` | string \| null | | Company phone |
| `invoice_title` | string \| null | | Invoice title |
| `principal_investigator` | string \| null | | PI name |
| `payment_method` | string \| null | | Payment method |
| `recipient_address` | string \| null | | Shipping address |
| `total_price` | number \| string \| null | | Total order price |
| `notes` | string \| null | | Order notes |
| `confidential` | boolean | `false` | Confidential flag |
| `primer_count` | integer | `0` | Number of primers |
| `total_bases` | integer | `0` | Total base count |
| `weekend_delivery` | boolean | `false` | Weekend delivery flag |
| `partial_delivery` | boolean | `false` | Partial delivery flag |
| `items` | PrimerItemCreate[] | `[]` | Primer items |

### Step 3: Build, confirm, and submit

1. Build the complete order JSON combining: order-level fields + primer items.
2. Show the final summary to the user and ask for explicit confirmation.
3. Run `scitex orders create-primer order.json -f json`.

## Sequencing Order Workflow

Use the same confirmed-template and contact-data flow for sequencing, but do not reuse primer item fields. Either download/upload the sequencing Excel template or inspect `SequencingOrderCreate` and `SequencingItemCreate`, review the parsed/constructed sequencing items with the user, then explicitly confirm and run `scitex orders create-sequencing order.json -f json`.

## Rules

- Confirm before creating an order, changing order status, resending email, submitting for approval, approving, rejecting, or placing an order.
- Prefer Excel upload/download commands when the user is working from supplier spreadsheets.
- Do not assume supplier-specific fields; check OpenAPI and user-provided files.
