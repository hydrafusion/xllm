# xllm

Example cli utility for running LLMs.

## Installation

```Bash
cargo install xllm

xllm --init # This will generate a config file

```

## Dev Mode

```bash
# Run xllm CLI from workspace root
cargo run -p xllm -- -m haiku3 "How can i use xllm?" --file ./example.py

# Or from the xllm directory
cd xllm && cargo run -- -m haiku3 "How can i use xllm?" --file ./example.py
```

## Building from source

```bash
# Build the entire workspace
cargo build --release 

# Run xllm from workspace root
cargo run -p xllm -- -m haiku3 "How can i use xllm?" --file ./example.py

# Or install the xllm binary manually
cargo install --path ./xllm
```

## Future Features

- [ ] Add support for multiple LLMs

- [ ] Add support for Ollama

- [x] Add support for String + File Concatination

- [ ] Add support for open context window with specific name This will use
  hashbrown with window name below.

```Bash
# Example of opening a context window with a specific name
xllm -m haiku3 -o window_name "How can i use xllm?" --file ./example.py
```

## Todos

- [ ] Move todos into issues

- [ ] Add xllm init to create a config file.

- [ ] Enable thinking

- [ ] Custom Error Handling's

- [ ] Parameter: Output formatting (default is markdown)

- [ ] Is it better to pipe to glow <https://github.com/charmbracelet/glow>?

- [ ] Create xllm --help

- [ ] Create Cargo Documentation

- [ ] Build for multiple architectures

## Usage

- file is optional, if provided, it will be concatenated with the prompt

```Bash
xllm "How can I create python enums?" --file exmple.py
```

### Nice Features

#### gRPC Proxy Server

The `xllm-proxy` provides a secure, efficient reverse proxy for API requests:

```bash
# Start the proxy server
cargo run -p xllm-proxy

# Configure xllm to use the proxy
xllm --init  # Creates config file
# Edit config.toml to enable proxy mode:
# [global]
# proxy = true
# proxy_url = "http://127.0.0.1:50051"
```

**Benefits:**
- 🔒 **Enhanced Security**: gRPC encryption vs plain HTTP
- ⚡ **Better Performance**: Protobuf compression and gRPC multiplexing  
- 🛡️ **Type Safety**: Strongly typed message definitions
- 📊 **Request Logging**: Centralized logging and monitoring

A really nice feature would be using a proxy to handle the request, and the cli
tool will compress encript the request.
