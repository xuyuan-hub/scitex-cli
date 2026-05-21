# Biolab CLI 安装指南

以下步骤面向 AI Agent。部分步骤需要用户在浏览器中配合完成飞书 OAuth 授权。

## 环境要求

开始安装之前，请确保环境中已具备：

- Node.js（npm/npx，仅安装 Agent Skills 时需要）
- 系统凭据库可用：Windows Credential Manager、macOS Keychain，或 Linux Secret Service/keyutils
- 如果运行在 Docker/K8s 等容器中，CLI 会在 keyring 不可用时自动使用容器内本地 token 文件，无需重启容器或额外挂载
- Rust 1.70+（仅源码构建需要）

## 第 1 步 安装 CLI

优先使用 GitHub Release 中的预编译二进制：

https://github.com/xuyuan-hub/biolab-cli/releases/latest

根据系统下载对应文件：

| 平台 | Release 资产 |
|------|--------------|
| Linux | `biolab_unix` |
| Windows | `biolab_win.zip` |
| macOS x86_64 | `biolab_mac_amd64` |
| macOS arm64 | `biolab_mac_arm64` |

Linux / macOS：

```shell
chmod +x biolab_unix
sudo mv biolab_unix /usr/local/bin/biolab
biolab --help
```

Windows PowerShell：

```powershell
# 解压 biolab_win.zip，将 biolab_win.exe 重命名为 biolab.exe，并放入 PATH 中的目录
biolab --help
```

如果需要从源码构建：

```shell
git clone https://github.com/xuyuan-hub/biolab-cli.git
cd biolab-cli
cargo build --release

# Linux / macOS
./target/release/biolab --help

# Windows
.\target\release\biolab.exe --help
```

## 第 2 步 安装 CLI Skills

Agent 自动化操作订单、库存、模板、课题组等功能前，建议安装配套 Skills。

如果已经安装 `biolab`：

```shell
biolab skills install --scope global
biolab skills check
```

也可以使用通用 skills 安装方式：

```shell
npx -y skills add xuyuan-hub/biolab-cli -y -g
```

## 第 3 步 登录

Agent 运行以下命令，并提取终端输出中的认证链接发给用户。

```shell
biolab login --background
```

用户在浏览器中打开认证链接并完成飞书授权后，后台轮询进程会自动保存 token。Agent 不需要占住终端等待授权完成。

默认情况下，token 会保存到系统凭据库中。若 Agent 运行在 headless Docker/K8s 容器里，系统凭据库通常不可用，CLI 会自动回退到容器内本地 token 文件。不要打印、复制或记录 token。

## 第 4 步 验证

```shell
biolab status
biolab me -f json
```

如果 `biolab me -f json` 返回当前用户信息，说明 CLI 安装和登录已完成。

## 常用命令

```shell
# 查看当前用户
biolab me -f json

# 查看订单
biolab orders list -f json

# 查看库存
biolab inventory list -f json

# 查看模板
biolab templates list -f json

# 查看课题组信息
biolab lab info -f json
```

## 配置项

| 配置项 | 默认值 | 覆盖方式 |
|--------|--------|----------|
| API 地址 | `http://8.136.56.203/api/v1` | `BIOLAB_BASE_URL` |
| Token | 系统凭据库；容器内自动 fallback 到本地 token 文件 | `BIOLAB_TOKEN` |
| 宿主机明文 token 文件回退 | 默认关闭 | `BIOLAB_INSECURE_TOKEN_FILE=1` |
| 禁用容器 token 文件 fallback | 默认关闭 | `BIOLAB_DISABLE_CONTAINER_TOKEN_FILE=1` |

容器内 fallback 不需要重启容器或挂载 secret，适合正在运行的 Agent 容器。token 只保存在当前容器文件系统中；容器删除后需要重新登录。`BIOLAB_INSECURE_TOKEN_FILE=1` 只应在可信 headless 宿主机环境中临时使用。常规本地环境应使用系统凭据库。

## 故障处理

如果 `biolab login` 提示系统凭据库存储失败：

1. 确认系统凭据库可用。
2. 如果 Agent 在 Docker/K8s 容器中，直接使用 `biolab login --background`；CLI 会自动使用容器内 token 文件。
3. 在非容器 Linux headless 环境中配置 Secret Service/keyutils，或使用受控的临时环境变量。
4. 不要手动把 token 写入共享目录、项目目录或日志文件。

如果命令返回未登录或 token 失效：

```shell
biolab logout
biolab login --background
biolab status
```

## 更多能力

```shell
biolab --help
biolab orders --help
biolab inventory --help
biolab templates --help
biolab lab --help
```
