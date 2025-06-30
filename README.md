# xllm

An extensable cli utility for running LLMs.

## Dev Mode

```bash
cargo run -- -m haiku3 "How can i use xllm?" --file ./example.py

```

## Building from source

```bash
cargo build --release 
cargo install --path .
```

## Future Features

- [ ] Add support for multiple LLMs

- [ ] Add support for Ollama

- [x] Add support for String + File Concatination

- [ ] Add support for open context window with specific name

```Bash

xllm -m haiku3 -o window_name "How can i use xllm?" --file ./example.py
```

## Todos

- [ ] Custom Error Handling's

- [ ] Parameter: Output formatting (default is markdown)

- [ ] Is it better to pipe to glow <https://github.com/charmbracelet/glow>?

- [ ] Create xllm --help

- [ ] Create Cargo Documentation

## Usage

```Bash
xllm "How can I create python enums?" --file ./exmple.py
```
