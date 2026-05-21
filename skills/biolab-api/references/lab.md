# Lab Reference

Use this reference before managing lab membership, invitations, join applications, or approval rules.

## Core Roles

Roles are backend strings, currently accepted by request schemas as free-form strings with a maximum length. Common project roles include `pi`, `procurement`, `finance`, `warehouse`, and `member`; check current lab policy before assigning roles.

## Recommended Commands

```bash
biolab lab info -f json
biolab lab members -f json
biolab lab invite <email> member
biolab lab update-role <USER_ID> procurement
biolab lab invitations -f json
biolab lab applications -f json
biolab lab approval-rules -f json
```

## Schema Check

Before preparing lab JSON or role changes, inspect the backend OpenAPI schema:

```text
<BIOLAB_BASE_URL>/openapi.json
```

Current CLI commands map to these backend schemas:

- `LabCreate`: requires `name`
- `LabUpdate`: optional `name`, `require_approval`
- `LabMemberUpdate`: requires `role`
- `LabInviteRequest`: requires `email`; optional `role`
- `LabJoinRequest`: optional `role`
- `LabApprovalRuleCreate`: optional `order_type`, `max_price`, `approver_role`

## Agent Rules

- Confirm before removing members, changing roles, approving applications, rejecting applications, or editing approval rules.
- Use the lowest sufficient role when inviting or updating users.
- Read `lab members -f json` before changing a member role unless the user provides an exact `user_id`.
- Do not assume role hierarchy is enforced by the CLI; verify backend/lab policy before making role changes.

## Approval Rules

Approval rules are lab-level workflow configuration. Use `approval-rules` to inspect, `add-rule` with a JSON string matching `LabApprovalRuleCreate`, and `remove-rule` to delete a rule.
