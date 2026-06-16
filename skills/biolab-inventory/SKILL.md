---
name: biolab-inventory
description: "Use when listing, checking, creating, adjusting, transferring, or consuming Biolab generic inventory: item definitions, stock batches, locations, stock transactions, experiment inventory checks, checkin, checkout, FIFO checkout by item, and task-linked inventory usage."
metadata:
  requires:
    bins: ["biolab"]
  cliHelp: "biolab inventory --help"
---

# Biolab Inventory

Before starting, read `../biolab-shared/SKILL.md` for auth, safety, and OpenAPI rules.

Use this skill for generic inventory, not only primer stock. The current CLI supports backend OpenAPI inventory endpoints that already exist; do not invent reservation, atomic lock, or purchase-order commands.

## Read Commands

```bash
biolab inventory items --search <NAME> --category <CATEGORY> --supplier <SUPPLIER> -f json
biolab inventory item <ITEM_ID> -f json
biolab inventory list --name <NAME> --location-id <LOCATION_ID> --low-stock -f json
biolab inventory list --primer-name <OLD_PRIMER_NAME> -f json
biolab inventory get <STOCK_ID> -f json
biolab inventory summary --search <NAME> --category <CATEGORY> -f json
biolab inventory transactions <STOCK_ID> -f json
biolab inventory transactions-all --type checkout --item-id <ITEM_ID> --search <NAME> -f json
biolab inventory locations -f json
biolab inventory preferences --workflow-type <TYPE> -f json
```

`--primer-name` is only a compatibility alias for old primer workflows. Prefer `--name` for generic inventory.

## Write Commands

Confirm with the user before mutating inventory unless the user explicitly asked to execute the mutation.

```bash
biolab inventory create-item item.json -f json
biolab inventory update-item <ITEM_ID> item_update.json -f json
biolab inventory disable-item <ITEM_ID> -f json
biolab inventory create-stock stock.json -f json
biolab inventory checkin <STOCK_ID> --quantity <QTY> --purpose "<WHY>" -f json
biolab inventory checkout <STOCK_ID> --quantity <QTY> --recipient "<WHO>" --purpose "<WHY>" --experiment-ref <EXP> --task-id <TASK_ID> --part-id <PART_ID> --requirement-key <KEY> -f json
biolab inventory checkout-item <ITEM_ID> --quantity <QTY> --purpose "<WHY>" --experiment-ref <EXP> --task-id <TASK_ID> --part-id <PART_ID> --requirement-key <KEY> -f json
biolab inventory adjust <STOCK_ID> --quantity -1 --type loss --reason "<REASON>" -f json
biolab inventory transfer <STOCK_ID> --location-id <LOCATION_ID> --reason "<REASON>" -f json
biolab inventory create-location "<NAME>" --parent-id <PARENT_ID> -f json
```

Rules:

- `checkin`, `checkout`, and `checkout-item` quantities must be positive.
- `adjust` quantity may be positive or negative, but not zero. Allowed types are `correction`, `loss`, `damage`, and `expire`.
- Prefer `checkout-item` when an experiment specifies the item but not the exact batch; the backend performs FIFO stock-out.
- Prefer `checkout` when a specific stock batch was selected or physically used.
- Always include `experiment_ref` when there is experiment context.
- Always include `task_id`, `part_id`, and `requirement_key` when inventory is consumed for a scheduled experiment task.

## Inventory Check (LLM-Driven Active Search)

Do NOT use `biolab inventory check` as the primary discovery method. Its literal name matching cannot handle Chinese/English variations, abbreviations, or synonyms.

Instead, the LLM must actively search for all requirements in bulk using the `--filters` parameter with `like` operator and `or` combine logic. This replaces many individual `--search` calls with 1-2 queries.

### Filter Schema

```text
--filters 'JSON [{field, operator, value, combine}]'
```

**Operators:** `eq`/`equal`, `neq`/`not_equal`, `lt`/`less_than`, `gt`/`greater_than`, `lte`/`less_or_equal`, `gte`/`greater_or_equal`, `like`

**Combine:** `and`, `or`

**Fields (items):** `name`, `category`, `supplier`, `catalog_number`, `unit`, `specification`, `is_active`, `created_at`, `updated_at`

**Fields (summary):** `name`, `category`, `supplier`, `catalog_number`, `unit`, `specification`, `is_active`, `created_at`

`like` uses SQL `LIKE` patterns: `%word%` for substring match, `word%` for prefix, `%word` for suffix.

### Step 1 — Search for ALL items with one OR-combined query

Build a single filter array with one entry per search term, all combined with `or`:

```bash
biolab inventory items --filters '[
  {"field":"name","operator":"like","value":"%连接酶%","combine":"or"},
  {"field":"name","operator":"like","value":"%EcoRI%","combine":"or"},
  {"field":"name","operator":"like","value":"%内切酶%","combine":"or"},
  {"field":"name","operator":"like","value":"%聚合酶%","combine":"or"},
  {"field":"name","operator":"like","value":"%感受态%","combine":"or"},
  {"field":"name","operator":"like","value":"%DNA%","combine":"or"},
  {"field":"name","operator":"like","value":"%胶回收%","combine":"or"},
  {"field":"name","operator":"like","value":"%培养基%","combine":"or"}
]' -f json
```

Cover each requirement with multiple search terms (Chinese, English, abbreviation). The LLM is responsible for picking a diverse set of terms that span all requirements in one query.

### Step 2 — Check stock for matched items in bulk

Use the same OR-combined filter pattern to get stock summaries for every matched item at once:

```bash
biolab inventory summary --filters '[
  {"field":"name","operator":"like","value":"%连接酶%","combine":"or"},
  {"field":"name","operator":"like","value":"%XhoI%","combine":"or"},
  {"field":"name","operator":"like","value":"%高保真%","combine":"or"},
  {"field":"name","operator":"like","value":"%FastPure%","combine":"or"}
]' -f json
```

Use the actual matched item names from Step 1 to narrow the summary filter terms.

### Step 3 — LLM judges the match

The LLM decides whether a search result satisfies the requirement:
- Name similarity (Chinese ↔ English, partial match, supplier naming)
- Category match
- Unit compatibility (flag mismatches; do not auto-convert)
- Stock sufficiency

### Step 4 — Report

For each requirement, report: matched item, stock batch(es), remaining quantity, unit. If a requirement cannot be found after trying all reasonable search terms, mark it as missing.

### Why not `biolab inventory check`

The aggregate `biolab inventory check requirements.json` command uses literal name matching. It will miss items whose inventory names differ from the requirement name — for example, searching "T4 DNA Ligase" will not find "T4 DNA连接酶", and searching "XhoI" will not find "XhoI内切酶". LLM-driven bulk search with `like` + `or` is the correct approach.

## Experiment Rule

For planning: search inventory only. Do not checkout while merely drafting a plan.

For execution: re-search inventory immediately before physical execution, then run `checkout` or `checkout-item` for each consumed requirement and record the task/part/requirement fields.

If stock is missing or insufficient, stop before creating a task marked executable. Move the missing material into the ordering workflow if the available order APIs support that material type.
