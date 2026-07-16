---
name: scitex-lab
version: 0.1.0
description: "Use when managing Scientex lab information, members, roles, invitations, join applications, approval settings, or lab approval rules."
metadata:
  requires:
    bins: ["scitex"]
  cliHelp: "scitex lab --help"
---

# Scientex Lab

**Before starting, read `../scitex-shared/SKILL.md` for auth, safety, and OpenAPI rules.**

## Commands

```bash
scitex lab info -f json
scitex lab create <NAME> -f json
scitex lab update '{"require_approval":true}' -f json
scitex lab members -f json
scitex lab update-role <USER_ID> member -f json
scitex lab remove-member <USER_ID> -f json
scitex lab invite <EMAIL> member -f json
scitex lab invitations -f json
scitex lab accept-invite <INVITATION_ID> -f json
scitex lab decline-invite <INVITATION_ID> -f json
scitex lab join <LAB_ID> member -f json
scitex lab applications -f json
scitex lab approve-app <APPLICATION_ID> -f json
scitex lab reject-app <APPLICATION_ID> -f json
scitex lab approval-rules -f json
scitex lab add-rule '{"order_type":"primer_synthesis","max_price":500,"approver_role":"pi"}' -f json
scitex lab remove-rule <RULE_ID> -f json
```

## Schema

Inspect `<SCIENTEX_BASE_URL>/openapi.json` before preparing lab JSON or role changes.

Relevant schemas:

- `LabCreate`: requires `name`
- `LabUpdate`: optional `name`, `require_approval`
- `LabMemberUpdate`: requires `role`
- `LabInviteRequest`: requires `email`; optional `role`
- `LabJoinRequest`: optional `role`
- `LabApprovalRuleCreate`: optional `order_type`, `max_price`, `approver_role`

Roles are backend strings. Common project roles include `pi`, `procurement`, `finance`, `warehouse`, and `member`; verify current lab policy before assigning roles.

## Rules

- Confirm before removing members, changing roles, approving applications, rejecting applications, or editing approval rules. First retrieve the invitation, application, member, or rule list and verify the exact target ID and current state.
- Use the lowest sufficient role.
- Read `lab members -f json` before changing a member role unless the user provides an exact `user_id`.
- Do not assume role hierarchy is enforced by the CLI.
