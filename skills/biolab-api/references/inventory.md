# Inventory Reference

Use this reference before checking stock, moving stock in or out, or changing storage locations.

## Core Concepts

- Stock represents an available primer/material quantity.
- Transactions record `checkin` and `checkout` movements.
- Storage locations may be hierarchical through `parent_id`.
- Storage preferences exist in the backend OpenAPI, but the current CLI does not expose preference commands.

## Recommended Commands

```bash
biolab inventory list -f json
biolab inventory list --low-stock -f json
biolab inventory get <STOCK_ID> -f json
biolab inventory stats -f json
biolab inventory checkin <STOCK_ID> --quantity 5 --purpose "restock"
biolab inventory checkout <STOCK_ID> --quantity 2 --purpose "PCR" --experiment-ref "EXP-001"
biolab inventory locations -f json
biolab inventory create-location "Box A" --parent-id <LOCATION_ID>
```

## Schema Check

Before preparing inventory JSON or interpreting fields, inspect the backend OpenAPI schema:

```text
<BIOLAB_BASE_URL>/openapi.json
```

Current CLI commands map to these backend schemas:

- `CheckinRequest`: requires `quantity`; optional `purpose`
- `CheckoutRequest`: requires `quantity`; optional `purpose`, `experiment_ref`
- `StorageLocationCreate`: requires `name`; optional `parent_id`
- `StoragePreferenceCreate`: backend-only in the current CLI

## Agent Rules

- Confirm user intent before checkin/checkout because these mutate inventory.
- Quantity must be a positive number.
- Use `inventory get` before checkout when the current remaining quantity matters.
- Include `experiment_ref` for checkout whenever the user provides experiment context.
- Do not invent CLI commands for backend endpoints that are not exposed by `biolab inventory --help`.
