# xllm

An extensable cli utility for running LLMs.

## Installation

```Basho
cargo install xllm

```

## Dev Mode

```bash
cargo run -- -m haiku3 "How can i use xllm?" --file ./example.py

```

## Building from source

```bash
cargo build --release 
cargo run -- -m haiku3 "How can i use xllm?" --file ./example.py

## Or install it manually
cargo install --path .
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
