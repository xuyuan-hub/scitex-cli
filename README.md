# biolab-cli

[Rust](https://www.rust-lang.org/) [![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT) [![CI](https://github.com/xuyuan-hub/biolab-cli/actions/workflows/release.yml/badge.svg)](https://github.com/xuyuan-hub/biolab-cli/actions/workflows/release.yml)

[English](#biolab-cli) | [中文版](#biolab-cli-1)

The official CLI client for the Biolab lab management system — built for humans and AI Agents. Covers core lab domains including orders, inventory, templates, and lab administration, with AI Agent Skills for zero-setup automated workflows.

[Installation](#installation--quick-start) · [AI Agent Skills](#ai-agent-skills) · [Auth](#authentication) · [Commands](#command-system) · [Output Formats](#output-formats) · [Security](#security--risk-warnings) · [Architecture](#architecture) · [Contributing](#contributing)

## Why biolab-cli?

* **Agent-Native Design** — Structured Skills out of the box (orders, inventory, templates, lab) — AI Agents can operate the lab with zero extra setup
* **Wide Coverage** — 6 business domains (orders, inventory, templates, lab, projects, users), 40+ commands
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
| 🗂️ Projects | Project CRUD, status updates, and project membership management |
| 🌱 Project Workflows | Project-scoped seed/germplasm and planting workflows by slug |
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
| Windows | `biolab_win.zip` |

```bash
# Linux / macOS
chmod +x biolab_unix && sudo mv biolab_unix /usr/local/bin/biolab

# Windows
# Extract biolab_win.zip, rename biolab_win.exe to biolab.exe, and place it in PATH
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

# Optional: check for newer releases
biolab update check

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

> Run `biolab login`. It outputs an auth URL, starts a background poller, and returns immediately. Send the URL to the user to open in their browser. After the user authorizes, the background poller saves the token.

```bash
biolab login
```

**Step 3 — Install Agent Skills (required for automated workflows)**

```bash
# Universal — supports Hermes, Claude Code, Codex, Cursor, OpenCode, and other skills-compatible agents
npx -y skills add xuyuan-hub/biolab-cli -y -g

# Equivalent via the CLI
biolab skills install --global
```

**Step 4 — Verify**

```bash
biolab me -f json
```

If this returns user info, the setup is complete.

## AI Agent Skills

After installing skills via `npx skills add` or `biolab skills install`, the Agent gains access to the following structured skills. `biolab skills install` delegates to the standard `skills` installer so Hermes and other agents can refresh their own skill indexes correctly.

| Skill | Description |
|-------|-------------|
| `biolab-shared` | Core CLI usage, auth, credential chain, OpenAPI schema rules, error handling |
| `biolab-orders` | Primer synthesis and sequencing orders, order JSON, Excel upload/download, status changes |
| `biolab-templates` | Order-info templates for company, invoice, PI, payment, recipient, and notes |
| `biolab-inventory` | Generic inventory items, stock batches, inventory checks, checkin/checkout, FIFO checkout, adjust/transfer, transactions, storage locations |
| `biolab-experiment` | Experiment planning and execution workflow with inventory checks before task creation and task-linked checkout during execution |
| `biolab-admin` | Admin task type catalog creation and deletion |
| `biolab-lab` | Lab info, members, roles, invitations, applications, approval rules |
| `biolab-project` | Project slug workflows for germplasm, sequencing files, stocks, planting orders, and harvests |
| `biolab-users` | Login status, authenticated profile, contact fields, password changes |

The Agent skills are installed by the standard `skills` installer. Domain skills point back to `biolab-shared` for auth and OpenAPI schema rules.

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
biolab login

# Check status
biolab status

# Logout
biolab logout
```

## Updates

```bash
biolab update check
```

The update check compares the local CLI version with the latest GitHub Release and prints the recommended asset for the current platform. It does not automatically replace the running binary.

Token is stored in the OS keychain by default and is valid for 8 days. In Docker/K8s containers, if keyring is unavailable, the CLI automatically uses a container-local token file so Agent login does not require restarting the container or mounting a secret. `BIOLAB_TOKEN` can override storage for CI or temporary sessions. Legacy `~/.biolab_token` files are migrated into the keychain when possible on non-container hosts; host plaintext file storage is disabled unless `BIOLAB_INSECURE_TOKEN_FILE=1` is explicitly set in a trusted headless environment.

## Command System

Use `--help` for live command details:

```bash
biolab --help
biolab orders --help
biolab inventory --help
biolab lab --help
biolab projects --help
biolab project tashan --help
biolab admin --help
```

For the longer command catalog, see [docs/命令参考.md](docs/命令参考.md).

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

`cargo test` runs before build — 45 unit tests must pass.

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
* **覆盖全面** —— 6 大业务域（订单、库存、模板、课题组、项目、用户），40+ 命令
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
| 🗂️ 项目 | 项目 CRUD、状态更新、项目成员管理 |
| 🌱 项目工作流 | 按 slug 操作项目内种质、种子、种植和收获工作流 |
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
| Windows | `biolab_win.zip` |

```bash
# Linux / macOS
chmod +x biolab_unix && sudo mv biolab_unix /usr/local/bin/biolab

# Windows
# 解压 biolab_win.zip，将 biolab_win.exe 重命名为 biolab.exe 并放入 PATH
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

# 可选：检查是否有新版本
biolab update check

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

> 运行 `biolab login`，会打印一个认证 URL、启动后台轮询进程并立即返回。将 URL 发给用户在浏览器中打开，授权后后台进程会自动保存 token。无论本地或远程终端均可正常工作。

```bash
biolab login
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
# 通用安装（支持 Hermes、Claude Code、Codex、Cursor、OpenCode 等兼容 skills 的 agent）
npx -y skills add xuyuan-hub/biolab-cli -y -g

# 或使用 CLI 安装；会委托给 npx skills add
biolab skills install --global
```

| Skill | 描述 |
|-------|------|
| `biolab-shared` | CLI 核心用法、认证、凭据链、OpenAPI schema 规则、错误处理 |
| `biolab-orders` | 引物合成和测序订单、订单 JSON、Excel 上传/下载、状态变更 |
| `biolab-templates` | 订单信息模板：单位、发票、PI、付款方式、收货地址、备注 |
| `biolab-inventory` | 通用物料、库存批次、库存检查、入库/出库、FIFO 出库、调整/转移、交易记录、存储位置 |
| `biolab-experiment` | 实验方案与执行工作流：创建任务前检查库存，执行阶段关联任务出库 |
| `biolab-admin` | 管理端任务类型目录的创建与删除 |
| `biolab-lab` | 课题组信息、成员、角色、邀请、入组申请、审批规则 |
| `biolab-project` | 按项目 slug 操作种质、测序文件、库存、种植单和收获记录 |
| `biolab-users` | 登录状态、当前用户资料、联系人字段、密码修改 |

Agent skills 会由标准 `skills` 安装器一次性安装；领域 skill 会回指 `biolab-shared` 获取认证和 OpenAPI schema 规则。

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
biolab login

# 检查状态
biolab status

# 登出
biolab logout
```

## 更新

```bash
biolab update check
```

更新检查会比较本地 CLI 版本和 GitHub 最新 Release，并输出当前平台推荐下载资产。它不会自动替换正在运行的二进制文件。

Token 默认存储在 OS 密钥链中，有效期 8 天。在 Docker/K8s 容器中，如果 keyring 不可用，CLI 会自动使用容器内本地 token 文件，Agent 登录无需重启容器或挂载 secret。可通过 `BIOLAB_TOKEN` 环境变量覆盖。非容器宿主机上的遗留 `~/.biolab_token` 文件会尽量迁移到密钥链；宿主机明文文件存储默认关闭，只有在可信 headless 环境中显式设置 `BIOLAB_INSECURE_TOKEN_FILE=1` 才会启用。

## 命令系统

使用 `--help` 查看实时命令：

```bash
biolab --help
biolab orders --help
biolab inventory --help
biolab lab --help
biolab projects --help
biolab project tashan --help
biolab admin --help
```

完整命令清单见 [docs/命令参考.md](docs/命令参考.md)。

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

构建前运行 `cargo test` —— 45 个单元测试必须通过。

打标签推送（如 `v0.1.0`）自动创建 GitHub Release 并附带二进制文件。

## 贡献

欢迎社区贡献！如果发现 bug 或有功能建议，请提交 [Issue](https://github.com/xuyuan-hub/biolab-cli/issues) 或 [Pull Request](https://github.com/xuyuan-hub/biolab-cli/pulls)。

重大变更建议先通过 Issue 讨论。

## 许可证

本项目使用 **MIT 许可证**。
