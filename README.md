# pdf-oxide-wasi

A WASI binary wrapper around [pdf_oxide](https://github.com/yfedoseev/pdf_oxide) — pipe a PDF in via stdin, get markdown out via stdout.

**Single file. No Python. No Node. No native deps. Runs anywhere wasmtime or wasmer runs.**

```bash
cat document.pdf | wasmtime pdf-oxide-wasi.wasm
```

---

## Why

pdf_oxide already compiles to `wasm32-unknown-unknown` for the browser. This repo adds `wasm32-wasip1` (WASI) support — the server/CLI/agent-framework variant. A WASI binary runs under wasmtime, wasmer, Node.js (WASI module), and Deno without any language runtime or package manager.

Built while integrating pdf_oxide into [BeigeBox](https://github.com/RALaBarge/beigebox), an OpenAI-compatible LLM proxy with a WASI transform pipeline and an operator agent. This fills the gap between pdf_oxide's native CLI and any agent framework that can invoke WASM modules.

---

## Usage

```bash
# Full document as markdown
cat document.pdf | wasmtime pdf-oxide-wasi.wasm

# Plain text instead of markdown
cat document.pdf | wasmtime pdf-oxide-wasi.wasm -- --format text

# Specific page range (1-indexed)
cat document.pdf | wasmtime pdf-oxide-wasi.wasm -- --pages 1-5

# Tables only
cat document.pdf | wasmtime pdf-oxide-wasi.wasm -- --tables-only
```

**Exit codes:** `0` success, `1` error. Errors go to stderr.

**Output format:** pages separated by `<!-- page N -->` comments, form fields appended as a markdown list if present.

---

## Runtime compatibility

| Runtime | Works |
|---|---|
| wasmtime (Linux / macOS / Windows) | ✅ |
| wasmer | ✅ |
| Node.js (`--experimental-wasi-unstable-api`) | ✅ |
| Deno (`--allow-read`) | ✅ |
| Browser (direct) | ❌ needs a WASI shim |

---

## Building from source

Requires nightly Rust (pdf_oxide uses `ceil_char_boundary`, still unstable as of Rust 1.90):

```bash
rustup toolchain install nightly
rustup target add wasm32-wasip1 --toolchain nightly
cargo +nightly build --target wasm32-wasip1 --release
# output: target/wasm32-wasip1/release/pdf-oxide-wasi.wasm
```

Binary size: ~3.5 MB (release, LTO, stripped).

---

## Repo structure

```
pdf-oxide-wasi/
├── src/main.rs       — WASI wrapper: read stdin, call pdf_oxide, write stdout
├── Cargo.toml        — target wasm32-wasip1, pdf_oxide dep
└── README.md
```

---

## Context

- pdf_oxide crate: [crates.io/crates/pdf_oxide](https://crates.io/crates/pdf_oxide) by [@yfedoseev](https://github.com/yfedoseev)
- Related: [pdf_oxide_cli](https://crates.io/crates/pdf_oxide_cli) (native CLI), [pdf_oxide_mcp](https://crates.io/crates/pdf_oxide_mcp) (MCP server)
- BeigeBox: [github.com/RALaBarge/beigebox](https://github.com/RALaBarge/beigebox)
