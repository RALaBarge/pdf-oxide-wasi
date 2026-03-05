# pdf-oxide-wasi

> pdf_oxide compiled to WASI — extract text, tables, and markdown from PDFs via stdin/stdout.

**Status: in progress — see NEXT SESSION below.**

---

## What this is

[pdf_oxide](https://github.com/yfedoseev/pdf_oxide) is a Rust-backed PDF library doing 0.8ms/doc with 100% pass rate on 3,830 test PDFs. It ships Python bindings and a JS/browser WASM build. This repo wraps it as a proper WASI binary — stdin in, markdown out — so it works with any WASI runtime (wasmtime, wasmer, Node.js WASI, browser) and any agent framework without pulling in Python or Node.

Discovered while building [BeigeBox](https://github.com/RALaBarge/beigebox), an OpenAI-compatible LLM proxy that has a WASI transform pipeline and an operator agent. BeigeBox already uses wasmtime. The goal was a PDF tool the operator could call as a WASM module — zero deps, fully self-contained, ships as a `.wasm` binary in GitHub Releases.

This is useful beyond BeigeBox: any agentskills.io-compatible agent (Claude Code, Cursor, Gemini CLI, OpenHands) can consume the bundled `SKILL.md` and invoke the binary via wasmtime.

## Intended interface

```bash
# stdin = raw PDF bytes, stdout = markdown
cat document.pdf | wasmtime pdf-oxide-wasi.wasm

# flags (planned)
cat document.pdf | wasmtime pdf-oxide-wasi.wasm -- --format text
cat document.pdf | wasmtime pdf-oxide-wasi.wasm -- --pages 1-5
cat document.pdf | wasmtime pdf-oxide-wasi.wasm -- --tables-only
```

Exit code 0 on success, 1 on error. Errors go to stderr.

## Planned repo structure

```
pdf-oxide-wasi/
├── src/
│   └── main.rs              ← thin WASI wrapper: read stdin, call pdf_oxide, write stdout
├── Cargo.toml               ← target wasm32-wasip1, pdf_oxide dep
├── SKILL.md                 ← agentskills.io compatible skill
├── .github/
│   └── workflows/
│       └── release.yml      ← cargo build --target wasm32-wasip1 on tag push
│                               uploads .wasm to GitHub Releases
└── README.md
```

## Key open question

pdf_oxide already compiles to `wasm32-unknown-unknown` (browser WASM). The question is whether `wasm32-wasip1` works cleanly — WASI adds filesystem/stdio syscalls that the dependency tree needs to handle. Worth checking if yfedoseev would accept a WASI target upstream (open an issue) but build it here first either way.

---

## NEXT SESSION — pick up here

**Goal:** get a working `.wasm` binary that reads a PDF from stdin and writes markdown to stdout.

### Step 1 — scaffold the Rust project

```bash
cd /home/jinx/ai-stack/pdf-oxide-wasi
cargo init --name pdf-oxide-wasi
rustup target add wasm32-wasip1
```

### Step 2 — write src/main.rs

```rust
use std::io::{self, Read, Write};

fn main() {
    let mut buf = Vec::new();
    io::stdin().read_to_end(&mut buf).expect("failed to read stdin");

    match pdf_oxide::PdfDocument::from_bytes(&buf) {
        Ok(doc) => {
            let count = doc.page_count();
            for i in 0..count {
                if let Ok(md) = doc.to_markdown(i, true) {
                    if !md.trim().is_empty() {
                        println!("<!-- page {} -->\n{}", i + 1, md);
                    }
                }
            }
        }
        Err(e) => {
            eprintln!("pdf-oxide-wasi: error: {}", e);
            std::process::exit(1);
        }
    }
}
```

> **Note:** `PdfDocument::from_bytes()` may not exist yet in pdf_oxide's public API — check the crate docs. If not, file an issue upstream or use the CLI as a subprocess fallback.

### Step 3 — Cargo.toml

```toml
[package]
name = "pdf-oxide-wasi"
version = "0.1.0"
edition = "2021"

[[bin]]
name = "pdf-oxide-wasi"
path = "src/main.rs"

[dependencies]
pdf_oxide = "0.10"

[profile.release]
opt-level = "z"
lto = true
strip = true
```

### Step 4 — attempt the build

```bash
cargo build --target wasm32-wasip1 --release
```

If pdf_oxide's deps don't compile to WASI cleanly, the errors will tell you which crates are the problem. Common culprits: anything doing thread spawning, dynamic linking, or platform-specific syscalls. Options if it fails:
- Patch deps with WASI-compatible alternatives
- Open upstream issue on pdf_oxide asking for WASI target support
- Wrap the CLI binary via WASI instead (heavier but works)

### Step 5 — smoke test

```bash
cat some.pdf | wasmtime target/wasm32-wasip1/release/pdf-oxide-wasi.wasm
```

### Step 6 — write SKILL.md + GitHub Actions release workflow

Once the binary works, add the agentskills.io SKILL.md and a release workflow that builds and uploads the `.wasm` on git tag.

### Step 7 — wire back into BeigeBox

In `beigebox/beigebox/tools/pdf_reader.py`, replace the Python pdf_oxide call with a wasmtime subprocess call to the downloaded `.wasm`. BeigeBox already has wasmtime in requirements.

---

## Context

- Discovered via: conversation about higress, pdf_oxide, forge, handy while building BeigeBox
- Related repos: [anthropics/skills](https://github.com/anthropics/skills), [agentskills.io](https://agentskills.io), [K-Dense-AI/claude-scientific-skills](https://github.com/K-Dense-AI/claude-scientific-skills)
- BeigeBox already has: wasmtime in requirements.txt, WASM transform pipeline in wasm_runtime.py, pdf_reader Python tool (fallback), pdf-processing Agent Skill in 2600/skills/
