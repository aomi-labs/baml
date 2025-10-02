# Repository Guidelines

## Project Structure & Module Organization
This crate generates the Rust SDK for BAML and lives under `engine/generators/languages/rust`. Core orchestration sits in `src/lib.rs`, with IR-to-Rust transforms in `src/ir_to_rust/` and shared helpers split across `src/functions.rs`, `src/generated_types.rs`, and `src/utils.rs`. Askama templates that define emitted files live in `src/_templates/`, while reproducible integration fixtures are checked into `generated_tests/` (each folder is a standalone Cargo crate compiled by the harness). `askama.toml` tweaks template search paths, and `Cargo.toml` holds crate metadata and workspace dependencies.

## Build, Test, and Development Commands
Run `cargo fmt --package generators-rust` to apply the shared formatting profile defined in `engine/rustfmt.toml`. `cargo check --package generators-rust` validates the generator compiles without emitting artifacts. Use `cargo clippy --package generators-rust --all-targets` before review to surface lints that template edits can silently introduce. Execute `cargo test --package generators-rust` to render fixtures in `generated_tests/*` and ensure the generated crates compile and diff cleanly via the local `test-harness`.

## Coding Style & Naming Conventions
Follow Rust 2021 defaults with 4-space indentation, trailing commas on multi-line structures, and standard naming: modules/functions in `snake_case`, public types in `PascalCase`, and constants in `SCREAMING_SNAKE_CASE`. Prefer expression-oriented helpers over imperative mutation; generator functions should return `anyhow::Result` for recoverable failures. When touching templates, reuse helpers already defined in `_templates/` and keep rendered filenames in sync with the `collector.add_file` calls in `src/lib.rs`.

## Testing Guidelines
Module-level unit tests belong alongside implementation code under `#[cfg(test)]`. Integration coverage relies on the `test-harness` dev-dependency, which iterates through fixtures in `generators/data/<suite>/rust` and validates the rendered output under `generated_tests/`. Add or clone a fixture when introducing new IR features so regressions surface during `cargo test`. Use standard `cargo test` filtering (for example `cargo test --package generators-rust sample`) to focus on a single suite, and set `RUN_GENERATOR_TESTS=1` to run the downstream `cargo test --verbose` inside each generated crate.

## Commit & Pull Request Guidelines
Recent history favors concise, imperative subjects such as "Add asserts and classes tests"; mirror that style and keep titles under ~60 characters. Describe notable template or IR changes in the body, including regeneration steps if contributors must run `baml-cli generate`. Pull requests should link any tracked issues, list validation (`cargo fmt`, `cargo clippy`, `cargo test`), and attach diffs or logs that demonstrate regenerated artifacts when applicable.
