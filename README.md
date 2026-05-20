# biolab-cli

[Rust](https://www.rust-lang.org/) [![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT) [![CI](https://github.com/xuyuan-hub/biolab-cli/actions/workflows/release.yml/badge.svg)](https://github.com/xuyuan-hub/biolab-cli/actions/workflows/release.yml)

[English](#biolab-cli) | [中文版](#biolab-cli-1)

The official CLI client for the Biolab lab management system — built for humans and AI Agents. Covers core lab domains including orders, inventory, templates, and lab administration, with AI Agent Skills for zero-setup automated workflows.

[Installation](#installation--quick-start) · [AI Agent Skills](#ai-agent-skills) · [Auth](#authentication) · [Commands](#command-system) · [Output Formats](#output-formats) · [Security](#security--risk-warnings) · [Architecture](#architecture) · [Contributing](#contributing)

## Why biolab-cli?

* **Agent-Native Design** — Structured Skills out of the box (orders, inventory, templates, lab) — AI Agents can operate the lab with zero extra setup
* **Wide Coverage** — 5 business domains (orders, inventory, templates, lab, users), 30+ commands
* **AI-Friendly** — JSON output (`-f json`), structured skill reference docs, deterministic command patterns
* **Cross-Platform** — Pre-built binaries for Linux, macOS (x86_64 + arm64), Windows
* **Up and Running in 2 Minutes** — One login, interactive OAuth, from install to first API call in 3 steps
* **Secure & Controllable** — Feishu OAuth, local token storage with 8-day expiry, no credential sharing
* **Three-Layer Architecture** — CLI commands (human-friendly) → Domain services (structured API) → Raw HTTP (extensible)

## Features

| Domain | Capabilities |
|--------|--------------|
| 📦 Orders | Create, query, update primer synthesis & sequencing orders; download Excel; upload supplier templates; resend pending orders |
| 🧪 Inventory | List, filter, checkin/checkout stock; location management; low-stock alerts; stats |
| 📋 Templates | CRUD for order-info templates (company, address, PI, payment defaults); set default |
| 👥 Lab | Lab creation, member management (invite/join/role), approval rules, applications |
| 👤 Users | View/update profile, change password, permission checks |

## Installation & Quick Start

### Requirements

Before you start, make sure you have:

* A pre-built binary (download from [releases](https://github.com/xuyuan-hub/biolab-cli/releases/latest)) or Rust 1.70+ to build from source

### Quick Start (Human Users)

> **Note for AI assistants:** If you are an AI Agent helping the user with installation, jump directly to [Quick Start (AI Agent)](#quick-start-ai-agent), which contains all the steps you need to complete.

#### Install

**Option 1 — Download pre-built binary (recommended):**

| Platform | Binary |
|----------|--------|
| Linux | `biolab_unix` |
| macOS (x86_64) | `biolab_mac_amd64` |
| macOS (arm64) | `biolab_mac_arm64` |
| Windows | `biolab_win.exe` |

```bash
# Linux / macOS
chmod +x biolab_unix && sudo mv biolab_unix /usr/local/bin/biolab

# Windows
# Rename biolab_win.exe to biolab.exe and place in PATH
```

**Option 2 — Build from source:**

Requires Rust 1.70+.

```bash
git clone https://github.com/xuyuan-hub/biolab-cli.git
cd biolab-cli
cargo build --release
# Binary: target/release/biolab
```

#### Configure & Use

```bash
# 1. Login (outputs a verification URL — open in any browser)
biolab login

# 2. Verify
biolab status

# 3. Start using
biolab me
biolab orders list
```

### Quick Start (AI Agent)

> The following steps are for AI Agents. Some steps require the user to complete actions in a browser.

**Step 1 — Check if already authenticated**

```bash
biolab status
```

If not logged in, proceed to Step 2.

**Step 2 — Login**

> Run `biolab login --background`. It outputs an auth URL, starts a background poller, and returns immediately. Send the URL to the user to open in their browser. After the user authorizes, the background poller saves the token.

```bash
biolab login --background
```

**Step 3 — Install Agent Skills (required for automated workflows)**

```bash
# Universal — supports Claude Code, Codex, Cursor, OpenCode, 51+ agents
npx skills add xuyuan-hub/biolab-cli -y -g
```

**Step 4 — Verify**

```bash
biolab me -f json
```

If this returns user info, the setup is complete.

## AI Agent Skills

After installing skills via `npx skills add` or `biolab skills install`, the Agent gains access to the following structured skills:

| Skill | Description |
|-------|-------------|
| `biolab-api` | Core CLI usage, auth, credential chain, error handling (auto-loaded by all other skills) |
| `orders` | Order schemas, status machine (`pending → ordered → received → stored`), supplier differences (sangon vs biosune), primer & sequencing order fields |
| `inventory` | Stock/checkin/checkout schemas, location hierarchy, low-stock detection |
| `templates` | Template fields for order defaults (company, address, PI, payment method), CRUD operations |
| `lab` | Lab CRUD, member management (5 roles: pi > procurement > finance > warehouse > member), approval rules |
| `users` | User info, phone/email requirements, permission model, signup |

The Agent skill reference docs are located in `.claude/skills/biolab-api/references/` after installation.

### Agent Order Workflow

When asked to create an order, the Agent follows this sequence:

1. **Check auth** — `biolab status`. Not logged in = 401 failure.
2. **Get user info** — `biolab me -f json`. Check `phone_number` is not empty.
3. **Check default template** — `biolab templates get-default primer_synthesis -f json`. Template stores company, address, PI, payment.
4. **Confirm options** — Supplier / purification method / specs — never assume defaults.
5. **Extract primers** — From user-provided documents (Excel/text/chat).
6. **Build JSON & submit** — Merge template defaults + user options + extracted primers → temp JSON → `biolab orders create-primer <file>`.

## Authentication

| Command | Description |
|---------|-------------|
| `login` | Custom CLI poll flow — outputs auth URL, polls for JWT |
| `logout` | Remove local token from the OS keychain and delete any legacy `~/.biolab_token` file |
| `status` | Show current login status |

```bash
# Interactive login
biolab login

# Agent-friendly login: print auth URL and continue polling in background
biolab login --background

# Check status
biolab status

# Logout
biolab logout
```

Token is stored in the OS keychain by default and is valid for 8 days. In Docker/K8s containers, if keyring is unavailable, the CLI automatically uses a container-local token file so Agent login does not require restarting the container or mounting a secret. `BIOLAB_TOKEN` can override storage for CI or temporary sessions. Legacy `~/.biolab_token` files are migrated into the keychain when possible on non-container hosts; host plaintext file storage is disabled unless `BIOLAB_INSECURE_TOKEN_FILE=1` is explicitly set in a trusted headless environment.

## Command System

### Orders

```bash
# List orders (with pagination)
biolab orders list --skip 0 --limit 100

# Order detail (with items)
biolab orders get <ID>

# Create primer synthesis order (from JSON file)
biolab orders create-primer order.json

# Create sequencing order
biolab orders create-sequencing order.json

# Update order
biolab orders update <ID> '{"status":"received"}'

# Resend pending order
biolab orders resend <ID>

# Download order Excel
biolab orders download <ID> [output.xlsx]

# Download supplier templates
biolab orders download-primer-template
biolab orders download-sequencing-template

# Upload Excel for parsing
biolab orders upload-primer-excel file.xlsx
biolab orders upload-sequencing-excel file.xlsx
```

### Inventory

```bash
# Stock list
biolab inventory list

# Filter by name
biolab inventory list --primer-name 'FWD'

# Low stock only
biolab inventory list --low-stock

# Stock detail (with transaction history)
biolab inventory get <ID>

# Stats (total, low-stock count)
biolab inventory stats

# Checkin (add stock)
biolab inventory checkin <ID> --quantity 5 --purpose "restock"

# Checkout (remove stock)
biolab inventory checkout <ID> --quantity 2 --purpose "PCR" --experiment-ref "EXP-001"

# Locations
biolab inventory locations
biolab inventory create-location "Freezer A" [--parent-id <ID>]
```

### Templates

```bash
biolab templates list
biolab templates get <ID>
biolab templates get-default primer_synthesis
biolab templates create <json-file>
biolab templates update <ID> <json-file>
biolab templates delete <ID>
biolab templates set-default <ID>
```

### Lab

```bash
biolab lab info
biolab lab create <name>
biolab lab update <json>
biolab lab members
biolab lab update-role <user_id> <role>
biolab lab remove-member <user_id>
biolab lab invite <email> [member]
biolab lab invitations
biolab lab accept-invite <id>
biolab lab decline-invite <id>
biolab lab join <lab_id> [role]
biolab lab applications
biolab lab approve-app <id>
biolab lab reject-app <id>
biolab lab approval-rules
biolab lab add-rule <json>
biolab lab remove-rule <id>
```

### Users

```bash
biolab me
biolab me update '{"phone_number":"13800000000"}'
biolab me change-password --current 'old' --new 'new'
```

## Output Formats

All commands support `-f json` for machine-readable output:

```bash
biolab me -f json            # Full JSON (for Agent parsing)
biolab orders list -f json   # Structured array
biolab inventory stats -f json
```

Default (text) output uses colored formatting for human readability.

## Security & Risk Warnings

This tool can be invoked by AI Agents to automate lab operations on the Biolab platform. After you authorize via Feishu OAuth, the AI Agent will act under your user identity within the authorized scope, which may lead to high-risk consequences such as:

* Creating orders with incorrect parameters
* Modifying inventory without proper verification
* Changing lab member roles or approval rules

To reduce these risks:

* Token expires after 8 days — requires re-authentication
* Agent skills are read-only reference docs — they do not execute anything on their own
* All commands require explicit user intent — the Agent should confirm before creating or modifying data
* Use `-f json` output to review what the Agent is about to submit before execution

Please fully understand all usage risks. By using this tool, you are deemed to voluntarily assume all related responsibilities.

## Configuration

| Setting | Default | Override |
|---------|---------|----------|
| API Base URL | `http://8.136.56.203/api/v1` | `BIOLAB_BASE_URL` env var |
| Token | OS keychain; container-local file fallback in Docker/K8s | `BIOLAB_TOKEN` env var; legacy `~/.biolab_token` migration; `BIOLAB_INSECURE_TOKEN_FILE=1` for explicit host plaintext fallback |

## Architecture

The project follows a three-layer architecture in Rust:

```
src/
├── main.rs              # Thin CLI entry (imports from library)
├── lib.rs               # All mod declarations; public API re-exports
├── errors.rs            # BiolabError enum (thiserror)
├── client.rs            # BiolabClient factory
├── http.rs              # Raw HTTP methods (reqwest + rustls)
├── api_response.rs      # Response envelope unwrapping
├── types.rs             # Serde structs + custom deserializers
├── auth.rs              # Feishu OAuth flow
├── output.rs            # JSON vs colored text formatting
├── config.rs            # Token management (env → file → OAuth)
├── commands/            # clap subcommand args + run() handlers
└── services/            # impl BiolabClient blocks + unit tests
    └── helpers.rs       # Shared: empty_body, single_field_body, url_encode
```

See [CLAUDE.md](CLAUDE.md) for detailed development notes.

## CI

Multi-platform builds via GitHub Actions on every push:

* Linux (x86_64 via musl)
* Windows (x86_64)
* macOS (x86_64 + arm64)

`cargo test` runs before build — 23 unit tests must pass.

Tagged pushes (e.g. `v0.1.0`) auto-create GitHub Releases with binaries.

## Contributing

Community contributions are welcome! If you find a bug or have feature suggestions, please submit an [Issue](https://github.com/xuyuan-hub/biolab-cli/issues) or [Pull Request](https://github.com/xuyuan-hub/biolab-cli/pulls).

## License

This project is licensed under the **MIT License**.

---

# biolab-cli

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT) [![CI](https://github.com/xuyuan-hub/biolab-cli/actions/workflows/release.yml/badge.svg)](https://github.com/xuyuan-hub/biolab-cli/actions/workflows/release.yml)

[English](#biolab-cli) · [中文版](#biolab-cli-1)

实验管理系统官方 CLI 客户端 —— 为用户和 AI Agent 设计。覆盖订单、库存、信息模板、课题组管理等核心业务领域，内置 AI Agent Skills，支持零配置自动化工作流。

[安装](#安装与快速开始) · [AI Agent Skills](#ai-agent-skills-1) · [认证](#认证) · [命令](#命令系统) · [输出格式](#输出格式) · [安全](#安全与风险提示) · [架构](#架构) · [贡献](#贡献)

## 为什么用 biolab-cli？

* **Agent 原生** —— 内置结构化 Skills（订单、库存、模板、课题组）—— AI Agent 无需额外配置即可操作实验系统
* **覆盖全面** —— 5 大业务域（订单、库存、模板、课题组、用户），30+ 命令
* **AI 友好** —— JSON 输出（`-f json`）、结构化 skill 参考文档、确定性命令模式
* **跨平台** —— 预编译 Linux、macOS（x86_64 + arm64）、Windows 二进制
* **2 分钟上手** —— 一次登录、交互式 OAuth，3 步即可调用第一个 API
* **安全可控** —— 飞书 OAuth、本地 token 8 天过期、无需共享凭据
* **三层架构** —— CLI 命令（用户友好）→ 领域服务（结构化 API）→ 原始 HTTP（可扩展）

## 功能

| 领域 | 能力 |
|------|------|
| 📦 订单 | 创建、查询、更新引物合成和测序订单；下载 Excel；上传供应商模板；重发待处理订单 |
| 🧪 库存 | 列表、筛选、出入库操作；存储位置管理；低库存预警；统计 |
| 📋 信息模板 | 订单信息模板 CRUD（单位、地址、PI、付款方式默认值）；设默认模板 |
| 👥 课题组 | 课题组创建、成员管理（邀请/加入/角色）、审批规则、入组申请 |
| 👤 用户 | 查看/更新个人信息、修改密码、权限检查 |

## 安装与快速开始

### 前置要求

开始前请确保：

* 已下载预编译二进制（从 [releases](https://github.com/xuyuan-hub/biolab-cli/releases/latest)）或 Rust 1.70+ 可从源码编译

### 用户快速开始

> **AI Agent 注意：** 如果你是在帮用户安装，请直接跳到 [Agent 快速开始](#agent-快速开始)，那里包含你需要完成的所有步骤。

#### 安装

**方式一 —— 下载预编译二进制（推荐）：**

| 平台 | 文件名 |
|------|--------|
| Linux | `biolab_unix` |
| macOS (x86_64) | `biolab_mac_amd64` |
| macOS (arm64) | `biolab_mac_arm64` |
| Windows | `biolab_win.exe` |

```bash
# Linux / macOS
chmod +x biolab_unix && sudo mv biolab_unix /usr/local/bin/biolab

# Windows
# 将 biolab_win.exe 重命名为 biolab.exe 并放入 PATH
```

**方式二 —— 从源码编译：**

需要 Rust 1.70+。

```bash
git clone https://github.com/xuyuan-hub/biolab-cli.git
cd biolab-cli
cargo build --release
# 可执行文件：target/release/biolab
```

#### 配置与使用

```bash
# 1. 登录（浏览器打开飞书 OAuth）
biolab login

# 2. 验证
biolab status

# 3. 开始使用
biolab me
biolab orders list
```

### Agent 快速开始

> 以下步骤面向 AI Agent。部分步骤需要用户在浏览器中完成操作。

**第一步 —— 检查是否已登录**

```bash
biolab status
```

如果未登录，进入第二步。

**第二步 —— 登录**

> 运行 `biolab login --background`，会打印一个认证 URL、启动后台轮询进程并立即返回。将 URL 发给用户在浏览器中打开，授权后后台进程会自动保存 token。无论本地或远程终端均可正常工作。

```bash
biolab login --background
```

**第三步 —— 安装 Agent Skills（自动化工作流必需）**

```bash
biolab skills install
```

**第四步 —— 验证**

```bash
biolab me -f json
```

如果返回用户信息，安装完成。

## AI Agent Skills

执行以下任一命令后，Agent 可使用以下结构化 skills：

```bash
# 通用安装（支持 Claude Code, Codex, Cursor, OpenCode 等 51+ agent）
npx skills add xuyuan-hub/biolab-cli -y -g

# 或使用 CLI 内置安装（仅 Claude / Codex）
biolab skills install
```

| Skill | 描述 |
|-------|------|
| `biolab-api` | CLI 核心用法、认证、凭据链、错误处理（其他 skills 自动加载） |
| `orders` | 订单 Schema、状态机（`待下单 → 已下单 → 已收货 → 已入库`）、供应商差异（生工 vs 铂尚）、引物与测序订单字段 |
| `inventory` | 库存/出入库 Schema、位置层级、低库存检测 |
| `templates` | 订单默认模板字段（单位、地址、PI、付款方式）、CRUD 操作 |
| `lab` | 课题组 CRUD、成员管理（5 种角色：pi > procurement > finance > warehouse > member）、审批规则 |
| `users` | 用户信息、手机号/邮箱要求、权限模型、注册 |

Agent skill 参考文档位于 `.claude/skills/biolab-api/references/`。

### Agent 下单工作流

被要求创建订单时，Agent 按以下顺序执行：

1. **检查登录** —— `biolab status`。未登录 = 401 失败。
2. **获取用户信息** —— `biolab me -f json`。检查 `phone_number` 是否为空。
3. **检查默认模板** —— `biolab templates get-default primer_synthesis -f json`。模板存储单位、地址、PI、付款方式。
4. **确认选项** —— 供应商 / 纯化方式 / 规格 —— 绝不假设默认值。
5. **提取引物** —— 从用户提供的文档（Excel/文本/聊天记录）中提取。
6. **构建 JSON 并提交** —— 合并模板默认值 + 用户确认的选项 + 提取的引物 → 临时 JSON 文件 → `biolab orders create-primer <文件>`。

## 认证

| 命令 | 描述 |
|------|------|
| `login` | 自定义 CLI 轮询流程 — 输出认证 URL，轮询获取 JWT |
| `logout` | 删除 OS 密钥链中的本地 token，并清理遗留 `~/.biolab_token` 文件 |
| `status` | 显示当前登录状态 |

```bash
# 交互式登录
biolab login

# Agent 友好登录：打印认证 URL 后在后台等待用户授权
biolab login --background

# 检查状态
biolab status

# 登出
biolab logout
```

Token 默认存储在 OS 密钥链中，有效期 8 天。在 Docker/K8s 容器中，如果 keyring 不可用，CLI 会自动使用容器内本地 token 文件，Agent 登录无需重启容器或挂载 secret。可通过 `BIOLAB_TOKEN` 环境变量覆盖。非容器宿主机上的遗留 `~/.biolab_token` 文件会尽量迁移到密钥链；宿主机明文文件存储默认关闭，只有在可信 headless 环境中显式设置 `BIOLAB_INSECURE_TOKEN_FILE=1` 才会启用。

## 命令系统

### 订单

```bash
# 订单列表
biolab orders list --skip 0 --limit 100

# 订单详情（含物品列表）
biolab orders get <ID>

# 创建引物合成订单
biolab orders create-primer order.json

# 创建测序订单
biolab orders create-sequencing order.json

# 更新订单
biolab orders update <ID> '{"status":"received"}'

# 重发待处理订单
biolab orders resend <ID>

# 下载订单 Excel
biolab orders download <ID> [输出路径]

# 下载供应商模板
biolab orders download-primer-template
biolab orders download-sequencing-template

# 上传 Excel 解析
biolab orders upload-primer-excel file.xlsx
biolab orders upload-sequencing-excel file.xlsx
```

### 库存

```bash
# 库存列表
biolab inventory list

# 按名称筛选
biolab inventory list --primer-name 'FWD'

# 仅显示低库存
biolab inventory list --low-stock

# 库存详情（含交易记录）
biolab inventory get <ID>

# 统计（总数、低库存数）
biolab inventory stats

# 入库
biolab inventory checkin <ID> --quantity 5 --purpose "补货"

# 出库
biolab inventory checkout <ID> --quantity 2 --purpose "PCR" --experiment-ref "EXP-001"

# 存储位置
biolab inventory locations
biolab inventory create-location "冰箱 A" [--parent-id <ID>]
```

### 信息模板

```bash
biolab templates list
biolab templates get <ID>
biolab templates get-default primer_synthesis
biolab templates create <json文件>
biolab templates update <ID> <json文件>
biolab templates delete <ID>
biolab templates set-default <ID>
```

### 课题组

```bash
biolab lab info
biolab lab create <名称>
biolab lab update <json>
biolab lab members
biolab lab update-role <user_id> <role>
biolab lab remove-member <user_id>
biolab lab invite <邮箱> [member]
biolab lab invitations
biolab lab accept-invite <id>
biolab lab decline-invite <id>
biolab lab join <lab_id> [role]
biolab lab applications
biolab lab approve-app <id>
biolab lab reject-app <id>
biolab lab approval-rules
biolab lab add-rule <json>
biolab lab remove-rule <id>
```

### 用户

```bash
biolab me
biolab me update '{"phone_number":"13800000000"}'
biolab me change-password --current '旧密码' --new '新密码'
```

## 输出格式

所有命令支持 `-f json` 机器可读输出：

```bash
biolab me -f json            # 完整 JSON（Agent 解析用）
biolab orders list -f json   # 结构化数组
biolab inventory stats -f json
```

默认（text）输出使用彩色格式化，适合人类阅读。

## 安全与风险提示

本工具可被 AI Agent 调用以自动化实验平台操作。飞书 OAuth 授权后，AI Agent 将以你的用户身份在授权范围内执行操作，可能导致以下高风险后果：

* 使用错误参数创建订单
* 未经充分验证就修改库存
* 更改课题组成员角色或审批规则

为降低风险：

* Token 8 天过期 —— 需定期重新认证
* Agent skills 为只读参考文档 —— 不会自行执行任何操作
* 所有命令需明确用户意图 —— Agent 在创建或修改数据前应确认
* 使用 `-f json` 输出在执行前审查 Agent 准备提交的内容

请充分理解所有使用风险。使用本工具即视为自愿承担全部责任。

## 配置

| 配置项 | 默认值 | 环境变量覆盖 |
|--------|--------|-------------|
| API 地址 | `http://8.136.56.203/api/v1` | `BIOLAB_BASE_URL` |
| Token | OS 密钥链；Docker/K8s 容器内本地文件 fallback | `BIOLAB_TOKEN`；遗留 `~/.biolab_token` 迁移；显式 `BIOLAB_INSECURE_TOKEN_FILE=1` 宿主机明文回退 |

## 架构

项目采用 Rust 三层架构：

```
src/
├── main.rs              # 薄 CLI 入口（从 library 导入）
├── lib.rs               # 所有 mod 声明；公共 API 导出
├── errors.rs            # BiolabError 枚举（thiserror）
├── client.rs            # BiolabClient 工厂
├── http.rs              # 原始 HTTP 方法（reqwest + rustls）
├── api_response.rs      # 响应信封解包
├── types.rs             # Serde 结构体 + 自定义反序列化
├── auth.rs              # 飞书 OAuth 流程
├── output.rs            # JSON vs 彩色文本格式化
├── config.rs            # Token 管理（env → file → OAuth）
├── commands/            # clap 子命令参数 + run() 处理器
└── services/            # impl BiolabClient 块 + 单元测试
    └── helpers.rs       # 共享：empty_body, single_field_body, url_encode
```

详见 [CLAUDE.md](CLAUDE.md)。

## CI

通过 GitHub Actions 每次推送自动构建：

* Linux（x86_64，musl）
* Windows（x86_64）
* macOS（x86_64 + arm64）

构建前运行 `cargo test` —— 23 个单元测试必须通过。

打标签推送（如 `v0.1.0`）自动创建 GitHub Release 并附带二进制文件。

## 贡献

欢迎社区贡献！如果发现 bug 或有功能建议，请提交 [Issue](https://github.com/xuyuan-hub/biolab-cli/issues) 或 [Pull Request](https://github.com/xuyuan-hub/biolab-cli/pulls)。

重大变更建议先通过 Issue 讨论。

## 许可证

本项目使用 **MIT 许可证**。
