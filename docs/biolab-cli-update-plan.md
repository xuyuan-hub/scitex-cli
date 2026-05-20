# biolab-cli 项目更新方案

## 背景与目标

`biolab-cli` 当前已经具备基础命令能力：登录、用户信息、订单、模板、库存、课题组管理，以及初步的 Agent skill 安装命令。下一步目标不是继续堆命令，而是把 CLI 整理成“人和 AI Agent 都能稳定使用”的产品型工具。

本方案用于约束后续更新：先明确模块边界和验收标准，再按阶段实施。

## 当前状态

已具备：

- Rust CLI 主入口和 clap 子命令路由。
- Feishu OAuth 登录和本地 token 存储。
- 订单、模板、库存、课题组、用户相关 API 调用。
- JSON 文件创建订单和模板。
- 初步 Agent skill：`biolab skills install/check/path`。

主要问题：

- `client.rs` 曾同时承载 HTTP transport、API endpoint、响应解包和文件上传，职责过重。
- 输出层还没有统一 envelope，`OutputFormat` 在部分函数里没有完全生效。
- 认证流程偏人工阻塞，不适合 Agent split-flow。
- 模板和订单工作流还没有打通，仍要求用户或 Agent 手动拼完整 JSON。
- 发布链路还缺 CI release、checksums 和二进制产物策略。

## 模块边界规划

### 1. API 响应模块

目标文件：`src/api_response.rs`

职责：

- 统一处理 HTTP 响应状态码。
- 统一处理后端 `{ data: ... }` envelope。
- 区分空结果和解析失败。
- 保留可读错误，便于 Agent 判断下一步。

验收标准：

- `client.rs` 不再直接定义 `parse_response`、`extract_array`、`extract_object`。
- 数组解析失败必须返回错误，不能静默返回 `[]`。
- 现有类型反序列化测试通过。

### 2. HTTP transport 模块

目标文件：`src/http.rs`

职责：

- 构造带 token 的 `reqwest::Client`。
- 提供 GET/POST/PATCH/PUT/DELETE 基础能力。
- 提供文件下载和 Excel 上传能力。
- 不承载具体业务 endpoint 语义。

验收标准：

- HTTP 请求拼接、token header、multipart 上传从领域服务里移出。
- 新增 HTTP 能力时优先改 `http.rs`。

### 3. 服务层模块

目标目录：`src/services/`

拆分：

- `services/users.rs`
- `services/orders.rs`
- `services/templates.rs`
- `services/inventory.rs`
- `services/lab.rs`

职责：

- 每个服务模块只负责对应业务域 API。
- `BiolabClient` 保留为对外门面，commands 层可以继续调用现有方法。
- 新增业务 endpoint 时只改对应服务模块和命令模块。

验收标准：

- `client.rs` 不再承载全部业务 API。
- 单个领域的 endpoint 聚合在对应 `services/*.rs`。
- commands 层无需理解 HTTP 细节。

### 4. 输出契约模块

目标文件：`src/output.rs`

职责：

- 明确 text/json 两种输出。
- JSON 输出可被 Agent 稳定解析。
- 错误、notice、数据、human text 分离。

验收标准：

- `print_result` 尊重 `OutputFormat`，或重命名为明确的 JSON 输出函数。
- 下载类命令、创建类命令、列表类命令的输出策略一致。

### 5. Agent Skill 文档模块

目标目录：`skills/biolab-api/references/`

文件：

- `orders.md`
- `inventory.md`
- `templates.md`
- `lab.md`
- `users.md`

职责：

- 为 Agent 提供领域规则、常用命令、危险操作、JSON 输入示例。
- `SKILL.md` 做路由和总规则，细节放到 references。

验收标准：

- `CLAUDE.md` 中提到的 references 文件实际存在。
- `SKILL.md` 明确要求在复杂订单、库存、课题组操作前读取对应 reference。

### 6. 业务工作流命令

建议新增：

- `orders validate <file>`
- `orders create-primer --from-template <template-id> --items <file>`
- `orders create-sequencing --from-template <template-id> --items <file>`
- `orders preview <file>`
- `templates export <id> <file>`

职责：

- 降低 Agent 手写完整 JSON 的失败率。
- 把模板能力真正接入订单创建流程。

验收标准：

- 常见下单流程可以通过模板加 items 文件完成。
- JSON 输入错误能在本地提前发现。

## 已完成更新

第一轮：响应解析与 Agent skill references。

- 新增 `src/api_response.rs`。
- 从 `client.rs` 移出响应解析函数。
- 修正数组解析失败静默吞错的问题。
- 新增 `skills/biolab-api/references/` 基础文档。
- 更新 `SKILL.md`，让 Agent 知道何时读取 reference。

第二轮：服务层重构。

- 新增 `src/http.rs`。
- 新增 `src/services/` 并拆分 users、orders、templates、inventory、lab。
- 收缩 `src/client.rs`，只保留 `BiolabClient` 门面构造和 `BiolabError`。
- 保留现有 `BiolabClient` public method 名称，避免 commands 层大改。

## 后续优先级

1. 统一输出 envelope，让 text/json 行为一致。
2. 拆分认证流程，增加 Agent 友好的 `auth status --json`、`login --no-wait`、`callback/poll`。
3. 打通 templates 和 orders，新增 validate、preview、from-template 类工作流命令。
4. 拆分 `types.rs` 为领域类型模块，并逐步把状态、角色、订单类型收敛为 enum。
5. 补发布链路：CI checks、release workflow、checksums、安装脚本或明确二进制策略。

## 验收标准

每轮完成后应满足：

- `cargo fmt --check` 通过。
- `cargo check --offline` 通过。
- `cargo test --offline` 通过。
- 不引入无关依赖。
- 不提交 `Cargo.lock` 的非功能性解析漂移。

## 风险与注意事项

- 当前 `Cargo.lock` 在本地解析时可能出现依赖解析漂移，非功能变更不应提交 lockfile 噪音。
- 后端 API envelope 的真实形态如果不止 `{ data }`，后续应补充更多 fixture 测试。
- 现有 Rust 注释和 CLI help 文案中有乱码，需要后续单独做编码与文案清理。
