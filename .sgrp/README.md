# PCF Development Guide

This directory contains the project-level rules for **PCF (Processing Compact File)**. Read this file before changing code, the language model, the CLI, the runtime, or repository tooling.

PCF is an experimental Rust workspace. The specification and implementation are still evolving, but changes must remain internally consistent and must not silently break the existing public behaviour.

## 1. Development rules

- Keep changes focused and reviewable.
- Prefer the smallest implementation that correctly solves the problem.
- Do not add a dependency when the Rust standard library or an existing workspace dependency is sufficient.
- Do not add a dependency only for convenience.
- Preserve the layered architecture of the workspace.
- Keep PCF Core independent from SSAR-specific behaviour.
- Keep parser, lexer, AST, diagnostics, runtime, formatter, ecosystem, and CLI responsibilities separated.
- Do not introduce C/C++ header files for Rust declarations.
- Keep source spans byte-based and convert them to human-readable line/column information only at presentation boundaries.
- Treat diagnostics as structured data, not as terminal-only strings.
- Update tests when behaviour changes.
- Update documentation when a command, syntax rule, public type, invariant, or workflow changes.
- Do not remove license or attribution information.

## 2. Workspace architecture

The workspace is split into crates so that lower-level components do not depend on higher-level application concerns.

```text
pcf-source
    |
pcf-span
    |
pcf-token
    |
pcf-lexer
    |
pcf-ast
    |
pcf-parser
    |
pcf-diagnostics
    |
pcf-resolver
    |
pcf-analyzer
    |
pcf-validator
    |
pcf-runtime
    |
pcf-stdlib
    |
pcf-formatter
    |
pcf-ecosystem
    +-- pcf-ecosystem-ssar
    +-- pcf-ecosystem-openscript
    |
pcf
    |
pcf-cli
    |
pcf-test
```

The exact dependency graph is defined by the individual `Cargo.toml` files. Do not bypass that architecture by importing a high-level crate into a lower-level crate.

### Core responsibilities

`pcf-source` owns source-oriented abstractions.

`pcf-span` owns source spans and positions.

`pcf-token` defines the lexical token model and keyword mapping.

`pcf-lexer` converts source text into tokens and structured diagnostics.

`pcf-ast` defines the syntax tree shared by parser and later stages.

`pcf-parser` converts tokens into an AST and produces syntax diagnostics.

`pcf-diagnostics` defines diagnostic severity, codes, labels, rendering support, and sinks.

`pcf-resolver` is responsible for name/module resolution.

`pcf-analyzer` performs semantic analysis such as unused, unreachable, duplicate, and import analysis when implemented.

`pcf-validator` performs AST validation against PCF rules.

`pcf-runtime` owns execution values, scopes, permissions, capabilities, registries, and evaluation.

`pcf-stdlib` owns standard-library functionality.

`pcf-formatter` owns source formatting.

`pcf-ecosystem` defines integration boundaries. Ecosystem-specific crates remain outside PCF Core semantics.

`pcf` is the public facade combining the principal PCF stages.

`pcf-cli` is the terminal interface. Terminal presentation must not leak into Core data structures.

`pcf-test` contains reusable test-facing helpers.

## 3. Rust requirements

The project currently declares Rust `>=1.90.0` in `.sgrp/ref.json`. Use a current stable Rust toolchain that supports edition 2024.

Do not change the minimum Rust version only to use a newer language feature unless the project requirement is intentionally changed as part of the same work.

The workspace uses Cargo resolver 2 and edition 2024.

## 4. Dependencies

Existing workspace dependencies are preferred:

- `anyhow` for application-level error propagation.
- `clap` for CLI argument parsing.
- `indexmap` where insertion-ordered maps are actually required.
- `miette` for diagnostic infrastructure where applicable.
- `serde` and `serde_json` for serialization boundaries.
- `thiserror` for typed library errors.
- `toml` for TOML configuration boundaries.

Do not introduce another crate if the required operation can be implemented clearly with `std` or an already available dependency.

When a dependency is changed, update `Cargo.lock` with Cargo rather than editing the lockfile manually.

## 5. Source and ownership rules

Prefer borrowing over cloning when ownership does not require a clone.

Do not clone tokens, AST nodes, diagnostics, or source text merely to make a borrow checker error disappear. Restructure the code when that is clearer and cheaper.

Use `String` only when the value owns text. Use `&str` for borrowed source or display-only data.

Use `Path` rather than `PathBuf` for functions that only need to inspect a path.

Use `std::mem::take` when an owned collection is being moved out of a mutable parser/state object and the old collection is no longer needed.

Avoid allocations in hot lexer/parser loops when a borrowed representation is sufficient. Do not sacrifice correctness or readability for micro-optimisation.

## 6. Lexer contract

The lexer must consume the complete input without panicking.

Every returned token has a byte-based `Span` into the original source.

The lexer emits an `Eof` token for normal lexing, including an empty source.

Whitespace other than newline is insignificant. Newlines are represented explicitly because the parser currently accepts them as statement separators.

Line comments beginning with `//` and comments beginning with `#` are ignored until the next newline.

Strings use double quotes. Supported escapes are defined by `pcf-lexer/src/string.rs`.

Invalid characters must produce a structured error diagnostic and an `Unknown` token so later processing remains recoverable.

Invalid numeric literals must not silently become valid-looking tokens. Numeric conversion failures must produce a diagnostic and an `Unknown` token.

Do not use `unwrap()` or indexing in the lexer for data that can be malformed by user input.

## 7. Parser contract

The parser consumes tokens and returns `ParseResult`:

```rust
pub struct ParseResult {
    pub program: Option<pcf_ast::Program>,
    pub diagnostics: Vec<pcf_diagnostics::Diagnostic>,
}
```

A parser failure caused by user source should be represented by diagnostics, not a panic.

The parser should recover where practical so that multiple diagnostics can be reported in one pass.

The parser must tolerate an empty token slice and must not assume that callers always supplied an `Eof` token.

Statement separators currently include newline and semicolon.

The currently implemented executable statement is the static form:

```text
output "text"
```

`output` currently requires a string literal. Runtime expression evaluation for `output` is not implied by this syntax rule.

Keep AST spans tied to the source tokens that produced them. Do not calculate spans from formatted output.

## 8. AST rules

AST types are public through `pcf-ast` and selected re-exports from `pcf`.

Do not create duplicate representations of the same syntax concept in the parser and AST crates.

`Expression` contains `LiteralExpression`, identifier, unary, binary, and group forms. `OutputStatement` currently contains a `LiteralExpression` because static output is intentionally restricted to literals.

When adding an AST node:

1. Add the type in the appropriate AST module.
2. Re-export it from `pcf-ast/src/lib.rs` when it is part of the public AST API.
3. Update parser construction.
4. Update span handling.
5. Update resolver/analyzer/validator/runtime consumers when applicable.
6. Add focused tests.
7. Update documentation/specification if the syntax is user-visible.

## 9. Diagnostics

Diagnostics have a severity, stable code, message, labels, and notes.

Use stable diagnostic codes for machine-readable identification. Do not change an existing code for cosmetic wording changes.

The primary label should identify the most relevant source location. Secondary labels may provide additional context.

Diagnostics must remain useful without the terminal UI.

The CLI may render diagnostics, but Core crates should return structured diagnostics rather than terminal-specific strings.

Current lexer diagnostic codes include:

- `PCF0001` invalid character.
- `PCF0002` invalid string escape.
- `PCF0003` unterminated string literal.
- `PCF0004` invalid numeric literal.

Current parser diagnostic codes include:

- `PCF1000` unexpected token.
- `PCF1001` invalid value after `output`.

New codes should be documented and should not collide with existing codes.

## 10. Public facade rules

`crates/pcf/src/lib.rs` is the high-level API boundary.

`pcf::parse` should provide a simple success/failure API.

`pcf::check` should perform lexing and parsing once, return all collected diagnostics, and expose statically discoverable `output` contents through `CheckOutput`.

`pcf::execute` is the runtime execution boundary. It must not be described as executing functionality that the evaluator does not implement yet.

When changing a public function, check every workspace caller before changing its signature.

## 11. Check output contract

`pcf::check` collects static string contents from `OutputStatement` nodes in source order.

This is intentionally analysis-time output. It does not execute the program.

For example:

```text
output "hello"
output "world"
```

produces two check outputs:

```text
hello
world
```

The output keeps its source span so editor integrations can associate the result with the originating statement.

Nested blocks and function bodies should be traversed when collecting static outputs. Declarations that are not executable statement bodies should not be treated as output automatically.

## 12. CLI contract

Build and invoke the CLI with:

```text
cargo run -p pcf-cli -- <command> <file>
```

The commands are:

```text
pcf inspect <file>
pcf inspect <file> --tokens
pcf inspect <file> --ast
pcf inspect <file> --tokens --ast
pcf parse <file>
pcf check <file>
pcf run <file>
```

`inspect` defaults to token output. `--ast` selects AST output. Supplying both flags shows both.

`parse` parses once and prints the resulting AST plus parser/lexer diagnostics.

`check` validates the source without executing it. It prints diagnostic details and the statically discovered output content.

`run` is the execution entry point. Its actual behaviour is limited by the currently implemented runtime evaluator. Do not make documentation claim that `run` executes features that are still stubs.

All CLI commands return exit code `0` when no error diagnostic is present and `1` when an error is present or execution fails.

A file read failure is an application error and exits with code `1` through `main.rs`.

CLI output is presentation data. Do not parse CLI output inside Core code.

## 13. CLI performance rules

Do not lex and parse the same source twice when one pass can provide the required result.

The CLI `parse` command therefore uses the lexer and parser result directly rather than calling `pcf::parse` and then `pcf::check` over the same source.

Keep diagnostic counting linear in the number of diagnostics.

Avoid unnecessary intermediate vectors when an iterator or direct traversal is clearer.

Terminal formatting is not normally the bottleneck; correctness and stable output take priority over premature formatting micro-optimisation.

## 14. Terminal UI rules

`pcf-cli/src/ui.rs` owns terminal report formatting.

The report uses a tree-like structure with fields, lines, sections, blanks, and a footer.

Field alignment is calculated per node level. Do not hard-code spaces into individual command output strings.

Diagnostic locations are calculated from byte offsets against the original UTF-8 source and displayed as one-based line and column values.

Do not interpret a byte offset as a character index.

Duration formatting uses nanoseconds, microseconds, milliseconds, or seconds depending on magnitude.

Keep terminal rendering deterministic so snapshot-style tests remain stable.

## 15. Runtime rules

The runtime contains values, scopes, permissions, capabilities, registries, native functions, and evaluation infrastructure.

A runtime feature is not complete merely because an AST node exists. A complete runtime feature normally requires:

1. Syntax support.
2. AST representation.
3. Resolution/validation where required.
4. Runtime evaluation.
5. Diagnostics for invalid runtime states.
6. Tests.
7. CLI or public API integration where applicable.
8. Documentation.

Do not silently return `Null` as a substitute for unimplemented observable behaviour unless the API explicitly defines that behaviour as a placeholder.

## 16. Testing requirements

Before submitting a change, run as many of the following as the environment supports:

```text
cargo fmt --all -- --check
cargo check --workspace
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

For CLI changes, additionally test:

```text
cargo run -p pcf-cli -- inspect <file>
cargo run -p pcf-cli -- inspect <file> --tokens
cargo run -p pcf-cli -- inspect <file> --ast
cargo run -p pcf-cli -- inspect <file> --tokens --ast
cargo run -p pcf-cli -- parse <file>
cargo run -p pcf-cli -- check <file>
cargo run -p pcf-cli -- run <file>
```

Test at least these source classes:

```text
output "hello"
output "hello"; output "world"
output "hello"\noutput "world"
output
output 42
unknown
@
"unterminated
999999999999999999999999999999999999999

```

Also test an empty file and UTF-8 source. The expected result must be determined from the language rules, not from what a previous implementation happened to print.

## 17. Quality-control checklist

Before considering a change complete:

- [ ] Code compiles with the workspace toolchain.
- [ ] `cargo fmt --all -- --check` passes.
- [ ] `cargo test --workspace` passes.
- [ ] `cargo clippy --workspace --all-targets -- -D warnings` passes where Clippy is available.
- [ ] No new dependency was added unnecessarily.
- [ ] No user-input path can panic unexpectedly.
- [ ] Diagnostics are structured and have stable codes where appropriate.
- [ ] Error exit codes are correct.
- [ ] CLI commands do not repeat expensive lex/parse work unnecessarily.
- [ ] AST spans remain correct.
- [ ] UTF-8 source positions are handled correctly.
- [ ] Public API changes have all callers updated.
- [ ] Tests cover the changed behaviour and important failure cases.
- [ ] Documentation reflects the implementation.
- [ ] No generated `target/`, temporary files, logs, or editor files are committed.
- [ ] Licensing and attribution remain intact.

If a check could not be executed, state that explicitly in the change report instead of claiming it passed.

## 18. Repository hygiene

The repository ignores Cargo build output under `/target/`, PCF output under `/.pcf/` and `/out/`, IDE state, operating-system files, and temporary logs.

`Cargo.lock` is committed because this is an application/tooling workspace and reproducible dependency resolution is useful.

Do not commit local build artifacts.

Empty directories are not represented by Git. A `.gitkeep` is acceptable only when an empty directory itself is intentionally part of the repository structure, such as a reserved `benches/` directory. Do not add `.gitkeep` files without a reason.

## 19. Source headers

PCF does not use C/C++-style header files.

Rust declarations belong in Rust modules and public APIs are exposed through `pub` items and `pub use` re-exports.

Do not create artificial `.h` files for Rust modules.

PCF source files also do not require a repeated OpenStudio-style metadata header. Package metadata belongs in `Cargo.toml`, repository metadata belongs in repository configuration, and licensing follows the project license/SPDX convention.

If a future FFI binding genuinely requires C headers, keep those headers inside the binding boundary rather than adding them to PCF Core.

## 20. Ecosystem separation

PCF Core must remain usable without SSAR-specific runtime assumptions.

SSAR-specific functionality belongs in `pcf-ecosystem-ssar` or another appropriate integration boundary.

OpenScript-specific behaviour belongs in `pcf-ecosystem-openscript` when it is not part of the PCF specification itself.

Do not add an ecosystem dependency to Core simply because an integration currently needs it.

## 21. Commit labels

Commit titles use one primary label:

`[Core]` — PCF language, AST, parser, lexer, core semantics, or fundamental data model.

`[Feature]` — a new user-visible/project capability.

`[Fix]` — correction of incorrect behaviour.

`[Refactor]` — internal restructuring without intentional behaviour change.

`[Std]` — standard-library functionality.

`[Ecosystem]` — ecosystem integration.

`[Docs]` — documentation/specification.

`[Test]` — tests whose main purpose is testing infrastructure or coverage.

`[Build]` — Cargo/workspace/build/packaging changes.

`[CI]` — GitHub Actions and continuous integration.

`[Chore]` — maintenance work.

`[Misc]` — genuinely uncategorised repository work.

Choose the label that describes the primary purpose, not the number of files changed.

Examples:

```text
[Fix] Report invalid numeric literals
[Core] Add expression precedence
[Refactor] Split parser recovery logic
[Docs] Document CLI contracts
[Test] Add parser recovery cases
[Build] Update workspace metadata
[CI] Add workspace quality checks
```

## 22. Change procedure

For a non-trivial change:

1. Read this file and the relevant crate's `module.md`.
2. Inspect the current implementation and its tests.
3. Identify the public behaviour and invariants affected.
4. Make the smallest coherent change.
5. Add or update focused tests immediately.
6. Format the affected Rust files.
7. Run workspace checks.
8. Test the CLI manually when the CLI is affected.
9. Review the diff for accidental changes, duplicated work, unnecessary allocations, and dependency changes.
10. Update documentation/specification.
11. Re-run the relevant checks after documentation and code changes.
12. Record any checks that could not be run.

Do not mark a task complete solely because the code looks correct. The implementation, tests, public API, CLI behaviour, and documentation must agree.

## 23. Current implementation boundaries

PCF is not yet a complete programming language runtime.

The repository contains the architecture for many language/runtime layers, while some modules remain intentionally minimal or are placeholders for future implementation.

Do not fill empty architectural modules with speculative functionality merely to make the repository look complete.

When implementing a new feature, implement it end-to-end only when its syntax and semantics are sufficiently defined. Otherwise, keep the placeholder explicit and document the boundary.

## 24. Final verification record

For every maintenance task, verify at minimum:

```text
Repository state
    -> correct base revision
    -> expected files changed only
    -> no generated artifacts

Rust
    -> format
    -> check
    -> tests
    -> clippy where available

Language
    -> lexer valid input
    -> lexer invalid input
    -> parser valid input
    -> parser recovery
    -> AST spans
    -> diagnostics

CLI
    -> inspect
    -> parse
    -> check
    -> run
    -> exit status
    -> UTF-8 output

Documentation
    -> command syntax
    -> architecture
    -> known limitations
    -> development rules
```

Never report a control as passed if it was not actually executed.

## Repository

Repository: https://github.com/ssar-group/pcf

Website: https://ssar-group.com

---

_Copyright © 2026 SSAR Group._
