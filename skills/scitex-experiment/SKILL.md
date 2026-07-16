---
name: scitex-experiment
description: "Use when planning or executing inventory-aware Scientex experiment work, including task-linked inventory checks, restocking decisions, and checkout during execution."
metadata:
  requires:
    bins: ["scitex"]
  cliHelp: "scitex inventory --help; scitex tasks --help"
---

# Scientex Experiment

Before API calls, read:

- `../scitex-shared/SKILL.md`
- `../scitex-inventory/SKILL.md`
- `../scitex-task/SKILL.md` when creating or executing tasks

## Core Workflow

1. Extract inventory requirements from the experiment plan. Give every requirement a stable `requirement_key`, such as `pcr.polymerase`, `pcr.dntp`, or `sequencing.primer_f`.
2. **Actively search** for each requirement in inventory. Start with an exact catalog number/name plus `--category`, `--supplier`, or filters when known; expand to Chinese/English synonyms only for zero or ambiguous results. For each requirement:

   a. Search for matching items:
   ```bash
   scitex inventory items --search "<term1>" -f json
   scitex inventory items --search "<term2>" -f json
   ```

   b. Once a candidate `item_id` is selected, check its stock without treating the result as a reservation:
   ```bash
   scitex inventory check requirements.json -f json
   ```

   c. If the item is found and has stock, record the item_id, stock batch id(s), remaining quantity, and unit. Re-search immediately before execution.

3. The LLM is responsible for matching: determine whether a search result really satisfies the requirement (name similarity, category match, unit compatibility). Do not expect exact name matches — inventory names may use Chinese, abbreviations, or supplier-specific naming.
4. If a requirement can be matched to in-stock items after thorough searching, mark it available.
5. If a requirement cannot be found after trying all reasonable search terms, mark it missing and report it. Do not claim the experiment is executable until all requirements are resolved.
6. For missing primer stock, use the primer order workflow when appropriate. For generic reagents/consumables, tell the user the current CLI does not create generic purchase orders unless such an order endpoint exists.
7. During execution, re-check inventory with active search and then checkout the actual consumed stock.

## Planning vs Execution

Planning phase:

- Actively search inventory for each requirement.
- Record matched item_ids, stock batch ids, quantities, units, and missing items in the plan.
- Do not mutate inventory.
- Do not treat stock discovery as reservation.

Execution phase:

- Re-search inventory as close as possible to physical execution.
- Use `checkout-item` when the item is known but no batch was chosen.
- Use `checkout` when a specific stock batch was selected.
- Include `experiment_ref`, `task_id`, `part_id`, and `requirement_key` on checkout whenever available.

Example:

```bash
scitex inventory checkout-item <ITEM_ID> \
  --quantity 10 \
  --purpose "PCR setup" \
  --experiment-ref EXP-20260615-001 \
  --task-id <TASK_ID> \
  --part-id <PART_ID> \
  --requirement-key pcr.dntp \
  -f json
```

## Ordering Rule

If stock is unavailable:

- Primer synthesis: use the Scientex orders skill/commands when the sequence and order information are available.
- Sequencing services: use sequencing order flow when the experiment requires sequencing service, not inventory checkout.
- Generic reagent or consumable: report the missing stock and ask the user how to order or restock unless a generic purchase endpoint exists in OpenAPI.

Never fake an order, reservation, or stock lock in the task payload.

## Task Integration

When creating experiment tasks:

- Include the inventory requirements in `input_data`.
- Include the active search results and stock matches in `input_data` or an attached plan when useful.
- Do not mark the task ready for execution if required inventory is unavailable.

When completing execution:

- Checkout consumed inventory first.
- Include checkout transaction IDs in the task result or experiment execution record.
- If checkout fails due to insufficient stock, stop execution and report the failing requirement.
