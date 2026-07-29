#!/usr/bin/env python3
"""Preflight a Tashan custom seed-type JSON payload without mutating server state."""

from __future__ import annotations

import argparse
import json
import re
import sys
import urllib.error
import urllib.request
from pathlib import Path
from typing import Any


KEY_PATTERN = re.compile(r"^[A-Za-z][A-Za-z0-9_]*$")
BASE_STRING_LIMITS = {
    "code": 32,
    "name": 255,
    "category": 64,
    "numbering_prefix": 16,
}
VALUE_TYPES = {
    "string",
    "integer",
    "decimal",
    "date",
    "enum",
    "multi_enum",
    "json_object",
}
REQUIRED_AT = {"optional", "preparing", "completion"}
EDITORS = {"leader", "staff", "both"}
KNOWN_ROLES = {
    "sample_name",
    "inventory_weight",
    "container",
    "seed_count",
    "source_parent",
    "generation",
    "storage_site",
    "storage_position",
    "intake_date",
    "operator",
}


def issue(issues: list[dict[str, str]], path: str, message: str) -> None:
    issues.append({"path": path, "message": message})


def is_number(value: Any) -> bool:
    return isinstance(value, (int, float)) and not isinstance(value, bool)


def read_json(source: str) -> Any:
    if source == "-":
        return json.load(sys.stdin)
    return json.loads(Path(source).read_text(encoding="utf-8"))


def read_openapi(source: str) -> dict[str, Any]:
    try:
        if source.startswith(("http://", "https://")):
            request = urllib.request.Request(source, headers={"Accept": "application/json"})
            with urllib.request.urlopen(request, timeout=15) as response:
                raw = response.read().decode("utf-8")
        else:
            raw = Path(source).read_text(encoding="utf-8")
    except (OSError, urllib.error.URLError, UnicodeDecodeError) as error:
        raise ValueError(f"无法读取 OpenAPI：{error}") from error

    try:
        document = json.loads(raw)
    except json.JSONDecodeError as error:
        raise ValueError(f"OpenAPI 不是有效 JSON：{error}") from error
    if not isinstance(document, dict):
        raise ValueError("OpenAPI 根节点必须是 JSON 对象")
    return document


def schema_from_openapi(document: dict[str, Any]) -> dict[str, Any]:
    try:
        schema = document["components"]["schemas"]["SeedObjectTypeConfigCreate"]
    except (KeyError, TypeError) as error:
        raise ValueError("OpenAPI 未包含 SeedObjectTypeConfigCreate") from error
    if not isinstance(schema, dict):
        raise ValueError("SeedObjectTypeConfigCreate 不是 JSON 对象")
    return schema


def validate_openapi_contract(
    payload: dict[str, Any], schema: dict[str, Any], errors: list[dict[str, str]], warnings: list[dict[str, str]]
) -> None:
    required = schema.get("required", [])
    if not isinstance(required, list):
        issue(warnings, "OpenAPI.required", "不是数组；已跳过动态必填字段检查")
    else:
        for field in required:
            if isinstance(field, str) and field not in payload:
                issue(errors, field, "为当前 OpenAPI 的必填字段")
        if "main_schema" not in required:
            issue(warnings, "OpenAPI.required", "当前 OpenAPI 未把 main_schema 标为必填；仍按种子字段合同进行本地预检")

    properties = schema.get("properties", {})
    if not isinstance(properties, dict):
        return
    for name, definition in properties.items():
        if name not in payload or not isinstance(definition, dict):
            continue
        max_length = definition.get("maxLength")
        if isinstance(max_length, int) and isinstance(payload[name], str) and len(payload[name]) > max_length:
            issue(errors, name, f"长度为 {len(payload[name])}，超过当前 OpenAPI 上限 {max_length}")


def validate_field_contract(payload: dict[str, Any], errors: list[dict[str, str]], warnings: list[dict[str, str]]) -> None:
    schema = payload.get("main_schema")
    if not isinstance(schema, dict):
        issue(errors, "main_schema", "必须是包含 fields 数组的对象")
        return

    schema_version = schema.get("schema_version")
    if schema_version is None:
        issue(warnings, "main_schema.schema_version", "建议显式使用当前字段合同版本 2")
    elif schema_version != 2:
        issue(warnings, "main_schema.schema_version", "当前项目文档使用版本 2；请确认后端是否支持该版本")

    fields = schema.get("fields")
    if not isinstance(fields, list) or not fields:
        issue(errors, "main_schema.fields", "必须是非空数组")
        return

    keys: set[str] = set()
    headers: set[str] = set()
    orders: set[int] = set()
    roles: dict[str, tuple[int, dict[str, Any]]] = {}

    for index, field in enumerate(fields):
        path = f"main_schema.fields[{index}]"
        if not isinstance(field, dict):
            issue(errors, path, "必须是对象")
            continue

        for required in ("key", "label", "header", "order", "value_type", "required_at", "editor"):
            if required not in field:
                issue(errors, f"{path}.{required}", "缺少必填字段")

        key = field.get("key")
        if not isinstance(key, str) or not KEY_PATTERN.fullmatch(key):
            issue(errors, f"{path}.key", "必须以英文字母开头，且只能包含字母、数字和下划线")
        elif key in keys:
            issue(errors, f"{path}.key", f"与另一字段重复：{key}")
        else:
            keys.add(key)

        for name in ("label", "header"):
            value = field.get(name)
            if not isinstance(value, str) or not value.strip():
                issue(errors, f"{path}.{name}", "必须是非空字符串")
        header = field.get("header")
        if isinstance(header, str) and header.strip():
            if header in headers:
                issue(errors, f"{path}.header", f"与另一字段重复：{header}")
            else:
                headers.add(header)

        order = field.get("order")
        if type(order) is not int or order < 0:
            issue(errors, f"{path}.order", "必须是大于等于 0 的整数")
        elif order in orders:
            issue(errors, f"{path}.order", f"与另一字段重复：{order}")
        else:
            orders.add(order)

        value_type = field.get("value_type")
        if value_type not in VALUE_TYPES:
            issue(errors, f"{path}.value_type", f"必须是以下之一：{', '.join(sorted(VALUE_TYPES))}")
        if field.get("required_at") not in REQUIRED_AT:
            issue(errors, f"{path}.required_at", f"必须是以下之一：{', '.join(sorted(REQUIRED_AT))}")
        if field.get("editor") not in EDITORS:
            issue(errors, f"{path}.editor", f"必须是以下之一：{', '.join(sorted(EDITORS))}")

        for name in ("employee_visible", "sensitive"):
            if name in field and not isinstance(field[name], bool):
                issue(errors, f"{path}.{name}", "必须是布尔值")
        if field.get("employee_visible") is True and field.get("editor") not in {"staff", "both"}:
            issue(errors, f"{path}.employee_visible", "员工可见字段的 editor 必须为 staff 或 both")
        if field.get("employee_visible") is True and field.get("sensitive") is True:
            issue(errors, path, "敏感字段不能同时对员工可见")
        if field.get("editor") in {"staff", "both"} and field.get("employee_visible") is not True:
            issue(warnings, path, "员工可编辑但未标记 employee_visible；该字段不会进入现场工单")

        validation = field.get("validation", {})
        if not isinstance(validation, dict):
            issue(errors, f"{path}.validation", "必须是对象")
            validation = {}
        for name in ("minimum", "maximum"):
            if name in validation and not is_number(validation[name]):
                issue(errors, f"{path}.validation.{name}", "必须是数值")
        if is_number(validation.get("minimum")) and is_number(validation.get("maximum")) and validation["minimum"] > validation["maximum"]:
            issue(errors, f"{path}.validation", "minimum 不能大于 maximum")
        if "precision" in validation and (type(validation["precision"]) is not int or validation["precision"] < 0):
            issue(errors, f"{path}.validation.precision", "必须是大于等于 0 的整数")
        if "max_length" in validation and (type(validation["max_length"]) is not int or validation["max_length"] < 1):
            issue(errors, f"{path}.validation.max_length", "必须是正整数")

        if value_type == "enum" and not isinstance(validation.get("allowed_values"), list):
            issue(errors, f"{path}.validation.allowed_values", "enum 字段必须提供允许值数组")
        if value_type == "multi_enum" and not isinstance(validation.get("multi_allowed_values"), list):
            issue(errors, f"{path}.validation.multi_allowed_values", "multi_enum 字段必须提供允许值数组")

        role = field.get("role")
        if role is not None:
            if not isinstance(role, str) or not role:
                issue(errors, f"{path}.role", "必须是非空字符串或 null")
            elif role in roles:
                issue(errors, f"{path}.role", f"与 fields[{roles[role][0]}] 重复绑定：{role}")
            else:
                roles[role] = (index, field)
                if role not in KNOWN_ROLES:
                    issue(warnings, f"{path}.role", "不是当前已识别的通用角色；不会映射到通用库存或追踪能力")

    required_roles = {
        "sample_name": {"required_at": "preparing"},
        "inventory_weight": {"value_type": "decimal", "required_at": "completion"},
        "container": {"required_at": "completion"},
    }
    for role, constraints in required_roles.items():
        if role not in roles:
            issue(errors, "main_schema.fields", f"缺少 role={role} 的字段")
            continue
        index, field = roles[role]
        for name, expected in constraints.items():
            if field.get(name) != expected:
                issue(errors, f"main_schema.fields[{index}].{name}", f"role={role} 必须为 {expected!r}")
        if role == "inventory_weight":
            validation = field.get("validation")
            if not isinstance(validation, dict) or not is_number(validation.get("minimum")) or validation["minimum"] < 0:
                issue(errors, f"main_schema.fields[{index}].validation.minimum", "库存重量必须设置大于等于 0 的最小值")


def validate(payload: Any, openapi: dict[str, Any] | None) -> dict[str, Any]:
    errors: list[dict[str, str]] = []
    warnings: list[dict[str, str]] = []
    if not isinstance(payload, dict):
        issue(errors, "$", "种子类型配置必须是 JSON 对象")
        return {"valid": False, "errors": errors, "warnings": warnings, "field_count": 0}

    for name, limit in BASE_STRING_LIMITS.items():
        value = payload.get(name)
        if not isinstance(value, str) or not value.strip():
            issue(errors, name, "必须是非空字符串")
        elif len(value) > limit:
            issue(errors, name, f"长度为 {len(value)}，超过上限 {limit}")

    validate_field_contract(payload, errors, warnings)
    if openapi is not None:
        validate_openapi_contract(payload, schema_from_openapi(openapi), errors, warnings)

    fields = payload.get("main_schema", {}).get("fields", []) if isinstance(payload.get("main_schema"), dict) else []
    return {
        "valid": not errors,
        "errors": errors,
        "warnings": warnings,
        "field_count": len(fields) if isinstance(fields, list) else 0,
        "openapi_checked": openapi is not None,
    }


def print_result(result: dict[str, Any], as_json: bool) -> None:
    if as_json:
        print(json.dumps(result, ensure_ascii=False, indent=2))
        return
    if result["valid"]:
        print(f"预检通过：{result['field_count']} 个字段。")
    else:
        print(f"预检失败：{len(result['errors'])} 个错误。")
    for item in result["errors"]:
        print(f"ERROR {item['path']}: {item['message']}")
    for item in result["warnings"]:
        print(f"WARNING {item['path']}: {item['message']}")


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description="预检 Tashan 自定义种子类型 JSON；不发送任何写请求。")
    parser.add_argument("config", help="配置 JSON 文件路径；使用 - 从标准输入读取")
    parser.add_argument(
        "--openapi",
        metavar="PATH_OR_URL",
        help="可选的最新 OpenAPI JSON 文件路径或 URL；校验 SeedObjectTypeConfigCreate 必填项和长度限制",
    )
    parser.add_argument("--json", action="store_true", help="以 JSON 输出预检结果")
    return parser.parse_args()


def main() -> int:
    args = parse_args()
    try:
        payload = read_json(args.config)
        openapi = read_openapi(args.openapi) if args.openapi else None
        result = validate(payload, openapi)
    except (OSError, ValueError, json.JSONDecodeError) as error:
        result = {
            "valid": False,
            "errors": [{"path": "$", "message": str(error)}],
            "warnings": [],
            "field_count": 0,
            "openapi_checked": bool(args.openapi),
        }
    print_result(result, args.json)
    return 0 if result["valid"] else 1


if __name__ == "__main__":
    raise SystemExit(main())
