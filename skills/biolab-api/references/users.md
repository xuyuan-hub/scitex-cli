# Users Reference

Use this reference before reading or updating the authenticated user's account.

## Recommended Commands

```bash
biolab me -f json
biolab me update '{"phone_number":"13800000000"}'
biolab me change-password --current <OLD_PASSWORD> --new <NEW_PASSWORD>
biolab status
biolab logout
```

## Schema Check

Before updating profile or password fields, inspect the backend OpenAPI schema:

```text
<BIOLAB_BASE_URL>/openapi.json
```

Current CLI commands map to these backend schemas:

- `UserUpdateMe`: optional `full_name`, `email`, `phone_number`
- `UpdatePassword`: requires `current_password`, `new_password`

## Agent Rules

- Do not print tokens, passwords, or secrets.
- Prefer `biolab status` before assuming the token is valid.
- Confirm before updating profile fields.
- Never ask the user to paste a password into chat unless the user explicitly chooses that flow; prefer interactive/manual handling for password changes.
- Do not invent admin user-management commands from backend `users` endpoints; this skill's user workflow is scoped to the authenticated account unless the CLI exposes more commands.
