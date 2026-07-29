# SeedLot Weight Ledger

Read this reference for formal post-intake inventory. `seed stocks` is a read-only migration view; use `seed lots` for new inventory operations.

```bash
scitex project tashan seed lots list [--type CODE] [--all] -f json
scitex project tashan seed lots get <LOT_ID> -f json
scitex project tashan seed lots movements <LOT_ID> -f json
scitex project tashan seed lots reservations <LOT_ID> -f json
scitex project tashan seed lots reserve <LOT_ID> --weight-g <DECIMAL_G> --yes -f json
scitex project tashan seed reservations release <RESERVATION_ID> --yes -f json
scitex project tashan seed lots checkout <LOT_ID> --weight-g <DECIMAL_G> [--reservation ID] --yes -f json
scitex project tashan seed lots transfer <LOT_ID> [--location-id ID] [--site TEXT] [--location-text TEXT] [--note TEXT] --yes -f json
scitex project tashan seed lots adjust <LOT_ID> --type <adjustment|loss|migration_correction> --weight-delta-g <DECIMAL_G> --reason TEXT --yes -f json
```

All weights are decimal strings in grams with at most four decimal places. Do not use floating-point arithmetic or maintain a local balance cache. Read the lot, movement, or reservation before proposing a write.

For each state-changing action, state and confirm the exact lot, decimal weight, reservation (when applicable), target placement, movement type, and reason. `reserve`, `release`, `checkout`, `transfer`, and `adjust` require `--yes` in non-interactive execution. Report server-returned movement, reservation, placement identifiers, and balances rather than calculating them locally.
