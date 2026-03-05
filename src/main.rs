/// pdf-oxide-wasi: read a PDF from stdin, write markdown to stdout.
///
/// Interface:
///   stdin  — raw PDF bytes
///   stdout — markdown (page-by-page, with <!-- page N --> separators)
///   stderr — error messages
///   exit 0 — success
///   exit 1 — error (unreadable PDF, empty input, etc.)
///
/// Optional args (positional, in order):
///   --format markdown|text   default: markdown
///   --pages N-M              page range, 1-indexed inclusive (default: all)
///   --tables-only            emit only table blocks
use std::io::{self, Read, Write};
use std::process;
use pdf_oxide::converters::ConversionOptions;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let opts = parse_args(&args[1..]);

    // Read entire stdin into a buffer
    let mut buf = Vec::new();
    if let Err(e) = io::stdin().read_to_end(&mut buf) {
        eprintln!("pdf-oxide-wasi: failed to read stdin: {e}");
        process::exit(1);
    }
    if buf.is_empty() {
        eprintln!("pdf-oxide-wasi: no input (pipe a PDF via stdin)");
        process::exit(1);
    }

    let mut doc = match pdf_oxide::PdfDocument::open_from_bytes(buf) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("pdf-oxide-wasi: failed to parse PDF: {e}");
            process::exit(1);
        }
    };

    let page_count = match doc.page_count() {
        Ok(n) => n,
        Err(e) => {
            eprintln!("pdf-oxide-wasi: failed to get page count: {e}");
            process::exit(1);
        }
    };

    if page_count == 0 {
        eprintln!("pdf-oxide-wasi: PDF has no pages");
        process::exit(1);
    }

    // Resolve page range (1-indexed input → 0-indexed internal)
    let (first, last) = opts.page_range
        .map(|(a, b)| (a.saturating_sub(1), (b.min(page_count)).saturating_sub(1)))
        .unwrap_or((0, page_count - 1));

    let md_options = ConversionOptions {
        include_form_fields: true,
        ..ConversionOptions::default()
    };

    let stdout = io::stdout();
    let mut out = io::BufWriter::new(stdout.lock());

    for i in first..=last {
        let content = match opts.format {
            Format::Markdown => doc.to_markdown(i, &md_options).unwrap_or_default(),
            Format::Text => doc.extract_text(i).unwrap_or_default(),
        };

        if opts.tables_only {
            // Filter: only emit lines that are part of markdown table blocks
            let table_lines: Vec<&str> = content
                .lines()
                .filter(|l| l.trim_start().starts_with('|'))
                .collect();
            if !table_lines.is_empty() {
                writeln!(out, "<!-- page {} -->", i + 1).ok();
                for l in table_lines {
                    writeln!(out, "{l}").ok();
                }
            }
        } else if !content.trim().is_empty() {
            writeln!(out, "<!-- page {} -->", i + 1).ok();
            writeln!(out, "{content}").ok();
        }
    }
}

// ---------------------------------------------------------------------------

#[derive(Debug)]
enum Format {
    Markdown,
    Text,
}

#[derive(Debug)]
struct Opts {
    format: Format,
    page_range: Option<(usize, usize)>,
    tables_only: bool,
}

fn parse_args(args: &[String]) -> Opts {
    let mut format = Format::Markdown;
    let mut page_range = None;
    let mut tables_only = false;

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--format" => {
                i += 1;
                if i < args.len() {
                    format = match args[i].as_str() {
                        "text" => Format::Text,
                        _ => Format::Markdown,
                    };
                }
            }
            "--pages" => {
                i += 1;
                if i < args.len() {
                    page_range = parse_range(&args[i]);
                }
            }
            "--tables-only" => {
                tables_only = true;
            }
            _ => {}
        }
        i += 1;
    }

    Opts { format, page_range, tables_only }
}

fn parse_range(s: &str) -> Option<(usize, usize)> {
    if let Some((a, b)) = s.split_once('-') {
        let start: usize = a.trim().parse().ok()?;
        let end: usize = b.trim().parse().ok()?;
        Some((start, end))
    } else {
        let n: usize = s.trim().parse().ok()?;
        Some((n, n))
    }
}
