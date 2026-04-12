Title: WASI binary wrapper for pdf_oxide — stdin→stdout, works with wasmtime/wasmer/Deno

---

Hey, I'm Ryan and I am making a LLM middleware.  I support custom tooling in it, WASM/WASI being a target and I put together a pdf_oxide WASI version for my use.  Please feel free to take it if you would like.

What it does: pipe a PDF in via stdin, get markdown out via stdout. singl e Rust binary, wasmer or wasitime is the only pre-req

    cat document.pdf | wasmtime pdf-oxide-wasi.wasm
    cat document.pdf | wasmtime pdf-oxide-wasi.wasm -- --format text
    cat document.pdf | wasmtime pdf-oxide-wasi.wasm -- --pages 1-5

One note for anyone trying to reproduce: pdf_oxide 0.3.14 uses
ceil_char_boundary, which is still nightly-only as of Rust 1.90, so it
requires `cargo +nightly build`. If there's interest in making this a
first-class target, stabilizing that would unblock it on stable.

I didn't see a wasm32-wasip1 release in pdf_oxide_cli or anywhere upstream —
happy to contribute this back as a subdirectory, a CI release target, or
leave it as a standalone repo, whichever works best for you. just wanted you to know it's possible and working!
