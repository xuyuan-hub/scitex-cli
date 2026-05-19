# biolab-cli

A Rust CLI client for the Biolab lab management system. Manage primer synthesis and sequencing orders, inventory, and lab administration from the command line.

## Installation

### Download Pre-built Binarys

Download from the [latest release](https://github.com/xuyuan-hub/biolab-cli/releases/latest).

| Platform | Binary |
|----------|--------|
| Linux (x86_64) | `biolab-linux-amd64` |
| Linux (arm64) | `biolab-linux-arm64` |
| macOS (x86_64) | `biolab-macos-amd64` |
| macOS (arm64) | `biolab-macos-arm64` |
| Windows (x86_64) | `biolab-windows-amd64.exe` |

```bash
# Linux / macOS
chmod +x biolab-* && sudo mv biolab-* /usr/local/bin/biolab

# Windows
# Move biolab-windows-amd64.exe to your PATH or use the full path
```

### Build from Source

Requires Rust 1.70+.

```bash
cargo build --release
```

The binary is at `target/release/biolab` (or `biolab.exe` on Windows).

## Quick Start

### 1. Login

```bash
biolab login   # Feishu OAuth (token valid for 8 days)
biolab status  # check login status
```

### 2. Orders

```bash
# List orders
biolab orders list

# Create primer synthesis order
biolab orders create-primer order.json

# View order detail
biolab orders get <ID>

# Download order Excel
biolab orders download <ID>
```

### 3. Inventory

```bash
biolab inventory list
biolab inventory list --low-stock
biolab inventory checkin <ID> --quantity 5 --purpose "实验用"
biolab inventory checkout <ID> --quantity 2 --purpose "PCR"
```

### 4. Templates

Store recurring defaults (company, address, PI, payment method).

```bash
biolab templates list
biolab templates get-default primer_synthesis
```

### 5. Lab

```bash
biolab lab info
biolab lab members
biolab lab invite <email> [member]
```

### 6. Account

```bash
biolab me
biolab me update '{"phone_number":"13800000000"}'
biolab logout
```

## Configuration

| Setting | Default | Override |
|---------|---------|----------|
| API Base URL | `http://8.136.56.203/api/v1` | `BIOLAB_BASE_URL` env var |
| Token | `~/.biolab_token` | `BIOLAB_TOKEN` env var |

## Output Formats

All commands support `-f json` for machine-readable output:

```bash
biolab me -f json
biolab orders list -f json
```

## Architecture

See [CLAUDE.md](CLAUDE.md) for project structure and development notes.

## License

MIT