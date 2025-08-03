# xllm

A CLI utility for running LLMs with optional gRPC proxy support.

## Installation

### Option 1: Install from crates.io (Basic functionality)

```bash
cargo install xllm
xllm --init  # This will generate a config file
```

### Option 2: Clone for full features (Recommended if you want gRPC proxy support)

```bash
git clone https://github.com/hydrafusion/xllm.git
cd xllm
cargo build --release

# Use the built binary
./target/release/xllm --init
```

**Note**: The published crate on crates.io only includes basic Claude API functionality. For the full experience including the gRPC proxy server (`xllm-proxy`), you need to clone the repository.

## Dev Mode

```bash
# Clone the repository first
git clone https://github.com/hydrafusion/xllm.git
cd xllm

# Run xllm CLI from workspace root
cargo run -p xllm -- -m haiku3 "How can i use xllm?" --file ./example.py

# Or from the xllm directory
cd xllm && cargo run -- -m haiku3 "How can i use xllm?" --file ./example.py
```

## Building from source

```bash
# Clone the repository
git clone https://github.com/hydrafusion/xllm.git
cd xllm

# Build the entire workspace (includes proxy server)
cargo build --release 

# Run xllm from workspace root
cargo run -p xllm -- -m haiku3 "How can i use xllm?" --file ./example.py

# Or install the xllm binary manually
cargo install --path ./xllm
```

## Workspace Structure

This repository contains multiple packages:

- **`xllm`** - The main CLI application (published on crates.io)
- **`xllm-proxy`** - TCP proxy server for enhanced security (clone only)
